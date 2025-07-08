#![cfg(feature = "serde")]

use deadpool_redis::Runtime;
use futures::FutureExt;
use redis::cmd;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
struct Config {
    #[serde(default)]
    redis: deadpool_redis::Config,
}

impl Config {
    pub fn from_env() -> Self {
        config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap()
    }
}

fn create_pool() -> deadpool_redis::Pool {
    let cfg = Config::from_env();
    cfg.redis.create_pool(Some(Runtime::Tokio1)).unwrap()
}

#[tokio::test]
async fn test_pipeline() {
    use deadpool_redis::redis::pipe;
    let pool = create_pool();
    let mut conn = pool.get().await.unwrap();
    let (value,): (String,) = pipe()
        .cmd("SET")
        .arg("deadpool/pipeline_test_key")
        .arg("42")
        .ignore()
        .cmd("GET")
        .arg("deadpool/pipeline_test_key")
        .query_async(&mut conn)
        .await
        .unwrap();
    assert_eq!(value, "42".to_string());
}

#[tokio::test]
async fn test_high_level_commands() {
    use deadpool_redis::redis::AsyncCommands;
    let pool = create_pool();
    let mut conn = pool.get().await.unwrap();
    conn.set::<_, _, ()>("deadpool/hlc_test_key", 42)
        .await
        .unwrap();
    let value: isize = conn.get("deadpool/hlc_test_key").await.unwrap();
    assert_eq!(value, 42);
}

#[tokio::test]
async fn test_aborted_command() {
    let pool = create_pool();

    {
        let mut conn = pool.get().await.unwrap();
        // Poll the future once. This does execute the query but does not
        // wait for the response. The connection now has a request queued
        // and the response to it will be returned when using the connection
        // the next time:
        // https://github.com/bikeshedder/deadpool/issues/97
        // https://github.com/mitsuhiko/redis-rs/issues/489
        cmd("PING")
            .arg("wrong")
            .query_async::<String>(&mut conn)
            .now_or_never();
    }
    {
        let mut conn = pool.get().await.unwrap();
        let value: String = cmd("PING")
            .arg("right")
            .query_async(&mut conn)
            .await
            .unwrap();
        assert_eq!(value, "right");
    }
}

#[tokio::test]
async fn test_recycled() {
    let pool = create_pool();

    let client_id = {
        let mut conn = pool.get().await.unwrap();

        cmd("CLIENT")
            .arg("ID")
            .query_async::<i64>(&mut conn)
            .await
            .unwrap()
    };

    {
        let mut conn = pool.get().await.unwrap();

        let new_client_id = cmd("CLIENT")
            .arg("ID")
            .query_async::<i64>(&mut conn)
            .await
            .unwrap();

        assert_eq!(
            client_id, new_client_id,
            "the redis connection was not recycled"
        );
    }
}

#[tokio::test]
async fn test_recycled_with_watch() {
    use deadpool_redis::redis::{pipe, Value};

    let pool = create_pool();

    const WATCHED_KEY: &str = "deadpool/watched_test_key";
    const TXN_KEY: &str = "deadpool/txn_test_key";

    // Start transaction on one key and return connection to pool
    let client_with_watch_id = {
        let mut conn = pool.get().await.unwrap();

        let client_id = cmd("CLIENT")
            .arg("ID")
            .query_async::<i64>(&mut conn)
            .await
            .unwrap();

        cmd("WATCH")
            .arg(WATCHED_KEY)
            .query_async::<()>(&mut conn)
            .await
            .unwrap();

        client_id
    };

    {
        let mut txn_conn = pool.get().await.unwrap();

        let new_client_id = cmd("CLIENT")
            .arg("ID")
            .query_async::<i64>(&mut txn_conn)
            .await
            .unwrap();

        // Ensure that's the same connection as the one in first transaction
        assert_eq!(
            client_with_watch_id, new_client_id,
            "the redis connection with transaction was not recycled"
        );

        // Start transaction on another key
        cmd("WATCH")
            .arg(TXN_KEY)
            .query_async::<()>(&mut txn_conn)
            .await
            .unwrap();

        {
            let mut writer_conn = pool.get().await.unwrap();

            // Overwrite key from first transaction from another connection
            cmd("SET")
                .arg(WATCHED_KEY)
                .arg("v")
                .query_async::<()>(&mut writer_conn)
                .await
                .unwrap();
        }

        // Expect that new transaction is not aborted by irrelevant key
        let txn_result = pipe()
            .atomic()
            .set(TXN_KEY, "foo")
            .query_async::<Value>(&mut txn_conn)
            .await
            .unwrap();
        assert_eq!(
            txn_result,
            Value::Array(vec![Value::Okay]),
            "redis transaction in recycled connection aborted",
        );
    }
}

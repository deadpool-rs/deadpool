#![cfg(all(feature = "cluster", feature = "serde"))]

use deadpool_redis::cluster::Runtime;
use futures::FutureExt;
use redis::cmd;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
struct Config {
    #[serde(default)]
    redis_cluster: deadpool_redis::cluster::Config,
}

impl Config {
    pub fn from_env() -> Self {
        config::Config::builder()
            .add_source(
                config::Environment::default()
                    .separator("__")
                    .try_parsing(true)
                    .list_separator(","),
            )
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap()
    }
}

fn create_pool() -> deadpool_redis::cluster::Pool {
    let cfg = Config::from_env();
    cfg.redis_cluster
        .create_pool(Some(Runtime::Tokio1))
        .unwrap()
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
async fn test_read_from_replicas() {
    use deadpool_redis::redis::pipe;
    let mut cfg = Config::from_env();
    cfg.redis_cluster.read_from_replicas = true;
    assert!(cfg.redis_cluster.read_from_replicas);

    let pool = cfg
        .redis_cluster
        .create_pool(Some(Runtime::Tokio1))
        .unwrap();
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
    let _: () = conn.set("deadpool/hlc_test_key", 42).await.unwrap();
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

    let connection_name = "unique_connection_name";

    let connection_details_1 = {
        let mut conn = pool.get().await.unwrap();
        cmd("CLIENT")
            .arg("SETNAME")
            .arg(connection_name)
            .query_async::<()>(&mut conn)
            .await
            .unwrap();

        let current_name: Option<String> = cmd("CLIENT")
            .arg("GETNAME")
            .query_async(&mut conn)
            .await
            .unwrap();

        current_name
    };

    let connection_details_2 = {
        let mut conn = pool.get().await.unwrap();
        let current_name: Option<String> = cmd("CLIENT")
            .arg("GETNAME")
            .query_async(&mut conn)
            .await
            .unwrap();

        current_name
    };

    assert_eq!(
        connection_details_1, connection_details_2,
        "The Redis connection was not recycled: different connection name"
    );
}

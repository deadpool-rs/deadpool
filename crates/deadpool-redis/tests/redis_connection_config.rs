#![cfg(feature = "serde")]

use std::time::Duration;

use deadpool_redis::Runtime;
use redis::AsyncCommands;
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

fn redis_url() -> String {
    Config::from_env().redis.url.unwrap_or_default()
}

/// Verifies that disabling the response timeout allows commands that take longer than the
/// default timeout. Uses `BLPOP` on an empty list with a 1-second server-side timeout.
#[tokio::test]
async fn test_response_timeout_disabled() {
    let pool = deadpool_redis::Config::from_url(redis_url())
        .with_response_timeout(None)
        .create_pool(Some(Runtime::Tokio1))
        .unwrap();

    let mut conn = pool.get().await.unwrap();

    let result: Option<(String, String)> = conn
        .blpop("deadpool/test_timeout_disabled", 1.0)
        .await
        .unwrap();

    assert_eq!(result, None);
}

/// Verifies that setting an explicit response timeout causes commands exceeding it to fail.
#[tokio::test]
async fn test_response_timeout_causes_timeout() {
    let pool = deadpool_redis::Config::from_url(redis_url())
        .with_response_timeout(Some(Duration::from_millis(100)))
        .create_pool(Some(Runtime::Tokio1))
        .unwrap();

    let mut conn = pool.get().await.unwrap();

    let start = std::time::Instant::now();
    let result: Result<Option<(String, String)>, _> =
        conn.blpop("deadpool/test_timeout_short", 1.0).await;
    let elapsed = start.elapsed();

    assert!(result.is_err(), "expected timeout error");
    assert!(
        elapsed < Duration::from_millis(500),
        "should have timed out well before the 1s BLPOP, took {:?}",
        elapsed
    );
}

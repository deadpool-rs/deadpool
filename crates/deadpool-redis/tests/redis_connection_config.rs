#![cfg(feature = "serde")]

use std::time::Duration;

use deadpool_redis::{Manager, Pool, Runtime};
use redis::{AsyncCommands, AsyncConnectionConfig};
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

fn create_pool_with_config(connection_config: AsyncConnectionConfig) -> Pool {
    let manager = Manager::new_with_config(redis_url(), connection_config).unwrap();
    Pool::builder(manager)
        .max_size(1)
        .runtime(Runtime::Tokio1)
        .build()
        .unwrap()
}

fn create_pool_default() -> Pool {
    let manager = Manager::new(redis_url()).unwrap();
    Pool::builder(manager)
        .max_size(1)
        .runtime(Runtime::Tokio1)
        .build()
        .unwrap()
}

/// Verifies that `new_with_config` with `set_response_timeout(None)` allows commands that take
/// longer than the default 500ms timeout.
///
/// Uses `BLPOP` on an empty list with a 1-second timeout. With the default `AsyncConnectionConfig`
/// (500ms response timeout), this would fail. With `set_response_timeout(None)`, it waits the full
/// second and returns nil.
#[tokio::test]
async fn test_response_timeout_can_be_disabled() {
    let config = AsyncConnectionConfig::new().set_response_timeout(None);
    let pool = create_pool_with_config(config);
    let mut conn = pool.get().await.unwrap();

    let result: Option<(String, String)> = conn
        .blpop("deadpool/nonexistent_timeout_test_key", 1.0)
        .await
        .unwrap();
    assert_eq!(result, None);
}

/// Verifies that the default `Manager::new` (without config) uses the redis crate's default
/// timeouts, which causes blocking commands exceeding 500ms to fail.
#[tokio::test]
async fn test_default_manager_times_out_on_slow_commands() {
    let pool = create_pool_default();
    let mut conn = pool.get().await.unwrap();

    let start = std::time::Instant::now();
    let result: Result<Option<(String, String)>, _> = conn
        .blpop("deadpool/nonexistent_default_timeout_key", 1.0)
        .await;
    let elapsed = start.elapsed();

    assert!(
        result.is_err(),
        "expected timeout error with default config"
    );
    assert!(
        elapsed < Duration::from_millis(900),
        "should have timed out before the 1s BLPOP completed, took {:?}",
        elapsed
    );
}

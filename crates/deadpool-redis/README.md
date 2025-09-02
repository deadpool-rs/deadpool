# Deadpool for Redis [![Latest Version](https://img.shields.io/crates/v/deadpool-redis.svg)](https://crates.io/crates/deadpool-redis) [![Build Status](https://img.shields.io/github/actions/workflow/status/deadpool-rs/deadpool/deadpool-redis.yml?branch=main)](https://github.com/deadpool-rs/deadpool/actions/workflows/deadpool-redis.yml?query=branch%3Amain) ![Unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg "Unsafe forbidden") [![Rust 1.75+](https://img.shields.io/badge/rustc-1.75+-lightgray.svg "Rust 1.75+")](https://blog.rust-lang.org/2023/12/28/Rust-1.75.0.html)

Deadpool is a dead simple async pool for connections and objects
of any type.

This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
manager for [`redis`](https://crates.io/crates/redis).

## Features

| Feature          | Description                                                           | Extra dependencies                                | Default |
| ---------------- | --------------------------------------------------------------------- | ------------------------------------------------- | ------- |
| `rt_tokio_1`     | Enable support for [tokio](https://crates.io/crates/tokio) crate      | `deadpool/rt_tokio_1`, `redis/tokio-comp`         | yes     |
| `rt_async-std_1` | Enable support for [async-std](https://crates.io/crates/async-std) crate | `deadpool/rt_async-std_1`, `redis/async-std-comp` | no      |
| `serde`          | Enable support for [serde](https://crates.io/crates/serde) crate      | `deadpool/serde`, `serde/derive`                  | no      |
| `cluster`        | Enable support for Redis Cluster                                      | `redis/cluster-async`                             | no      |
| `sentinel`       | Enable high-level interfaces for communication with Redis sentinels   | `redis/sentinel`, `tokio/sync`                    | no      |

All of the features of [redis](https://crates.io/crates/redis) are also re-exported.
For example, the feature `tls-rustls` does enable the feature `tls-rustls` from the `redis` crate.

## Example

```rust
use std::env;

use deadpool_redis::{redis::{cmd, FromRedisValue}, Config, Runtime};

#[tokio::main]
async fn main() {
    let mut cfg = Config::from_url(env::var("REDIS__URL").unwrap());
    let pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();
    {
        let mut conn = pool.get().await.unwrap();
        cmd("SET")
            .arg(&["deadpool/test_key", "42"])
            .query_async::<()>(&mut conn)
            .await.unwrap();
    }
    {
        let mut conn = pool.get().await.unwrap();
        let value: String = cmd("GET")
            .arg(&["deadpool/test_key"])
            .query_async(&mut conn)
            .await.unwrap();
        assert_eq!(value, "42".to_string());
    }
}
```

### Example with `config` and `dotenvy` crate

```rust
use deadpool_redis::{redis::{cmd, FromRedisValue}, Runtime};
use dotenvy::dotenv;

#[derive(Debug, serde::Deserialize)]
struct Config {
    #[serde(default)]
    redis: deadpool_redis::Config
}

impl Config {
      pub fn from_env() -> Result<Self, config::ConfigError> {
         config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .build()?
            .try_deserialize()
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let cfg = Config::from_env().unwrap();
    let pool = cfg.redis.create_pool(Some(Runtime::Tokio1)).unwrap();
    {
        let mut conn = pool.get().await.unwrap();
        cmd("SET")
            .arg(&["deadpool/test_key", "42"])
            .query_async::<()>(&mut conn)
            .await.unwrap();
    }
    {
        let mut conn = pool.get().await.unwrap();
        let value: String = cmd("GET")
            .arg(&["deadpool/test_key"])
            .query_async(&mut conn)
            .await.unwrap();
        assert_eq!(value, "42".to_string());
    }
}
```

## Example (Cluster)

```rust
use std::env;
use deadpool_redis::{redis::{cmd, FromRedisValue}};
use deadpool_redis::cluster::{Config, Runtime};

#[tokio::main]
async fn main() {
    let redis_urls = env::var("REDIS_CLUSTER__URLS")
        .unwrap()
        .split(',')
        .map(String::from)
        .collect::<Vec<_>>();
    let mut cfg = Config::from_urls(redis_urls);
    let pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();
    {
        let mut conn = pool.get().await.unwrap();
        cmd("SET")
            .arg(&["deadpool/test_key", "42"])
            .query_async::<()>(&mut conn)
            .await.unwrap();
    }
    {
        let mut conn = pool.get().await.unwrap();
        let value: String = cmd("GET")
            .arg(&["deadpool/test_key"])
            .query_async(&mut conn)
            .await.unwrap();
        assert_eq!(value, "42".to_string());
    }
}
```

### Example with `config` and `dotenvy` crate

```rust
use deadpool_redis::redis::{cmd, FromRedisValue};
use deadpool_redis::cluster::{Runtime};
use dotenvy::dotenv;

#[derive(Debug, serde::Deserialize)]
struct Config {
    #[serde(default)]
    redis_cluster: deadpool_redis::cluster::Config
}

impl Config {
      pub fn from_env() -> Result<Self, config::ConfigError> {
         config::Config::builder()
            .add_source(
                config::Environment::default()
                .separator("__")
                .try_parsing(true)
                .list_separator(","),
            )
            .build()?
            .try_deserialize()
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let cfg = Config::from_env().unwrap();
    let pool = cfg.redis_cluster.create_pool(Some(Runtime::Tokio1)).unwrap();
    {
        let mut conn = pool.get().await.unwrap();
        cmd("SET")
            .arg(&["deadpool/test_key", "42"])
            .query_async::<()>(&mut conn)
            .await.unwrap();
    }
    {
        let mut conn = pool.get().await.unwrap();
        let value: String = cmd("GET")
            .arg(&["deadpool/test_key"])
            .query_async(&mut conn)
            .await.unwrap();
        assert_eq!(value, "42".to_string());
    }
}
```

## Example (Sentinel)

```rust
use std::env;
use deadpool_redis::{redis::{cmd, FromRedisValue}, sentinel::SentinelServerType};
use deadpool_redis::sentinel::{Config, Runtime};
use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    use deadpool_redis::redis::pipe;
    let redis_urls = env::var("REDIS_SENTINEL__URLS")
        .unwrap()
        .split(',')
        .map(String::from)
        .collect::<Vec<_>>();

    let master_name = env::var("REDIS_SENTINEL__MASTER_NAME").unwrap();
    let mut cfg = Config::from_urls(redis_urls, master_name, SentinelServerType::Master);
    let pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();

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
```


### Example with `config` and `dotenvy` crate
```rust
use serde::{Deserialize, Serialize};
use deadpool_redis::{redis, Runtime};
use dotenvy::dotenv;

#[derive(Debug, Default, Deserialize, Serialize)]
struct Config {
    #[serde(default)]
    redis_sentinel: deadpool_redis::sentinel::Config,
}

impl Config {
    pub fn from_env() -> Self {
        config::Config::builder()
            .add_source(
                config::Environment::default()
                    .separator("__")
                    .try_parsing(true)
                    .list_separator(",")
                    .with_list_parse_key("redis_sentinel.urls"),
            )
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap()
    }
}

fn create_pool() -> deadpool_redis::sentinel::Pool {
    let cfg = Config::from_env();

    cfg.redis_sentinel
        .create_pool(Some(Runtime::Tokio1))
        .unwrap()
}

#[tokio::main]
async fn main() {
    dotenv().ok();
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

```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

# Deadpool for Lapin [![Latest Version](https://img.shields.io/crates/v/deadpool-lapin.svg)](https://crates.io/crates/deadpool-lapin) ![Unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg "Unsafe forbidden") [![Rust 1.75+](https://img.shields.io/badge/rustc-1.75+-lightgray.svg "Rust 1.75+")](https://blog.rust-lang.org/2023/12/28/Rust-1.75.0.html)

Deadpool is a dead simple async pool for connections and objects
of any type.

This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
manager for [`lapin`](https://crates.io/crates/lapin).

## Features

| Feature          | Description                                                           | Extra dependencies               | Default |
| ---------------- | --------------------------------------------------------------------- | -------------------------------- | ------- |
| `rt_tokio_1`     | Enable support for [tokio](https://crates.io/crates/tokio) crate      | `deadpool/rt_tokio_1`            | yes     |
| `rt_async-std_1` | Enable support for [async-std](https://crates.io/crates/async-std) crate | `deadpool/rt_async-std_1`        | no      |
| `serde`          | Enable support for [serde](https://crates.io/crates/serde) crate      | `deadpool/serde`, `serde/derive` | no      |

## Example with `tokio-amqp` crate

```rust,no_run
use std::sync::Arc;

use deadpool_lapin::{Config, Manager, Pool, Runtime};
use deadpool_lapin::lapin::{
    options::BasicPublishOptions,
    BasicProperties,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cfg = Config::default();
    cfg.url = Some("amqp://127.0.0.1:5672/%2f".into());
    let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
    for _ in 1..10 {
        let mut connection = pool.get().await?;
        let channel = connection.create_channel().await?;
        channel.basic_publish(
            "",
            "hello",
            BasicPublishOptions::default(),
            b"hello from deadpool",
            BasicProperties::default(),
        ).await?;
    }
    Ok(())
}
```

## Example with `config`, `dotenvy` and `tokio-amqp` crate

```rust
use std::sync::Arc;

use deadpool_lapin::Runtime;
use deadpool_lapin::lapin::{
    options::BasicPublishOptions,
    BasicProperties,
};
use dotenvy::dotenv;

#[derive(Debug, serde::Deserialize)]
struct Config {
    #[serde(default)]
    amqp: deadpool_lapin::Config
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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let mut cfg = Config::from_env().unwrap();
    let pool = cfg.amqp.create_pool(Some(Runtime::Tokio1)).unwrap();
    for _ in 1..10 {
        let mut connection = pool.get().await?;
        let channel = connection.create_channel().await?;
        channel.basic_publish(
            "",
            "hello",
            BasicPublishOptions::default(),
            b"hello from deadpool",
            BasicProperties::default(),
        ).await?;
    }
    Ok(())
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

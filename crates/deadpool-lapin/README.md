# Deadpool for Lapin [![Latest Version](https://img.shields.io/crates/v/deadpool-lapin.svg)](https://crates.io/crates/deadpool-lapin) [![Build Status](https://img.shields.io/github/actions/workflow/status/deadpool-rs/deadpool/deadpool-lapin.yml?branch=main)](https://github.com/deadpool-rs/deadpool/actions/workflows/deadpool-lapin.yml?query=branch%3Amain) ![Unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg "Unsafe forbidden") [![Rust 1.85+](https://img.shields.io/badge/rustc-1.85+-lightgray.svg "Rust 1.85+")](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/)

Deadpool is a dead simple async pool for connections and objects
of any type.

This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
manager for [`lapin`](https://crates.io/crates/lapin).

## Features

| Feature          | Description                                                              | Extra dependencies               | Default |
| ---------------- | ------------------------------------------------------------------------ | -------------------------------- | ------- |
| `rt_tokio_1`     | Enable support for [tokio](https://crates.io/crates/tokio) crate         | `deadpool/rt_tokio_1`            | yes     |
| `rt_smol_2`      | Enable support for [smol](https://crates.io/crates/smol) crate           | `deadpool/rt_smol_2`             | no      |
| `serde`          | Enable support for [serde](https://crates.io/crates/serde) crate         | `deadpool/serde`, `serde/derive` | no      |

## Example with `tokio-amqp` crate

```rust,no_run
use std::sync::Arc;

use deadpool_lapin::{Config, Manager, Pool, Runtime};
use deadpool_lapin::lapin::{
    options::BasicPublishOptions,
    ConnectionProperties,
    BasicProperties,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cfg = Config::default();
    cfg.url = Some("amqp://127.0.0.1:5672/%2f".into());
    let pool = cfg.create_pool(ConnectionProperties::default, Runtime::Tokio1)?;
    for _ in 1..10 {
        let mut connection = pool.get().await?;
        let channel = connection.create_channel().await?;
        channel.basic_publish(
            "".into(),
            "hello".into(),
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
    ConnectionProperties,
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
    let pool = cfg
        .amqp
        .create_pool(ConnectionProperties::default, Runtime::Tokio1)
        .unwrap();
    for _ in 1..10 {
        let mut connection = pool.get().await?;
        let channel = connection.create_channel().await?;
        channel.basic_publish(
            "".into(),
            "hello".into(),
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

# Deadpool for libsql [![Latest Version](https://img.shields.io/crates/v/deadpool-libsql.svg)](https://crates.io/crates/deadpool-libsql) ![Unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg "Unsafe forbidden") [![Rust 1.81+](https://img.shields.io/badge/rustc-1.81+-lightgray.svg "Rust 1.81+")](https://blog.rust-lang.org/2023/12/28/Rust-1.81.0.html)

Deadpool is a dead simple async pool for connections and objects
of any type.

This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
manager for [`libsql`](https://crates.io/crates/libsql).

## Features

| Feature          | Description                                                           | Extra dependencies               | Default |
| ---------------- | --------------------------------------------------------------------- | -------------------------------- | ------- |
| `rt_tokio_1`     | Enable support for [tokio](https://crates.io/crates/tokio) crate      | `deadpool/rt_tokio_1`            | yes     |
| `rt_async-std_1` | Enable support for [async-std](https://crates.io/crates/async-std) crate | `deadpool/rt_async-std_1`        | no      |
| `serde`          | Enable support for [serde](https://crates.io/crates/serde) crate      | `deadpool/serde`, `serde/derive` | no      |

All of the features of [libsql](https://crates.io/crates/libsql) are also re-exported.
For example, the feature `core` does enable the feature `core` from the `libsql` crate.

## Example

```rust
use std::sync::Arc;

use deadpool_libsql::{Manager, Pool};
use deadpool_libsql::libsql::{Builder, params};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = deadpool_libsql::libsql::Builder::new_local("libsql.db")
        .build()
        .await?;

    let manager = Manager::from_libsql_database(db);
    let pool = Pool::builder(manager).build()?;

    let conn = pool.get().await?;
    let mut rows = conn.query("SELECT 1", params![]).await?;
    let row = rows.next().await?.unwrap();
    let result: i64 = row.get(0)?;

    Ok(())
}
```

## Example with `config` and `dotenvy` crate

```env
# .env
LIBSQL__DATABASE=Local
LIBSQL__PATH=deadpool.db
```

```rust,ignore
use deadpool_libsql::{libsql::params, Runtime};
use dotenvy::dotenv;

#[derive(Debug, serde::Deserialize)]
struct Config {
    libsql: deadpool_libsql::Config,
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
    let cfg = Config::from_env().expect("Invalid config");
    let pool = cfg.libsql.create_pool(Some(Runtime::Tokio1)).await?;
    for i in 1..10i64 {
        let conn = pool.get().await?;
        let mut rows = conn.query("SELECT 1 + $1", params![i]).await?;
        let row = rows.next().await?.unwrap();
        let value: i64 = row.get(0)?;
        assert_eq!(value, i + 1);
    }
    Ok(())
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

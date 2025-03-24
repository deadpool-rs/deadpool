# Deadpool for R2D2 Managers [![Latest Version](https://img.shields.io/crates/v/deadpool-r2d2.svg)](https://crates.io/crates/deadpool-r2d2) ![Unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg "Unsafe forbidden") [![Rust 1.75+](https://img.shields.io/badge/rustc-1.75+-lightgray.svg "Rust 1.75+")](https://blog.rust-lang.org/2023/12/28/Rust-1.75.0.html)

Deadpool is a dead simple async pool for connections and objects
of any type.

This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
manager for [`r2d2`](https://crates.io/crates/r2d2) managers.

## Features

| Feature | Description | Extra dependencies | Default |
| ------- | ----------- | ------------------ | ------- |
| `rt_tokio_1` | Enable support for [tokio](https://crates.io/crates/tokio) crate | `deadpool/rt_tokio_1` | yes |
| `rt_async-std_1` | Enable support for [async-std](https://crates.io/crates/async-std) crate | `deadpool/rt_async-std_1` | no |
| `serde` | Enable support for [serde](https://crates.io/crates/serde) crate | `deadpool/serde` | no |
| `tracing` | Enable support for [tracing](https://github.com/tokio-rs/tracing) by propagating Spans in the `interact()` calls. Enable this if you use the `tracing` crate and you want to get useful traces from within `interact()` calls. | `deadpool-sync/tracing`, `tracing` | no |

## Example

```rust
use std::env;

use deadpool_r2d2::Runtime;
use r2d2_postgres::postgres::Error as PgError;

type PgManager = deadpool_r2d2::Manager<
    r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>,
>;
type PgPool = deadpool_r2d2::Pool<PgManager>;

fn create_pool(max_size: usize) -> PgPool {
    let mut pg_config = r2d2_postgres::postgres::Config::new();
    pg_config.host(&env::var("PG__HOST").unwrap());
    pg_config.user(&env::var("PG__USER").unwrap());
    pg_config.password(&env::var("PG__PASSWORD").unwrap());
    pg_config.dbname(&env::var("PG__DBNAME").unwrap());
    let r2d2_manager =
        r2d2_postgres::PostgresConnectionManager::new(pg_config, r2d2_postgres::postgres::NoTls);
    let manager = PgManager::new(r2d2_manager, Runtime::Tokio1);
    let pool = PgPool::builder(manager)
        .max_size(max_size)
        .build()
        .unwrap();
    pool
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_pool(2);
    let client = pool.get().await.unwrap();
    let answer: i32 = client
        .interact(|client| client.query_one("SELECT 42", &[]).map(|row| row.get(0)))
        .await??;
    assert_eq!(answer, 42);
    Ok(())
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

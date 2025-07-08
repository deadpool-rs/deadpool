# Deadpool for Diesel [![Latest Version](https://img.shields.io/crates/v/deadpool-diesel.svg)](https://crates.io/crates/deadpool-diesel) ![Unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg "Unsafe forbidden") [![Rust 1.75+](https://img.shields.io/badge/rustc-1.75+-lightgray.svg "Rust 1.75+")](https://blog.rust-lang.org/2023/12/28/Rust-1.75.0.html)

Deadpool is a dead simple async pool for connections and objects
of any type.

This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
manager for [`diesel`](https://crates.io/crates/diesel) connections.

## Features

| Feature | Description | Extra dependencies | Default |
| ------- | ----------- | ------------------ | ------- |
| `sqlite` | Enable `sqlite` feature in `diesel` crate | `diesel/sqlite` | no |
| `postgres` | Enable `postgres` feature in `diesel` crate | `diesel/postgres` | no |
| `mysql` | Enable `mysql` feature in `diesel` crate | `diesel/mysql` | no |
| `rt_tokio_1` | Enable support for [tokio](https://crates.io/crates/tokio) crate | `deadpool/rt_tokio_1` | yes |
| `rt_async-std_1` | Enable support for [async-std](https://crates.io/crates/async-std) crate | `deadpool/rt_async-std_1` | no |
| `serde` | Enable support for [serde](https://crates.io/crates/serde) crate | `deadpool/serde` | no |
| `tracing` | Enable support for [tracing](https://github.com/tokio-rs/tracing) by propagating Spans in the `interact()` calls. Enable this if you use the `tracing` crate and you want to get useful traces from within `interact()` calls. | `deadpool-sync/tracing`, `tracing` | no |

## Example

```rust
use deadpool_diesel::sqlite::{Runtime, Manager, Pool};
use diesel::{prelude::*, select, sql_types::Text};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = Manager::new(":memory:", Runtime::Tokio1);
    let pool = Pool::builder(manager)
        .max_size(8)
        .build()
        .unwrap();
    let conn = pool.get().await?;
    let result = conn.interact(|conn| {
        let query = select("Hello world!".into_sql::<Text>());
        query.get_result::<String>(conn)
    }).await??;
    assert!(result == "Hello world!");
    Ok(())
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

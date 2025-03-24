# Deadpool for SQLite [![Latest Version](https://img.shields.io/crates/v/deadpool-sqlite.svg)](https://crates.io/crates/deadpool-sqlite) ![Unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg "Unsafe forbidden") [![Rust 1.75+](https://img.shields.io/badge/rustc-1.75+-lightgray.svg "Rust 1.75+")](https://blog.rust-lang.org/2023/12/28/Rust-1.75.0.html)

Deadpool is a dead simple async pool for connections and objects
of any type.

This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
manager for [`rusqlite`](https://crates.io/crates/rusqlite)
and provides a wrapper that ensures correct use of the connection
inside a separate thread.

## Features

| Feature | Description | Extra dependencies | Default |
| ------- | ----------- | ------------------ | ------- |
| `rt_tokio_1` | Enable support for [tokio](https://crates.io/crates/tokio) crate | `deadpool/rt_tokio_1` | yes |
| `rt_async-std_1` | Enable support for [async-std](https://crates.io/crates/async-std) crate | `deadpool/rt_async-std_1` | no |
| `serde` | Enable support for [serde](https://crates.io/crates/serde) crate | `deadpool/serde`, `serde/derive` | no |
| `tracing` | Enable support for [tracing](https://github.com/tokio-rs/tracing) by propagating Spans in the `interact()` calls. Enable this if you use the `tracing` crate and you want to get useful traces from within `interact()` calls. | `deadpool-sync/tracing`, `tracing` | no |

## Example

```rust
use deadpool_sqlite::{Config, Runtime};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cfg = Config::new("db.sqlite3");
    let pool = cfg.create_pool(Runtime::Tokio1).unwrap();
    let conn = pool.get().await.unwrap();
    let result: i64 = conn
        .interact(|conn| {
            let mut stmt = conn.prepare("SELECT 1")?;
            let mut rows = stmt.query([])?;
            let row = rows.next()?.unwrap();
            row.get(0)
        })
        .await??;
    assert_eq!(result, 1);
    Ok(())
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

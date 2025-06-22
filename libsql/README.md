# Deadpool for Lapin [![Latest Version](https://img.shields.io/crates/v/deadpool-lapin.svg)](https://crates.io/crates/deadpool-lapin) ![Unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg "Unsafe forbidden") [![Rust 1.75+](https://img.shields.io/badge/rustc-1.75+-lightgray.svg "Rust 1.75+")](https://blog.rust-lang.org/2023/12/28/Rust-1.75.0.html)

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

    let manager = Manager::new(db);
    let pool = Pool::builder(manager).build()?;

    let conn = pool.get().await?;
    let mut rows = conn.query("SELECT 1", params![]).await?;
    let row = rows.next().await?.unwrap();
    let result: i64 = row.get(0)?;

    Ok(())
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

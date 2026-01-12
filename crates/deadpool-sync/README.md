# Deadpool for synchroneous code [![Latest Version](https://img.shields.io/crates/v/deadpool-sync.svg)](https://crates.io/crates/deadpool-sync) [![Build Status](https://img.shields.io/github/actions/workflow/status/deadpool-rs/deadpool/deadpool-sync.yml?branch=main)](https://github.com/deadpool-rs/deadpool/actions/workflows/deadpool-sync.yml?query=branch%3Amain) ![Unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg "Unsafe forbidden") [![Rust 1.85+](https://img.shields.io/badge/rustc-1.85+-lightgray.svg "Rust 1.85+")](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/)

Deadpool is a dead simple async pool for connections and objects
of any type.

This crates provides helpers for writing pools for objects that don't
support async and need to be run inside a thread.

**Note:** This crate is intended for making the development of
`deadpool-*` crates easier. Other libraries and binary projects
normally should not use this directly and use some provided
reexports by the crates using it.

## Features

| Feature   | Description                                                                                                                                                                                                                    | Extra dependencies | Default |
| --------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------ | ------- |
| `tracing` | Enable support for [tracing](https://github.com/tokio-rs/tracing) by propagating Spans in the `interact()` calls. Enable this if you use the `tracing` crate and you want to get useful traces from within `interact()` calls. | `tracing`          | no      |

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0)>
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT)>

at your option.

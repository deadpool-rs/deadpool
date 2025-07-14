# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased]

- Bump up MSRV to `1.82`
- Re-export `rusqlite` feature: `column_metadata`

## [0.12.1] - 2025-07-09

- Fix `docs.rs` documentation compilation

## [0.12.0] - 2025-07-08

- Update `rusqlite` dependency to version `0.37`

## [0.11.0] - 2025-05-19

- Re-export `rusqlite` features
- Update `rusqlite` dependency to version `0.35`

## [0.10.0] - 2025-02-12

- Update `rusqlite` dependency to version `0.33`
- Bump up MSRV to `1.80` to match the one of `rusqlite`

## [0.9.0] - 2024-10-24

- Update `rusqlite` dependency to version `0.32.1`
- Bump up MSRV to `1.77` to match the one of `rusqlite`

## [0.8.1] - 2024-05-04

- Update `deadpool` dependency to version `0.12`
- Add `LICENSE-APACHE` and `LICENSE-MIT` files to published crates

## [0.8.0] - 2024-04-01

- Update `rusqlite` dependency to version `0.31`
- Remove `async_trait` dependency

## [0.7.0] - 2023-11-14

- Update `rusqlite` dependency to version `0.30`

## [0.6.0] - 2023-09-26

- Update `deadpool` dependency to version `0.10`
- Update `rusqlite` dependency to version `0.29`
- Add `tracing` feature
- Bump up MSRV to `1.63` to match the one of `tokio`

## [0.5.0] - 2022-07-18

- Update `rusqlite` dependency to version `0.28`

## [0.4.0] - 2022-04-04

- Update `rusqlite` dependency to version `0.27`

## [0.3.1] - 2021-11-15

- Config structs now implement `Serialize`

## [0.3.0] - 2021-10-26

- __Breaking:__ Replace `deadpool::managed::sync` by
  `deadpool-sync::SyncWrapper` which fixes the return type
  of the `interact` method.

## [0.2.0] - 2021-10-18

- __Breaking:__ Replace `config` feature with `serde` (opted out by default)
- Add support for multiple runtimes
- Fix panic handling inside the `interact` method
- Wrap blocking drop method of connections inside `spawn_blocking`
- Remove unused `futures`, `tokio` and `log` dependencies

## [0.1.0] - 2021-05-30

- First release

<!-- next-url -->
[Unreleased]: https://github.com/bikeshedder/deadpool/compare/deadpool-sqlite-v0.12.1...HEAD
[0.12.1]: https://github.com/bikeshedder/deadpool/compare/deadpool-sqlite-v0.12.0...deadpool-sqlite-v0.12.1
[0.12.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-sqlite-v0.11.0...deadpool-sqlite-v0.12.0
[0.11.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-sqlite-v0.10.0...deadpool-sqlite-v0.11.0
[0.10.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-sqlite-v0.9.0...deadpool-sqlite-v0.10.0
[0.9.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-sqlite-v0.8.1...deadpool-sqlite-v0.9.0
[0.8.1]: https://github.com/bikeshedder/deadpool/compare/deadpool-sqlite-v0.8.0...deadpool-sqlite-v0.8.1
[0.8.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-sqlite-v0.7.0...deadpool-sqlite-v0.8.0
[0.7.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-sqlite-v0.6.0...deadpool-sqlite-v0.7.0
[0.6.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-sqlite-v0.5.0...deadpool-sqlite-v0.6.0
[0.5.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-sqlite-v0.4.0...deadpool-sqlite-v0.5.0
[0.4.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-sqlite-v0.3.0...deadpool-sqlite-v0.4.0
[0.3.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-sqlite-v0.2.0...deadpool-sqlite-v0.3.0
[0.2.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-sqlite-v0.1.0...deadpool-sqlite-v0.2.0
[0.1.0]: https://github.com/bikeshedder/deadpool/releases/tag/deadpool-sqlite-v0.1.0

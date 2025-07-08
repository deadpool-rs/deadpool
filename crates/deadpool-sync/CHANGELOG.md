# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased]

## [0.1.4] - 2024-06-04

- Fix `panic` when dropping a `SyncWrapper` while it is still executing the `interact` method.

## [0.1.3] - 2024-05-24

- Add `LICENSE-APACHE` and `LICENSE-MIT` files to published crates

## [0.1.2] - 2023-09-26

- Add `tracing` feature to `README`
- Fix MSRV (this was the reason for yanking the `0.1.1` release) and bump it up to `1.63` to match the one of `tokio`

## [0.1.1] - 2023-09-06 (yanked)

- Replace `deadpool` dependency by `deadpool-runtime`. This is a
  non-breaking change as the `Runtime` and `SpawnBlockingError`
  types were a re-export from `deadpool-runtime` anyways.

## [0.1.0] - 2021-10-26

- First release

<!-- next-url -->
[Unreleased]: https://github.com/bikeshedder/deadpool/compare/deadpool-sync-v0.1.4...HEAD
[0.1.4]: https://github.com/bikeshedder/deadpool/compare/deadpool-sync-v0.1.3...deadpool-sync-v0.1.4
[0.1.3]: https://github.com/bikeshedder/deadpool/compare/deadpool-sync-v0.1.2...deadpool-sync-v0.1.3
[0.1.2]: https://github.com/bikeshedder/deadpool/compare/deadpool-sync-v0.1.1...deadpool-sync-v0.1.2
[0.1.1]: https://github.com/bikeshedder/deadpool/compare/deadpool-sync-v0.1.0...deadpool-sync-v0.1.1
[0.1.0]: https://github.com/bikeshedder/deadpool/releases/tag/deadpool-sync-v0.1.0
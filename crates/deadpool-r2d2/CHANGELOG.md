# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased]

## [0.4.1] - 2024-05-04

- Update `deadpool` dependency to version `0.12`
- Add `LICENSE-APACHE` and `LICENSE-MIT` files to published crates

## [0.4.0] - 2024-04-01

- Update `deadpool` dependency to version `0.11`
- Remove `async_trait` dependency
- Bump up MSRV to `1.75`

## [0.3.0] - 2023-09-26

- Update `deadpool` dependency to version `0.10`
- Add `tracing` feature
- Bump up MSRV to `1.63` to match the one of `tokio`

## [0.2.0] - 2021-10-26

- __Breaking:__ Replace `deadpool::managed::sync` by
  `deadpool-sync::SyncWrapper` which fixes the return type
  of the `interact` method.

## [0.1.0] - 2021-10-18

- First release

<!-- next-url -->
[Unreleased]: https://github.com/bikeshedder/deadpool/compare/deadpool-r2d2-v0.4.1...HEAD
[0.4.1]: https://github.com/bikeshedder/deadpool/compare/deadpool-r2d2-v0.4.0...deadpool-r2d2-v0.4.1
[0.4.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-r2d2-v0.3.0...deadpool-r2d2-v0.4.0
[0.3.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-r2d2-v0.2.0...deadpool-r2d2-v0.3.0
[0.2.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-r2d2-v0.1.0...deadpool-r2d2-v0.2.0
[0.1.0]: https://github.com/bikeshedder/deadpool/releases/tag/deadpool-r2d2-v0.1.0

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased]

- Fix deprecation warning introduced in diesel `2.2.0`
- Update `diesel` dependency to version `2.2.0`
- Bump up MSRV to `1.84`
- Re-export `diesel` features
- Mimick `default` features of `diesel` crate

## [0.6.1] - 2024-05-04

- Update `deadpool` dependency to version `0.12`

## [0.6.0] - 2024-04-01

- Update `deadpool` dependency to version `0.11`
- Remove `async_trait` dependency
- Bump up MSRV to `1.75`

## [0.5.0] - 2023-09-26

- Update `deadpool` dependency to version `0.10`
- Add `tracing` feature
- Check for open transactions before recycling connections
- Allow to configure a custom recycle check function to customize ping queries for different database backends
- Bump up MSRV to `1.63` to match the one of `tokio`

## [0.4.1] - 2022-12-09

- Fix error handling when recycling connections

## [0.4.0] - 2022-09-12

- Update `diesel` dependency to version `2.0.0`

## [0.3.1] - 2021-11-03

- Add missing reexports from the `mysql` and `postgres` modules.

## [0.3.0] - 2021-10-26

- __Breaking:__ Replace `deadpool::managed::sync` by
  `deadpool-sync::SyncWrapper` which fixes the return type
  of the `interact` method.

## [0.2.0] - 2021-07-18

- __Breaking:__ Replace `config` feature with `serde` (opted out by default)
- Fix `std::error::Error::source` implementations for library errors
- Remove unused `tokio` dependency
- Async unaware diesel connections are now wrapped inside
  a `deadpool::managed::sync::SyncWrapper` which ensures that
  all database operations are run in a separate threads.

## [0.1.2] - 2021-07-16

- Remove `unsafe impl` by better usage of `PhantomData`

## [0.1.1] - 2021-07-14

- Fix title and crates.io badge in README

## [0.1.0] - 2021-07-14

- First release

<!-- next-url -->
[Unreleased]: https://github.com/bikeshedder/deadpool/compare/deadpool-diesel-v0.6.1...HEAD
[0.6.1]: https://github.com/bikeshedder/deadpool/compare/deadpool-diesel-v0.6.0...deadpool-diesel-v0.6.1
[0.6.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-diesel-v0.5.0...deadpool-diesel-v0.6.0
[0.5.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-diesel-v0.4.1...deadpool-diesel-v0.5.0
[0.4.1]: https://github.com/bikeshedder/deadpool/compare/deadpool-diesel-v0.4.0...deadpool-diesel-v0.4.1
[0.4.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-diesel-v0.3.1...deadpool-diesel-v0.4.0
[0.3.1]: https://github.com/bikeshedder/deadpool/compare/deadpool-diesel-v0.3.0...deadpool-diesel-v0.3.1
[0.3.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-diesel-v0.2.0...deadpool-diesel-v0.3.0
[0.2.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-diesel-v0.1.2...deadpool-diesel-v0.2.0
[0.1.2]: https://github.com/bikeshedder/deadpool/compare/deadpool-diesel-v0.1.1...deadpool-diesel-v0.1.2
[0.1.1]: https://github.com/bikeshedder/deadpool/compare/deadpool-diesel-v0.1.0...deadpool-diesel-v0.1.1
[0.1.0]: https://github.com/bikeshedder/deadpool/releases/tag/deadpool-diesel-v0.1.0

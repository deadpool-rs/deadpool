# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased]

- Re-export `lapin` features:
    - `default-runtime`
    - `rustls--aws_lc_rs`
    - `rustls--ring`
    - `unstable`

## [0.13.0] - 2024-07-07

- Re-export `lapin` features
- Update `lapin` dependency to version `3`

## [0.12.1] - 2024-05-04

- Update `deadpool` dependency to version `0.12`
- Add `LICENSE-APACHE` and `LICENSE-MIT` files to published crates

## [0.12.0] - 2024-04-01

- Update `deadpool` dependency to version `0.11`
- Remove `async_trait` dependency
- Bump up MSRV to `1.75`

## [0.11.0] - 2023-09-26

- Update `deadpool` dependency to version `0.10`
- Bump up MSRV to `1.63` to match the one of `tokio`

## [0.10.0] - 2022-02-22

- Update `lapin` dependency to version `2.0`

## [0.9.1] - 2021-11-15

- Config structs now implement `Serialize`

## [0.9.0] - 2021-10-18

- __Breaking:__ Replace `config` feature with `serde` (opted out by default)
- Remove unused `futures` and `log` dependencies
- Update `deadpool` to `0.9`

## [0.8.0] - 2021-05-21

- Update `config` dependency to version `0.11`
- Remove deprecated `from_env` methods
- Add `rt_tokio_1` and `rt_async-std_1` features

## [0.7.0] - 2020-12-26

- Update `tokio` dependency to version `1`
- Update `tokio-amqp` dependency to version `1`
- Mark `Config::from_env` as deprecated
- Re-export `deadpool::managed::PoolConfig`

## [0.6.2] - 2020-11-04

- Add support for `ConnectionProperties`

## [0.6.1] - 2020-11-04

- Add support for `deadpool 0.5` (`tokio 0.2`) and `deadpool 0.6` (`tokio 0.3`)

## [0.6.0] - 2020-07-14

- Update `lapin` dependency to version 1.0.0
- Re-export for `lapin` crate

## [0.5.0] - 2020-03-13

- First release

<!-- next-url -->
[Unreleased]: https://github.com/bikeshedder/deadpool/compare/deadpool-lapin-v0.13.0...HEAD
[0.13.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-lapin-v0.12.1...deadpool-lapin-v0.13.0
[0.12.1]: https://github.com/bikeshedder/deadpool/compare/deadpool-lapin-v0.12.0...deadpool-lapin-v0.12.1
[0.12.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-lapin-v0.11.0...deadpool-lapin-v0.12.0
[0.11.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-lapin-v0.10.0...deadpool-lapin-v0.11.0
[0.10.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-lapin-v0.9.1...deadpool-lapin-v0.10.0
[0.9.1]: https://github.com/bikeshedder/deadpool/compare/deadpool-lapin-v0.9.0...deadpool-lapin-v0.9.1
[0.9.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-lapin-v0.8.0...deadpool-lapin-v0.9.0
[0.8.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-lapin-v0.7.0...deadpool-lapin-v0.8.0
[0.7.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-lapin-v0.6.2...deadpool-lapin-v0.7.0
[0.6.2]: https://github.com/bikeshedder/deadpool/compare/deadpool-lapin-v0.6.1...deadpool-lapin-v0.6.2
[0.6.1]: https://github.com/bikeshedder/deadpool/compare/deadpool-lapin-v0.6.0...deadpool-lapin-v0.6.1
[0.6.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-lapin-v0.5.0...deadpool-lapin-v0.6.0
[0.5.0]: https://github.com/bikeshedder/deadpool/releases/tag/deadpool-lapin-v0.5.0

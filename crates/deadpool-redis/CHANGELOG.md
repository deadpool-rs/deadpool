# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased]

- Bump up MSRV to `1.82`
- Re-export `redis` features:
  - `safe_iterators`
  - `smol-comp`
  - `smol-native-tls-comp`
  - `smol-rustls-comp`
  - `vector-sets`

## [0.22.0] - 2025-07-07

- Update `redis` dependency to version `0.32`

## [0.21.1] - 2025-05-20

- Fix `sentinel` feature when `serde` feature is not enabled

## [0.21.0] - 2025-05-19

- Update `redis` dependency to version `0.31`

## [0.20.0] - 2025-02-27

- Re-export `redis` features
- Update `redis` dependency to version `0.29`

## [0.19.0] - 2024-02-12

- Update `redis` dependency to version `0.28`

## [0.18.0] - 2024-09-20

- Update `redis` dependency to version `0.27`

## [0.17.2] - 2024-09-19

- Fix optional `serde` dependency

## [0.17.1] - 2024-09-18

- Export missing config structure:
  - `ProtocolVersion`

## [0.17.0] - 2024-09-09

- **Breaking:** Add `read_from_replicas` field to Redis cluster `Config` struct
- Add support for `redis::sentinel`

## [0.16.0] - 2024-08-05

- Update `redis` dependency to version `0.26`

## [0.15.1] - 2024-05-04

- Update `deadpool` dependency to version `0.12`
- Add `LICENSE-APACHE` and `LICENSE-MIT` files to published crates

## [0.15.0] - 2024-04-01

- Update `redis` dependency to version `0.25`
- Update `deadpool` dependency to version `0.11`
- Remove `async_trait` dependency
- Bump up MSRV to `1.75`

## [0.14.0] - 2023-12-15

- Merge `deadpool-redis-cluster` into `deadpool-redis`.
- Remove `redis_cluster_async` dependency in favor of `redis::cluster` / `redis::cluster_async`.
- Update `redis` dependency to version `0.24`
- Bump up MSRV to `1.63` to match the one of `redis`

## [0.13.0] - 2023-09-26

- Update `deadpool` dependency to version `0.10`
- Bump up MSRV to `1.63` to match the one of `tokio`

## [0.12.0] - 2023-04-24

- Update `redis` dependency to version `0.23`

## [0.11.1] - 2022-12-09

- Export missing config structures:
  - `ConnectionAddr`
  - `ConnectionInfo`
  - `RedisConnectionInfo`
- Add `Config::from_connection_info` method

## [0.11.0] - 2022-10-26

- Update `redis` dependency to version `0.22`

## [0.10.2] - 2022-01-03

- Fix bug causing connections not to be recycled

## [0.10.1] - 2021-11-15

- Config structs now implement `Serialize`

## [0.10.0] - 2021-10-18

- **Breaking:** Replace `config` feature with `serde` (opted out by default)
- Fix `std::error::Error::source` implementations for library errors
- Remove unused `log` dependency

## [0.9.0] - 2021-08-19

- Remove `deadpool_redis::Connection` type alias
- Rename `deadpool_redis::ConnectionWrapper` to `Connection`
- Update `redis` dependency to version `0.21`
- Add `Config::from_url` method

## [0.8.1] - 2021-06-01

- Export `PoolCreateError`

## [0.6.2] and [0.7.2] - 2021-06-01

Release of 0.6 and 0.7 with the following feature backported:

- Make connection recycling more robust by checking the PING
  response. This works around `Cmd::query_async` not being drop
  safe in `redis` version `0.10` and earlier.

## [0.8.0] - 2021-05-21

- Update `config` dependency to version `0.11`
- Remove deprecated `from_env` methods
- Remove wrappers for `Cdm` and `Pipe`. The pool now returns a
  `ConnectionWrapper` rather than an `Object<ConnectionWrapper>` which
  implements the `redis::aio::ConnectionLike` trait and therefore can
  be used with plain `Cmd` and `Pipe` objects from the `redis` crate.
- Add support for new `redis::ConnectionInfo` structure.
- Change `redis` dependency to version `0.20`
- Make connection recycling more robust by checking the PING
  response. This works around `Cmd::query_async` not being drop
  safe in `redis` version `0.10` and earlier.
- Add `rt_tokio_1` and `rt_async-std_1` features

## [0.7.1] - 2021-02-22

- Change `redis` dependency to version range `0.19` to `0.20`

## [0.7.0] - 2020-12-26

- Update `deadpool` dependency to version `0.3`
- Update `redis` dependency to version `0.19`
- Disable `redis` default features
- Mark `Config::from_env` as deprecated
- Re-export `deadpool::managed::PoolConfig`

## [0.6.1] - 2020-08-27

- Change `redis` dependency to version range `0.15` to `0.17`

## [0.6.0] - 2020-07-14

- Update `redis` dependency to version `0.16.0`
- Re-export `redis` crate

## [0.5.2] - 2020-02-04

- Update `redis` dependency to version `0.15.1`
- Add `#[derive(Clone)]` to `Config` struct
- Add `Connection` type alias

## [0.5.1] - 2020-01-18

- Disable `default-features` for `deadpool` dependency

## [0.5.0] - 2020-01-16

- Update `redis` dependency to version `0.14.0`
- Rename `query` to `query_async` to match API of `redis` crate
- Rename `execute` to `execute_async` to match API of `redis` crate
- Add support for `config` crate

## [0.4.1] - 2019-12-31

- Add `PoolError` type alias

## [0.4.0] - 2019-12-16

- Rename `Connection` to `ConnectionWrapper`
- Add `Connection` type alias

## [0.3.0] - 2019-12-13

- Add pipeline support
- Add wrappers for `Cmd` and `Pipeline` to mimick the API of the `redis` crate
- Make recycling more robust by changing the `Manager::recycle` to a non
  consuming API.
- Add proper connection using the `PING` command

## [0.2.0] - 2019-12-03

- First release

<!-- next-url -->
[Unreleased]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.22.0...HEAD
[0.22.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.21.1...deadpool-redis-v0.22.0
[0.21.1]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.21.0...deadpool-redis-v0.21.1
[0.21.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.20.0...deadpool-redis-v0.21.0
[0.20.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.19.0...deadpool-redis-v0.20.0
[0.19.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.18.0...deadpool-redis-v0.19.0
[0.18.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.17.2...deadpool-redis-v0.18.0
[0.17.2]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.17.1...deadpool-redis-v0.17.2
[0.17.1]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.17.0...deadpool-redis-v0.17.1
[0.17.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.16.0...deadpool-redis-v0.17.0
[0.16.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.15.1...deadpool-redis-v0.16.0
[0.15.1]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.15.0...deadpool-redis-v0.15.1
[0.15.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.14.0...deadpool-redis-v0.15.0
[0.14.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.13.0...deadpool-redis-v0.14.0
[0.13.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.12.0...deadpool-redis-v0.13.0
[0.12.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.11.1...deadpool-redis-v0.12.0
[0.11.1]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.11.0...deadpool-redis-v0.11.1
[0.11.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.10.2...deadpool-redis-v0.11.0
[0.10.2]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.10.1...deadpool-redis-v0.10.2
[0.10.1]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.10.0...deadpool-redis-v0.10.1
[0.10.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.9.0...deadpool-redis-v0.10.0
[0.9.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.8.1...deadpool-redis-v0.9.0
[0.8.1]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.8.0...deadpool-redis-v0.8.1
[0.8.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.7.1...deadpool-redis-v0.8.0
[0.7.2]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.7.1...deadpool-redis-v0.7.2
[0.7.1]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.7.0...deadpool-redis-v0.7.1
[0.7.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.6.1...deadpool-redis-v0.7.0
[0.6.2]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.6.1...deadpool-redis-v0.6.2
[0.6.1]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.6.0...deadpool-redis-v0.6.1
[0.6.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.5.2...deadpool-redis-v0.6.0
[0.5.2]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.5.1...deadpool-redis-v0.5.2
[0.5.1]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.5.0...deadpool-redis-v0.5.1
[0.5.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.4.1...deadpool-redis-v0.5.0
[0.4.1]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.4.0...deadpool-redis-v0.4.1
[0.4.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.3.0...deadpool-redis-v0.4.0
[0.3.0]: https://github.com/bikeshedder/deadpool/compare/deadpool-redis-v0.2.0...deadpool-redis-v0.3.0
[0.2.0]: https://github.com/bikeshedder/deadpool/releases/tag/deadpool-redis-v0.2.0

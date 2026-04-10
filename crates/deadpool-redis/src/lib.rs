#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(
    nonstandard_style,
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links
)]
#![forbid(non_ascii_idents, unsafe_code)]
#![warn(
    deprecated_in_future,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    unused_import_braces,
    unused_labels,
    unused_lifetimes,
    unused_qualifications,
    unused_results
)]
#![allow(clippy::uninlined_format_args)]

#[cfg(feature = "cluster")]
pub mod cluster;
mod config;

#[cfg(feature = "sentinel")]
pub mod sentinel;

use std::{
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

const DEFAULT_CONNECTION_TIMEOUT: Option<Duration> = Some(Duration::from_secs(1));
const DEFAULT_RESPONSE_TIMEOUT: Option<Duration> = Some(Duration::from_millis(500));

use deadpool::managed;
use redis::{
    AsyncConnectionConfig, Client, IntoConnectionInfo, RedisError, RedisResult,
    aio::{ConnectionLike, MultiplexedConnection},
};

pub use redis;

pub use self::config::{
    Config, ConfigError, ConnectionAddr, ConnectionInfo, ProtocolVersion, RedisConnectionInfo,
};

pub use deadpool::managed::reexports::*;
deadpool::managed_reexports!("redis", Manager, Connection, RedisError, ConfigError);

/// Type alias for using [`deadpool::managed::RecycleResult`] with [`redis`].
type RecycleResult = managed::RecycleResult<RedisError>;

/// Wrapper around [`redis::aio::MultiplexedConnection`].
///
/// This structure implements [`redis::aio::ConnectionLike`] and can therefore
/// be used just like a regular [`redis::aio::MultiplexedConnection`].
#[allow(missing_debug_implementations)] // `redis::aio::MultiplexedConnection: !Debug`
pub struct Connection {
    conn: Object,
}

impl Connection {
    /// Takes this [`Connection`] from its [`Pool`] permanently.
    ///
    /// This reduces the size of the [`Pool`].
    #[must_use]
    pub fn take(this: Self) -> MultiplexedConnection {
        Object::take(this.conn)
    }
}

impl From<Object> for Connection {
    fn from(conn: Object) -> Self {
        Self { conn }
    }
}

impl Deref for Connection {
    type Target = MultiplexedConnection;

    fn deref(&self) -> &MultiplexedConnection {
        &self.conn
    }
}

impl DerefMut for Connection {
    fn deref_mut(&mut self) -> &mut MultiplexedConnection {
        &mut self.conn
    }
}

impl AsRef<MultiplexedConnection> for Connection {
    fn as_ref(&self) -> &MultiplexedConnection {
        &self.conn
    }
}

impl AsMut<MultiplexedConnection> for Connection {
    fn as_mut(&mut self) -> &mut MultiplexedConnection {
        &mut self.conn
    }
}

impl ConnectionLike for Connection {
    fn req_packed_command<'a>(
        &'a mut self,
        cmd: &'a redis::Cmd,
    ) -> redis::RedisFuture<'a, redis::Value> {
        self.conn.req_packed_command(cmd)
    }

    fn req_packed_commands<'a>(
        &'a mut self,
        cmd: &'a redis::Pipeline,
        offset: usize,
        count: usize,
    ) -> redis::RedisFuture<'a, Vec<redis::Value>> {
        self.conn.req_packed_commands(cmd, offset, count)
    }

    fn get_db(&self) -> i64 {
        self.conn.get_db()
    }
}

/// [`Manager`] for creating and recycling [`redis`] connections.
///
/// [`Manager`]: managed::Manager
pub struct Manager {
    client: Client,
    connection_config: Option<AsyncConnectionConfig>,
    ping_number: AtomicUsize,
}

impl std::fmt::Debug for Manager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Manager")
            .field("client", &self.client)
            .field("ping_number", &self.ping_number)
            .finish()
    }
}

/// Connection configuration for the [`Manager`].
///
/// # Example
///
/// ```rust
/// use std::time::Duration;
/// use deadpool_redis::{ManagerConfig, Manager};
///
/// let manager = Manager::new_with_config(
///     "redis://127.0.0.1",
///     ManagerConfig {
///         connection_timeout: Some(Duration::from_secs(5)),
///         response_timeout: None,
///     },
/// )
/// .unwrap();
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ManagerConfig {
    /// Timeout for establishing a connection.
    ///
    /// Set to `None` to disable. Defaults to 1 second.
    pub connection_timeout: Option<Duration>,

    /// Timeout for waiting for a response.
    ///
    /// Set to `None` to disable. Defaults to 500 milliseconds.
    pub response_timeout: Option<Duration>,
}

impl Default for ManagerConfig {
    fn default() -> Self {
        Self {
            connection_timeout: DEFAULT_CONNECTION_TIMEOUT,
            response_timeout: DEFAULT_RESPONSE_TIMEOUT,
        }
    }
}

impl Manager {
    /// Creates a new [`Manager`] from the given `params`.
    ///
    /// # Errors
    ///
    /// If establishing a new [`Client`] fails.
    pub fn new<T: IntoConnectionInfo>(params: T) -> RedisResult<Self> {
        Ok(Self {
            client: Client::open(params)?,
            connection_config: None,
            ping_number: AtomicUsize::new(0),
        })
    }

    /// Creates a new [`Manager`] from the given `params` and
    /// [`ManagerConfig`].
    ///
    /// # Errors
    ///
    /// If establishing a new [`Client`] fails.
    pub fn new_with_config<T: IntoConnectionInfo>(
        params: T,
        config: ManagerConfig,
    ) -> RedisResult<Self> {
        let connection_config = AsyncConnectionConfig::new()
            .set_connection_timeout(config.connection_timeout)
            .set_response_timeout(config.response_timeout);

        Ok(Self {
            client: Client::open(params)?,
            connection_config: Some(connection_config),
            ping_number: AtomicUsize::new(0),
        })
    }
}

impl managed::Manager for Manager {
    type Type = MultiplexedConnection;
    type Error = RedisError;

    async fn create(&self) -> Result<MultiplexedConnection, RedisError> {
        let conn = match &self.connection_config {
            Some(config) => {
                self.client
                    .get_multiplexed_async_connection_with_config(config)
                    .await?
            }
            None => self.client.get_multiplexed_async_connection().await?,
        };

        Ok(conn)
    }

    async fn recycle(&self, conn: &mut MultiplexedConnection, _: &Metrics) -> RecycleResult {
        let ping_number = self.ping_number.fetch_add(1, Ordering::Relaxed).to_string();
        // Using pipeline to avoid roundtrip for UNWATCH
        let (n,) = redis::Pipeline::with_capacity(2)
            .cmd("UNWATCH")
            .ignore()
            .cmd("PING")
            .arg(&ping_number)
            .query_async::<(String,)>(conn)
            .await?;
        if n == ping_number {
            Ok(())
        } else {
            Err(managed::RecycleError::message("Invalid PING response"))
        }
    }
}

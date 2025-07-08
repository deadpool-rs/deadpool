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
};

use deadpool::managed;
use redis::{
    aio::{ConnectionLike, MultiplexedConnection},
    Client, IntoConnectionInfo, RedisError, RedisResult,
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
#[derive(Debug)]
pub struct Manager {
    client: Client,
    ping_number: AtomicUsize,
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
            ping_number: AtomicUsize::new(0),
        })
    }
}

impl managed::Manager for Manager {
    type Type = MultiplexedConnection;
    type Error = RedisError;

    async fn create(&self) -> Result<MultiplexedConnection, RedisError> {
        let conn = self.client.get_multiplexed_async_connection().await?;
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

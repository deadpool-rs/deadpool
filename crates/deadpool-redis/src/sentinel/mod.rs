//! This module extends the library to support Redis Cluster.
use std::{
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicUsize, Ordering},
};

use redis;
use redis::aio::MultiplexedConnection;
use redis::sentinel::SentinelClient;
use redis::{aio::ConnectionLike, IntoConnectionInfo, RedisError, RedisResult};
use tokio::sync::Mutex;

use deadpool::managed;
pub use deadpool::managed::reexports::*;

pub use crate::sentinel::config::SentinelNodeConnectionInfo;
pub use crate::sentinel::config::SentinelServerType;
pub use crate::sentinel::config::TlsMode;

pub use self::config::{Config, ConfigError};

mod config;

deadpool::managed_reexports!(
    "redis_sentinel",
    Manager,
    Connection,
    RedisError,
    ConfigError
);

type RecycleResult = managed::RecycleResult<RedisError>;

/// Wrapper around [`redis::aio::MultiplexedConnection`].
///
/// This structure implements [`redis::aio::ConnectionLike`] and can therefore
/// be used just like a regular [`redis::aio::MultiplexedConnection`].
#[allow(missing_debug_implementations)] // `redis::cluster_async::ClusterConnection: !Debug`
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

/// [`Manager`] for creating and recycling [`redis::aio::MultiplexedConnection`] connections.
///
/// [`Manager`]: managed::Manager
pub struct Manager {
    client: Mutex<SentinelClient>,
    ping_number: AtomicUsize,
}

impl std::fmt::Debug for Manager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Manager")
            .field("client", &format!("{:p}", &self.client))
            .field("ping_number", &self.ping_number)
            .finish()
    }
}

impl Manager {
    /// Creates a new [`Manager`] from the given `params`.
    ///
    /// # Errors
    ///
    /// If establishing a new [`SentinelClient`] fails.
    pub fn new<T: IntoConnectionInfo>(
        param: Vec<T>,
        service_name: String,
        node_connection_info: Option<SentinelNodeConnectionInfo>,
        server_type: SentinelServerType,
    ) -> RedisResult<Self> {
        Ok(Self {
            client: Mutex::new(SentinelClient::build(
                param,
                service_name,
                node_connection_info.map(|i| i.into()),
                server_type.into(),
            )?),
            ping_number: AtomicUsize::new(0),
        })
    }
}

impl managed::Manager for Manager {
    type Type = MultiplexedConnection;
    type Error = RedisError;

    async fn create(&self) -> Result<MultiplexedConnection, RedisError> {
        let mut client = self.client.lock().await;
        let conn = client.get_async_connection().await?;
        Ok(conn)
    }

    async fn recycle(&self, conn: &mut MultiplexedConnection, _: &Metrics) -> RecycleResult {
        let ping_number = self.ping_number.fetch_add(1, Ordering::Relaxed).to_string();
        let n = redis::cmd("PING")
            .arg(&ping_number)
            .query_async::<String>(conn)
            .await?;
        if n == ping_number {
            Ok(())
        } else {
            Err(managed::RecycleError::message("Invalid PING response"))
        }
    }
}

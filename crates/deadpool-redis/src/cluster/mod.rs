//! This module extends the library to support Redis Cluster.
mod config;

use std::{
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicUsize, Ordering},
};

use deadpool::managed;
use redis::{aio::ConnectionLike, IntoConnectionInfo, RedisError, RedisResult};

use redis;
pub use redis::cluster::{ClusterClient, ClusterClientBuilder};
pub use redis::cluster_async::ClusterConnection;

pub use self::config::{Config, ConfigError};

pub use deadpool::managed::reexports::*;
deadpool::managed_reexports!(
    "redis_cluster",
    Manager,
    Connection,
    RedisError,
    ConfigError
);

type RecycleResult = managed::RecycleResult<RedisError>;

/// Wrapper around [`redis::cluster_async::ClusterConnection`].
///
/// This structure implements [`redis::aio::ConnectionLike`] and can therefore
/// be used just like a regular [`redis::cluster_async::ClusterConnection`].
#[allow(missing_debug_implementations)] // `redis::cluster_async::ClusterConnection: !Debug`
pub struct Connection {
    conn: Object,
}

impl Connection {
    /// Takes this [`Connection`] from its [`Pool`] permanently.
    ///
    /// This reduces the size of the [`Pool`].
    #[must_use]
    pub fn take(this: Self) -> ClusterConnection {
        Object::take(this.conn)
    }
}

impl From<Object> for Connection {
    fn from(conn: Object) -> Self {
        Self { conn }
    }
}

impl Deref for Connection {
    type Target = ClusterConnection;

    fn deref(&self) -> &ClusterConnection {
        &self.conn
    }
}

impl DerefMut for Connection {
    fn deref_mut(&mut self) -> &mut ClusterConnection {
        &mut self.conn
    }
}

impl AsRef<ClusterConnection> for Connection {
    fn as_ref(&self) -> &ClusterConnection {
        &self.conn
    }
}

impl AsMut<ClusterConnection> for Connection {
    fn as_mut(&mut self) -> &mut ClusterConnection {
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

/// [`Manager`] for creating and recycling [`redis::cluster_async`] connections.
///
/// [`Manager`]: managed::Manager
pub struct Manager {
    client: ClusterClient,
    ping_number: AtomicUsize,
}

// `redis::cluster_async::ClusterClient: !Debug`
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
    /// If establishing a new [`ClusterClientBuilder`] fails.
    pub fn new<T: IntoConnectionInfo>(
        params: Vec<T>,
        read_from_replicas: bool,
    ) -> RedisResult<Self> {
        let mut client = ClusterClientBuilder::new(params);
        if read_from_replicas {
            client = client.read_from_replicas();
        }
        Ok(Self {
            client: client.build()?,
            ping_number: AtomicUsize::new(0),
        })
    }
}

impl managed::Manager for Manager {
    type Type = ClusterConnection;
    type Error = RedisError;

    async fn create(&self) -> Result<ClusterConnection, RedisError> {
        let conn = self.client.get_async_connection().await?;
        Ok(conn)
    }

    async fn recycle(&self, conn: &mut ClusterConnection, _: &Metrics) -> RecycleResult {
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

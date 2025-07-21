//! This module contains all the configuration structures

#[cfg(any(feature = "core", feature = "replication", feature = "sync"))]
use std::path::PathBuf;
#[cfg(any(feature = "replication", feature = "sync"))]
use std::time::Duration;

use deadpool::{
    managed::{CreatePoolError, PoolConfig},
    Runtime,
};
use libsql::Builder;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{Manager, Pool, PoolBuilder};

/// Configuration object.
///
/// # Example (from environment)
///
/// By enabling the `serde` feature you can read the configuration using the
/// [`config`](https://crates.io/crates/config) crate as following:
/// ```env
/// LIBSQL__DATABASE=Local
/// LIBSQL__PATH=db.sqlite
/// ```
/// ```rust
/// #[derive(serde::Deserialize, serde::Serialize)]
/// struct Config {
///     libsql: deadpool_libsql::config::Config,
/// }
/// impl Config {
///     pub fn from_env() -> Result<Self, config::ConfigError> {
///         let mut cfg = config::Config::builder()
///            .add_source(config::Environment::default().separator("__"))
///            .build()?;
///            cfg.try_deserialize()
///     }
/// }
/// ```
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Config {
    /// Database configuration.
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub database: Database,
    /// Pool configuration.
    #[cfg_attr(feature = "serde", serde(default))]
    pub pool: PoolConfig,
}

impl Config {
    /// Create a new [`Config`] with the given database
    #[must_use]
    pub fn new(database: Database) -> Self {
        Self {
            database,
            pool: PoolConfig::default(),
        }
    }

    /// Create a new [`Pool`] using this [`Config`].
    ///
    /// # Errors
    ///
    /// See [`CreatePoolError`] for details.
    pub async fn create_pool(
        self,
        runtime: Option<Runtime>,
    ) -> Result<Pool, CreatePoolError<ConfigError>> {
        let mut builder = self.builder().await.map_err(CreatePoolError::Config)?;
        if let Some(runtime) = runtime {
            builder = builder.runtime(runtime);
        }
        builder.build().map_err(CreatePoolError::Build)
    }

    /// Creates a new [`PoolBuilder`] using this [`Config`].
    ///
    /// # Errors
    ///
    /// See [`ConfigError`] for details.
    pub async fn builder(self) -> Result<PoolBuilder, ConfigError> {
        let config = self.pool;
        let manager = Manager::from_config(self).await?;
        Ok(Pool::builder(manager).config(config))
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "database"))]
/// This is a 1:1 mapping of [libsql::Builder] to a (de)serializable
/// config structure
pub enum Database {
    /// See: [libsql::Builder::new_local]
    #[cfg(feature = "core")]
    Local(Local),
    /// See: [libsql::Builder::new_local_replica]
    #[cfg(feature = "replication")]
    LocalReplica(LocalReplica),
    /// See: [libsql::Builder::new_remote]
    #[cfg(feature = "remote")]
    Remote(Remote),
    /// See: [libsql::Builder::new_remote_replica]
    #[cfg(feature = "replication")]
    RemoteReplica(RemoteReplica),
    /// See: [libsql::Builder::new_synced_database]
    #[cfg(feature = "sync")]
    SyncedDatabase(SyncedDatabase),
}

impl Database {
    pub(crate) async fn libsql_database(&self) -> Result<libsql::Database, libsql::Error> {
        match self {
            #[cfg(feature = "core")]
            Self::Local(x) => x.libsql_database().await,
            #[cfg(feature = "replication")]
            Self::LocalReplica(x) => x.libsql_database().await,
            #[cfg(feature = "remote")]
            Self::Remote(x) => x.libsql_database().await,
            #[cfg(feature = "replication")]
            Self::RemoteReplica(x) => x.libsql_database().await,
            #[cfg(feature = "sync")]
            Self::SyncedDatabase(x) => x.libsql_database().await,
            #[cfg(not(any(
                feature = "core",
                feature = "replication",
                feature = "remote",
                feature = "sync"
            )))]
            _ => compile_error!("At least one of the following features must be enabled: core, replication, remote, sync"),
        }
    }
}

#[cfg(feature = "core")]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[allow(missing_docs)]
pub struct Local {
    pub path: PathBuf,
    pub encryption_config: Option<EncryptionConfig>,
    pub flags: Option<OpenFlags>,
}

#[cfg(feature = "core")]
impl Local {
    async fn libsql_database(&self) -> Result<libsql::Database, libsql::Error> {
        let mut builder = Builder::new_local(&self.path);
        if let Some(encryption_config) = &self.encryption_config {
            builder = builder.encryption_config(encryption_config.to_libsql());
        }
        if let Some(flags) = &self.flags {
            builder = builder.flags(flags.to_libsql());
        }
        builder.build().await
    }
}

#[cfg(any(feature = "core", feature = "replication"))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[allow(missing_docs)]
pub struct EncryptionConfig {
    pub cipher: Cipher,
    pub encryption_key: bytes::Bytes,
}

#[cfg(feature = "core")]
impl EncryptionConfig {
    fn to_libsql(&self) -> libsql::EncryptionConfig {
        libsql::EncryptionConfig {
            cipher: self.cipher.to_libsql(),
            encryption_key: self.encryption_key.clone(),
        }
    }
}

#[cfg(any(feature = "core", feature = "replication"))]
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
/// This is a 1:1 copy of [libsql::Cipher] with (de)serialization support
pub enum Cipher {
    #[default]
    #[cfg_attr(feature = "serde", serde(rename = "aes256cbc"))]
    /// AES 256 Bit CBC - No HMAC (wxSQLite3)
    Aes256Cbc,
}

#[cfg(feature = "core")]
impl Cipher {
    fn to_libsql(self) -> libsql::Cipher {
        match self {
            Self::Aes256Cbc => libsql::Cipher::Aes256Cbc,
        }
    }
}

#[cfg(any(feature = "core", feature = "replication"))]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[allow(missing_docs)]
pub struct OpenFlags {
    pub read_only: bool,
    pub read_write: bool,
    pub create: bool,
}

#[cfg(any(feature = "core", feature = "replication"))]
impl OpenFlags {
    fn to_libsql(self) -> libsql::OpenFlags {
        (if self.read_only {
            libsql::OpenFlags::SQLITE_OPEN_READ_ONLY
        } else {
            libsql::OpenFlags::empty()
        }) | (if self.read_write {
            libsql::OpenFlags::SQLITE_OPEN_READ_WRITE
        } else {
            libsql::OpenFlags::empty()
        }) | (if self.create {
            libsql::OpenFlags::SQLITE_OPEN_CREATE
        } else {
            libsql::OpenFlags::empty()
        })
    }
}

#[cfg(feature = "replication")]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[allow(missing_docs)]
pub struct LocalReplica {
    pub path: PathBuf,
    pub encryption_config: Option<EncryptionConfig>,
    pub flags: Option<OpenFlags>,
}

#[cfg(feature = "replication")]
impl LocalReplica {
    async fn libsql_database(&self) -> Result<libsql::Database, libsql::Error> {
        let mut builder = Builder::new_local_replica(&self.path);
        if let Some(flags) = &self.flags {
            builder = builder.flags(flags.to_libsql());
        }
        // FIXME add support for http_request_callback ?
        builder.build().await
    }
}

#[cfg(feature = "remote")]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[allow(missing_docs)]
pub struct Remote {
    pub url: String,
    pub auth_token: String,
    pub namespace: Option<String>,
    pub remote_encryption: Option<EncryptionContext>,
}

#[cfg(feature = "remote")]
impl Remote {
    async fn libsql_database(&self) -> Result<libsql::Database, libsql::Error> {
        // connector, namespace, remote_encryption
        let mut builder = Builder::new_remote(self.url.clone(), self.auth_token.clone());
        // TODO connector
        if let Some(namespace) = &self.namespace {
            builder = builder.namespace(namespace);
        }
        #[allow(unused)]
        if let Some(encryption_context) = &self.remote_encryption {
            #[cfg(feature = "sync")]
            {
                builder = builder.remote_encryption(encryption_context.to_libsql());
            }
            #[cfg(not(feature = "sync"))]
            return Err(libsql::Error::Misuse(
                "Remote encryption unavailable: sync feature of libsql is disabled".into(),
            ));
        }
        builder.build().await
    }
}

#[cfg(feature = "replication")]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[allow(missing_docs)]
pub struct RemoteReplica {
    pub path: PathBuf,
    pub url: String,
    pub auth_token: String,
    // TODO connector
    pub encryption_config: Option<EncryptionConfig>,
    // TODO http_request_callback
    pub namespace: Option<String>,
    pub read_your_writes: Option<bool>,
    pub remote_encryption: Option<EncryptionContext>,
    pub sync_interval: Option<Duration>,
    pub sync_protocol: Option<SyncProtocol>,
}

#[cfg(feature = "replication")]
impl RemoteReplica {
    async fn libsql_database(&self) -> Result<libsql::Database, libsql::Error> {
        // connector, namespace, remote_encryption
        let mut builder =
            Builder::new_remote_replica(&self.path, self.url.clone(), self.auth_token.clone());
        // FIXME add support for connector
        #[allow(unused)]
        if let Some(encryption_config) = &self.encryption_config {
            #[cfg(feature = "core")]
            {
                builder = builder.encryption_config(encryption_config.to_libsql());
            }
            #[cfg(not(feature = "core"))]
            return Err(libsql::Error::Misuse("RemoteReplicate::encryption_config unavailable: core feature of libsql is disabled".into()));
        }
        // FIXME add support for http_request_callback ?
        if let Some(namespace) = &self.namespace {
            builder = builder.namespace(namespace);
        }
        if let Some(read_your_writes) = self.read_your_writes {
            builder = builder.read_your_writes(read_your_writes);
        }
        #[allow(unused)]
        if let Some(encryption_context) = &self.remote_encryption {
            #[cfg(feature = "sync")]
            {
                builder = builder.remote_encryption(encryption_context.to_libsql());
            }
            #[cfg(not(feature = "sync"))]
            return Err(libsql::Error::Misuse("RemoteReplication::encryption_context unavailable: sync feature of libsql is disabled".into()));
        }
        if let Some(sync_interval) = &self.sync_interval {
            builder = builder.sync_interval(*sync_interval);
        }
        #[allow(unused)]
        if let Some(sync_protocol) = &self.sync_protocol {
            #[cfg(feature = "sync")]
            {
                builder = builder.sync_protocol(sync_protocol.to_libsql());
            }
            #[cfg(not(feature = "sync"))]
            return Err(libsql::Error::Misuse(
                "RemoteReplication::sync_protocol unavailable: sync feature of libsql is disabled"
                    .into(),
            ));
        }
        builder.build().await
    }
}

#[cfg(any(feature = "remote", feature = "replication", feature = "sync"))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
/// This is a 1:1 copy of [libsql::EncryptionContext] with (de)serialization support
pub struct EncryptionContext {
    /// The base64-encoded key for the encryption, sent on every request.
    pub key: EncryptionKey,
}

#[cfg(feature = "sync")]
impl EncryptionContext {
    #[cfg(feature = "sync")]
    fn to_libsql(&self) -> libsql::EncryptionContext {
        libsql::EncryptionContext {
            key: self.key.to_libsql(),
        }
    }
}

#[cfg(any(feature = "remote", feature = "replication", feature = "sync"))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
/// This is a 1:1 copy of [libsql::EncryptionKey] with (de)serialization support
pub enum EncryptionKey {
    /// The key is a base64-encoded string.
    Base64Encoded(String),
    /// The key is a byte array.
    Bytes(Vec<u8>),
}

#[cfg(any(feature = "remote", feature = "sync"))]
impl EncryptionKey {
    #[cfg(feature = "sync")]
    fn to_libsql(&self) -> libsql::EncryptionKey {
        #[cfg(feature = "sync")]
        match self {
            Self::Base64Encoded(string) => libsql::EncryptionKey::Base64Encoded(string.clone()),
            Self::Bytes(bytes) => libsql::EncryptionKey::Bytes(bytes.clone()),
        }
    }
}

#[cfg(feature = "replication")]
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
/// This is a 1:1 copy of the [libsql::SyncProtocol] with (de)serialization support
pub enum SyncProtocol {
    #[allow(missing_docs)]
    V1,
    #[allow(missing_docs)]
    V2,
}

#[cfg(all(feature = "replication", feature = "sync"))]
impl SyncProtocol {
    fn to_libsql(self) -> libsql::SyncProtocol {
        match self {
            Self::V1 => libsql::SyncProtocol::V1,
            Self::V2 => libsql::SyncProtocol::V2,
        }
    }
}

#[cfg(feature = "sync")]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[allow(missing_docs)]
pub struct SyncedDatabase {
    pub path: PathBuf,
    pub url: String,
    pub auth_token: String,
    // TODO connector
    pub read_your_writes: Option<bool>,
    pub remote_encryption: Option<EncryptionContext>,
    pub remote_writes: Option<bool>,
    pub set_push_batch_size: Option<u32>,
    pub sync_interval: Option<Duration>,
}

#[cfg(feature = "sync")]
impl SyncedDatabase {
    async fn libsql_database(&self) -> Result<libsql::Database, libsql::Error> {
        let mut builder =
            Builder::new_synced_database(&self.path, self.url.clone(), self.auth_token.clone());
        // TODO connector
        if let Some(read_your_writes) = self.read_your_writes {
            builder = builder.read_your_writes(read_your_writes);
        }
        if let Some(encryption_context) = &self.remote_encryption {
            builder = builder.remote_encryption(encryption_context.to_libsql());
        }
        if let Some(remote_writes) = &self.remote_writes {
            builder = builder.remote_writes(*remote_writes);
        }
        if let Some(push_batch_size) = &self.set_push_batch_size {
            builder = builder.set_push_batch_size(*push_batch_size);
        }
        if let Some(sync_interval) = &self.sync_interval {
            builder = builder.sync_interval(*sync_interval);
        }
        builder.build().await
    }
}

/// This error is returned if there is something wrong with the libSQL configuration.
pub type ConfigError = libsql::Error;

use std::{convert::Infallible, path::PathBuf};

use crate::{CreatePoolError, Manager, Pool, PoolBuilder, PoolConfig, Runtime};

/// Configuration object.
///
/// # Example (from environment)
///
/// By enabling the `serde` feature you can read the configuration using the
/// [`config`](https://crates.io/crates/config) crate as following:
/// ```env
/// SQLITE__PATH=db.sqlite3
/// SQLITE__POOL__MAX_SIZE=16
/// SQLITE__POOL__TIMEOUTS__WAIT__SECS=5
/// SQLITE__POOL__TIMEOUTS__WAIT__NANOS=0
/// ```
/// ```rust
/// #[derive(serde::Deserialize, serde::Serialize)]
/// struct Config {
///     sqlite: deadpool_sqlite::Config,
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
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Config {
    /// Path to SQLite database file.
    pub path: PathBuf,

    /// [`Pool`] configuration.
    pub pool: Option<PoolConfig>,

    /// [`OpenFlags`] used when opening the SQLite connection.
    ///
    /// When set to `None` the rusqlite default flags are used, which is the
    /// same behaviour as calling [`rusqlite::Connection::open`].
    pub flags: Option<OpenFlags>,
}

impl Config {
    /// Create a new [`Config`] with the given `path` of SQLite database file.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            pool: None,
            flags: None,
        }
    }

    /// Creates a new [`Pool`] using this [`Config`].
    ///
    /// # Errors
    ///
    /// See [`CreatePoolError`] for details.
    pub fn create_pool(&self, runtime: Runtime) -> Result<Pool, CreatePoolError> {
        self.builder(runtime)
            .map_err(CreatePoolError::Config)?
            .build()
            .map_err(CreatePoolError::Build)
    }

    /// Creates a new [`PoolBuilder`] using this [`Config`].
    ///
    /// # Errors
    ///
    /// See [`ConfigError`] for details.
    pub fn builder(&self, runtime: Runtime) -> Result<PoolBuilder, ConfigError> {
        let manager = Manager::from_config(self, runtime);
        Ok(Pool::builder(manager)
            .config(self.get_pool_config())
            .runtime(runtime))
    }

    /// Returns [`deadpool::managed::PoolConfig`] which can be used to construct
    /// a [`deadpool::managed::Pool`] instance.
    #[must_use]
    pub fn get_pool_config(&self) -> PoolConfig {
        self.pool.unwrap_or_default()
    }
}

/// (De)serializable representation of the [`rusqlite::OpenFlags`] used when
/// opening a connection.
///
/// This mirrors the subset of flags most commonly needed and keeps the
/// [`Config`] usable with the [`config`](https://crates.io/crates/config)
/// crate, which [`rusqlite::OpenFlags`] does not support on its own.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct OpenFlags {
    /// Open the database in read-only mode. See
    /// [`rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY`].
    pub read_only: bool,

    /// Open the database for reading and writing. See
    /// [`rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE`].
    pub read_write: bool,

    /// Create the database file if it does not exist. See
    /// [`rusqlite::OpenFlags::SQLITE_OPEN_CREATE`].
    pub create: bool,
}

impl OpenFlags {
    pub(crate) fn to_rusqlite(self) -> rusqlite::OpenFlags {
        (if self.read_only {
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
        } else {
            rusqlite::OpenFlags::empty()
        }) | (if self.read_write {
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
        } else {
            rusqlite::OpenFlags::empty()
        }) | (if self.create {
            rusqlite::OpenFlags::SQLITE_OPEN_CREATE
        } else {
            rusqlite::OpenFlags::empty()
        })
    }
}

/// This error is returned if there is something wrong with the SQLite configuration.
///
/// This is just a type alias to [`Infallible`] at the moment as there
/// is no validation happening at the configuration phase.
pub type ConfigError = Infallible;

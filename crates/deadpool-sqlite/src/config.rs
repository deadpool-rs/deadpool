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
}

impl Config {
    /// Create a new [`Config`] with the given `path` of SQLite database file.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            pool: None,
        }
    }

    /// Creates a new [`Pool`] using this [`Config`].
    ///
    /// # Errors
    ///
    /// See [`CreatePoolError`] for details.
    ///
    /// [`RedisError`]: redis::RedisError
    pub fn create_pool(&self, runtime: Runtime) -> Result<Pool, CreatePoolError> {
        self.builder(runtime)
            .map_err(CreatePoolError::Config)?
            .runtime(runtime)
            .build()
            .map_err(CreatePoolError::Build)
    }

    /// Creates a new [`PoolBuilder`] using this [`Config`].
    ///
    /// # Errors
    ///
    /// See [`ConfigError`] for details.
    ///
    /// [`RedisError`]: redis::RedisError
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

/// This error is returned if there is something wrong with the SQLite configuration.
///
/// This is just a type alias to [`Infallible`] at the moment as there
/// is no validation happening at the configuration phase.
pub type ConfigError = Infallible;

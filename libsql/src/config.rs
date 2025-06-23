use std::convert::Infallible;

use deadpool::managed::{CreatePoolError, PoolConfig};
use libsql::Database;

use crate::{Manager, Pool, PoolBuilder};

/// libsql configuration object
///
/// # Example
///
/// ```rust
///# use deadpool_libsql::libsql::Builder;
///
/// let path = "libsql.db";
/// let database = Builder::new_local(path).build().await?;
/// let pool = Config::new(database).create_pool()?;
/// ```
#[derive(Debug)]
pub struct Config {
    /// libsql database
    pub database: Database,
    /// [`Pool`] configuration
    pub pool: Option<PoolConfig>,
}

impl Config {
    /// Create a new [`Config`] with the given database
    #[must_use]
    pub fn new(database: Database) -> Self {
        Self {
            database,
            pool: None,
        }
    }

    /// Create a new [`Pool`] using this [`Config`].
    ///
    /// # Errors
    ///
    /// See [`CreatePoolError`] for details.
    pub fn create_pool(self) -> Result<Pool, CreatePoolError<ConfigError>> {
        self.builder()
            .map_err(CreatePoolError::Config)?
            .build()
            .map_err(CreatePoolError::Build)
    }

    /// Creates a new [`PoolBuilder`] using this [`Config`].
    ///
    /// # Errors
    ///
    /// See [`ConfigError`] for details.
    pub fn builder(self) -> Result<PoolBuilder, ConfigError> {
        let config = self.get_pool_config();
        let manager = Manager::from_config(self);
        Ok(Pool::builder(manager).config(config))
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

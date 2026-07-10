use std::convert::Infallible;

use crate::{CreatePoolError, Manager, Pool, PoolBuilder, PoolConfig};

/// Configuration object.
///
/// # Example (from environment)
///
/// You can read the configuration using the
/// [`config`](https://crates.io/crates/config) crate as following:
/// ```env
/// MEMCACHED__ADDR=127.0.0.1:11211
/// MEMCACHED__POOL__MAX_SIZE=16
/// MEMCACHED__POOL__TIMEOUTS__WAIT__SECS=5
/// MEMCACHED__POOL__TIMEOUTS__WAIT__NANOS=0
/// ```
/// ```rust,ignore
/// #[derive(serde::Deserialize, serde::Serialize)]
/// struct Config {
///     memcached: deadpool_memcached::Config,
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
pub struct Config {
    /// Memcached address.
    pub addr: Option<String>,

    /// [`Pool`] configuration.
    pub pool: Option<PoolConfig>,
}

impl Config {
    /// Creates a new [`Config`] with the given Memcached address.
    #[must_use]
    pub fn new(addr: impl Into<String>) -> Self {
        Self {
            addr: Some(addr.into()),
            pool: None,
        }
    }

    /// Creates a new [`Pool`] using this [`Config`].
    ///
    /// # Errors
    ///
    /// See [`CreatePoolError`] for details.
    pub fn create_pool(&self) -> Result<Pool, CreatePoolError> {
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
    pub fn builder(&self) -> Result<PoolBuilder, ConfigError> {
        let manager = Manager::new(self.get_addr());
        Ok(Pool::builder(manager).config(self.get_pool_config()))
    }

    /// Returns address used to connect to the Memcached server.
    pub fn get_addr(&self) -> &str {
        self.addr.as_deref().unwrap_or("127.0.0.1:11211")
    }

    /// Returns [`deadpool::managed::PoolConfig`] which can be used to construct
    /// a [`deadpool::managed::Pool`] instance.
    #[must_use]
    pub fn get_pool_config(&self) -> PoolConfig {
        self.pool.unwrap_or_default()
    }
}

/// This error is returned if there is something wrong with the Memcached configuration.
///
/// This is just a type alias to [`Infallible`] at the moment as there
/// is no validation happening at the configuration phase.
pub type ConfigError = Infallible;

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn default_addr_is_localhost() {
        let cfg = Config::default();
        assert_eq!(cfg.get_addr(), "127.0.0.1:11211");
    }

    #[test]
    fn new_sets_addr() {
        let cfg = Config::new("cache.internal:11211");
        assert_eq!(cfg.get_addr(), "cache.internal:11211");
    }
}

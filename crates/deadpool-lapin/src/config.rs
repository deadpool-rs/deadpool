use std::fmt;
use std::sync::Arc;

use crate::{CreatePoolError, Manager, Pool, PoolBuilder, PoolConfig, Runtime};

/// Configuration object.
///
/// # Example (from environment)
///
/// By enabling the `serde` feature you can read the configuration using the
/// [`config`](https://crates.io/crates/config) crate as following:
/// ```env
/// AMQP__URL=amqp://127.0.0.1:5672/%2f
/// AMQP__POOL__MAX_SIZE=16
/// AMQP__POOL__TIMEOUTS__WAIT__SECS=2
/// AMQP__POOL__TIMEOUTS__WAIT__NANOS=0
/// ```
/// ```rust
/// #[derive(serde::Deserialize)]
/// struct Config {
///     amqp: deadpool_lapin::Config,
/// }
///
/// impl Config {
///     pub fn from_env() -> Result<Self, config::ConfigError> {
///         let mut cfg = config::Config::builder()
///            .add_source(config::Environment::default().separator("__"))
///            .build()?;
///            cfg.try_deserialize()
///     }
/// }
/// ```
#[derive(Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Config {
    /// AMQP server URL.
    pub url: Option<String>,

    /// [`Pool`] configuration.
    pub pool: Option<PoolConfig>,
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("url", &self.url)
            .field("pool", &self.pool)
            .finish()
    }
}

impl Config {
    /// Creates a new [`Pool`] using this [`Config`].
    ///
    /// # Errors
    ///
    /// See [`CreatePoolError`] for details.
    pub fn create_pool<F>(
        &self,
        connection_properties: F,
        runtime: Runtime,
    ) -> Result<Pool, CreatePoolError>
    where
        F: Fn() -> lapin::ConnectionProperties + Send + Sync + 'static,
    {
        self.builder(connection_properties, runtime)
            .map_err(CreatePoolError::Config)?
            .build()
            .map_err(CreatePoolError::Build)
    }

    /// Creates a new [`PoolBuilder`] using this [`Config`].
    ///
    /// # Errors
    ///
    /// See [`ConfigError`] for details.
    pub fn builder<F>(
        &self,
        connection_properties: F,
        runtime: Runtime,
    ) -> Result<PoolBuilder, ConfigError>
    where
        F: Fn() -> lapin::ConnectionProperties + Send + Sync + 'static,
    {
        let url = self.get_url().to_string();
        let pool_config = self.get_pool_config();
        let connection_properties: Arc<
            dyn Fn() -> lapin::ConnectionProperties + Send + Sync + 'static,
        > = Arc::new(connection_properties);
        let manager = match runtime {
            #[cfg(feature = "rt_tokio_1")]
            Runtime::Tokio1 => {
                let runtime = async_rs::Runtime::tokio_current();
                let connection_properties = connection_properties.clone();
                Manager::new(url, move || connection_properties(), runtime)
            }
            #[cfg(feature = "rt_smol_2")]
            Runtime::Smol2 => {
                let runtime = async_rs::Runtime::smol();
                let connection_properties = connection_properties.clone();
                Manager::new(url, move || connection_properties(), runtime)
            }
            #[allow(unreachable_patterns)]
            _ => return Err(ConfigError::UnsupportedRuntime(runtime)),
        };

        Ok(Pool::builder(manager).config(pool_config).runtime(runtime))
    }

    /// Returns URL which can be used to connect to the database.
    pub fn get_url(&self) -> &str {
        self.url.as_deref().unwrap_or("amqp://127.0.0.1:5672/%2f")
    }

    /// Returns [`deadpool::managed::PoolConfig`] which can be used to construct
    /// a [`deadpool::managed::Pool`] instance.
    #[must_use]
    pub fn get_pool_config(&self) -> PoolConfig {
        self.pool.unwrap_or_default()
    }
}

/// This error is returned if there is something wrong with the lapin configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ConfigError {
    /// The selected [`Runtime`] is not supported by this crate build.
    #[error("unsupported runtime for deadpool-lapin: {0:?}")]
    UnsupportedRuntime(Runtime),
}

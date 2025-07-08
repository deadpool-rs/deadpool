use std::convert::Infallible;

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

    /// Connection properties.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub connection_properties: lapin::ConnectionProperties,
}

pub(crate) struct ConnProps<'a>(pub(crate) &'a lapin::ConnectionProperties);
impl std::fmt::Debug for ConnProps<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectionProperties")
            .field("locale", &self.0.locale)
            .field("client_properties", &self.0.client_properties)
            .finish_non_exhaustive()
    }
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("url", &self.url)
            .field("pool", &self.pool)
            .field(
                "connection_properties",
                &ConnProps(&self.connection_properties),
            )
            .finish()
    }
}

impl Config {
    /// Creates a new [`Pool`] using this [`Config`].
    ///
    /// # Errors
    ///
    /// See [`CreatePoolError`] for details.
    pub fn create_pool(&self, runtime: Option<Runtime>) -> Result<Pool, CreatePoolError> {
        self.builder(runtime)
            .build()
            .map_err(CreatePoolError::Build)
    }

    /// Creates a new [`PoolBuilder`] using this [`Config`].
    pub fn builder(&self, runtime: Option<Runtime>) -> PoolBuilder {
        let url = self.get_url().to_string();
        let pool_config = self.get_pool_config();

        let conn_props = self.connection_properties.clone();
        let conn_props = match runtime {
            None => conn_props,
            #[cfg(feature = "rt_tokio_1")]
            Some(Runtime::Tokio1) => conn_props
                .with_executor(tokio_executor_trait::Tokio::current())
                .with_reactor(tokio_reactor_trait::Tokio),
            #[cfg(feature = "rt_async-std_1")]
            Some(Runtime::AsyncStd1) => conn_props
                .with_executor(async_executor_trait::AsyncStd)
                .with_reactor(async_reactor_trait::AsyncIo),
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        };

        let mut builder = Pool::builder(Manager::new(url, conn_props)).config(pool_config);

        if let Some(runtime) = runtime {
            builder = builder.runtime(runtime)
        }

        builder
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
///
/// This is just a type alias to [`Infallible`] at the moment as there
/// is no validation happening at the configuration phase.
pub type ConfigError = Infallible;

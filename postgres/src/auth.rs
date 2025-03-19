/// Token fetcher for authentication.
///
/// This enum represents different methods of fetching authentication tokens:
/// - Default: Uses a static password/token
/// - AWS RDS: Dynamically fetches tokens for AWS RDS IAM authentication
#[derive(Debug)]
pub(super) enum AuthTokenFetcher {
    /// Default authentication using a static password/token
    Default(Vec<u8>),
    #[cfg(feature = "aws")]
    /// AWS RDS IAM authentication token fetcher
    AwsRds(crate::aws::AwsRdsInner),
}

impl AuthTokenFetcher {
    /// Creates a new AuthTokenFetcher based on the provided configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The manager configuration
    /// * `pg_config` - The PostgreSQL connection configuration
    ///
    /// # Returns
    ///
    /// Returns an AuthTokenFetcher configured based on the provided settings
    pub(super) fn for_config(config: &super::ManagerConfig, pg_config: &super::PgConfig) -> Self {
        #[cfg(not(feature = "aws"))]
        {
            let _ = config;
            Self::default(pg_config)
        }
        #[cfg(feature = "aws")]
        {
            crate::aws::for_config(config, pg_config)
        }
    }

    /// Creates a default token fetcher with a static password.
    pub(super) fn default(pg_config: &super::PgConfig) -> Self {
        Self::Default(pg_config.get_password().unwrap_or_default().to_vec())
    }

    /// Fetches a new token if needed.
    ///
    /// For AWS RDS authentication, this will fetch a new token if the current one
    /// has expired. For default authentication, this is a no-op.
    pub(super) async fn fetch_token_if_needed(&self) {
        match self {
            #[cfg(feature = "aws")]
            AuthTokenFetcher::AwsRds(inner) => crate::aws::fetch_token_if_needed(inner).await,
            _ => {},
        }
    }

    /// Executes a provided function with the current authentication token.
    ///
    /// This method retrieves the current authentication token and passes it to the provided
    /// function `f`. The function `f` is then executed with the token as its argument.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure or function that takes a byte slice (`&[u8]`) representing the
    ///         authentication token and returns a value of type `R`.
    ///
    /// # Returns
    ///
    /// Returns the result of the provided function `f` executed with the current authentication token.
    pub(super) async fn with_token<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        match self {
            AuthTokenFetcher::Default(token) => f(token),
            #[cfg(feature = "aws")]
            AuthTokenFetcher::AwsRds(inner) => f(inner.read().await.token().as_bytes()),
        }
    }
}

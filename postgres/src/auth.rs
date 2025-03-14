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
        #[cfg(not(feature = "aws"))] {
            let _ = config;
            Self::default(pg_config)
        }
        #[cfg(feature = "aws")] {
            crate::aws::for_config(config, pg_config)
        }
    }

    /// Creates a default token fetcher with a static password.
    pub(super)fn default(pg_config: &super::PgConfig) -> Self {
        Self::Default(pg_config.get_password().unwrap_or_default().to_vec())
    }

    /// Fetches a new token if needed.
    /// 
    /// For AWS RDS authentication, this will fetch a new token if the current one
    /// has expired. For default authentication, this is a no-op.
    pub(super) async fn fetch_token(&self) {
        match self {
            #[cfg(feature = "aws")]
            AuthTokenFetcher::AwsRds(inner) => crate::aws::fetch_token(inner).await,
            _ => {}
        }
    }

    /// Checks if a new token needs to be fetched.
    /// 
    /// # Returns
    /// 
    /// Returns true if a new token should be fetched, false otherwise.
    pub(super) async fn is_fetch_needed(&self) -> bool {
        match self {
            #[cfg(feature = "aws")]
            AuthTokenFetcher::AwsRds(inner) => inner.read().await.is_fetch_needed(),
            _ => false,
        }
    }

    /// Gets the current authentication token.
    /// 
    /// # Returns
    /// 
    /// Returns the current authentication token as a byte vector.
    pub(super) async fn token(&self) -> Vec<u8> {
        match self {
            AuthTokenFetcher::Default(token) => token.clone(),
            #[cfg(feature = "aws")]
            AuthTokenFetcher::AwsRds(inner) => inner.read().await.token().as_bytes().to_vec(),
        }
    }
}

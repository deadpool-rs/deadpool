use std::time::Duration;

use tokio::{sync::RwLock, time::Instant};
use tokio_postgres::config::Host;

use crate::auth::AuthTokenFetcher;

use super::ManagerConfig;

/// Default expiration time in seconds for AWS RDS IAM authentication tokens.
/// AWS recommends using a token lifetime between 5 minutes and 15 minutes.
const DEFAULT_EXPIRES_IN: u64 = 900;

pub(super) type AwsRdsInner = RwLock<AuthTokenFetcherInner>;

/// Configuration for AWS RDS IAM authentication.
///
/// This struct holds configuration options for AWS RDS IAM authentication,
/// including the AWS region and token expiration duration.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct AwsRdsSignerConfig {
    region: Option<String>,
    expires_in: Duration,
}

impl AwsRdsSignerConfig {
    /// Creates a new builder for AwsRdsSignerConfig.
    ///
    /// Use this method to start building a new configuration with custom settings.
    ///
    /// # Returns
    ///
    /// Returns a new `AwsRdsSignerConfigBuilder` instance.
    pub fn builder() -> AwsRdsSignerConfigBuilder {
        AwsRdsSignerConfigBuilder::default()
    }

    /// Gets the configured AWS region.
    ///
    /// # Returns
    ///
    /// Returns an `Option<&str>` containing the configured AWS region if set,
    /// or `None` if no region was configured.
    pub fn region(&self) -> Option<&str> {
        self.region.as_deref()
    }

    /// Gets the configured token expiration duration.
    ///
    /// # Returns
    ///
    /// Returns a `Duration` representing how long generated tokens will be valid for.
    /// If not explicitly set during configuration, this will be the default duration
    /// of 900 seconds (15 minutes).
    pub fn expires_in(&self) -> Duration {
        self.expires_in
    }
}

/// Builder for creating an `AwsRdsSignerConfig` instance.
///
/// This builder provides a fluent interface for configuring AWS RDS IAM authentication
/// settings. Use this to create a new `AwsRdsSignerConfig` with custom settings.
#[derive(Debug, Default)]
pub struct AwsRdsSignerConfigBuilder {
    region: Option<String>,
    expires_in: Option<Duration>,
}

impl AwsRdsSignerConfigBuilder {
    /// Sets the AWS region for the signer.
    ///
    /// # Arguments
    ///
    /// * `region` - The AWS region where the RDS instance is located (e.g., "us-east-1")
    ///
    /// # Returns
    ///
    /// Returns the builder instance for method chaining.
    pub fn region(mut self, region: impl Into<String>) -> Self {
        let region = region.into();
        if region.is_empty() {
            self.region = None;
        } else {
            self.region = Some(region);
        }
        self
    }

    /// Sets the expiration duration for generated authentication tokens.
    ///
    /// # Arguments
    ///
    /// * `duration` - The duration after which the generated token will expire.
    ///                If not set, defaults to 900 seconds (15 minutes).
    ///
    /// # Returns
    ///
    /// Returns the builder instance for method chaining.
    pub fn expires_in(mut self, duration: impl Into<Duration>) -> Self {
        self.expires_in = Some(duration.into());
        self
    }

    /// Builds and returns the final `AwsRdsSignerConfig` instance.
    ///
    /// If `expires_in` was not set, it will use the default value of 900 seconds (15 minutes).
    /// The region is optional and will be `None` if not set.
    ///
    /// # Returns
    ///
    /// Returns a configured `AwsRdsSignerConfig` instance.
    pub fn build(self) -> AwsRdsSignerConfig {
        AwsRdsSignerConfig {
            region: self.region,
            expires_in: self
                .expires_in
                .unwrap_or(Duration::from_secs(DEFAULT_EXPIRES_IN)),
        }
    }
}

impl ManagerConfig {
    /// Creates an AWS RDS signer configured with the connection details.
    ///
    /// # Arguments
    ///
    /// * `config` - The Postgres connection configuration
    ///
    /// # Returns
    ///
    /// Returns a configured `aws_rds_signer::Signer` instance ready for generating authentication tokens.
    pub(super) fn get_rds_signer(&self, config: &tokio_postgres::Config) -> aws_rds_signer::Signer {
        let Some(signer_config) = &self.aws_rds_signer_config else {
            tracing::warn!("AWS RDS signer config is not set, using default signer");
            return aws_rds_signer::Signer::default();
        };
        let host = host_to_string(&config.get_hosts()[0]);
        let port = config.get_ports()[0];
        let mut signer = aws_rds_signer::Signer::builder().host(host).port(port);
        if let Some(region) = signer_config.region() {
            signer = signer.region(region);
        }
        if let Some(user) = &config.get_user() {
            signer = signer.user(user.to_string());
        }
        tracing::debug!(target: "deadpool.postgres", "AWS RDS signer: {:?}", signer);
        signer.build()
    }

    /// Checks if AWS RDS IAM authentication is enabled.
    ///
    /// # Returns
    ///
    /// Returns `true` if AWS RDS IAM authentication is configured and enabled,
    /// `false` otherwise.
    pub fn is_rds_signer_enabled(&self) -> bool {
        self.aws_rds_signer_config.is_some()
    }
}

/// Converts a Postgres host configuration to a string representation.
///
/// # Arguments
///
/// * `host` - The host configuration, either TCP or Unix socket
///
/// # Returns
///
/// Returns a string representation of the host:
/// * For TCP hosts, returns the hostname
/// * For Unix sockets, returns the path as a string
fn host_to_string(host: &Host) -> String {
    match host {
        Host::Tcp(host) => host.to_string(),
        Host::Unix(path) => path.to_string_lossy().to_string(),
    }
}

/// Internal state for AWS RDS authentication token fetching.
#[derive(Debug)]
pub(super) struct AuthTokenFetcherInner {
    /// Duration after which a token expires and needs to be refreshed
    expires_in: Duration,
    /// Timestamp of when the last token was fetched
    last_token_fetch: Option<Instant>,
    /// AWS RDS signer for generating authentication tokens
    signer: aws_rds_signer::Signer,
    /// The current authentication token
    token: String,
}

impl AuthTokenFetcher {
    /// Creates a new AWS RDS token fetcher.
    ///
    /// # Arguments
    ///
    /// * `expires_in` - Duration after which the token expires
    /// * `signer` - AWS RDS signer for generating tokens
    ///
    /// # Returns
    ///
    /// Returns a new AuthTokenFetcher configured for AWS RDS authentication
    pub(super) fn aws_rds(
        expires_in: Duration,
        signer: aws_rds_signer::Signer,
    ) -> AuthTokenFetcher {
        AuthTokenFetcher::AwsRds(RwLock::new(AuthTokenFetcherInner {
            expires_in,
            last_token_fetch: None,
            signer,
            token: String::new(),
        }))
    }
}

impl AuthTokenFetcherInner {
    /// Checks if a new token needs to be fetched based on expiration time.
    ///
    /// # Returns
    ///
    /// Returns true if the current token has expired or no token exists,
    /// false otherwise.
    pub(super) fn is_fetch_needed(&self) -> bool {
        if let Some(last_token_fetch) = self.last_token_fetch {
            if last_token_fetch.elapsed() < self.expires_in {
                return false;
            }
            tracing::debug!(target: "deadpool.postgres", "Token expired, fetch needed");
        } else {
            tracing::debug!(target: "deadpool.postgres", "No token found, fetch needed");
        }
        true
    }

    /// Fetches a new authentication token from AWS RDS.
    ///
    /// Updates the internal token and last fetch timestamp if successful.
    /// Logs an error if token fetching fails.
    pub(super) async fn fetch_token(&mut self) {
        tracing::debug!(target: "deadpool.postgres", "Fetching RDS token");
        match self.signer.fetch_token().await {
            Ok(token) => {
                self.token = token;
                self.last_token_fetch = Some(Instant::now());
                tracing::debug!(
                    target: "deadpool.postgres",
                    "RDS token fetched successfully at {:?}",
                    self.last_token_fetch
                );
            }
            Err(e) => {
                tracing::error!(target: "deadpool.postgres", "Failed to fetch RDS signer token: {}", e);
            }
        }
    }

    /// Gets the current authentication token.
    ///
    /// # Returns
    ///
    /// Returns the current authentication token as a string slice.
    pub(super) fn token(&self) -> &str {
        &self.token
    }
}

/// Fetches a new token if needed by checking expiration and updating the token.
///
/// # Arguments
///
/// * `inner` - The AuthTokenFetcherInner containing token state
pub(super) async fn fetch_token(inner: &AwsRdsInner) {
    if inner.read().await.is_fetch_needed() {
        inner.write().await.fetch_token().await;
    }
}

pub(super) fn for_config(
    config: &ManagerConfig,
    pg_config: &tokio_postgres::Config,
) -> AuthTokenFetcher {
    config.aws_rds_signer_config.as_ref().map_or(
        AuthTokenFetcher::default(pg_config),
        |signer_config| {
            tracing::debug!(target: "deadpool.postgres", "Creating AuthTokenFetcher with config: {:?}", signer_config);
            AuthTokenFetcher::aws_rds(signer_config.expires_in(), config.get_rds_signer(pg_config))
        },
    )
}

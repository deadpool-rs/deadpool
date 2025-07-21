use thiserror::Error;

/// This error is returned when the connection fails
#[derive(Debug, Error)]
pub enum ConnectionError {
    /// The error was reported by the [libsql::Connection].
    #[error("Libsql returned an error: {0}")]
    Libsql(#[from] libsql::Error),
    /// The test query was executed but the database returned
    /// an unexpected response.
    #[error("Test query failed: {0}")]
    TestQueryFailed(&'static str),
}

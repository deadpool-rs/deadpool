use std::fmt;

/// Possible errors returned when managing [`Connection`]s.
///
/// [`Connection`]: crate::Connection
#[derive(Debug)]
pub enum Error {
    /// Failed to establish a [`Connection`].
    ///
    /// [`Connection`]: crate::Connection
    Connection(diesel::ConnectionError),

    /// Failed to ping the database.
    Ping(diesel::result::Error),

    /// The transaction manager of a given
    /// connection is in a broken state. That usually
    /// means that it contains an open uncommited transaction
    BrokenTransactionManger,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connection(e) => write!(f, "Failed to establish connection: {}", e),
            Self::Ping(e) => write!(f, "Failed to ping database: {}", e),
            Self::BrokenTransactionManger => write!(f, "Broken transaction manager"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Connection(e) => Some(e),
            Self::Ping(e) => Some(e),
            Self::BrokenTransactionManger => None,
        }
    }
}

impl From<diesel::ConnectionError> for Error {
    fn from(e: diesel::ConnectionError) -> Self {
        Self::Connection(e)
    }
}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Self {
        Self::Ping(e)
    }
}

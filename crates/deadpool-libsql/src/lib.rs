#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(
    nonstandard_style,
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links
)]
#![forbid(non_ascii_idents, unsafe_code)]
#![warn(
    deprecated_in_future,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    unused_import_braces,
    unused_labels,
    unused_lifetimes,
    unused_qualifications,
    unused_results
)]
#![allow(clippy::uninlined_format_args)]

use std::sync::atomic::{AtomicU64, Ordering};

use deadpool::managed::{self, RecycleError};

pub mod config;
pub use config::Config;
mod errors;

pub use libsql;

pub use deadpool::managed::reexports::*;
pub use errors::ConnectionError;
deadpool::managed_reexports!(
    "libsql",
    Manager,
    Connection,
    ConnectionError,
    config::ConfigError
);

/// Type alias for ['Object']
pub type Connection = managed::Object<Manager>;

/// [`Manager`] for creating and recycling [`libsql::Connection`].
///
/// [`Manager`]: managed::Manager
#[derive(Debug)]
pub struct Manager {
    database: libsql::Database,
    test_query_count: AtomicU64,
}

impl Manager {
    /// Creates a new [`Manager`] using the given [`libsql::Database`].
    pub fn from_libsql_database(database: libsql::Database) -> Self {
        Self {
            database,
            test_query_count: AtomicU64::new(0),
        }
    }

    /// Creates a new [`Manager`] using the given [`config::Config`].
    pub async fn from_config(config: Config) -> Result<Self, libsql::Error> {
        config
            .database
            .libsql_database()
            .await
            .map(Self::from_libsql_database)
    }

    async fn run_test_query(&self, conn: &libsql::Connection) -> Result<(), ConnectionError> {
        let test_query_count = self.test_query_count.fetch_add(1, Ordering::Relaxed);
        // A call to the database to check that it is accessible
        let row = conn
            .query("SELECT ?", [test_query_count])
            .await?
            .next()
            .await?
            .ok_or(ConnectionError::TestQueryFailed(
                "No rows returned from database for test query",
            ))?;
        let value: u64 = row.get(0)?;

        if value == test_query_count {
            Ok(())
        } else {
            Err(ConnectionError::TestQueryFailed(
                "Unexpected value returned for test query",
            ))
        }
    }
}

impl managed::Manager for Manager {
    type Type = libsql::Connection;
    type Error = ConnectionError;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let conn = self.database.connect()?;
        // Libsql establishes the database connection lazily. Thus the
        // only way to check if the connection is in a useable state is
        // to run a test query.
        self.run_test_query(&conn).await?;
        Ok(conn)
    }

    async fn recycle(
        &self,
        conn: &mut Self::Type,
        _: &Metrics,
    ) -> managed::RecycleResult<Self::Error> {
        self.run_test_query(conn)
            .await
            .map_err(RecycleError::Backend)
    }
}

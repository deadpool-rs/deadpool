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

mod config;
pub use self::config::{Config, ConfigError};

pub use libsql;
use libsql::Database;

pub use deadpool::managed::reexports::*;
deadpool::managed_reexports!("libsql", Manager, Connection, libsql::Error, ConfigError);

/// Type alias for ['Object']
pub type Connection = managed::Object<Manager>;

/// [`Manager`] for creating and recycling [`libsql::Connection`].
///
/// [`Manager`]: managed::Manager
#[derive(Debug)]
pub struct Manager {
    database: Database,
    recycle_count: AtomicU64,
}

impl Manager {
    /// Creates a new [`Manager`] using the given libsql database.
    pub fn new(database: Database) -> Self {
        Self {
            database,
            recycle_count: AtomicU64::new(0),
        }
    }

    /// Creates a new [`Manager`] using the given [`Config`].
    pub fn from_config(config: Config) -> Self {
        Self::new(config.database)
    }
}

impl managed::Manager for Manager {
    type Type = libsql::Connection;
    type Error = libsql::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let db = self.database.connect()?;
        Ok(db)
    }

    async fn recycle(
        &self,
        conn: &mut Self::Type,
        _: &Metrics,
    ) -> managed::RecycleResult<Self::Error> {
        let recycle_count = self.recycle_count.fetch_add(1, Ordering::Relaxed);

        // A call to the database to check that it is accessible
        let row = conn
            .query("SELECT ?", [recycle_count])
            .await?
            .next()
            .await?
            .ok_or(RecycleError::message(
                "No rows returned from database on recycle count",
            ))?;

        let value: u64 = row.get(0)?;

        if value == recycle_count {
            Ok(())
        } else {
            Err(RecycleError::message("Recycle count mismatch"))
        }
    }
}

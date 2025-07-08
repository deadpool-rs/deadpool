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

mod config;

use deadpool::managed;
use lapin::{ConnectionProperties, Error};

pub use lapin;

pub use self::config::{Config, ConfigError};

pub use deadpool::managed::reexports::*;
deadpool::managed_reexports!(
    "lapin",
    Manager,
    managed::Object<Manager>,
    Error,
    ConfigError
);

/// Type alias for ['Object']
pub type Connection = managed::Object<Manager>;

type RecycleResult = managed::RecycleResult<Error>;
type RecycleError = managed::RecycleError<Error>;

/// [`Manager`] for creating and recycling [`lapin::Connection`].
///
/// [`Manager`]: managed::Manager
pub struct Manager {
    addr: String,
    connection_properties: ConnectionProperties,
}

impl std::fmt::Debug for Manager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Manager")
            .field("addr", &self.addr)
            .field(
                "connection_properties",
                &config::ConnProps(&self.connection_properties),
            )
            .finish()
    }
}

impl Manager {
    /// Creates a new [`Manager`] using the given AMQP address and
    /// [`lapin::ConnectionProperties`].
    #[must_use]
    pub fn new<S: Into<String>>(addr: S, connection_properties: ConnectionProperties) -> Self {
        Self {
            addr: addr.into(),
            connection_properties,
        }
    }
}

impl managed::Manager for Manager {
    type Type = lapin::Connection;
    type Error = Error;

    async fn create(&self) -> Result<lapin::Connection, Error> {
        let conn =
            lapin::Connection::connect(self.addr.as_str(), self.connection_properties.clone())
                .await?;
        Ok(conn)
    }

    async fn recycle(&self, conn: &mut lapin::Connection, _: &Metrics) -> RecycleResult {
        match conn.status().state() {
            lapin::ConnectionState::Connected => Ok(()),
            other_state => Err(RecycleError::message(format!(
                "lapin connection is in state: {:?}",
                other_state
            ))),
        }
    }
}

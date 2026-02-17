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

use std::{future::Future, pin::Pin};

use deadpool::managed;
use lapin::Error;

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
type ConnectFuture = Pin<Box<dyn Future<Output = Result<lapin::Connection, Error>> + Send>>;
type ConnectFn = dyn Fn(String, lapin::ConnectionProperties) -> ConnectFuture + Send + Sync;

/// [`Manager`] for creating and recycling [`lapin::Connection`].
///
/// [`Manager`]: managed::Manager
pub struct Manager {
    addr: String,
    connect: Box<ConnectFn>,
    connection_properties: Box<dyn Fn() -> lapin::ConnectionProperties + Send + Sync>,
}

impl std::fmt::Debug for Manager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Manager")
            .field("addr", &self.addr)
            .finish_non_exhaustive()
    }
}

impl Manager {
    /// Creates a new [`Manager`] using the given AMQP address and
    /// [`async_rs::Runtime`] and a [`lapin::ConnectionProperties`] factory.
    #[must_use]
    pub fn new<S, F, RK>(addr: S, connection_properties: F, runtime: async_rs::Runtime<RK>) -> Self
    where
        S: Into<String>,
        F: Fn() -> lapin::ConnectionProperties + Send + Sync + 'static,
        RK: async_rs::traits::RuntimeKit + Send + Sync + Clone + 'static,
    {
        let connect = move |addr: String, conn_props: lapin::ConnectionProperties| {
            let runtime = runtime.clone();
            Box::pin(async move {
                lapin::Connection::connect_with_runtime(addr.as_str(), conn_props, runtime).await
            }) as ConnectFuture
        };

        Self {
            addr: addr.into(),
            connect: Box::new(connect),
            connection_properties: Box::new(connection_properties),
        }
    }
}

impl managed::Manager for Manager {
    type Type = lapin::Connection;
    type Error = Error;

    async fn create(&self) -> Result<lapin::Connection, Error> {
        let conn_props = (self.connection_properties)();
        let conn = (self.connect)(self.addr.clone(), conn_props).await?;
        Ok(conn)
    }

    async fn recycle(&self, conn: &mut lapin::Connection, _: &Metrics) -> RecycleResult {
        if conn.status().connected() {
            Ok(())
        } else {
            Err(RecycleError::message(format!(
                "lapin connection is not connected: {:?}",
                conn.status()
            )))
        }
    }
}

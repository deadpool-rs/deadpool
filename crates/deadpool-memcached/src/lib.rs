//! # Deadpool for Memcache
//!
//! Deadpool is a dead simple async pool for connections and objects of any type.
//!
//! This crate implements a [`deadpool`](https://crates.io/crates/deadpool) manager for
//! [`async-memcached`](https://crates.io/crates/async-memcached).  We specifically force users to
//! connect via TCP as there is no existing mechanism to parameterize how to deal with different
//! unerlying connection types at the moment.
#![deny(warnings, missing_docs)]
use std::convert::Infallible;

use async_memcached::{Client, Error};

/// Type alias for using [`deadpool::managed::RecycleResult`] with [`redis`].
type RecycleResult = deadpool::managed::RecycleResult<Error>;

type ConfigError = Infallible;

pub use deadpool::managed::reexports::*;
deadpool::managed_reexports!("memcached", Manager, Client, Error, ConfigError);

/// The manager for creating and recyling memcache connections
pub struct Manager {
    addr: String,
}

impl Manager {
    /// Create a new manager for the given address.
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}

impl deadpool::managed::Manager for Manager {
    type Type = Client;
    type Error = Error;

    async fn create(&self) -> Result<Client, Error> {
        Client::new(&self.addr).await
    }

    async fn recycle(&self, conn: &mut Client, _: &Metrics) -> RecycleResult {
        match conn.version().await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

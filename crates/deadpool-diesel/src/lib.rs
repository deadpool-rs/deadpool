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

mod error;
mod manager;

#[cfg(feature = "mysql")]
#[cfg_attr(docsrs, doc(cfg(feature = "mysql")))]
pub mod mysql;
#[cfg(feature = "postgres")]
#[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]
pub mod postgres;
#[cfg(feature = "sqlite")]
#[cfg_attr(docsrs, doc(cfg(feature = "sqlite")))]
pub mod sqlite;

use deadpool::managed;

pub use deadpool::managed::reexports::*;
pub use deadpool_sync::reexports::*;
// Normally backend implementations don't export the generic `Pool`
// type. `deadpool-diesel` is different in that regards as it is
// generic itself.
pub use deadpool::managed::Pool;

pub use self::{
    error::Error,
    manager::{Manager, ManagerConfig, RecycleCheckCallback, RecyclingMethod},
};

/// Type alias for using [`deadpool::managed::PoolError`] with [`diesel`].
pub type PoolError = managed::PoolError<Error>;

/// Connection which is returned by the [`Pool`].
pub type Connection<C> = deadpool_sync::SyncWrapper<C>;

//! Type aliases for using `deadpool-diesel` with SQLite.

/// Manager for SQLite connections
pub type Manager = crate::Manager<diesel::SqliteConnection>;

pub use deadpool::managed::reexports::*;
pub use deadpool_sync::reexports::*;
deadpool::managed_reexports!(
    "diesel",
    Manager,
    deadpool::managed::Object<Manager>,
    crate::Error,
    std::convert::Infallible
);

/// Type alias for [`Object`]
pub type Connection = Object;

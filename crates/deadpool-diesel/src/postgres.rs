//! Type aliases for using `deadpool-diesel` with PostgreSQL.

/// Manager for PostgreSQL connections
pub type Manager = crate::Manager<diesel::PgConnection>;

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

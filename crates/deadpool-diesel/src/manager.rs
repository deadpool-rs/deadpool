use std::{borrow::Cow, fmt, marker::PhantomData, sync::Arc};

use deadpool::{
    managed::{self, Metrics, RecycleError, RecycleResult},
    Runtime,
};
use deadpool_sync::SyncWrapper;
use diesel::{query_builder::QueryFragment, IntoSql, RunQueryDsl};

use crate::Error;

/// [`Connection`] [`Manager`] for use with [`diesel`].
///
/// See the [`deadpool` documentation](deadpool) for usage examples.
///
/// [`Manager`]: managed::Manager
/// [`Connection`]: crate::Connection
pub struct Manager<C> {
    database_url: String,
    runtime: Runtime,
    manager_config: Arc<ManagerConfig<C>>,
    _marker: PhantomData<fn() -> C>,
}

/// Type of the recycle check callback for the [`RecyclingMethod::CustomFunction`] variant
pub type RecycleCheckCallback<C> = dyn Fn(&mut C) -> Result<(), Error> + Send + Sync;

/// Possible methods of how a connection is recycled.
pub enum RecyclingMethod<C> {
    /// Only check for open transactions when recycling existing connections
    /// Unless you have special needs this is a safe choice.
    ///
    /// If the database connection is closed you will recieve an error on the first place
    /// you actually try to use the connection
    Fast,
    /// In addition to checking for open transactions a test query is executed
    ///
    /// This is slower, but guarantees that the database connection is ready to be used.
    Verified,
    /// Like `Verified` but with a custom query
    CustomQuery(Cow<'static, str>),
    /// Like `Verified` but with a custom callback that allows to perform more checks
    ///
    /// The connection is only recycled if the callback returns `Ok(())`
    CustomFunction(Box<RecycleCheckCallback<C>>),
}

// We use manual implementation here instead of `#[derive(Default)]` as of MSRV 1.63, it generates
// redundant `C: Default` bound, which imposes problems in the code.
// TODO: Use `#[derive(Default)]` with `#[default]` attribute once MSRV is bumped to 1.66 or above.
impl<C> Default for RecyclingMethod<C> {
    fn default() -> Self {
        Self::Fast
    }
}

/// Configuration object for a Manager.
///
/// This currently only makes it possible to specify which [`RecyclingMethod`]
/// should be used when retrieving existing objects from the [`Pool`].
///
/// [`Pool`]: crate::Pool
#[derive(Debug)]
pub struct ManagerConfig<C> {
    /// Method of how a connection is recycled. See [RecyclingMethod].
    pub recycling_method: RecyclingMethod<C>,
}

impl<C> Default for ManagerConfig<C> {
    fn default() -> Self {
        Self {
            recycling_method: Default::default(),
        }
    }
}

impl<C: fmt::Debug> fmt::Debug for RecyclingMethod<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fast => write!(f, "Fast"),
            Self::Verified => write!(f, "Verified"),
            Self::CustomQuery(arg0) => f.debug_tuple("CustomQuery").field(arg0).finish(),
            Self::CustomFunction(_) => f.debug_tuple("CustomFunction").finish(),
        }
    }
}

// Implemented manually to avoid unnecessary trait bound on `C` type parameter.
impl<C> fmt::Debug for Manager<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Manager")
            .field("database_url", &self.database_url)
            .field("runtime", &self.runtime)
            .field("_marker", &self._marker)
            .finish()
    }
}

impl<C> Manager<C>
where
    C: diesel::Connection,
{
    /// Creates a new [`Manager`] which establishes [`Connection`]s to the given
    /// `database_url`.
    ///
    /// [`Connection`]: crate::Connection
    #[must_use]
    pub fn new<S: Into<String>>(database_url: S, runtime: Runtime) -> Self {
        Self::from_config(database_url, runtime, Default::default())
    }

    /// Creates a new [`Manager`] which establishes [`Connection`]s to the given
    /// `database_url` with a specific [`ManagerConfig`].
    ///
    /// [`Connection`]: crate::Connection
    #[must_use]
    pub fn from_config(
        database_url: impl Into<String>,
        runtime: Runtime,
        manager_config: ManagerConfig<C>,
    ) -> Self {
        Manager {
            database_url: database_url.into(),
            runtime,
            manager_config: Arc::new(manager_config),
            _marker: PhantomData,
        }
    }
}

impl<C> managed::Manager for Manager<C>
where
    C: diesel::Connection + 'static,
    diesel::helper_types::select<diesel::dsl::AsExprOf<i32, diesel::sql_types::Integer>>:
        QueryFragment<C::Backend>,
    diesel::query_builder::SqlQuery: QueryFragment<C::Backend>,
{
    type Type = crate::Connection<C>;
    type Error = Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let database_url = self.database_url.clone();
        SyncWrapper::new(self.runtime, move || {
            C::establish(&database_url).map_err(Into::into)
        })
        .await
    }

    async fn recycle(&self, obj: &mut Self::Type, _: &Metrics) -> RecycleResult<Self::Error> {
        if obj.is_mutex_poisoned() {
            return Err(RecycleError::message(
                "Mutex is poisoned. Connection is considered unusable.",
            ));
        }
        let config = Arc::clone(&self.manager_config);
        obj.interact(move |conn| config.recycling_method.perform_recycle_check(conn))
            .await
            .map_err(|e| RecycleError::message(format!("Panic: {:?}", e)))
            .and_then(|r| r.map_err(RecycleError::Backend))
    }
}

impl<C> RecyclingMethod<C>
where
    C: diesel::Connection,
    diesel::helper_types::select<diesel::dsl::AsExprOf<i32, diesel::sql_types::Integer>>:
        QueryFragment<C::Backend>,
    diesel::query_builder::SqlQuery: QueryFragment<C::Backend>,
{
    fn perform_recycle_check(&self, conn: &mut C) -> Result<(), Error> {
        use diesel::connection::TransactionManager;

        // first always check for open transactions because
        // we really do not want to have a connection with a
        // dangling transaction in our connection pool
        if C::TransactionManager::is_broken_transaction_manager(conn) {
            return Err(Error::BrokenTransactionManger);
        }
        match self {
            // For fast we are basically done
            RecyclingMethod::Fast => {}
            // For verified we perform a `SELECT 1` statement
            // We use the DSL here to make this somewhat independent from
            // the backend SQL dialect
            RecyclingMethod::Verified => {
                let _ = diesel::select(1.into_sql::<diesel::sql_types::Integer>())
                    .execute(conn)
                    .map_err(Error::Ping)?;
            }
            // For custom query we just execute the user provided query
            RecyclingMethod::CustomQuery(query) => {
                let _ = diesel::sql_query(query.as_ref())
                    .execute(conn)
                    .map_err(Error::Ping)?;
            }
            // for custom function we call the relevant closure
            RecyclingMethod::CustomFunction(check) => check(conn)?,
        }
        Ok(())
    }
}

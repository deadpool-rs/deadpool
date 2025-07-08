use std::{fmt, sync::Arc};

use deadpool::{
    managed::{self, Metrics, RecycleError, RecycleResult},
    Runtime,
};
use deadpool_sync::SyncWrapper;

/// [`Manager`] for use with [`r2d2`] [managers](r2d2::ManageConnection).
///
/// See the [`deadpool` documentation](deadpool) for usage examples.
///
/// [`Manager`]: managed::Manager
pub struct Manager<M: r2d2::ManageConnection> {
    r2d2_manager: Arc<M>,
    runtime: Runtime,
}

// Implemented manually to avoid unnecessary trait bound on `C` type parameter.
impl<M: r2d2::ManageConnection> fmt::Debug for Manager<M>
where
    M: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Manager")
            .field("r2d2_manager", &self.r2d2_manager)
            .field("runtime", &self.runtime)
            .finish()
    }
}

impl<M: r2d2::ManageConnection> Manager<M> {
    /// Creates a new [`Manager`] using the given [`r2d2`] [manager](r2d2::ManageConnection)
    #[must_use]
    pub fn new(r2d2_manager: M, runtime: Runtime) -> Self {
        Manager {
            runtime,
            r2d2_manager: Arc::new(r2d2_manager),
        }
    }
}

impl<M: r2d2::ManageConnection> managed::Manager for Manager<M>
where
    M::Error: Send,
{
    type Type = SyncWrapper<M::Connection>;
    type Error = M::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let r2d2_manager = self.r2d2_manager.clone();
        SyncWrapper::new(self.runtime, move || r2d2_manager.connect()).await
    }

    async fn recycle(&self, obj: &mut Self::Type, _: &Metrics) -> RecycleResult<Self::Error> {
        if obj.is_mutex_poisoned() {
            return Err(RecycleError::message(
                "Mutex is poisoned. Connection is considered unusable.",
            ));
        }
        let r2d2_manager = self.r2d2_manager.clone();
        obj.interact::<_, RecycleResult<Self::Error>>(move |obj| {
            if r2d2_manager.has_broken(obj) {
                Err(RecycleError::message("Connection is broken"))
            } else {
                r2d2_manager.is_valid(obj).map_err(RecycleError::Backend)
            }
        })
        .await
        .map_err(|e| RecycleError::message(format!("Interaction failed: {}", e)))?
    }
}

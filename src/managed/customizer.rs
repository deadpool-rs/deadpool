use std::{borrow::BorrowMut, future::Future, pin::Pin};

use super::Manager;

/// An object modifier can be used to modify objects created by
/// a manager. It is applied by calling the [`ObjectCustomizer::wrap_manager`]
/// method which takes a [Manager] and returns a [WrappedManager].
pub enum ObjectCustomizer<T, W> {
    /// Use this variant if the function you are passing is know
    /// to never block. The function will be run as is for every
    /// new connection.
    NonBlocking(fn(obj: T) -> W),
    /// Use this variant if the given code is using async-await.
    /// If you need to call blocking sync code you can also use
    /// this variant and use something like `tokio::task::spawn_blocking`
    /// internally.
    Async(fn(obj: T) -> Pin<Box<dyn Future<Output = W> + 'static + Send>>),
}

impl<T, W> ObjectCustomizer<T, W> {
    /// Apply this [ObjectCustomizer] to a given [Manager]
    /// returning a [WrappedManager].
    pub fn wrap_manager<M>(self, manager: M) -> WrappedManager<M, W>
    where
        M: Manager<Type = T>,
    {
        WrappedManager {
            manager,
            customizer: self,
        }
    }
}

/// This struct wraps an existing [Manager] and runs the given
/// [ObjectCustomizer] every time a new object is created.
/// This struct can be created using the [ObjectCustomizer::wrap_manager]
/// method.
pub struct WrappedManager<M, W>
where
    M: Manager,
{
    manager: M,
    customizer: ObjectCustomizer<M::Type, W>,
}

#[async_trait::async_trait]
impl<M, W> Manager for WrappedManager<M, W>
where
    M: Manager,
    W: BorrowMut<M::Type> + Send,
{
    type Type = W;
    type Error = M::Error;
    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let obj = self.manager.create().await?;
        let obj = match self.customizer {
            ObjectCustomizer::NonBlocking(f) => f(obj),
            ObjectCustomizer::Async(f) => f(obj).await,
        };
        Ok(obj)
    }
    async fn recycle(&self, obj: &mut Self::Type) -> super::RecycleResult<Self::Error> {
        let mut obj = obj.borrow_mut();
        self.manager.recycle(&mut obj).await
    }
}

use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use crate::managed::{Manager, Metrics, Pool, WeakPool};

/// Wrapper around the actual pooled object which implements [`Deref`],
/// [`DerefMut`] and [`Drop`] traits.
///
/// Use this object just as if it was of type `T` and upon leaving a scope the
/// [`Drop::drop()`] will take care of returning it to the pool.
#[must_use]
pub struct Object<M: Manager> {
    /// The actual object
    pub(crate) inner: Option<ObjectInner<M>>,

    /// Pool to return the pooled object to.
    pub(crate) pool: WeakPool<M>,
}

impl<M> fmt::Debug for Object<M>
where
    M: fmt::Debug + Manager,
    M::Type: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Object")
            .field("inner", &self.inner)
            .finish()
    }
}

#[derive(Debug)]
pub(crate) struct ObjectInner<M: Manager> {
    /// Actual pooled object.
    pub obj: M::Type,

    /// The ID of this object. IDs are assigned in increasing order and
    /// generally reflect the creation order of objects.
    ///
    /// This can be used to discard objects after a configuration change
    /// or simply identify an objects for debugging purposes.
    pub id: usize,

    /// Object metrics.
    pub metrics: Metrics,
}

impl<M: Manager> Object<M> {
    /// Takes this [`Object`] from its [`Pool`] permanently. This reduces the
    /// size of the [`Pool`].
    #[must_use]
    pub fn take(mut this: Self) -> M::Type {
        let mut inner = this.inner.take().unwrap().obj;
        if let Some(pool) = Object::pool(&this) {
            pool.inner.detach_object(&mut inner)
        }
        inner
    }

    /// Returns the unique ID of this object.
    ///
    /// Object IDs are strictly monotonically increasing — each new object
    /// receives an ID greater than that of the previously created object.
    /// However, IDs are not guaranteed to be consecutive; gaps may exist.
    pub fn id(this: &Self) -> usize {
        this.inner.as_ref().unwrap().id
    }

    /// Get object statistics
    pub fn metrics(this: &Self) -> &Metrics {
        &this.inner.as_ref().unwrap().metrics
    }

    /// Returns the [`Pool`] this [`Object`] belongs to.
    ///
    /// Since [`Object`]s only hold a [`std::sync::Weak`] reference to the
    /// [`Pool`] they come from, this can fail and return [`None`] instead.
    pub fn pool(this: &Self) -> Option<Pool<M>> {
        this.pool.upgrade()
    }
}

impl<M: Manager> Drop for Object<M> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.take() {
            if let Some(pool) = self.pool.upgrade() {
                pool.inner.return_object(inner)
            }
        }
    }
}

impl<M: Manager> Deref for Object<M> {
    type Target = M::Type;
    fn deref(&self) -> &M::Type {
        &self.inner.as_ref().unwrap().obj
    }
}

impl<M: Manager> DerefMut for Object<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner.as_mut().unwrap().obj
    }
}

impl<M: Manager> AsRef<M::Type> for Object<M> {
    fn as_ref(&self) -> &M::Type {
        self
    }
}

impl<M: Manager> AsMut<M::Type> for Object<M> {
    fn as_mut(&mut self) -> &mut M::Type {
        self
    }
}

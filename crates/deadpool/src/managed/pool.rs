#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
use std::{
    fmt,
    future::Future,
    marker::PhantomData,
    sync::{
        Arc, Weak,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use deadpool_runtime::{Runtime, timeout};
use tokio::sync::TryAcquireError;

use crate::{
    Status,
    managed::{
        Manager, Metrics, Object, PoolBuilder, PoolConfig, PoolError, TimeoutType, Timeouts,
        dropguard::DropGuard,
        hooks::Hooks,
        object::ObjectInner,
        slots::{Permit, Slots},
    },
};

/// Generic object and connection pool.
///
/// This struct can be cloned and transferred across thread boundaries and uses
/// reference counting for its internal state.
pub struct Pool<M: Manager, W: From<Object<M>> = Object<M>> {
    pub(crate) inner: Arc<PoolInner<M>>,
    pub(crate) _wrapper: PhantomData<fn() -> W>,
}

// Implemented manually to avoid unnecessary trait bound on `W` type parameter.
impl<M, W> fmt::Debug for Pool<M, W>
where
    M: fmt::Debug + Manager,
    M::Type: fmt::Debug,
    W: From<Object<M>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pool")
            .field("inner", &self.inner)
            .field("wrapper", &self._wrapper)
            .finish()
    }
}

impl<M: Manager, W: From<Object<M>>> Clone for Pool<M, W> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _wrapper: PhantomData,
        }
    }
}

impl<M: Manager, W: From<Object<M>>> Pool<M, W> {
    /// Instantiates a builder for a new [`Pool`].
    ///
    /// This is the only way to create a [`Pool`] instance.
    pub fn builder(manager: M) -> PoolBuilder<M, W> {
        PoolBuilder::new(manager)
    }

    pub(crate) fn from_builder(builder: PoolBuilder<M, W>) -> Self {
        Self {
            inner: Arc::new(PoolInner {
                manager: builder.manager,
                next_id: AtomicUsize::new(0),
                slots: Slots::new(builder.config.max_size),
                users: AtomicUsize::new(0),
                config: builder.config,
                hooks: builder.hooks,
                runtime: builder.runtime,
            }),
            _wrapper: PhantomData,
        }
    }

    /// Retrieves an [`Object`] from this [`Pool`] or waits for one to
    /// become available.
    ///
    /// # Errors
    ///
    /// See [`PoolError`] for details.
    pub async fn get(&self) -> Result<W, PoolError<M::Error>> {
        self.timeout_get(&self.timeouts()).await
    }

    /// Retrieves an [`Object`] from this [`Pool`] using a different `timeout`
    /// than the configured one.
    ///
    /// # Errors
    ///
    /// See [`PoolError`] for details.
    pub async fn timeout_get(&self, timeouts: &Timeouts) -> Result<W, PoolError<M::Error>> {
        let _ = self.inner.users.fetch_add(1, Ordering::Relaxed);
        let users_guard = DropGuard(|| {
            let _ = self.inner.users.fetch_sub(1, Ordering::Relaxed);
        });

        let non_blocking = match timeouts.wait {
            Some(t) => t.as_nanos() == 0,
            None => false,
        };

        let mut permit = if non_blocking {
            self.inner.slots.try_acquire().map_err(|e| match e {
                TryAcquireError::Closed => PoolError::Closed,
                TryAcquireError::NoPermits => PoolError::Timeout(TimeoutType::Wait),
            })?
        } else {
            apply_timeout(
                self.inner.runtime,
                TimeoutType::Wait,
                timeouts.wait,
                async {
                    self.inner
                        .slots
                        .acquire()
                        .await
                        .map_err(|_| PoolError::Closed)
                },
            )
            .await?
        };

        let inner_obj = loop {
            let inner_obj = if let Some(inner_obj) = permit.pop(self.inner.config.queue_mode) {
                self.try_recycle(timeouts, &mut permit, inner_obj).await?
            } else {
                self.try_create(timeouts, &mut permit).await?
            };
            if let Some(inner_obj) = inner_obj {
                break inner_obj;
            }
        };

        users_guard.disarm();
        permit.lease();

        Ok(Object {
            inner: Some(inner_obj),
            pool: self.weak(),
        }
        .into())
    }

    #[inline]
    async fn try_recycle(
        &self,
        timeouts: &Timeouts,
        permit: &mut Permit<'_, ObjectInner<M>>,
        inner_obj: ObjectInner<M>,
    ) -> Result<Option<ObjectInner<M>>, PoolError<M::Error>> {
        let mut unready_obj = UnreadyObject {
            inner: Some(inner_obj),
            permit,
            manager: &self.inner.manager,
        };
        let inner = unready_obj.inner();

        // Apply pre_recycle hooks
        if let Err(_e) = self.inner.hooks.pre_recycle.apply(inner).await {
            // TODO log pre_recycle error
            return Ok(None);
        }

        if apply_timeout(
            self.inner.runtime,
            TimeoutType::Recycle,
            timeouts.recycle,
            self.inner.manager.recycle(&mut inner.obj, &inner.metrics),
        )
        .await
        .is_err()
        {
            return Ok(None);
        }

        // Apply post_recycle hooks
        if let Err(_e) = self.inner.hooks.post_recycle.apply(inner).await {
            // TODO log post_recycle error
            return Ok(None);
        }

        inner.metrics.recycle_count += 1;
        #[cfg(not(target_arch = "wasm32"))]
        {
            inner.metrics.recycled = Some(Instant::now());
        }

        Ok(Some(unready_obj.ready()))
    }

    #[inline]
    async fn try_create(
        &self,
        timeouts: &Timeouts,
        permit: &mut Permit<'_, ObjectInner<M>>,
    ) -> Result<Option<ObjectInner<M>>, PoolError<M::Error>> {
        let inner_obj = ObjectInner {
            obj: apply_timeout(
                self.inner.runtime,
                TimeoutType::Create,
                timeouts.create,
                self.inner.manager.create(),
            )
            .await?,
            id: self.inner.next_id.fetch_add(1, Ordering::Relaxed),
            metrics: Metrics::default(),
        };
        if let Some(mut evicted) = permit.occupy() {
            self.inner.manager.detach(&mut evicted.obj);
        }
        let mut unready_obj = UnreadyObject {
            inner: Some(inner_obj),
            permit,
            manager: &self.inner.manager,
        };

        // Apply post_create hooks
        if let Err(e) = self
            .inner
            .hooks
            .post_create
            .apply(unready_obj.inner())
            .await
        {
            return Err(PoolError::PostCreateHook(e));
        }

        Ok(Some(unready_obj.ready()))
    }

    /**
     * Resize the pool. This change the `max_size` of the pool dropping
     * excess objects and/or making space for new ones.
     *
     * If the pool is closed this method does nothing. The [`Pool::status()`] method
     * always reports a `max_size` of 0 for closed pools.
     */
    pub fn resize(&self, max_size: usize) {
        drop(self.inner.slots.resize(max_size));
    }

    /// Retains only the objects specified by the given function.
    ///
    /// This function is typically used to remove objects from
    /// the pool based on their current state or metrics.
    ///
    /// **Caution:** This function blocks the entire pool while
    /// it is running. Therefore the given function should not
    /// block.
    ///
    /// The following example starts a background task that
    /// runs every 30 seconds and removes objects from the pool
    /// that haven't been used for more than one minute.
    ///
    /// ```rust,ignore
    /// let interval = Duration::from_secs(30);
    /// let max_age = Duration::from_secs(60);
    /// tokio::spawn(async move {
    ///     loop {
    ///         tokio::time::sleep(interval).await;
    ///         pool.retain(|_, metrics| metrics.last_used() < max_age);
    ///     }
    /// });
    /// ```
    pub fn retain(
        &self,
        mut predicate: impl FnMut(&M::Type, Metrics) -> bool,
    ) -> RetainResult<M::Type> {
        let (retained, removed) = self
            .inner
            .slots
            .retain(|obj| predicate(&obj.obj, obj.metrics));
        let removed = removed
            .into_iter()
            .map(|mut obj| {
                self.manager().detach(&mut obj.obj);
                obj.obj
            })
            .collect();
        RetainResult { retained, removed }
    }

    /// Get current timeout configuration
    pub fn timeouts(&self) -> Timeouts {
        self.inner.config.timeouts
    }

    /// Closes this [`Pool`].
    ///
    /// All current and future tasks waiting for [`Object`]s will return
    /// [`PoolError::Closed`] immediately.
    ///
    /// This operation resizes the pool to 0.
    pub fn close(&self) {
        drop(self.inner.slots.close());
    }

    /// Indicates whether this [`Pool`] has been closed.
    pub fn is_closed(&self) -> bool {
        self.inner.slots.is_closed()
    }

    /// Retrieves [`Status`] of this [`Pool`].
    #[must_use]
    pub fn status(&self) -> Status {
        let slots = self.inner.slots.status();
        let users = self.inner.users.load(Ordering::Relaxed);
        let (available, waiting) = if users < slots.size {
            (slots.size - users, 0)
        } else {
            (0, users - slots.size)
        };
        Status {
            max_size: slots.max_size,
            size: slots.size,
            available,
            waiting,
        }
    }

    /// Returns [`Manager`] of this [`Pool`].
    #[must_use]
    pub fn manager(&self) -> &M {
        &self.inner.manager
    }

    /// Returns a [`WeakPool<T>`] of this [`Pool`].
    pub fn weak(&self) -> WeakPool<M> {
        WeakPool {
            inner: Arc::downgrade(&self.inner),
            _wrapper: PhantomData,
        }
    }
}

/// A weak reference to a [`Pool<T>`], used to avoid keeping the pool alive.
///
/// `WeakPool<T>` is analogous to [`std::sync::Weak<T>`] for [`Pool<T>`], and
/// is typically used in situations where you need a non-owning reference to a pool,
/// such as in background tasks, managers, or callbacks that should not extend
/// the lifetime of the pool.
///
/// This allows components to retain a reference to the pool while avoiding
/// reference cycles or prolonging its lifetime unnecessarily.
///
/// To access the pool, use [`WeakPool::upgrade`] to attempt to get a strong reference.
#[derive(Debug)]
pub struct WeakPool<M: Manager, W: From<Object<M>> = Object<M>> {
    inner: Weak<PoolInner<M>>,
    _wrapper: PhantomData<fn() -> W>,
}

impl<M: Manager, W: From<Object<M>>> WeakPool<M, W> {
    /// Attempts to upgrade the `WeakPool` to a strong [`Pool<T>`] reference.
    ///
    /// If the pool has already been dropped (i.e., no strong references remain),
    /// this returns `None`.
    pub fn upgrade(&self) -> Option<Pool<M, W>> {
        Some(Pool {
            inner: self.inner.upgrade()?,
            _wrapper: PhantomData,
        })
    }
}

pub(crate) struct PoolInner<M: Manager> {
    manager: M,
    next_id: AtomicUsize,
    slots: Slots<ObjectInner<M>>,
    /// Number of [`Pool`] users. A user is both a future which is waiting for an [`Object`] or one
    /// with an [`Object`] which hasn't been returned, yet.
    users: AtomicUsize,
    config: PoolConfig,
    runtime: Option<Runtime>,
    hooks: Hooks<M>,
}

// Implemented manually to avoid unnecessary trait bound on the struct.
impl<M> fmt::Debug for PoolInner<M>
where
    M: fmt::Debug + Manager,
    M::Type: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PoolInner")
            .field("manager", &self.manager)
            .field("slots", &self.slots)
            .field("used", &self.users)
            .field("config", &self.config)
            .field("runtime", &self.runtime)
            .field("hooks", &self.hooks)
            .finish()
    }
}

impl<M: Manager> PoolInner<M> {
    pub(crate) fn return_object(&self, inner: ObjectInner<M>) {
        let _ = self.users.fetch_sub(1, Ordering::Relaxed);
        if let Some(mut inner) = self.slots.release(inner) {
            self.manager.detach(&mut inner.obj);
        }
    }
    pub(crate) fn detach_object(&self, obj: &mut M::Type) {
        let _ = self.users.fetch_sub(1, Ordering::Relaxed);
        self.slots.remove();
        self.manager.detach(obj);
    }
}

/// An object which is about to be handed out by the pool. If it's dropped
/// before being [ready](UnreadyObject::ready) the object is discarded and
/// its slot vacated.
struct UnreadyObject<'a, 'b, M: Manager> {
    inner: Option<ObjectInner<M>>,
    permit: &'a mut Permit<'b, ObjectInner<M>>,
    manager: &'a M,
}

impl<M: Manager> UnreadyObject<'_, '_, M> {
    fn ready(mut self) -> ObjectInner<M> {
        self.inner.take().unwrap()
    }
    fn inner(&mut self) -> &mut ObjectInner<M> {
        self.inner.as_mut().unwrap()
    }
}

impl<M: Manager> Drop for UnreadyObject<'_, '_, M> {
    fn drop(&mut self) {
        if let Some(mut inner) = self.inner.take() {
            self.permit.vacate();
            self.manager.detach(&mut inner.obj);
        }
    }
}

async fn apply_timeout<O, E>(
    runtime: Option<Runtime>,
    timeout_type: TimeoutType,
    duration: Option<Duration>,
    future: impl Future<Output = Result<O, impl Into<PoolError<E>>>>,
) -> Result<O, PoolError<E>> {
    match (runtime, duration) {
        (_, None) => future.await.map_err(Into::into),
        (Some(runtime), Some(duration)) => timeout(runtime, duration, future)
            .await
            .ok_or(PoolError::Timeout(timeout_type))?
            .map_err(Into::into),
        (None, Some(_)) => Err(PoolError::NoRuntimeSpecified),
    }
}

#[derive(Debug)]
/// This is the result returned by `Pool::retain`
pub struct RetainResult<T> {
    /// Number of retained objects
    pub retained: usize,
    /// Objects that were removed from the pool
    pub removed: Vec<T>,
}

impl<T> Default for RetainResult<T> {
    fn default() -> Self {
        Self {
            retained: Default::default(),
            removed: Default::default(),
        }
    }
}

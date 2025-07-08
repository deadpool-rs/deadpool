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

pub mod reexports;

use std::{
    any::Any,
    fmt,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, MutexGuard, PoisonError, TryLockError},
};

use deadpool_runtime::{Runtime, SpawnBlockingError};

/// Possible errors returned when [`SyncWrapper::interact()`] fails.
#[derive(Debug)]
pub enum InteractError {
    /// Provided callback has panicked.
    Panic(Box<dyn Any + Send + 'static>),

    /// Callback was aborted. This variant needs to exist for technical
    /// reasons but you should never actually be able to get this as a
    /// return value when calling `SyncWrapper::interact`.
    Aborted,
}

impl fmt::Display for InteractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Panic(_) => write!(f, "Panic"),
            Self::Aborted => write!(f, "Aborted"),
        }
    }
}

impl std::error::Error for InteractError {}

/// Wrapper for objects which only provides blocking functions that need to be
/// called on a separate thread.
///
/// Access to the wrapped object is provided via the [`SyncWrapper::interact()`]
/// method.
#[must_use]
pub struct SyncWrapper<T>
where
    T: Send + 'static,
{
    obj: Arc<Mutex<Option<T>>>,
    runtime: Runtime,
}

// Implemented manually to avoid unnecessary trait bound on `E` type parameter.
impl<T> fmt::Debug for SyncWrapper<T>
where
    T: fmt::Debug + Send + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyncWrapper")
            .field("obj", &self.obj)
            .field("runtime", &self.runtime)
            .finish()
    }
}

impl<T> SyncWrapper<T>
where
    T: Send + 'static,
{
    /// Creates a new wrapped object.
    pub async fn new<F, E>(runtime: Runtime, f: F) -> Result<Self, E>
    where
        F: FnOnce() -> Result<T, E> + Send + 'static,
        E: Send + 'static,
    {
        let result = match runtime.spawn_blocking(f).await {
            // FIXME: Panicking when the creation panics is not nice.
            // In order to handle this properly the Manager::create
            // methods needs to support a custom error enum which
            // supports a Panic variant.
            Err(SpawnBlockingError::Panic(e)) => panic!("{e:?}"),
            Ok(obj) => obj,
        };
        result.map(|obj| Self {
            obj: Arc::new(Mutex::new(Some(obj))),
            runtime,
        })
    }

    /// Interacts with the underlying object.
    ///
    /// Expects a closure that takes the object as its parameter.
    /// The closure is executed in a separate thread so that the async runtime
    /// is not blocked.
    pub async fn interact<F, R>(&self, f: F) -> Result<R, InteractError>
    where
        F: FnOnce(&mut T) -> R + Send + 'static,
        R: Send + 'static,
    {
        let arc = self.obj.clone();
        #[cfg(feature = "tracing")]
        let span = tracing::Span::current();
        self.runtime
            .spawn_blocking(move || {
                let mut guard = arc.lock().unwrap();
                let conn: &mut T = guard.as_mut().ok_or(InteractError::Aborted)?;
                #[cfg(feature = "tracing")]
                let _span = span.enter();
                Ok(f(conn))
            })
            .await
            .map_err(|SpawnBlockingError::Panic(p)| InteractError::Panic(p))?
    }

    /// Indicates whether the underlying [`Mutex`] has been poisoned.
    ///
    /// This happens when a panic occurs while interacting with the object.
    pub fn is_mutex_poisoned(&self) -> bool {
        self.obj.is_poisoned()
    }

    /// Lock the underlying mutex and return a guard for the inner
    /// object.
    pub fn lock(&self) -> Result<SyncGuard<'_, T>, PoisonError<MutexGuard<'_, Option<T>>>> {
        self.obj.lock().map(SyncGuard)
    }

    /// Try to lock the underlying mutex and return a guard for the
    /// inner object.
    pub fn try_lock(&self) -> Result<SyncGuard<'_, T>, TryLockError<MutexGuard<'_, Option<T>>>> {
        self.obj.try_lock().map(SyncGuard)
    }
}

impl<T> Drop for SyncWrapper<T>
where
    T: Send + 'static,
{
    fn drop(&mut self) {
        let arc = self.obj.clone();
        // Drop the `rusqlite::Connection` inside a `spawn_blocking`
        // as the `drop` function of it can block.
        self.runtime
            .spawn_blocking_background(move || match arc.lock() {
                Ok(mut guard) => drop(guard.take()),
                Err(e) => drop(e.into_inner().take()),
            })
            .unwrap();
    }
}

/// This guard is returned when calling `SyncWrapper::lock` or
/// `SyncWrapper::try_lock`. This is basicly just a wrapper around
/// a `MutexGuard` but hides some implementation details.
///
/// **Important:** Any blocking operation using this object
/// should be executed on a separate thread (e.g. via `spawn_blocking`).
#[derive(Debug)]
pub struct SyncGuard<'a, T: Send>(MutexGuard<'a, Option<T>>);

impl<T: Send> Deref for SyncGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl<T: Send> DerefMut for SyncGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}

impl<T: Send> AsRef<T> for SyncGuard<'_, T> {
    fn as_ref(&self) -> &T {
        self.0.as_ref().unwrap()
    }
}

impl<T: Send> AsMut<T> for SyncGuard<'_, T> {
    fn as_mut(&mut self) -> &mut T {
        self.0.as_mut().unwrap()
    }
}

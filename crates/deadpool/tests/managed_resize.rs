#![cfg(feature = "managed")]

use std::{convert::Infallible, time::Duration};

use deadpool::managed::{self, Metrics, Object, RecycleResult, Timeouts};

type Pool = managed::Pool<Manager, Object<Manager>>;

struct Manager {}

impl managed::Manager for Manager {
    type Type = ();
    type Error = Infallible;

    async fn create(&self) -> Result<(), Infallible> {
        Ok(())
    }

    async fn recycle(&self, _conn: &mut (), _: &Metrics) -> RecycleResult<Infallible> {
        Ok(())
    }
}

// Regression test for https://github.com/deadpool-rs/deadpool/issues/380
#[tokio::test]
async fn test_grow_reuse_existing() {
    // Shrink doesn't discard objects currently borrowed from the pool but
    // keeps track of them so that repeatedly growing and shrinking will
    // not cause excessive object creation. This logic used to contain a bug
    // causing an overflow.
    let mgr = Manager {};
    let pool = Pool::builder(mgr).max_size(2).build().unwrap();
    let obj1 = pool.get().await.unwrap();
    let obj2 = pool.get().await.unwrap();
    assert!(pool.status().size == 2);
    assert!(pool.status().max_size == 2);
    pool.resize(0);
    // At this point the two objects are still tracked
    assert!(pool.status().size == 2);
    assert!(pool.status().max_size == 0);
    pool.resize(1);
    // Only one of the objects should be returned to the pool
    assert!(pool.status().size == 2);
    assert!(pool.status().max_size == 1);
    drop(obj1);
    // The first drop brings the size to 1.
    assert!(pool.status().size == 1);
    assert!(pool.status().max_size == 1);
    drop(obj2);
    assert!(pool.status().size == 1);
    assert!(pool.status().max_size == 1);
}

#[tokio::test]
async fn resize_pool_shrink() {
    let mgr = Manager {};
    let pool = Pool::builder(mgr).max_size(2).build().unwrap();
    let obj0 = pool.get().await.unwrap();
    let obj1 = pool.get().await.unwrap();
    pool.resize(1);
    assert_eq!(pool.status().max_size, 1);
    assert_eq!(pool.status().size, 2);
    drop(obj1);
    assert_eq!(pool.status().max_size, 1);
    assert_eq!(pool.status().size, 1);
    drop(obj0);
    assert_eq!(pool.status().max_size, 1);
    assert_eq!(pool.status().size, 1);
}

#[tokio::test]
async fn resize_pool_grow() {
    let mgr = Manager {};
    let pool = Pool::builder(mgr).max_size(1).build().unwrap();
    let obj0 = pool.get().await.unwrap();
    pool.resize(2);
    assert_eq!(pool.status().max_size, 2);
    assert_eq!(pool.status().size, 1);
    let obj1 = pool.get().await.unwrap();
    assert_eq!(pool.status().max_size, 2);
    assert_eq!(pool.status().size, 2);
    drop(obj1);
    assert_eq!(pool.status().max_size, 2);
    assert_eq!(pool.status().size, 2);
    drop(obj0);
    assert_eq!(pool.status().max_size, 2);
    assert_eq!(pool.status().size, 2);
}

#[tokio::test]
async fn resize_pool_shrink_grow() {
    let mgr = Manager {};
    let pool = Pool::builder(mgr).max_size(1).build().unwrap();
    let obj0 = pool.get().await.unwrap();
    pool.resize(2);
    pool.resize(0);
    pool.resize(5);
    assert_eq!(pool.status().max_size, 5);
    assert_eq!(pool.status().size, 1);
    drop(obj0);
    assert_eq!(pool.status().max_size, 5);
    assert_eq!(pool.status().size, 1);
}

#[tokio::test]
async fn resize_pool_grow_concurrent() {
    let mgr = Manager {};
    let pool = Pool::builder(mgr).max_size(0).build().unwrap();
    let join_handle = {
        let pool = pool.clone();
        tokio::spawn(async move { pool.get().await })
    };
    tokio::task::yield_now().await;
    assert_eq!(pool.status().max_size, 0);
    assert_eq!(pool.status().size, 0);
    assert_eq!(pool.status().available, 0);
    assert_eq!(pool.status().waiting, 1);
    pool.resize(1);
    assert_eq!(pool.status().max_size, 1);
    assert_eq!(pool.status().size, 0);
    assert_eq!(pool.status().available, 0);
    assert_eq!(pool.status().waiting, 1);
    tokio::task::yield_now().await;
    assert_eq!(pool.status().max_size, 1);
    assert_eq!(pool.status().size, 1);
    assert_eq!(pool.status().available, 0);
    assert_eq!(pool.status().waiting, 0);
    let obj0 = join_handle.await.unwrap().unwrap();
    assert_eq!(pool.status().max_size, 1);
    assert_eq!(pool.status().size, 1);
    assert_eq!(pool.status().available, 0);
    assert_eq!(pool.status().waiting, 0);
    drop(obj0);
    assert_eq!(pool.status().max_size, 1);
    assert_eq!(pool.status().size, 1);
    assert_eq!(pool.status().available, 1);
    assert_eq!(pool.status().waiting, 0);
}

#[tokio::test]
async fn close_resize() {
    let mgr = Manager {};
    let pool = Pool::builder(mgr).max_size(1).build().unwrap();
    pool.close();
    pool.resize(16);
    assert_eq!(pool.status().size, 0);
    assert_eq!(pool.status().max_size, 0);
}

/// Returns the number of objects that can be acquired from the pool
/// without waiting. All acquired objects are returned to the pool
/// before this function returns.
async fn acquirable(pool: &Pool) -> usize {
    let timeouts = Timeouts {
        wait: Some(Duration::ZERO),
        ..Timeouts::default()
    };
    let mut objs = Vec::new();
    while let Ok(obj) = pool.timeout_get(&timeouts).await {
        objs.push(obj);
    }
    objs.len()
}

// Regression test for https://github.com/deadpool-rs/deadpool/issues/504
#[tokio::test]
async fn resize_shrink_empty_pool() {
    let mgr = Manager {};
    let pool = Pool::builder(mgr).max_size(5).build().unwrap();
    pool.resize(0);
    assert_eq!(acquirable(&pool).await, 0);
    assert_eq!(pool.status().size, 0);
    pool.resize(2);
    assert_eq!(acquirable(&pool).await, 2);
    assert_eq!(pool.status().size, 2);
}

// Regression test for https://github.com/deadpool-rs/deadpool/issues/504
#[tokio::test]
async fn resize_shrink_partially_filled_pool() {
    let mgr = Manager {};
    let pool = Pool::builder(mgr).max_size(5).build().unwrap();
    drop((pool.get().await.unwrap(), pool.get().await.unwrap()));
    assert_eq!(pool.status().size, 2);
    pool.resize(1);
    assert_eq!(pool.status().size, 1);
    assert_eq!(acquirable(&pool).await, 1);
    assert_eq!(pool.status().size, 1);
}

// Regression test for https://github.com/deadpool-rs/deadpool/issues/504
#[tokio::test]
async fn resize_shrink_partially_filled_pool_with_borrowed_objects() {
    let mgr = Manager {};
    let pool = Pool::builder(mgr).max_size(5).build().unwrap();
    let obj0 = pool.get().await.unwrap();
    let obj1 = pool.get().await.unwrap();
    pool.resize(1);
    assert_eq!(acquirable(&pool).await, 0);
    drop(obj0);
    assert_eq!(acquirable(&pool).await, 0);
    drop(obj1);
    assert_eq!(acquirable(&pool).await, 1);
    assert_eq!(pool.status().size, 1);
}

#[tokio::test]
async fn resize_shrink_grow_with_borrowed_objects() {
    let mgr = Manager {};
    let pool = Pool::builder(mgr).max_size(2).build().unwrap();
    let obj0 = pool.get().await.unwrap();
    let obj1 = pool.get().await.unwrap();
    pool.resize(0);
    pool.resize(1);
    assert_eq!(acquirable(&pool).await, 0);
    drop(obj0);
    assert_eq!(acquirable(&pool).await, 0);
    drop(obj1);
    assert_eq!(acquirable(&pool).await, 1);
    assert_eq!(pool.status().size, 1);
}

#[tokio::test]
async fn resize_shrink_grow_empty_pool() {
    let mgr = Manager {};
    let pool = Pool::builder(mgr).max_size(5).build().unwrap();
    pool.resize(1);
    pool.resize(3);
    assert_eq!(acquirable(&pool).await, 3);
    assert_eq!(pool.status().size, 3);
}

mod failing {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    use deadpool::managed::{self, Metrics, RecycleResult};
    use tokio::sync::Notify;

    /// Manager whose object creation fails. The first creation blocks
    /// until `gate` is notified.
    pub(super) struct Manager {
        pub(super) gate: Arc<Notify>,
        pub(super) block: AtomicBool,
    }

    impl managed::Manager for Manager {
        type Type = ();
        type Error = ();

        async fn create(&self) -> Result<(), ()> {
            if self.block.swap(false, Ordering::SeqCst) {
                self.gate.notified().await;
            }
            Err(())
        }

        async fn recycle(&self, _: &mut (), _: &Metrics) -> RecycleResult<()> {
            Ok(())
        }
    }
}

// Regression test for https://github.com/deadpool-rs/deadpool/issues/504
#[tokio::test]
async fn resize_shrink_while_create_fails() {
    use std::sync::{Arc, atomic::AtomicBool};

    use deadpool::managed::{PoolError, TimeoutType};

    let gate = Arc::new(tokio::sync::Notify::new());
    let mgr = failing::Manager {
        gate: gate.clone(),
        block: AtomicBool::new(true),
    };
    let pool = managed::Pool::<failing::Manager>::builder(mgr)
        .max_size(2)
        .build()
        .unwrap();
    let join_handle = {
        let pool = pool.clone();
        tokio::spawn(async move { pool.get().await.map(drop) })
    };
    // Wait until the spawned task is blocked in `Manager::create`
    while pool.status().waiting == 0 {
        tokio::task::yield_now().await;
    }
    pool.resize(0);
    gate.notify_one();
    assert!(join_handle.await.unwrap().is_err());
    let timeouts = Timeouts {
        wait: Some(Duration::ZERO),
        ..Timeouts::default()
    };
    assert!(matches!(
        pool.timeout_get(&timeouts).await,
        Err(PoolError::Timeout(TimeoutType::Wait))
    ));
}

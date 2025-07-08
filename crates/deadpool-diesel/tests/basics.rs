#![cfg(feature = "sqlite")]

use tokio::sync::mpsc;

use deadpool_diesel::{
    sqlite::{Hook, HookError, Manager, Metrics, Pool, PoolError, Runtime},
    InteractError,
};

fn create_pool(max_size: usize) -> Pool {
    let manager = Manager::new(":memory:", Runtime::Tokio1);
    Pool::builder(manager).max_size(max_size).build().unwrap()
}

#[tokio::test]
async fn establish_basic_connection() {
    let pool = create_pool(2);

    let (s1, mut r1) = mpsc::channel(1);
    let (s2, mut r2) = mpsc::channel(1);

    let pool1 = pool.clone();
    let t1 = tokio::spawn(async move {
        let conn = pool1.get().await.unwrap();
        s1.send(()).await.unwrap();
        r2.recv().await.unwrap();
        drop(conn)
    });

    let pool2 = pool.clone();
    let t2 = tokio::spawn(async move {
        let conn = pool2.get().await.unwrap();
        s2.send(()).await.unwrap();
        r1.recv().await.unwrap();
        drop(conn)
    });

    t1.await.unwrap();
    t2.await.unwrap();

    drop(pool.get().await.unwrap());
}

#[tokio::test]
async fn pooled_connection_impls_connection() {
    use diesel::prelude::*;
    use diesel::select;
    use diesel::sql_types::Text;

    let pool = create_pool(1);
    let conn_result: Result<_, PoolError> = pool.get().await;
    let conn = conn_result.unwrap();
    let result: Result<Result<String, diesel::result::Error>, InteractError> = conn
        .interact(|conn| {
            let query = select("foo".into_sql::<Text>());
            query.get_result::<String>(conn)
        })
        .await;
    assert_eq!("foo", &result.unwrap().unwrap());
}

#[tokio::test]
async fn lock() {
    use diesel::prelude::*;
    use diesel::select;
    use diesel::sql_types::Text;

    let pool = create_pool(1);
    let wrapper = pool.get().await.unwrap();
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = wrapper.try_lock().unwrap();
        let query = select("foo".into_sql::<Text>());
        query.get_result::<String>(&mut *conn)
    })
    .await
    .unwrap()
    .unwrap();
    assert_eq!("foo", &result);
}

#[tokio::test]
async fn hooks() {
    let manager = Manager::new(":memory:", Runtime::Tokio1);
    Pool::builder(manager)
        .post_create(Hook::sync_fn(|_conn, _metrics: &Metrics| {
            Err(HookError::message("This is a static message"))
        }))
        .build()
        .unwrap();
}

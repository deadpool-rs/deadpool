//! Basic tests for deadpool-memcached
use std::env;

use async_memcached::AsciiProtocol;
use deadpool_memcached::{Manager, Pool};

fn create_pool() -> Option<Pool> {
    let addr = env::var("MEMCACHED__ADDR").ok()?;
    let manager = Manager::new(addr);
    Some(Pool::builder(manager).build().unwrap())
}

#[tokio::test]
async fn test_set_get() {
    let Some(pool) = create_pool() else {
        // Skip test when no Memcached server is configured.
        return;
    };
    let test_key = "test:basic:test_set_get";
    let test_value = "answer_42";
    let mut conn = pool.get().await.unwrap();
    let _ = conn.delete(test_key).await;
    assert_eq!(conn.get(test_key).await.unwrap(), None);
    conn.set(test_key, test_value, None, None).await.unwrap();
    let value =
        String::from_utf8(conn.get(test_key).await.unwrap().unwrap().data.unwrap()).unwrap();
    assert_eq!(value, test_value);
}

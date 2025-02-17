//! Basic tests for deadpool-memcached

use std::env;

use deadpool_memcached::{Manager, Pool};

fn create_pool() -> Pool {
    let addr = env::var("MEMCACHED__ADDR").unwrap();
    let manager = Manager::new(addr);
    Pool::builder(manager).build().unwrap()
}

#[tokio::test]
async fn test_set_get() {
    let test_key = "test:basic:test_set_get";
    let test_value = "answer_42";
    let pool = create_pool();
    let mut conn = pool.get().await.unwrap();
    let _ = conn.delete(test_key).await;
    assert_eq!(conn.get(test_key).await.unwrap(), None);
    conn.set(test_key, test_value, None, None).await.unwrap();
    let value = String::from_utf8(conn.get(test_key).await.unwrap().unwrap().data).unwrap();
    assert_eq!(value, test_value);
}

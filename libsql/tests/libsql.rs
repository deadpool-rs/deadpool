use deadpool_libsql::{Manager, Pool};
use libsql::params;

async fn create_pool() -> Pool {
    let database = deadpool_libsql::libsql::Builder::new_local("libsql.db")
        .build()
        .await
        .unwrap();
    let manager = Manager::new(database);
    Pool::builder(manager).build().unwrap()
}

#[tokio::test]
async fn basic() {
    let pool = create_pool().await;
    let conn = pool.get().await.unwrap();

    let mut stmt = conn.prepare("SELECT 1").await.unwrap();
    let mut rows = stmt.query(params![]).await.unwrap();
    let row = rows.next().await.unwrap().unwrap();
    let result: i64 = row.get(0).unwrap();

    assert_eq!(result, 1);
}

use deadpool_sqlite::{Config, InteractError, Pool, Runtime};

fn create_pool() -> Pool {
    let cfg = Config {
        path: "db.sqlite3".into(),
        pool: None,
    };
    cfg.create_pool(Runtime::Tokio1).unwrap()
}

#[tokio::test]
async fn basic() {
    let pool = create_pool();
    let conn = pool.get().await.unwrap();
    let result: i64 = conn
        .interact(|conn| {
            let mut stmt = conn.prepare("SELECT 1")?;
            let mut rows = stmt.query([])?;
            let row = rows.next()?.unwrap();
            row.get(0)
        })
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result, 1);
}

#[tokio::test]
async fn panic() {
    let pool = create_pool();
    {
        let conn = pool.get().await.unwrap();
        let result = conn
            .interact::<_, ()>(|_| {
                panic!("Whopsies!");
            })
            .await;
        assert!(matches!(result, Err(InteractError::Panic(_))))
    }
    // The previous callback panicked. The pool should recover from this.
    let conn = pool.get().await.unwrap();
    let result: i64 = conn
        .interact(|conn| {
            let mut stmt = conn.prepare("SELECT 1").unwrap();
            let mut rows = stmt.query([]).unwrap();
            let row = rows.next().unwrap().unwrap();
            row.get(0)
        })
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result, 1);
}

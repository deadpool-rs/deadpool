use libsql::params;

#[cfg(feature = "core")]
async fn create_pool() -> deadpool_libsql::Pool {
    let database = deadpool_libsql::libsql::Builder::new_local("libsql.db")
        .build()
        .await
        .unwrap();
    let manager = deadpool_libsql::Manager::from_libsql_database(database);
    deadpool_libsql::Pool::builder(manager).build().unwrap()
}

#[tokio::test]
#[cfg(feature = "core")]
async fn basic() {
    let pool = create_pool().await;
    let conn = pool.get().await.unwrap();

    let mut stmt = conn.prepare("SELECT 1").await.unwrap();
    let mut rows = stmt.query(params![]).await.unwrap();
    let row = rows.next().await.unwrap().unwrap();
    let result: i64 = row.get(0).unwrap();

    assert_eq!(result, 1);
}

/// This test makes sure that the connection fails when trying to
/// get a connection from the pool and not at pool creation time.
#[tokio::test]
#[cfg(feature = "core")]
async fn fail_at_connect_to_local() {
    use deadpool_libsql::config::OpenFlags;
    let config = deadpool_libsql::config::Config {
        database: deadpool_libsql::config::Database::Local(deadpool_libsql::config::Local {
            path: "/does-not-exist.db".into(),
            encryption_config: None,
            flags: Some(OpenFlags {
                create: false,
                read_only: false,
                read_write: true,
            }),
        }),
        pool: deadpool_libsql::PoolConfig::default(),
    };
    let pool = config.create_pool(None).await.unwrap();
    let result = pool.get().await;
    assert!(
        result.is_err(),
        "Connection unexpectedly established: {:?}",
        result.unwrap()
    );
}

/// This test makes sure that the connection fails when trying to
/// get a connection from the pool and not at pool creation time.
#[cfg(feature = "remote")]
#[tokio::test]
async fn fail_at_connect_to_remote() {
    let config = deadpool_libsql::config::Config {
        database: deadpool_libsql::config::Database::Remote(deadpool_libsql::config::Remote {
            url: "http://invalid-hostname.example.com:1337".into(),
            auth_token: "nothing here".into(),
            namespace: None,
            remote_encryption: None,
        }),
        pool: deadpool_libsql::PoolConfig::default(),
    };
    let pool = config.create_pool(None).await.unwrap();
    let result = pool.get().await;
    assert!(
        result.is_err(),
        "Connection unexpectedly established: {:?}",
        result.unwrap()
    );
}

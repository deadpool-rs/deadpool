use deadpool::managed::{Manager, Metrics, Pool, RecycleResult};
use deadpool_runtime::Runtime;
use deadpool_sync::SyncWrapper;

struct Computer {
    pub answer: usize,
}

struct ComputerManager {}

impl Manager for ComputerManager {
    type Type = SyncWrapper<Computer>;
    type Error = ();

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        SyncWrapper::new(Runtime::Tokio1, || Ok(Computer { answer: 42 })).await
    }

    async fn recycle(
        &self,
        _obj: &mut Self::Type,
        _metrics: &Metrics,
    ) -> RecycleResult<Self::Error> {
        Ok(())
    }
}

#[tokio::test]
async fn post_recycle() {
    let manager = ComputerManager {};
    let pool = Pool::<ComputerManager>::builder(manager)
        .max_size(1)
        .build()
        .unwrap();
    let obj = pool.get().await.unwrap();
    assert_eq!(obj.interact(|computer| computer.answer).await.unwrap(), 42);
    let guard = obj.lock().unwrap();
    assert_eq!(guard.answer, 42);
}

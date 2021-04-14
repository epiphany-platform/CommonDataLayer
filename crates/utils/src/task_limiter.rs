use std::future::Future;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Clone)]
pub struct TaskLimiter {
    semaphore: Arc<Semaphore>,
    limit: usize,
}

impl std::fmt::Debug for TaskLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskLimiter")
            .field("limit", &self.limit)
            .finish()
    }
}

impl TaskLimiter {
    pub fn new(limit: usize) -> TaskLimiter {
        Self {
            semaphore: Arc::new(Semaphore::new(limit)),
            limit,
        }
    }

    pub async fn run<Fun, Fut>(&self, task: Fun)
    where
        Fun: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let semaphore = Arc::clone(&self.semaphore);
        let permit = semaphore.acquire_owned().await;

        tokio::spawn(async move {
            let _permit = permit;
            task().await
        });
    }
}

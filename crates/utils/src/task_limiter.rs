use std::future::Future;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Clone)]
pub struct TaskLimiter {
    semaphore: Arc<Semaphore>,
}

impl TaskLimiter {
    pub fn new(limit: usize) -> TaskLimiter {
        Self {
            semaphore: Arc::new(Semaphore::new(limit)),
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

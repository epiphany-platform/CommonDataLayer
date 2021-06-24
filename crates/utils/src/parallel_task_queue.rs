use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

use tokio::sync::oneshot::{channel, Receiver, Sender};

use misc_utils::abort_on_poison;

type LocksDB = Arc<Mutex<HashMap<String, LockQueue>>>;

pub struct ParallelTaskQueue {
    locks: LocksDB,
}

impl ParallelTaskQueue {
    pub async fn acquire_permit(&self, key: String) -> LockGuard {
        let receiver = {
            let mut locks = self.locks.lock().unwrap();
            let lock_queue = LockQueue::new();
            let queue = (*locks).entry(key.clone()).or_insert(lock_queue);
            queue.add_task()
        };
        receiver.await.unwrap();

        LockGuard {
            db: self.locks.clone(),
            key,
        }
    }

    pub fn new() -> ParallelTaskQueue {
        ParallelTaskQueue {
            locks: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for ParallelTaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LockQueue {
    is_blocked: bool,
    queue: VecDeque<Sender<()>>,
}

impl LockQueue {
    fn new() -> LockQueue {
        LockQueue {
            is_blocked: false,
            queue: VecDeque::new(),
        }
    }

    fn add_task(&mut self) -> Receiver<()> {
        let (tx, rx) = channel();
        match self.is_blocked {
            true => {
                self.queue.push_back(tx);
            }
            false => {
                self.is_blocked = true;
                tx.send(()).expect("Receiver dropped");
            }
        };
        rx
    }

    fn end_task(&mut self) {
        match self.queue.pop_front() {
            None => {
                self.is_blocked = false;
            }
            Some(tx) => {
                let result = tx.send(());
                if result.is_err() {
                    // receiver dropped, start next task
                    self.end_task()
                }
            }
        }
    }
}

pub struct LockGuard {
    db: LocksDB,
    key: String,
}
impl Drop for LockGuard {
    fn drop(&mut self) {
        let mut guard = self.db.lock().unwrap_or_else(abort_on_poison);
        let entry = (*guard)
            .get_mut(&self.key)
            .expect("LockGuard already dropped");
        entry.end_task();
        if !entry.is_blocked {
            (*guard).remove(&self.key);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::atomic::{AtomicU32, Ordering},
        time::Duration,
    };

    use tokio::{
        sync::{oneshot, Barrier},
        time::sleep,
        try_join,
    };

    use super::*;

    async fn wait_for_barrier(
        wait_barrier: Arc<Barrier>,
        task_queue: Arc<ParallelTaskQueue>,
        lock_key: String,
    ) {
        let _guard = task_queue.acquire_permit(lock_key).await;
        wait_barrier.wait().await;
    }

    async fn return_task_execution_order(
        waiter: Receiver<()>,
        task_queue: Arc<ParallelTaskQueue>,
        lock_key: String,
        counter: Arc<AtomicU32>,
    ) -> u32 {
        let _guard = task_queue.acquire_permit(lock_key).await;
        waiter.await.unwrap();
        counter.fetch_add(1, Ordering::SeqCst)
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore)] // `Miri` doesn't support kqueue() syscall in this test
    async fn should_process_unrelated_tasks_in_parallel() -> anyhow::Result<()> {
        let task_queue = Arc::new(ParallelTaskQueue::new());
        let barrier = Arc::new(Barrier::new(2));
        let first_task = tokio::spawn(wait_for_barrier(
            barrier.clone(),
            task_queue.clone(),
            "A".to_string(),
        ));
        let second_task = tokio::spawn(wait_for_barrier(
            barrier.clone(),
            task_queue.clone(),
            "B".to_string(),
        ));
        sleep(Duration::from_millis(200)).await;
        try_join!(first_task, second_task)?;
        Ok(())
    }
    #[tokio::test]
    #[cfg_attr(miri, ignore)] // `Miri` doesn't support kqueue() syscall in this test
    async fn should_wait_for_related_task_to_finish() -> anyhow::Result<()> {
        let task_queue = Arc::new(ParallelTaskQueue::new());
        let (tx1, rx1) = oneshot::channel();
        let (tx2, rx2) = oneshot::channel();
        let (tx3, rx3) = oneshot::channel();
        let counter = Arc::new(AtomicU32::new(0));
        let first_task = tokio::spawn(return_task_execution_order(
            rx1,
            task_queue.clone(),
            "A".to_string(),
            counter.clone(),
        ));
        let second_task = tokio::spawn(return_task_execution_order(
            rx2,
            task_queue.clone(),
            "A".to_string(),
            counter.clone(),
        ));
        let third_task = tokio::spawn(return_task_execution_order(
            rx3,
            task_queue.clone(),
            "A".to_string(),
            counter.clone(),
        ));
        tx3.send(()).unwrap();
        sleep(Duration::from_millis(200)).await;
        tx2.send(()).unwrap();
        sleep(Duration::from_millis(200)).await;
        tx1.send(()).unwrap();
        let results = try_join!(first_task, second_task, third_task)?;
        assert_eq!(3, counter.load(Ordering::SeqCst));
        assert_eq!(0, results.0);
        assert_eq!(1, results.1);
        assert_eq!(2, results.2);
        Ok(())
    }
    #[tokio::test]
    #[cfg_attr(miri, ignore)] // `Miri` doesn't support kqueue() syscall in this test
    async fn should_clean_up_after_task_group_is_done() {
        let task_queue = Arc::new(ParallelTaskQueue::new());
        {
            let _guard = task_queue.acquire_permit("A".to_string()).await;
            // DO SOME WORK
        }
        assert_eq!(
            task_queue.locks.lock().unwrap().len(),
            0,
            "Lock key was not removed from parallel task queue"
        );
    }
}

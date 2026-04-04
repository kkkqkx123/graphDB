use crate::sync::batch::BatchProcessor;
use crate::sync::queue::SyncTaskQueue;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

pub struct SyncScheduler {
    batch_processor: Arc<Mutex<BatchProcessor>>,
    queue: Arc<SyncTaskQueue>,
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl SyncScheduler {
    pub fn new(batch_processor: Arc<Mutex<BatchProcessor>>, queue: Arc<SyncTaskQueue>) -> Self {
        Self {
            batch_processor,
            queue,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub fn start(&self) -> tokio::task::JoinHandle<()> {
        let batch_processor = self.batch_processor.clone();
        let running = self.running.clone();

        running.store(true, std::sync::atomic::Ordering::SeqCst);

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(100));

            while running.load(std::sync::atomic::Ordering::SeqCst) {
                ticker.tick().await;

                let mut processor = batch_processor.lock().await;
                let keys: Vec<_> = processor.buffers.keys().cloned().collect();

                for key in keys {
                    if processor.should_commit(&key) {
                        if let Err(e) = processor.commit_batch(key).await {
                            log::error!("Batch commit failed: {:?}", e);
                        }
                    }
                }
            }
        })
    }

    pub fn stop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

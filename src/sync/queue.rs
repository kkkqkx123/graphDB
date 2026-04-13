//! Asynchronous Task Queue
//!
//! Provide asynchronous batch task processing capability, supporting:
//! - Asynchronous task queue
//! - Batch processing timer
//! - Error retry mechanism
//! - Dead letter queue
//!
//! # Generic Usage Example
//!
//! ```rust
//! use crate::sync::queue::{AsyncQueue, QueueConfig, QueueHandler};
//!
//! // Used for any clonable type T
//! struct MyHandler;
//! impl QueueHandler<String> for MyHandler {
//!     fn handle_item(&self, item: &String) -> Result<(), crate::sync::queue::QueueError> {
//!         println!("Processing: {}", item);
//!         Ok(())
//!     }
//! }
//!
//! let config = QueueConfig::default();
//! let queue = AsyncQueue::new(config);
//! ```

use std::collections::VecDeque;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{Mutex, RwLock};

use crate::utils::output;
use tokio::time::Duration;

/// Queue Error Type
#[derive(Debug, Error)]
pub enum QueueError {
    /// processor error
    #[error("Handler error: {0}")]
    HandlerError(String),

    /// The queue is full.
    #[error("Queue is full")]
    QueueFull,

    /// Queue closed
    #[error("Queue is closed")]
    QueueClosed,

    /// internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<std::io::Error> for QueueError {
    fn from(err: std::io::Error) -> Self {
        QueueError::InternalError(err.to_string())
    }
}

pub type QueueResult<T> = Result<T, QueueError>;

/// Queue Configuration
#[derive(Debug, Clone)]
pub struct QueueConfig {
    /// Maximum queue capacity
    pub max_queue_size: usize,
    /// Maximum number of items for batch processing
    pub batch_size: usize,
    /// Time interval for batch processing (milliseconds)
    pub batch_interval_ms: u64,
    /// Maximum number of retries
    pub max_retries: u32,
    /// Maximum dead letter queue capacity
    pub dead_letter_queue_size: usize,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 10000,
            batch_size: 100,
            batch_interval_ms: 100, // 100ms
            max_retries: 3,
            dead_letter_queue_size: 1000,
        }
    }
}

/// Pending items
#[derive(Debug, Clone)]
struct PendingItem<T> {
    item: T,
    retry_count: u32,
}

/// Items in the dead letter queue
#[derive(Debug, Clone)]
pub struct DeadLetterItem<T> {
    pub item: T,
    pub error: String,
    pub retry_count: u32,
    pub timestamp: std::time::SystemTime,
}

/// Queue processor trait (generalized version)
pub trait QueueHandler<T>: Send + Sync {
    fn handle_item(&self, item: &T) -> Result<(), QueueError>;

    fn handle_batch(&self, items: &[T]) -> Result<(), QueueError> {
        // Default processing on a case-by-case basis
        for item in items {
            self.handle_item(item)?;
        }
        Ok(())
    }
}

/// Asynchronous queue (generalized version)
pub struct AsyncQueue<T>
where
    T: Clone + Send + Sync + 'static,
{
    config: QueueConfig,
    pending_queue: Arc<Mutex<VecDeque<PendingItem<T>>>>,
    dead_letter_queue: Arc<RwLock<VecDeque<DeadLetterItem<T>>>>,
    handler: Option<Arc<dyn QueueHandler<T>>>,
}

impl<T> std::fmt::Debug for AsyncQueue<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncQueue")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl<T> AsyncQueue<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Creating a new asynchronous queue
    pub fn new(config: QueueConfig) -> Self {
        let pending_queue = Arc::new(Mutex::new(VecDeque::with_capacity(config.max_queue_size)));
        let dead_letter_queue = Arc::new(RwLock::new(VecDeque::with_capacity(
            config.dead_letter_queue_size,
        )));

        Self {
            config,
            pending_queue,
            dead_letter_queue,
            handler: None,
        }
    }

    /// Setting up the processor
    pub fn set_handler(&mut self, handler: Arc<dyn QueueHandler<T>>) {
        self.handler = Some(handler);
    }

    /// Get the length of the pending queue
    pub async fn pending_count(&self) -> usize {
        self.pending_queue.lock().await.len()
    }

    /// Get dead letter queue length
    pub async fn dead_letter_count(&self) -> usize {
        self.dead_letter_queue.read().await.len()
    }

    /// Submitting items to the queue
    pub async fn submit(&self, item: T) -> Result<(), QueueError> {
        let mut queue = self.pending_queue.lock().await;

        if queue.len() >= self.config.max_queue_size {
            return Err(QueueError::QueueFull);
        }

        queue.push_back(PendingItem {
            item,
            retry_count: 0,
        });

        Ok(())
    }

    /// batch file
    async fn process_batch(&self) -> Result<usize, QueueError> {
        let handler = match &self.handler {
            Some(h) => h.clone(),
            None => {
                return Err(QueueError::HandlerError(
                    "No handler configured".to_string(),
                ))
            }
        };

        let mut queue = self.pending_queue.lock().await;
        if queue.is_empty() {
            return Ok(0);
        }

        // batch
        let batch_size = std::cmp::min(self.config.batch_size, queue.len());
        let mut batch: Vec<T> = Vec::with_capacity(batch_size);
        let mut pending_retry: Vec<PendingItem<T>> = Vec::new();

        for _ in 0..batch_size {
            if let Some(pending) = queue.pop_front() {
                batch.push(pending.item.clone());
                pending_retry.push(pending);
            }
        }

        drop(queue); // release a lock

        // Processing batch
        match handler.handle_batch(&batch) {
            Ok(()) => Ok(batch_size),
            Err(e) => {
                // Processing failed, try retrying
                for mut pending in pending_retry {
                    pending.retry_count += 1;
                    if pending.retry_count >= self.config.max_retries {
                        // Maximum number of retries exceeded, moved to dead letter queue
                        self.add_to_dead_letter(
                            pending.item.clone(),
                            format!("{:?}", e),
                            pending.retry_count,
                        )
                        .await;
                    } else {
                        // rejoin the tail end of the queue
                        let mut queue = self.pending_queue.lock().await;
                        queue.push_front(pending);
                    }
                }
                Err(QueueError::HandlerError(format!(
                    "Batch processing failed: {:?}",
                    e
                )))
            }
        }
    }

    /// Add to dead letter queue
    async fn add_to_dead_letter(&self, item: T, error: String, retry_count: u32) {
        let mut dlq = self.dead_letter_queue.write().await;

        if dlq.len() >= self.config.dead_letter_queue_size {
            // Queue is full. Remove oldest item.
            dlq.pop_front();
        }

        dlq.push_back(DeadLetterItem {
            item,
            error,
            retry_count,
            timestamp: std::time::SystemTime::now(),
        });
    }

    /// Getting items in the dead letter queue
    pub async fn get_dead_letter_items(&self, limit: usize) -> Vec<DeadLetterItem<T>> {
        let dlq = self.dead_letter_queue.read().await;
        dlq.iter().take(limit).cloned().collect()
    }

    /// Empty the dead letter queue
    pub async fn clear_dead_letter_queue(&self) {
        let mut dlq = self.dead_letter_queue.write().await;
        dlq.clear();
    }

    /// Start the background processing loop
    pub async fn start_processing(&self) -> Result<(), QueueError> {
        let interval = Duration::from_millis(self.config.batch_interval_ms);
        let mut timer = tokio::time::interval(interval);

        loop {
            timer.tick().await;

            // batch
            match self.process_batch().await {
                Ok(count) if count > 0 => {
                    // Successfully processed count
                }
                Ok(_) => {
                    // The queue is empty. Keep waiting.
                }
                Err(e) => {
                    // Logging errors without interrupting the processing loop
                    let _ = output::print_error(&format!("Error processing batch: {:?}", e));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_queue_submit() {
        let config = QueueConfig::default();
        let queue = AsyncQueue::new(config);

        // Submitting Test Items
        queue
            .submit("test".to_string())
            .await
            .expect("Submit should succeed");
        assert_eq!(queue.pending_count().await, 1);
    }

    #[tokio::test]
    async fn test_async_queue_batch_processing() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct TestHandler {
            count: AtomicUsize,
        }

        impl QueueHandler<String> for TestHandler {
            fn handle_item(&self, _item: &String) -> Result<(), QueueError> {
                self.count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        let config = QueueConfig {
            batch_size: 5,
            ..Default::default()
        };
        let mut queue = AsyncQueue::new(config);

        let handler = Arc::new(TestHandler {
            count: AtomicUsize::new(0),
        });
        queue.set_handler(handler.clone());

        // Submission of multiple items
        for i in 0..10 {
            queue
                .submit(format!("item_{}", i))
                .await
                .expect("Submit should succeed");
        }

        // batch
        let processed = queue.process_batch().await.expect("Process should succeed");
        assert_eq!(processed, 5);
        assert_eq!(handler.count.load(Ordering::SeqCst), 5);
        assert_eq!(queue.pending_count().await, 5);
    }
}

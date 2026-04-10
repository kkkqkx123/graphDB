//! Asynchronous Event Queue
//!
//! Provide asynchronous batch event processing capability, supporting:
//! - Asynchronous event queue
//! - Batch processing timer
//! - Error retry mechanism
//! - Dead letter queue
//!
//! # Generic Usage Example
//!
//! ```rust
//! use crate::event::async_queue::{AsyncQueue, QueueConfig, QueueHandler};
//!
//! // Used for any clonable type T
//! struct MyHandler;
//! impl QueueHandler<String> for MyHandler {
//!     fn handle_item(&self, item: &String) -> Result<(), crate::event::EventError> {
//!         println!("Processing: {}", item);
//!         Ok(())
//!     }
//! }
//!
//! let config = QueueConfig::default();
//! let queue = AsyncQueue::new(config);
//! ```

use crate::event::EventError;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::Duration;

/// 队列配置
#[derive(Debug, Clone)]
pub struct QueueConfig {
    /// 队列最大容量
    pub max_queue_size: usize,
    /// 批量处理的最大项数
    pub batch_size: usize,
    /// 批量处理的时间间隔（毫秒）
    pub batch_interval_ms: u64,
    /// 最大重试次数
    pub max_retries: u32,
    /// 死信队列最大容量
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

/// 待处理项
#[derive(Debug, Clone)]
struct PendingItem<T> {
    item: T,
    retry_count: u32,
}

/// 死信队列中的项
#[derive(Debug, Clone)]
pub struct DeadLetterItem<T> {
    pub item: T,
    pub error: String,
    pub retry_count: u32,
    pub timestamp: std::time::SystemTime,
}

/// 队列处理器 trait（泛型版本）
pub trait QueueHandler<T>: Send + Sync {
    fn handle_item(&self, item: &T) -> Result<(), EventError>;
    
    fn handle_batch(&self, items: &[T]) -> Result<(), EventError> {
        // 默认逐个处理
        for item in items {
            self.handle_item(item)?;
        }
        Ok(())
    }
}

/// 异步队列（泛型版本）
pub struct AsyncQueue<T> 
where
    T: Clone + Send + Sync + 'static,
{
    config: QueueConfig,
    pending_queue: Arc<Mutex<VecDeque<PendingItem<T>>>>,
    dead_letter_queue: Arc<RwLock<VecDeque<DeadLetterItem<T>>>>,
    handler: Option<Arc<dyn QueueHandler<T>>>,
    shutdown_tx: mpsc::Sender<()>,
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
    /// 创建新的异步队列
    pub fn new(config: QueueConfig) -> Self {
        let (shutdown_tx, _) = mpsc::channel(1);
        let pending_queue = Arc::new(Mutex::new(VecDeque::with_capacity(config.max_queue_size)));
        let dead_letter_queue = Arc::new(RwLock::new(VecDeque::with_capacity(
            config.dead_letter_queue_size,
        )));

        Self {
            config,
            pending_queue,
            dead_letter_queue,
            handler: None,
            shutdown_tx,
        }
    }

    /// 设置处理器
    pub fn set_handler(&mut self, handler: Arc<dyn QueueHandler<T>>) {
        self.handler = Some(handler);
    }

    /// 获取待处理队列长度
    pub async fn pending_count(&self) -> usize {
        self.pending_queue.lock().await.len()
    }

    /// 获取死信队列长度
    pub async fn dead_letter_count(&self) -> usize {
        self.dead_letter_queue.read().await.len()
    }

    /// 提交项到队列
    pub async fn submit(&self, item: T) -> Result<(), EventError> {
        let mut queue = self.pending_queue.lock().await;

        if queue.len() >= self.config.max_queue_size {
            return Err(EventError::QueueFull);
        }

        queue.push_back(PendingItem {
            item,
            retry_count: 0,
        });

        Ok(())
    }

    /// 批量处理
    async fn process_batch(&self) -> Result<usize, EventError> {
        let handler = match &self.handler {
            Some(h) => h.clone(),
            None => {
                return Err(EventError::HandlerError(
                    "No handler configured".to_string(),
                ))
            }
        };

        let mut queue = self.pending_queue.lock().await;
        if queue.is_empty() {
            return Ok(0);
        }

        // 取出一批
        let batch_size = std::cmp::min(self.config.batch_size, queue.len());
        let mut batch: Vec<T> = Vec::with_capacity(batch_size);
        let mut pending_retry: Vec<PendingItem<T>> = Vec::new();

        for _ in 0..batch_size {
            if let Some(pending) = queue.pop_front() {
                batch.push(pending.item.clone());
                pending_retry.push(pending);
            }
        }

        drop(queue); // 释放锁

        // 处理批次
        match handler.handle_batch(&batch) {
            Ok(()) => Ok(batch_size),
            Err(e) => {
                // 处理失败，尝试重试
                for mut pending in pending_retry {
                    pending.retry_count += 1;
                    if pending.retry_count >= self.config.max_retries {
                        // 超过最大重试次数，移入死信队列
                        self.add_to_dead_letter(
                            pending.item.clone(),
                            format!("{:?}", e),
                            pending.retry_count,
                        )
                        .await;
                    } else {
                        // 重新加入队列尾部
                        let mut queue = self.pending_queue.lock().await;
                        queue.push_front(pending);
                    }
                }
                Err(EventError::HandlerError(format!(
                    "Batch processing failed: {:?}",
                    e
                )))
            }
        }
    }

    /// 添加到死信队列
    async fn add_to_dead_letter(&self, item: T, error: String, retry_count: u32) {
        let mut dlq = self.dead_letter_queue.write().await;

        if dlq.len() >= self.config.dead_letter_queue_size {
            // 队列已满，移除最旧的项
            dlq.pop_front();
        }

        dlq.push_back(DeadLetterItem {
            item,
            error,
            retry_count,
            timestamp: std::time::SystemTime::now(),
        });
    }

    /// 获取死信队列中的项
    pub async fn get_dead_letter_items(&self, limit: usize) -> Vec<DeadLetterItem<T>> {
        let dlq = self.dead_letter_queue.read().await;
        dlq.iter().take(limit).cloned().collect()
    }

    /// 清空死信队列
    pub async fn clear_dead_letter_queue(&self) {
        let mut dlq = self.dead_letter_queue.write().await;
        dlq.clear();
    }

    /// 启动后台处理循环
    pub async fn start_processing(&self) -> Result<(), EventError> {
        let interval = Duration::from_millis(self.config.batch_interval_ms);
        let mut timer = tokio::time::interval(interval);

        loop {
            timer.tick().await;

            // 处理一批
            match self.process_batch().await {
                Ok(count) if count > 0 => {
                    // 成功处理 count 个
                }
                Ok(_) => {
                    // 队列为空，继续等待
                }
                Err(e) => {
                    // 记录错误，但不中断处理循环
                    eprintln!("Error processing batch: {:?}", e);
                }
            }
        }
    }
}

// 为 StorageEvent 保留类型别名和向后兼容的 API
use crate::event::StorageEvent;

/// 异步事件队列（向后兼容的类型别名）
pub type AsyncEventQueue = AsyncQueue<StorageEvent>;

/// 事件处理器（向后兼容的类型别名）
pub type EventHandler = dyn QueueHandler<StorageEvent>;

/// 死信事件（向后兼容的类型别名）
pub type DeadLetterEvent = DeadLetterItem<StorageEvent>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct TestHandler {
        count: AtomicUsize,
    }

    impl QueueHandler<StorageEvent> for TestHandler {
        fn handle_item(&self, _event: &StorageEvent) -> Result<(), EventError> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_async_queue_submit() {
        let config = QueueConfig::default();
        let queue = AsyncQueue::new(config);

        // 创建测试事件
        let event = StorageEvent::VertexInserted {
            space_id: 1,
            vertex: crate::core::Vertex {
                vid: Box::new(crate::core::Value::Int64(1)),
                id: 1,
                tags: vec![],
                properties: std::collections::HashMap::new(),
            },
            timestamp: 0,
        };

        queue.submit(event).await.expect("Submit should succeed");
        assert_eq!(queue.pending_count().await, 1);
    }

    #[tokio::test]
    async fn test_async_queue_batch_processing() {
        let config = QueueConfig {
            batch_size: 5,
            ..Default::default()
        };
        let mut queue = AsyncQueue::new(config);

        let handler = Arc::new(TestHandler {
            count: AtomicUsize::new(0),
        });
        queue.set_handler(handler.clone());

        // 提交多个事件
        for i in 0..10 {
            let event = StorageEvent::VertexInserted {
                space_id: 1,
                vertex: crate::core::Vertex {
                    vid: Box::new(crate::core::Value::Int64(i)),
                    id: i,
                    tags: vec![],
                    properties: std::collections::HashMap::new(),
                },
                timestamp: 0,
            };
            queue.submit(event).await.expect("Submit should succeed");
        }

        // 处理一批
        let processed = queue.process_batch().await.expect("Process should succeed");
        assert_eq!(processed, 5);
        assert_eq!(handler.count.load(Ordering::SeqCst), 5);
        assert_eq!(queue.pending_count().await, 5);
    }
}

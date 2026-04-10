//! 异步事件队列
//!
//! 提供异步批量事件处理能力，支持：
//! - 异步事件队列
//! - 批量处理定时器
//! - 错误重试机制
//! - 死信队列

use crate::event::{EventError, StorageEvent};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::{Duration, Interval};

/// 事件队列配置
#[derive(Debug, Clone)]
pub struct AsyncQueueConfig {
    /// 队列最大容量
    pub max_queue_size: usize,
    /// 批量处理的最大事件数
    pub batch_size: usize,
    /// 批量处理的时间间隔（毫秒）
    pub batch_interval_ms: u64,
    /// 最大重试次数
    pub max_retries: u32,
    /// 死信队列最大容量
    pub dead_letter_queue_size: usize,
}

impl Default for AsyncQueueConfig {
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

/// 待处理事件
#[derive(Debug, Clone)]
struct PendingEvent {
    event: StorageEvent,
    retry_count: u32,
}

/// 死信队列中的事件
#[derive(Debug, Clone)]
pub struct DeadLetterEvent {
    pub event: StorageEvent,
    pub error: String,
    pub retry_count: u32,
    pub timestamp: std::time::SystemTime,
}

/// 事件处理器 trait
pub trait EventHandler: Send + Sync {
    fn handle_event(&self, event: &StorageEvent) -> Result<(), EventError>;
    fn handle_batch(&self, events: &[StorageEvent]) -> Result<(), EventError> {
        // 默认逐个处理
        for event in events {
            self.handle_event(event)?;
        }
        Ok(())
    }
}

/// 异步事件队列
pub struct AsyncEventQueue {
    config: AsyncQueueConfig,
    pending_queue: Arc<Mutex<VecDeque<PendingEvent>>>,
    dead_letter_queue: Arc<RwLock<VecDeque<DeadLetterEvent>>>,
    handler: Option<Arc<dyn EventHandler>>,
    shutdown_tx: mpsc::Sender<()>,
}

impl AsyncEventQueue {
    /// 创建新的异步事件队列
    pub fn new(config: AsyncQueueConfig) -> Self {
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

    /// 设置事件处理器
    pub fn set_handler(&mut self, handler: Arc<dyn EventHandler>) {
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

    /// 提交事件到队列
    pub async fn submit(&self, event: StorageEvent) -> Result<(), EventError> {
        let mut queue = self.pending_queue.lock().await;

        if queue.len() >= self.config.max_queue_size {
            return Err(EventError::QueueFull);
        }

        queue.push_back(PendingEvent {
            event,
            retry_count: 0,
        });

        Ok(())
    }

    /// 批量处理事件
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

        // 取出一批事件
        let batch_size = std::cmp::min(self.config.batch_size, queue.len());
        let mut batch: Vec<StorageEvent> = Vec::with_capacity(batch_size);
        let mut pending_retry: Vec<PendingEvent> = Vec::new();

        for _ in 0..batch_size {
            if let Some(mut pending) = queue.pop_front() {
                batch.push(pending.event.clone());
                pending_retry.push(pending);
            }
        }

        drop(queue); // 释放锁

        // 处理批次
        match handler.handle_batch(&batch) {
            Ok(()) => Ok(batch_size),
            Err(e) => {
                // 处理失败，尝试重试
                let mut retry_count = 0;
                for mut pending in pending_retry {
                    pending.retry_count += 1;
                    if pending.retry_count >= self.config.max_retries {
                        // 超过最大重试次数，移入死信队列
                        self.add_to_dead_letter(
                            pending.event.clone(),
                            format!("{:?}", e),
                            pending.retry_count,
                        )
                        .await;
                    } else {
                        // 重新加入队列尾部
                        let mut queue = self.pending_queue.lock().await;
                        queue.push_front(pending);
                        retry_count += 1;
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
    async fn add_to_dead_letter(&self, event: StorageEvent, error: String, retry_count: u32) {
        let mut dlq = self.dead_letter_queue.write().await;

        if dlq.len() >= self.config.dead_letter_queue_size {
            // 队列已满，移除最旧的事件
            dlq.pop_front();
        }

        dlq.push_back(DeadLetterEvent {
            event,
            error,
            retry_count,
            timestamp: std::time::SystemTime::now(),
        });
    }

    /// 获取死信队列中的事件
    pub async fn get_dead_letter_events(&self, limit: usize) -> Vec<DeadLetterEvent> {
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

            // 处理一批事件
            match self.process_batch().await {
                Ok(count) if count > 0 => {
                    // 成功处理 count 个事件
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct TestHandler {
        count: AtomicUsize,
    }

    impl EventHandler for TestHandler {
        fn handle_event(&self, _event: &StorageEvent) -> Result<(), EventError> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_async_queue_submit() {
        let config = AsyncQueueConfig::default();
        let queue = AsyncEventQueue::new(config);

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
        let config = AsyncQueueConfig {
            batch_size: 5,
            ..Default::default()
        };
        let mut queue = AsyncEventQueue::new(config);

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

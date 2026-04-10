//! 事件系统模块
//!
//! 提供存储操作事件的发布订阅机制，用于实现数据与索引的自动同步。
//!
//! # 架构
//!
//! ```text
//! StorageClient -> EventEmittingStorage -> EventHub -> SyncManager
//! ```
//!
//! # 使用示例
//!
//! ```rust
//! let event_hub = Arc::new(MemoryEventHub::new());
//! let sync_manager = Arc::new(SyncManager::new(fulltext_coordinator, config));
//! let mut storage = EventEmittingStorage::new(inner_storage, event_hub.clone(), Some(sync_manager));
//! storage.enable_events(true);
//! ```

pub mod async_queue;
pub mod error;
pub mod hub;
pub mod types;

pub use async_queue::{AsyncQueue, QueueConfig, QueueHandler};
pub use error::*;
pub use hub::{EventHub, MemoryEventHub};
pub use types::*;

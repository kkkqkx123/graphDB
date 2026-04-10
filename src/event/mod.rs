//! 事件系统模块
//!
//! 提供存储操作事件的发布订阅机制，用于实现数据与索引的自动同步。
//!
//! # 架构
//!
//! ```text
//! StorageClient -> EventEmittingStorage -> EventHub -> EventHandler
//! ```
//!
//! # 使用示例
//!
//! ```rust
//! let event_hub = Arc::new(MemoryEventHub::new());
//! let mut storage = EventEmittingStorage::new(inner_storage, event_hub.clone());
//! storage.enable_events(true);
//!
//! event_hub.subscribe(EventType::VertexEvent, |event| {
//!     // 处理事件
//!     Ok(())
//! })?;
//! ```

pub mod async_queue;
pub mod error;
pub mod hub;
pub mod types;

pub use async_queue::*;
pub use error::*;
pub use hub::*;
pub use types::*;

//! Sync Module
//!
//! Synchronization system for fulltext and vector index updates.

pub mod batch;
pub mod manager;
pub mod persistence;
pub mod queue;
pub mod recovery;
pub mod task;

pub use batch::{BatchConfig, BufferError, TaskBuffer};
pub use manager::{SyncError, SyncManager, SyncMode};
pub use persistence::{FailedTask, PersistenceError, SyncPersistence, SyncState};
pub use queue::{AsyncQueue, DeadLetterItem, QueueConfig, QueueError, QueueHandler, QueueResult};
pub use recovery::{RecoveryConfig, RecoveryError, RecoveryManager, RecoveryResult};
pub use task::{SyncTask, TaskResult, VectorPointData};

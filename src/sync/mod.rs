//! Sync Module
//!
//! Synchronization system for fulltext index updates.

pub mod batch;
pub mod manager;
pub mod persistence;
pub mod recovery;
pub mod task;

pub use batch::{BatchConfig, BufferError, TaskBuffer};
pub use manager::{SyncError, SyncManager, SyncMode};
pub use persistence::{FailedTask, PersistenceError, SyncPersistence, SyncState};
pub use recovery::{RecoveryConfig, RecoveryError, RecoveryManager, RecoveryResult};
pub use task::{SyncTask, TaskResult};

//! Sync Module
//!
//! Synchronization system for fulltext and vector index updates.

pub mod batch;
pub mod manager;
pub mod persistence;
pub mod queue;
pub mod recovery;
pub mod task;
pub mod vector_sync;

pub use crate::search::SyncConfig;
pub use batch::{BatchConfig, BufferError, TaskBuffer};
pub use manager::{SyncError, SyncManager, SyncMode};
pub use persistence::{FailedTask, PersistenceError, SyncPersistence, SyncState};
pub use queue::{AsyncQueue, DeadLetterItem, QueueConfig, QueueError, QueueHandler, QueueResult};
pub use recovery::{RecoveryConfig, RecoveryError, RecoveryManager, RecoveryResult};
pub use task::{SyncTask, TaskResult, VectorPointData};
pub use vector_sync::{
    SearchOptions, VectorChangeContext, VectorChangeType, VectorIndexLocation,
    VectorSyncCoordinator,
};

// Re-export from transaction module
pub use crate::transaction::sync_handle::{
    IndexBufferConfig, PendingIndexUpdate, SyncHandle, SyncHandleState,
};

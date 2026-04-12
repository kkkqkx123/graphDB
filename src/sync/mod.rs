//! Sync Module
//!
//! Synchronization system for fulltext and vector index updates.

pub mod batch;
pub mod compensation;
pub mod coordinator;
pub mod dead_letter_queue;
pub mod external_index;
pub mod manager;
pub mod metrics;
pub mod persistence;
pub mod queue;
pub mod recovery;
pub mod retry;
pub mod task;
pub mod vector_batch;
pub mod vector_sync;
pub mod vector_transaction_buffer;

pub use crate::search::SyncConfig;
pub use batch::{
    BatchConfig, BatchError, BatchProcessor, GenericBatchProcessor, TransactionBatchBuffer,
    TransactionBuffer,
};
pub use compensation::{CompensationManager, CompensationResult, CompensationStats};
pub use coordinator::{
    ChangeContext, ChangeData, ChangeType, IndexType, SyncCoordinator, SyncCoordinatorError,
};
pub use dead_letter_queue::{DeadLetterEntry, DeadLetterQueue, DeadLetterQueueConfig};
pub use external_index::{ExternalIndexClient, IndexData, IndexOperation};
pub use manager::{SyncError, SyncManager};
pub use metrics::{SyncMetrics, SyncStats};
pub use persistence::{FailedTask, PersistenceError, SyncPersistence, SyncState};
pub use queue::{AsyncQueue, DeadLetterItem, QueueConfig, QueueError, QueueHandler, QueueResult};
pub use recovery::{RecoveryConfig, RecoveryError, RecoveryManager, RecoveryResult};
pub use retry::{with_retry, RetryConfig};
pub use task::{SyncTask, TaskResult, VectorPointData};
pub use vector_batch::{VectorBatchConfig, VectorBatchError, VectorBatchManager};
pub use vector_sync::{
    SearchOptions, VectorChangeContext, VectorChangeType, VectorIndexLocation,
    VectorSyncCoordinator,
};
pub use vector_transaction_buffer::{
    PendingVectorUpdate, VectorTransactionBuffer, VectorTransactionBufferConfig,
};

// Re-export from transaction module
pub use crate::transaction::sync_handle::{
    IndexBufferConfig, PendingIndexUpdate, SyncHandle, SyncHandleState,
};

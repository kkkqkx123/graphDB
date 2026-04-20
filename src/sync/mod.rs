//! Sync Module
//!
//! Synchronization system for fulltext and vector index updates.

pub mod batch;
pub mod coordinator;
pub mod dead_letter_queue;
pub mod external_index;
pub mod manager;
pub mod metrics;
pub mod retry;
pub mod task;
pub mod vector_batch;
pub mod vector_sync;
pub mod vector_transaction_buffer;
pub mod vector_types;

pub use crate::search::SyncConfig;
pub use batch::{
    BatchConfig, BatchError, BatchProcessor, GenericBatchProcessor, TransactionBatchBuffer,
    TransactionBuffer,
};
pub use coordinator::{
    ChangeContext, ChangeData, ChangeType, IndexType, SyncCoordinator, SyncCoordinatorError,
};
pub use dead_letter_queue::{DeadLetterEntry, DeadLetterQueue, DeadLetterQueueConfig};
pub use external_index::{ExternalIndexClient, IndexData, IndexOperation};
pub use manager::{SyncError, SyncManager};
pub use metrics::SyncMetrics;
pub use retry::{with_retry, RetryConfig};
pub use task::VectorPointData;
pub use vector_batch::{VectorBatchConfig, VectorBatchError, VectorBatchManager};
pub use vector_sync::{
    SearchOptions, VectorChangeContext, VectorIndexLocation, VectorSyncCoordinator,
};
pub use vector_types::VectorChangeType;
pub use vector_transaction_buffer::{
    PendingVectorUpdate, VectorTransactionBuffer, VectorTransactionBufferConfig,
};

/// Pending index update (moved from transaction::sync_handle)
#[derive(Debug, Clone)]
pub struct PendingIndexUpdate {
    /// Transaction ID
    pub txn_id: crate::transaction::types::TransactionId,
    /// Space ID
    pub space_id: u64,
    /// Tag Name
    pub tag_name: String,
    /// field name
    pub field_name: String,
    /// Document ID
    pub doc_id: String,
    /// Updated content (None means deleted)
    pub content: Option<String>,
    /// Previous content before update (for rollback)
    pub old_content: Option<String>,
}

impl PendingIndexUpdate {
    pub fn new(
        txn_id: crate::transaction::types::TransactionId,
        space_id: u64,
        tag_name: String,
        field_name: String,
        doc_id: String,
    ) -> Self {
        Self {
            txn_id,
            space_id,
            tag_name,
            field_name,
            doc_id,
            content: None,
            old_content: None,
        }
    }

    pub fn with_content(mut self, content: String) -> Self {
        self.content = Some(content);
        self
    }

    pub fn with_old_content(mut self, old_content: String) -> Self {
        self.old_content = Some(old_content);
        self
    }
}

/// Index buffer configuration (moved from transaction::sync_handle)
#[derive(Debug, Clone)]
pub struct IndexBufferConfig {
    pub max_buffer_size: usize,
    pub flush_timeout_ms: u64,
}

impl Default for IndexBufferConfig {
    fn default() -> Self {
        Self {
            max_buffer_size: 1000,
            flush_timeout_ms: 100,
        }
    }
}

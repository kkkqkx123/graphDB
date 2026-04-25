use async_trait::async_trait;
use std::sync::Arc;

use super::error::BatchResult;
use crate::sync::external_index::IndexOperation;

/// Trait for batch processing of index operations
///
/// Implementors of this trait provide buffered batch processing capabilities
/// for index operations, with support for background flushing and immediate mode.
#[async_trait]
pub trait BatchProcessor: Send + Sync + std::fmt::Debug {
    /// Add a single operation to the batch
    async fn add(&self, operation: IndexOperation) -> BatchResult<()>;

    /// Add multiple operations to the batch
    async fn add_batch(&self, operations: Vec<IndexOperation>) -> BatchResult<()>;

    /// Commit all pending operations immediately
    async fn commit_all(&self) -> BatchResult<()>;

    /// Commit operations that have exceeded the timeout
    async fn commit_timeout(&self) -> BatchResult<()>;

    /// Start background task for periodic flushing
    async fn start_background_task(self: Arc<Self>);

    /// Stop the background flushing task
    async fn stop_background_task(&self);

    /// Return as Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Trait for transaction-aware buffering of index operations
///
/// This trait provides two-phase commit support for index operations:
/// - Operations are buffered during transaction execution (prepare phase)
/// - On commit: operations are cleared from buffer (caller executes them)
/// - On rollback: operations are discarded
///
/// Note: The implementor only manages buffering, not execution.
/// The caller is responsible for executing operations after commit.
#[async_trait]
pub trait TransactionBuffer: Send + Sync {
    /// Buffer an operation for the given transaction (prepare phase)
    async fn prepare(
        &self,
        txn_id: crate::transaction::types::TransactionId,
        operation: IndexOperation,
    ) -> BatchResult<()>;

    /// Mark transaction as committed
    ///
    /// Note: This only clears the buffer. The caller must execute
    /// the operations using `take_operations()` before or after calling this.
    async fn commit(&self, txn_id: crate::transaction::types::TransactionId) -> BatchResult<()>;

    /// Rollback the transaction by discarding all buffered operations
    async fn rollback(&self, txn_id: crate::transaction::types::TransactionId) -> BatchResult<()>;

    /// Get the number of pending operations for a transaction
    fn pending_count(&self, txn_id: crate::transaction::types::TransactionId) -> usize;
}

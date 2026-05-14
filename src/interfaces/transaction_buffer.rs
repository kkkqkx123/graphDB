//! Transaction Buffer Interface
//!
//! Defines the interface for transaction-aware buffering of index operations.
//! This trait abstracts the buffering mechanism from the sync layer.

use async_trait::async_trait;

use crate::sync::external_index::IndexOperation;
use crate::sync::batch::error::BatchResult;
use crate::transaction::types::TransactionId;

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
        txn_id: TransactionId,
        operation: IndexOperation,
    ) -> BatchResult<()>;

    /// Mark transaction as committed
    ///
    /// Note: This only clears the buffer. The caller must execute
    /// the operations using `take_operations()` before or after calling this.
    async fn commit(&self, txn_id: TransactionId) -> BatchResult<()>;

    /// Rollback the transaction by discarding all buffered operations
    async fn rollback(&self, txn_id: TransactionId) -> BatchResult<()>;

    /// Get the number of pending operations for a transaction
    fn pending_count(&self, txn_id: TransactionId) -> usize;
}
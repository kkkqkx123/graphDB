use async_trait::async_trait;
use std::sync::Arc;

use super::error::BatchResult;
use crate::sync::external_index::IndexOperation;

#[async_trait]
pub trait BatchProcessor: Send + Sync + std::fmt::Debug {
    async fn add(&self, operation: IndexOperation) -> BatchResult<()>;

    async fn add_batch(&self, operations: Vec<IndexOperation>) -> BatchResult<()>;

    async fn commit_all(&self) -> BatchResult<()>;

    async fn commit_timeout(&self) -> BatchResult<()>;

    async fn start_background_task(self: Arc<Self>);

    async fn stop_background_task(&self);

    fn as_any(&self) -> &dyn std::any::Any;
}

#[async_trait]
pub trait TransactionBuffer: Send + Sync {
    async fn prepare(
        &self,
        txn_id: crate::transaction::types::TransactionId,
        operation: IndexOperation,
    ) -> BatchResult<()>;

    async fn commit(&self, txn_id: crate::transaction::types::TransactionId) -> BatchResult<()>;

    async fn rollback(&self, txn_id: crate::transaction::types::TransactionId) -> BatchResult<()>;

    fn pending_count(&self, txn_id: crate::transaction::types::TransactionId) -> usize;
}

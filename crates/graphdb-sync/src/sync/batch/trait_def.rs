use async_trait::async_trait;
use std::sync::Arc;

use super::error::BatchResult;
use crate::sync::types::IndexOperation;

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
}

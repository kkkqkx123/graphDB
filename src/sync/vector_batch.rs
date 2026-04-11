//! Vector Batch Manager
//!
//! Manages batch operations for vector index synchronization with support for:
//! - Transaction-aware buffering (two-phase commit)
//! - Async batch processing
//! - Configurable batch size and timeout

use crate::sync::vector_sync::{VectorChangeType, VectorSyncCoordinator};
use crate::search::SyncFailurePolicy;
use crate::transaction::types::TransactionId;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use vector_client::types::VectorPoint;

/// Collection identifier
type CollectionKey = (u64, String, String); // (space_id, tag_name, field_name)

/// Vector batch configuration
#[derive(Debug, Clone)]
pub struct VectorBatchConfig {
    /// Maximum batch size before auto-commit
    pub batch_size: usize,
    /// Time interval for batch commits
    pub commit_interval: Duration,
    /// Maximum wait time for a batch
    pub max_wait_time: Duration,
    /// Queue capacity for async operations
    pub queue_capacity: usize,
    /// Failure handling policy
    pub failure_policy: SyncFailurePolicy,
}

impl Default for VectorBatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 256,
            commit_interval: Duration::from_millis(1000),
            max_wait_time: Duration::from_secs(5),
            queue_capacity: 10000,
            failure_policy: SyncFailurePolicy::FailOpen,
        }
    }
}

impl VectorBatchConfig {
    pub fn new(
        batch_size: usize,
        commit_interval_ms: u64,
        queue_capacity: usize,
        failure_policy: SyncFailurePolicy,
    ) -> Self {
        Self {
            batch_size,
            commit_interval: Duration::from_millis(commit_interval_ms),
            max_wait_time: Duration::from_secs(5),
            queue_capacity,
            failure_policy,
        }
    }
}

/// Vector operation types
#[derive(Debug, Clone)]
pub enum VectorOperation {
    /// Upsert a vector point
    Upsert(VectorPoint),
    /// Delete a vector point by ID
    Delete(String),
}

/// Pending vector operation for transaction
#[derive(Debug, Clone)]
pub struct PendingVectorOperation {
    pub operation: VectorOperation,
    pub space_id: u64,
    pub tag_name: String,
    pub field_name: String,
    pub change_type: VectorChangeType,
}

/// Vector batch manager
pub struct VectorBatchManager {
    vector_coordinator: Arc<VectorSyncCoordinator>,
    config: VectorBatchConfig,
    /// Transaction buffers for two-phase commit
    pending_buffers: DashMap<TransactionId, Vec<PendingVectorOperation>>,
    /// Async upsert buffers by collection
    upsert_buffers: DashMap<CollectionKey, Vec<VectorPoint>>,
    /// Async delete buffers by collection
    delete_buffers: DashMap<CollectionKey, Vec<String>>,
    /// Last commit time by collection
    last_commit: DashMap<CollectionKey, std::time::Instant>,
    /// Background task handle
    background_task: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl std::fmt::Debug for VectorBatchManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VectorBatchManager")
            .field("config", &self.config)
            .field("pending_buffers_count", &self.pending_buffers.len())
            .field("upsert_buffers_count", &self.upsert_buffers.len())
            .finish_non_exhaustive()
    }
}

impl VectorBatchManager {
    /// Create a new vector batch manager
    pub fn new(
        vector_coordinator: Arc<VectorSyncCoordinator>,
        config: VectorBatchConfig,
    ) -> Self {
        Self {
            vector_coordinator,
            config,
            pending_buffers: DashMap::new(),
            upsert_buffers: DashMap::new(),
            delete_buffers: DashMap::new(),
            last_commit: DashMap::new(),
            background_task: Mutex::new(None),
        }
    }

    /// Get configuration
    pub fn config(&self) -> &VectorBatchConfig {
        &self.config
    }

    // ==================== Two-Phase Commit API ====================

    /// Buffer a vector operation within a transaction (Phase 1)
    pub async fn buffer_operation(
        &self,
        txn_id: TransactionId,
        operation: PendingVectorOperation,
    ) -> Result<(), VectorBatchError> {
        let mut buffer = self.pending_buffers.entry(txn_id).or_default();
        buffer.push(operation);

        // Check buffer size limit
        if buffer.len() > self.config.queue_capacity {
            return Err(VectorBatchError::BufferOverflow(format!(
                "Transaction {:?} buffer exceeded capacity {}",
                txn_id, self.config.queue_capacity
            )));
        }

        Ok(())
    }

    /// Commit all buffered operations for a transaction (Phase 2)
    pub async fn commit_transaction(
        &self,
        txn_id: TransactionId,
    ) -> Result<(), VectorBatchError> {
        let operations = self.pending_buffers.remove(&txn_id).map(|(_, ops)| ops);

        if let Some(ops) = operations {
            if ops.is_empty() {
                return Ok(());
            }

            // Group operations by collection
            let mut upserts_by_collection: HashMap<CollectionKey, Vec<VectorPoint>> =
                HashMap::new();
            let mut deletes_by_collection: HashMap<CollectionKey, Vec<String>> = HashMap::new();

            for op in ops {
                let key = (op.space_id, op.tag_name.clone(), op.field_name.clone());

                match op.operation {
                    VectorOperation::Upsert(point) => {
                        upserts_by_collection.entry(key).or_default().push(point);
                    }
                    VectorOperation::Delete(point_id) => {
                        deletes_by_collection.entry(key).or_default().push(point_id);
                    }
                }
            }

            // Execute upserts
            for (key, points) in upserts_by_collection {
                self.execute_upsert_batch(&key, points).await?;
            }

            // Execute deletes
            for (key, point_ids) in deletes_by_collection {
                self.execute_delete_batch(&key, point_ids).await?;
            }
        }

        Ok(())
    }

    /// Rollback (discard) all buffered operations for a transaction
    pub async fn rollback_transaction(
        &self,
        txn_id: TransactionId,
    ) -> Result<(), VectorBatchError> {
        self.pending_buffers.remove(&txn_id);
        Ok(())
    }

    /// Get pending operations count for a transaction
    pub fn pending_count(&self, txn_id: TransactionId) -> usize {
        self.pending_buffers
            .get(&txn_id)
            .map(|entry| entry.len())
            .unwrap_or(0)
    }

    // ==================== Async Batch API ====================

    /// Add an async upsert operation
    pub async fn add_upsert(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        point: VectorPoint,
    ) -> Result<(), VectorBatchError> {
        let key = (space_id, tag_name.to_string(), field_name.to_string());

        let mut buffer = self.upsert_buffers.entry(key.clone()).or_default();
        buffer.push(point);

        // Auto-commit if batch size reached
        if buffer.len() >= self.config.batch_size {
            let points = self
                .upsert_buffers
                .remove(&key)
                .map(|(_, v)| v)
                .unwrap_or_default();
            self.execute_upsert_batch(&key, points).await?;
        }

        // Update last commit time
        self.last_commit
            .entry(key)
            .or_insert_with(std::time::Instant::now);

        Ok(())
    }

    /// Add an async delete operation
    pub async fn add_deletion(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        point_id: String,
    ) -> Result<(), VectorBatchError> {
        let key = (space_id, tag_name.to_string(), field_name.to_string());

        let mut buffer = self.delete_buffers.entry(key.clone()).or_default();
        buffer.push(point_id);

        // Auto-commit if batch size reached
        if buffer.len() >= self.config.batch_size {
            let ids = self
                .delete_buffers
                .remove(&key)
                .map(|(_, v)| v)
                .unwrap_or_default();
            self.execute_delete_batch(&key, ids).await?;
        }

        // Update last commit time
        self.last_commit
            .entry(key)
            .or_insert_with(std::time::Instant::now);

        Ok(())
    }

    /// Commit all pending async operations
    pub async fn commit_all(&self) -> Result<(), VectorBatchError> {
        // Commit upserts
        let upsert_keys: Vec<_> = self.upsert_buffers.iter().map(|e| e.key().clone()).collect();
        for key in upsert_keys {
            if let Some((_, points)) = self.upsert_buffers.remove(&key) {
                if !points.is_empty() {
                    self.execute_upsert_batch(&key, points).await?;
                }
            }
        }

        // Commit deletes
        let delete_keys: Vec<_> = self.delete_buffers.iter().map(|e| e.key().clone()).collect();
        for key in delete_keys {
            if let Some((_, ids)) = self.delete_buffers.remove(&key) {
                if !ids.is_empty() {
                    self.execute_delete_batch(&key, ids).await?;
                }
            }
        }

        Ok(())
    }

    /// Commit operations that have exceeded the timeout
    pub async fn commit_timeout(&self) -> Result<(), VectorBatchError> {
        let now = std::time::Instant::now();

        // Check upsert buffers
        let upsert_keys: Vec<_> = self.upsert_buffers.iter().map(|e| e.key().clone()).collect();
        for key in upsert_keys {
            if let Some(last) = self.last_commit.get(&key) {
                if now.duration_since(*last) >= self.config.commit_interval {
                    if let Some((_, points)) = self.upsert_buffers.remove(&key) {
                        if !points.is_empty() {
                            self.execute_upsert_batch(&key, points).await?;
                            self.last_commit.insert(key.clone(), now);
                        }
                    }
                }
            }
        }

        // Check delete buffers
        let delete_keys: Vec<_> = self.delete_buffers.iter().map(|e| e.key().clone()).collect();
        for key in delete_keys {
            if let Some(last) = self.last_commit.get(&key) {
                if now.duration_since(*last) >= self.config.commit_interval {
                    if let Some((_, ids)) = self.delete_buffers.remove(&key) {
                        if !ids.is_empty() {
                            self.execute_delete_batch(&key, ids).await?;
                            self.last_commit.insert(key.clone(), now);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    // ==================== Internal Helpers ====================

    /// Execute upsert batch for a collection
    async fn execute_upsert_batch(
        &self,
        key: &CollectionKey,
        points: Vec<VectorPoint>,
    ) -> Result<(), VectorBatchError> {
        let (space_id, tag_name, field_name) = key;
        let collection_name = format!("space_{}_{}_{}", space_id, tag_name, field_name);

        let points_count = points.len();
        if points_count == 1 {
            if let Some(point) = points.into_iter().next() {
                self.vector_coordinator
                    .vector_manager()
                    .upsert(&collection_name, point)
                    .await
                    .map_err(|e| {
                        VectorBatchError::VectorError(format!("Upsert failed: {}", e))
                    })?;
            }
        } else if points_count > 1 {
            self.vector_coordinator
                .vector_manager()
                .upsert_batch(&collection_name, points)
                .await
                .map_err(|e| {
                    VectorBatchError::VectorError(format!("Batch upsert failed: {}", e))
                })?;
        }

        Ok(())
    }

    /// Execute delete batch for a collection
    async fn execute_delete_batch(
        &self,
        key: &CollectionKey,
        point_ids: Vec<String>,
    ) -> Result<(), VectorBatchError> {
        let (space_id, tag_name, field_name) = key;
        let collection_name = format!("space_{}_{}_{}", space_id, tag_name, field_name);

        let ids_count = point_ids.len();
        if ids_count == 1 {
            if let Some(id) = point_ids.first() {
                self.vector_coordinator
                    .vector_manager()
                    .delete(&collection_name, id)
                    .await
                    .map_err(|e| {
                        VectorBatchError::VectorError(format!("Delete failed: {}", e))
                    })?;
            }
        } else if ids_count > 1 {
            let refs: Vec<&str> = point_ids.iter().map(|s| s.as_str()).collect();
            self.vector_coordinator
                .vector_manager()
                .delete_batch(&collection_name, refs)
                .await
                .map_err(|e| {
                    VectorBatchError::VectorError(format!("Batch delete failed: {}", e))
                })?;
        }

        Ok(())
    }

    /// Start background batch commit task
    pub async fn start_background_task(self: Arc<Self>) {
        let mut handle = self.background_task.lock().await;
        if handle.is_some() {
            return; // Already running
        }

        let manager = self.clone();
        let interval = manager.config.commit_interval;

        let task = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                if let Err(e) = manager.commit_timeout().await {
                    tracing::error!("Background batch commit failed: {:?}", e);
                }
            }
        });

        *handle = Some(task);
    }

    /// Stop background task
    pub async fn stop_background_task(&self) {
        let mut handle = self.background_task.lock().await;
        if let Some(task) = handle.take() {
            task.abort();
        }
    }
}

impl Drop for VectorBatchManager {
    fn drop(&mut self) {
        // Abort background task
        if let Ok(mut handle) = self.background_task.try_lock() {
            if let Some(task) = handle.take() {
                task.abort();
            }
        }
    }
}

/// Vector batch errors
#[derive(Debug, thiserror::Error)]
pub enum VectorBatchError {
    #[error("Buffer overflow: {0}")]
    BufferOverflow(String),

    #[error("Vector error: {0}")]
    VectorError(String),

    #[error("Coordinator error: {0}")]
    CoordinatorError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_batch_config_default() {
        let config = VectorBatchConfig::default();
        assert_eq!(config.batch_size, 256);
        assert_eq!(config.commit_interval, Duration::from_millis(1000));
        assert_eq!(config.queue_capacity, 10000);
    }

    #[test]
    fn test_vector_batch_config_creation() {
        let config = VectorBatchConfig::new(512, 2000, 5000, SyncFailurePolicy::FailClosed);
        assert_eq!(config.batch_size, 512);
        assert_eq!(config.commit_interval, Duration::from_millis(2000));
        assert_eq!(config.queue_capacity, 5000);
        assert_eq!(config.failure_policy, SyncFailurePolicy::FailClosed);
    }
}

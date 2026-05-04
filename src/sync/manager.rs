//! Sync Manager
//!
//! Unified synchronization manager using SyncCoordinator.

use crate::core::error::CoordinatorError;
use crate::core::Value;
use crate::search::SyncConfig;
use crate::sync::coordinator::{ChangeType, SyncCoordinator};
use crate::sync::vector_sync::VectorSyncCoordinator;
use std::sync::Arc;
use tokio::sync::Mutex;

// Re-export vector_client types for unified API
pub use vector_client::{CollectionConfig, SearchResult};

pub struct SyncManager {
    sync_coordinator: Arc<SyncCoordinator>,
    vector_coordinator: Option<Arc<VectorSyncCoordinator>>,
    running: Arc<std::sync::atomic::AtomicBool>,
    dead_letter_queue: Option<Arc<crate::sync::DeadLetterQueue>>,
    #[allow(clippy::type_complexity)]
    handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl Clone for SyncManager {
    fn clone(&self) -> Self {
        Self {
            sync_coordinator: self.sync_coordinator.clone(),
            vector_coordinator: self.vector_coordinator.clone(),
            running: self.running.clone(),
            dead_letter_queue: self.dead_letter_queue.clone(),
            handle: Mutex::new(None),
        }
    }
}

impl std::fmt::Debug for SyncManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncManager")
            .field("sync_coordinator", &self.sync_coordinator)
            .field("vector_coordinator", &self.vector_coordinator)
            .field("running", &self.running)
            .finish_non_exhaustive()
    }
}

impl SyncManager {
    pub fn new(sync_coordinator: Arc<SyncCoordinator>) -> Self {
        Self {
            sync_coordinator,
            vector_coordinator: None,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            dead_letter_queue: None,
            handle: Mutex::new(None),
        }
    }

    pub fn with_vector_coordinator(
        mut self,
        vector_coordinator: Arc<VectorSyncCoordinator>,
    ) -> Self {
        self.vector_coordinator = Some(vector_coordinator);
        self
    }

    pub fn with_sync_config(
        sync_coordinator: Arc<SyncCoordinator>,
        _sync_config: SyncConfig,
    ) -> Self {
        Self::new(sync_coordinator)
    }

    pub fn with_dead_letter_queue(
        mut self,
        dead_letter_queue: Arc<crate::sync::DeadLetterQueue>,
    ) -> Self {
        self.dead_letter_queue = Some(dead_letter_queue);
        self
    }

    pub async fn start(&self) -> Result<(), SyncError> {
        if self.running.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }

        self.running
            .store(true, std::sync::atomic::Ordering::SeqCst);

        // Start background tasks in the coordinator
        self.sync_coordinator.start_background_tasks().await;

        Ok(())
    }

    pub async fn stop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);

        // Stop background tasks in the coordinator
        self.sync_coordinator.stop_background_tasks().await;

        // Wait for handle to complete
        if let Some(handle) = self.handle.lock().await.take() {
            let _ = handle.await;
        }
    }

    /// Vertex changes with transactions (synchronous buffering)
    pub fn on_vertex_change_with_txn(
        &self,
        txn_id: crate::transaction::types::TransactionId,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
        properties: &[(String, Value)],
        change_type: ChangeType,
    ) -> Result<(), SyncError> {
        // For each attribute create a context and buffer
        for (field_name, value) in properties {
            // Buffer full-text index operations
            if let Value::String(text) = value {
                let ctx = crate::sync::coordinator::ChangeContext::new_fulltext(
                    space_id,
                    tag_name,
                    field_name,
                    change_type,
                    vertex_id.to_string().unwrap_or_default(),
                    text.clone(),
                );
                self.sync_coordinator
                    .buffer_operation(txn_id, ctx)
                    .map_err(SyncError::from)?;
            }

            // Buffered Vector Indexing Operations
            if let Some(vector) = value.as_vector() {
                if let Some(ref vector_coord) = self.vector_coordinator {
                    let ctx = crate::sync::vector_sync::VectorChangeContext::new(
                        space_id,
                        tag_name,
                        field_name,
                        crate::sync::vector_sync::VectorChangeType::from(change_type),
                        crate::sync::vector_sync::VectorPointData {
                            id: vertex_id.to_string().unwrap_or_default(),
                            vector: vector.clone(),
                            payload: std::collections::HashMap::new(),
                        },
                    );
                    vector_coord
                        .buffer_vector_change(txn_id, ctx)
                        .map_err(|e| SyncError::VectorError(e.to_string()))?;
                }
            }
        }

        Ok(())
    }

    /// Edge insertion (synchronous buffering)
    pub fn on_edge_insert(
        &self,
        txn_id: crate::transaction::types::TransactionId,
        space_id: u64,
        edge: &crate::core::Edge,
    ) -> Result<(), SyncError> {
        // Extracting edge properties
        let props: Vec<(String, Value)> = edge
            .props
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        // Buffer full-text index operations for string properties
        for (field_name, value) in &props {
            if let Value::String(text) = value {
                let ctx = crate::sync::coordinator::ChangeContext::new_fulltext(
                    space_id,
                    &edge.edge_type,
                    field_name,
                    ChangeType::Insert,
                    format!("{}->{}", edge.src, edge.dst),
                    text.clone(),
                );
                self.sync_coordinator
                    .buffer_operation(txn_id, ctx)
                    .map_err(SyncError::from)?;
            }

            // Buffer vector index operations for vector properties
            if let Some(vector) = value.as_vector() {
                if let Some(ref vector_coord) = self.vector_coordinator {
                    if vector_coord.index_exists(space_id, &edge.edge_type, field_name) {
                        let ctx = crate::sync::vector_sync::VectorChangeContext::new(
                            space_id,
                            &edge.edge_type,
                            field_name,
                            crate::sync::vector_sync::VectorChangeType::from(ChangeType::Insert),
                            crate::sync::vector_sync::VectorPointData {
                                id: format!("{}->{}", edge.src, edge.dst),
                                vector: vector.clone(),
                                payload: std::collections::HashMap::new(),
                            },
                        );
                        vector_coord
                            .buffer_vector_change(txn_id, ctx)
                            .map_err(|e| SyncError::VectorError(e.to_string()))?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Edge deletion (synchronized buffering)
    pub fn on_edge_delete(
        &self,
        txn_id: crate::transaction::types::TransactionId,
        space_id: u64,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<(), SyncError> {
        // Create delete context for full-text index
        let ctx = crate::sync::coordinator::ChangeContext::new_fulltext(
            space_id,
            edge_type,
            "_id",
            ChangeType::Delete,
            format!("{}->{}", src, dst),
            String::new(),
        );
        self.sync_coordinator
            .buffer_operation(txn_id, ctx)
            .map_err(SyncError::from)?;

        Ok(())
    }

    pub fn on_vector_change_with_context_buffered(
        &self,
        txn_id: crate::transaction::types::TransactionId,
        ctx: crate::sync::vector_sync::VectorChangeContext,
    ) -> Result<(), SyncError> {
        if let Some(ref vector_coord) = self.vector_coordinator {
            vector_coord
                .buffer_vector_change(txn_id, ctx)
                .map_err(|e| SyncError::VectorError(e.to_string()))?;
        }
        Ok(())
    }

    pub async fn on_vector_change_with_context(
        &self,
        ctx: crate::sync::vector_sync::VectorChangeContext,
    ) -> Result<(), SyncError> {
        if self.vector_coordinator.is_none() {
            return Ok(());
        }

        if let Some(ref vector_coord) = self.vector_coordinator {
            vector_coord
                .on_vector_change(ctx)
                .await
                .map_err(|e| SyncError::VectorError(e.to_string()))?;
        }

        Ok(())
    }

    pub async fn commit_all(&self) -> Result<(), SyncError> {
        self.sync_coordinator.commit_all().await?;
        Ok(())
    }

    pub async fn prepare_transaction(
        &self,
        txn_id: crate::transaction::types::TransactionId,
    ) -> Result<(), SyncError> {
        self.sync_coordinator.prepare_transaction(txn_id).await?;
        Ok(())
    }

    pub async fn commit_transaction(
        &self,
        txn_id: crate::transaction::types::TransactionId,
    ) -> Result<(), SyncError> {
        // Commit fulltext index
        self.sync_coordinator.commit_transaction(txn_id).await?;

        // Commit vector index
        if let Some(ref vector_coord) = self.vector_coordinator {
            vector_coord
                .commit_transaction(txn_id)
                .await
                .map_err(|e| SyncError::VectorError(e.to_string()))?;
        }

        Ok(())
    }

    pub async fn rollback_transaction(
        &self,
        txn_id: crate::transaction::types::TransactionId,
    ) -> Result<(), SyncError> {
        self.sync_coordinator.rollback_transaction(txn_id).await?;

        // Also rollback vector index buffer
        if let Some(ref vector_coord) = self.vector_coordinator {
            vector_coord.rollback_transaction(txn_id).await;
        }

        Ok(())
    }

    // ===== Synchronous Wrappers for Transaction Operations =====
    //
    // These methods provide synchronous access to async transaction operations.
    // They automatically detect the runtime context and handle appropriately:
    // - In tokio context: Uses `block_in_place` to avoid blocking the executor
    // - Outside tokio: Uses `futures::executor::block_on`

    /// Synchronous wrapper for `prepare_transaction`
    ///
    /// Automatically detects tokio runtime context and handles appropriately.
    pub fn prepare_transaction_sync(
        &self,
        txn_id: crate::transaction::types::TransactionId,
    ) -> Result<(), SyncError> {
        self.execute_sync(|| self.prepare_transaction(txn_id))
    }

    /// Synchronous wrapper for `commit_transaction`
    ///
    /// Automatically detects tokio runtime context and handles appropriately.
    pub fn commit_transaction_sync(
        &self,
        txn_id: crate::transaction::types::TransactionId,
    ) -> Result<(), SyncError> {
        self.execute_sync(|| self.commit_transaction(txn_id))
    }

    /// Synchronous wrapper for `rollback_transaction`
    ///
    /// Automatically detects tokio runtime context and handles appropriately.
    pub fn rollback_transaction_sync(
        &self,
        txn_id: crate::transaction::types::TransactionId,
    ) -> Result<(), SyncError> {
        self.execute_sync(|| self.rollback_transaction(txn_id))
    }

    /// Execute an async operation synchronously with proper runtime detection
    fn execute_sync<F, Fut, T>(&self, f: F) -> Result<T, SyncError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, SyncError>>,
    {
        #[cfg(feature = "tokio")]
        {
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                return tokio::task::block_in_place(|| handle.block_on(f()));
            }
        }

        futures::executor::block_on(f())
    }

    /// Commit vector index transaction
    pub async fn commit_vector_transaction(
        &self,
        txn_id: crate::transaction::types::TransactionId,
    ) -> Result<(), SyncError> {
        if let Some(ref vector_coord) = self.vector_coordinator {
            vector_coord
                .commit_transaction(txn_id)
                .await
                .map_err(|e| SyncError::VectorError(e.to_string()))?;
        }
        Ok(())
    }

    pub fn sync_coordinator(&self) -> &Arc<SyncCoordinator> {
        &self.sync_coordinator
    }

    pub fn vector_coordinator(&self) -> Option<&Arc<VectorSyncCoordinator>> {
        self.vector_coordinator.as_ref()
    }

    /// Get the fulltext manager directly
    pub fn fulltext_manager(&self) -> Arc<crate::search::manager::FulltextIndexManager> {
        self.sync_coordinator.fulltext_manager().clone()
    }

    pub fn is_running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::SeqCst)
    }

    // Dead Letter Queue Management API

    /// Get all dead letter entries
    pub fn get_dead_letter_entries(&self) -> Vec<crate::sync::DeadLetterEntry> {
        if let Some(ref dlq) = self.dead_letter_queue {
            dlq.get_all()
        } else {
            vec![]
        }
    }

    /// Get unrecovered dead letter entries
    pub fn get_unrecovered_entries(&self) -> Vec<crate::sync::DeadLetterEntry> {
        if let Some(ref dlq) = self.dead_letter_queue {
            dlq.get_unrecovered()
        } else {
            vec![]
        }
    }

    /// Get old dead letter entries (older than specified duration)
    pub fn get_old_dead_letter_entries(
        &self,
        age: std::time::Duration,
    ) -> Vec<crate::sync::DeadLetterEntry> {
        if let Some(ref dlq) = self.dead_letter_queue {
            dlq.get_old_entries(age)
        } else {
            vec![]
        }
    }

    /// Remove a dead letter entry by index
    pub fn remove_dead_letter_entry(&self, index: usize) -> Option<crate::sync::DeadLetterEntry> {
        if let Some(ref dlq) = self.dead_letter_queue {
            dlq.remove(index)
        } else {
            None
        }
    }

    /// Get dead letter queue size
    pub fn get_dlq_size(&self) -> usize {
        if let Some(ref dlq) = self.dead_letter_queue {
            dlq.get_all().len()
        } else {
            0
        }
    }

    /// Get unrecovered dead letter queue size
    pub fn get_unrecovered_dlq_size(&self) -> usize {
        if let Some(ref dlq) = self.dead_letter_queue {
            dlq.get_unrecovered().len()
        } else {
            0
        }
    }

    // ===== Unified Index Management API =====

    /// Check if vector index exists
    pub fn vector_index_exists(&self, space_id: u64, tag_name: &str, field_name: &str) -> bool {
        if let Some(ref vector_coord) = self.vector_coordinator {
            vector_coord.index_exists(space_id, tag_name, field_name)
        } else {
            false
        }
    }

    /// Create vector index
    pub async fn create_vector_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        vector_size: usize,
        distance: vector_client::DistanceMetric,
    ) -> Result<String, SyncError> {
        if let Some(ref vector_coord) = self.vector_coordinator {
            vector_coord
                .create_vector_index(space_id, tag_name, field_name, vector_size, distance)
                .await
                .map_err(|e| SyncError::VectorError(e.to_string()))
        } else {
            Err(SyncError::Internal(
                "Vector coordinator not available".to_string(),
            ))
        }
    }

    /// Drop vector index
    pub async fn drop_vector_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Result<(), SyncError> {
        if let Some(ref vector_coord) = self.vector_coordinator {
            vector_coord
                .drop_vector_index(space_id, tag_name, field_name)
                .await
                .map_err(|e| SyncError::VectorError(e.to_string()))
        } else {
            Err(SyncError::Internal(
                "Vector coordinator not available".to_string(),
            ))
        }
    }

    /// Search vector index
    pub async fn search_vector(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        vector: &[f32],
        top_k: usize,
    ) -> Result<Vec<SearchResult>, SyncError> {
        if let Some(ref vector_coord) = self.vector_coordinator {
            let options = crate::sync::vector_sync::SearchOptions::new(
                space_id,
                tag_name,
                field_name,
                vector.to_vec(),
                top_k,
            );
            vector_coord
                .search_with_options(options)
                .await
                .map_err(|e| SyncError::VectorError(e.to_string()))
        } else {
            Err(SyncError::Internal(
                "Vector coordinator not available".to_string(),
            ))
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("Coordinator error: {0}")]
    CoordinatorError(#[from] CoordinatorError),

    #[error("Sync coordinator error: {0}")]
    SyncCoordinatorError(#[from] crate::sync::coordinator::SyncCoordinatorError),

    #[error("Buffer error: {0}")]
    BufferError(String),

    #[error("Vector error: {0}")]
    VectorError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

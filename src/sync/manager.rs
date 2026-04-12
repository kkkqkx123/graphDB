//! Sync Manager
//!
//! Unified synchronization manager using SyncCoordinator.

use crate::core::error::CoordinatorError;
use crate::core::Value;
use crate::search::SyncConfig;
use crate::sync::batch::BatchConfig;
use crate::sync::compensation::{CompensationManager, CompensationStats};
use crate::sync::coordinator::SyncCoordinator;
use crate::sync::recovery::RecoveryManager;
use crate::sync::vector_sync::VectorSyncCoordinator;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct SyncManager {
    sync_coordinator: Arc<SyncCoordinator>,
    vector_coordinator: Option<Arc<VectorSyncCoordinator>>,
    running: Arc<std::sync::atomic::AtomicBool>,
    recovery: Option<Arc<RecoveryManager>>,
    compensation_manager: Option<Arc<CompensationManager>>,
    dead_letter_queue: Option<Arc<crate::sync::DeadLetterQueue>>,
    #[allow(clippy::type_complexity)]
    handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
    #[allow(clippy::type_complexity)]
    compensation_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl Clone for SyncManager {
    fn clone(&self) -> Self {
        Self {
            sync_coordinator: self.sync_coordinator.clone(),
            vector_coordinator: self.vector_coordinator.clone(),
            running: self.running.clone(),
            recovery: self.recovery.clone(),
            compensation_manager: self.compensation_manager.clone(),
            dead_letter_queue: self.dead_letter_queue.clone(),
            handle: Mutex::new(None), // No cloning handle
            compensation_handle: Mutex::new(None), // No cloning handle
        }
    }
}

impl std::fmt::Debug for SyncManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncManager")
            .field("sync_coordinator", &self.sync_coordinator)
            .field("vector_coordinator", &self.vector_coordinator)
            .field("running", &self.running)
            .field("recovery", &self.recovery)
            .finish_non_exhaustive()
    }
}

impl SyncManager {
    pub fn new(sync_coordinator: Arc<SyncCoordinator>) -> Self {
        Self {
            sync_coordinator,
            vector_coordinator: None,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            recovery: None,
            compensation_manager: None,
            dead_letter_queue: None,
            handle: Mutex::new(None),
            compensation_handle: Mutex::new(None),
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

    pub fn with_recovery(
        sync_coordinator: Arc<SyncCoordinator>,
        _config: BatchConfig,
        data_dir: PathBuf,
    ) -> Self {
        let recovery = Arc::new(RecoveryManager::new(data_dir));

        Self {
            sync_coordinator,
            vector_coordinator: None,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            recovery: Some(recovery),
            compensation_manager: None,
            dead_letter_queue: None,
            handle: Mutex::new(None),
            compensation_handle: Mutex::new(None),
        }
    }

    pub fn with_compensation_manager(
        mut self,
        compensation_manager: Arc<CompensationManager>,
    ) -> Self {
        self.compensation_manager = Some(compensation_manager);
        self
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

        // Start recovery manager if present
        if let Some(ref recovery) = self.recovery {
            recovery.start().await?;
        }

        // Start compensation manager if present
        if let Some(ref compensation_manager) = self.compensation_manager {
            let compensation_handle = compensation_manager
                .clone()
                .start_background_task(std::time::Duration::from_secs(60))
                .await;
            
            *self.compensation_handle.lock().await = Some(compensation_handle);
            log::info!("Compensation manager started with automatic compensation enabled");
        }

        Ok(())
    }

    pub async fn stop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);

        // Stop background tasks in the coordinator
        self.sync_coordinator.stop_background_tasks().await;

        // Stop recovery manager if present
        if let Some(ref recovery) = self.recovery {
            recovery.stop().await;
        }

        // Stop compensation manager if present
        if let Some(ref compensation_manager) = self.compensation_manager {
            compensation_manager.stop();
        }

        // Wait for handle to complete
        if let Some(handle) = self.handle.lock().await.take() {
            let _ = handle.await;
        }

        // Wait for compensation handle to complete
        if let Some(comp_handle) = self.compensation_handle.lock().await.take() {
            let _ = comp_handle.await;
        }
    }

    /// Vertex change synchronization (transaction mode)
    /// Note: This method is now only used in non-transactional scenarios, which should use on_vertex_insert or on_vertex_change_with_txn
    pub async fn on_vertex_change(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
        properties: &[(String, Value)],
        change_type: crate::coordinator::ChangeType,
    ) -> Result<(), SyncError> {
        // Direct synchronous processing
        self.sync_coordinator
            .on_vertex_change(
                space_id,
                tag_name,
                vertex_id,
                properties,
                change_type.into(),
            )
            .await?;

        // Simultaneously handle vector index changes (if any)
        if let Some(ref vector_coord) = self.vector_coordinator {
            self.execute_vector_vertex_change_sync(
                space_id,
                tag_name,
                vertex_id,
                properties,
                change_type,
                vector_coord,
            )
            .await?;
        }

        Ok(())
    }

    /// Vertex insertion with transactions (synchronized buffering)
    pub fn on_vertex_insert(
        &self,
        _txn_id: crate::transaction::types::TransactionId,
        space_id: u64,
        vertex: &crate::core::Vertex,
    ) -> Result<(), SyncError> {
        // Create a change context
        let change_type = crate::coordinator::ChangeType::Insert;
        let vertex_id = &vertex.vid;

        // Extract all properties
        let props: Vec<(String, Value)> = vertex
            .tags
            .iter()
            .flat_map(|tag| tag.properties.iter().map(|(k, v)| (k.clone(), v.clone())))
            .collect();

        // Get the first tag name (if any)
        if let Some(first_tag) = vertex.tags.first() {
            let tag_name = &first_tag.name;

            // Buffering operations (synchronized calls)
            futures::executor::block_on(async {
                self.sync_coordinator
                    .on_vertex_change(space_id, tag_name, vertex_id, &props, change_type.into())
                    .await
            })
            .map_err(SyncError::from)?;
        }

        Ok(())
    }

    /// Vertex changes with transactions (synchronous buffering)
    pub fn on_vertex_change_with_txn(
        &self,
        txn_id: crate::transaction::types::TransactionId,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
        properties: &[(String, Value)],
        change_type: crate::coordinator::ChangeType,
    ) -> Result<(), SyncError> {
        // For each attribute create a context and buffer
        for (field_name, value) in properties {
            // Buffer full-text index operations
            if let Value::String(text) = value {
                let ctx = crate::sync::coordinator::ChangeContext::new_fulltext(
                    space_id,
                    tag_name,
                    field_name,
                    change_type.into(),
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

        // Gets the first attribute of type string for full-text indexing
        for (field_name, value) in &props {
            if let Value::String(text) = value {
                let ctx = crate::sync::coordinator::ChangeContext::new_fulltext(
                    space_id,
                    &edge.edge_type,
                    field_name,
                    crate::coordinator::ChangeType::Insert.into(),
                    format!("{}->{}", edge.src, edge.dst),
                    text.clone(),
                );
                self.sync_coordinator
                    .buffer_operation(txn_id, ctx)
                    .map_err(SyncError::from)?;
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
        // Create delete context
        let ctx = crate::sync::coordinator::ChangeContext::new_fulltext(
            space_id,
            edge_type,
            "_id", // Use special field names to identify edges
            crate::coordinator::ChangeType::Delete.into(),
            format!("{}->{}", src, dst),
            String::new(), // No text content is required when deleting
        );
        self.sync_coordinator
            .buffer_operation(txn_id, ctx)
            .map_err(SyncError::from)?;

        Ok(())
    }

    async fn execute_vector_vertex_change_sync(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
        properties: &[(String, Value)],
        change_type: crate::coordinator::ChangeType,
        vector_coord: &Arc<VectorSyncCoordinator>,
    ) -> Result<(), SyncError> {
        use std::collections::HashMap;

        for (field_name, value) in properties {
            if vector_coord.index_exists(space_id, tag_name, field_name) {
                let vector = value.as_vector().unwrap_or_default();
                let mut payload = HashMap::new();
                payload.insert("vertex_id".to_string(), vertex_id.clone());

                let ctx = crate::sync::vector_sync::VectorChangeContext::new(
                    space_id,
                    tag_name,
                    field_name,
                    crate::sync::vector_sync::VectorChangeType::from(change_type),
                    crate::sync::vector_sync::VectorPointData {
                        id: format!("{}", vertex_id),
                        vector,
                        payload,
                    },
                );

                vector_coord
                    .on_vector_change(ctx)
                    .await
                    .map_err(|e| SyncError::VectorError(e.to_string()))?;
            }
        }
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

        // Direct synchronous processing
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
            vector_coord.rollback_transaction(txn_id);
        }
        
        Ok(())
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

    /// Manually trigger compensation for all unrecovered entries
    pub async fn trigger_compensation(&self) -> Result<CompensationStats, SyncError> {
        if let Some(ref compensation_manager) = self.compensation_manager {
            Ok(compensation_manager.process_dead_letter_queue().await)
        } else {
            Err(SyncError::Internal("Compensation manager not initialized".to_string()))
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
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("Coordinator error: {0}")]
    CoordinatorError(#[from] CoordinatorError),

    #[error("Sync coordinator error: {0}")]
    SyncCoordinatorError(#[from] crate::sync::coordinator::SyncCoordinatorError),

    #[error("Recovery error: {0}")]
    RecoveryError(#[from] crate::sync::recovery::RecoveryError),

    #[error("Buffer error: {0}")]
    BufferError(String),

    #[error("Vector error: {0}")]
    VectorError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

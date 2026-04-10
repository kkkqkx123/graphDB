//! Sync Manager
//!
//! Unified synchronization manager with timer-based batch commit.

use crate::coordinator::{ChangeType, FulltextCoordinator};
use crate::core::error::CoordinatorError;
use crate::core::Value;
use crate::search::SyncConfig;
use crate::sync::batch::{BatchConfig, BufferError, TaskBuffer};
use crate::sync::recovery::RecoveryManager;
use crate::sync::task::SyncTask;
use crate::sync::vector_sync::{VectorPoint, VectorSyncCoordinator};
use crate::vector::VectorChangeType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{interval, Duration};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncMode {
    Sync,
    Async,
    Off,
}

pub struct SyncManager {
    fulltext_coordinator: Arc<FulltextCoordinator>,
    vector_coordinator: Option<Arc<VectorSyncCoordinator>>,
    buffer: Arc<TaskBuffer>,
    mode: Arc<RwLock<SyncMode>>,
    running: Arc<std::sync::atomic::AtomicBool>,
    recovery: Option<Arc<RecoveryManager>>,
    #[allow(clippy::type_complexity)]
    handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl Clone for SyncManager {
    fn clone(&self) -> Self {
        Self {
            fulltext_coordinator: self.fulltext_coordinator.clone(),
            vector_coordinator: self.vector_coordinator.clone(),
            buffer: self.buffer.clone(),
            mode: self.mode.clone(),
            running: self.running.clone(),
            recovery: self.recovery.clone(),
            handle: Mutex::new(None), // No cloning handle
        }
    }
}

impl std::fmt::Debug for SyncManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncManager")
            .field("fulltext_coordinator", &self.fulltext_coordinator)
            .field("vector_coordinator", &self.vector_coordinator)
            .field("buffer", &self.buffer)
            .field("mode", &self.mode)
            .field("running", &self.running)
            .field("recovery", &self.recovery)
            .finish_non_exhaustive()
    }
}

impl SyncManager {
    pub fn new(fulltext_coordinator: Arc<FulltextCoordinator>, config: BatchConfig) -> Self {
        let buffer = Arc::new(TaskBuffer::new(fulltext_coordinator.clone(), config));

        Self {
            fulltext_coordinator,
            vector_coordinator: None,
            buffer,
            mode: Arc::new(RwLock::new(SyncMode::Async)),
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            recovery: None,
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
        fulltext_coordinator: Arc<FulltextCoordinator>,
        sync_config: SyncConfig,
    ) -> Self {
        let batch_config = BatchConfig {
            batch_size: sync_config.batch_size,
            commit_interval: Duration::from_millis(sync_config.commit_interval_ms),
            max_wait_time: Duration::from_secs(5),
            queue_capacity: sync_config.queue_size,
            failure_policy: sync_config.failure_policy,
        };
        let buffer = Arc::new(TaskBuffer::new(fulltext_coordinator.clone(), batch_config));

        Self {
            fulltext_coordinator,
            vector_coordinator: None,
            buffer,
            mode: Arc::new(RwLock::new(sync_config.mode)),
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            recovery: None,
            handle: Mutex::new(None),
        }
    }

    pub fn with_mode(fulltext_coordinator: Arc<FulltextCoordinator>, mode: SyncMode) -> Self {
        let buffer = Arc::new(TaskBuffer::new(
            fulltext_coordinator.clone(),
            BatchConfig::default(),
        ));

        Self {
            fulltext_coordinator,
            vector_coordinator: None,
            buffer,
            mode: Arc::new(RwLock::new(mode)),
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            recovery: None,
            handle: Mutex::new(None),
        }
    }

    pub fn with_recovery(
        fulltext_coordinator: Arc<FulltextCoordinator>,
        config: BatchConfig,
        data_dir: PathBuf,
    ) -> Self {
        let buffer = Arc::new(TaskBuffer::new(
            fulltext_coordinator.clone(),
            config.clone(),
        ));
        let recovery = Arc::new(RecoveryManager::new(buffer.clone(), data_dir));

        Self {
            fulltext_coordinator,
            vector_coordinator: None,
            buffer,
            mode: Arc::new(RwLock::new(SyncMode::Async)),
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            recovery: Some(recovery),
            handle: Mutex::new(None),
        }
    }

    pub fn with_sync_config_and_recovery(
        fulltext_coordinator: Arc<FulltextCoordinator>,
        sync_config: SyncConfig,
        data_dir: PathBuf,
    ) -> Self {
        let batch_config = BatchConfig {
            batch_size: sync_config.batch_size,
            commit_interval: Duration::from_millis(sync_config.commit_interval_ms),
            max_wait_time: Duration::from_secs(5),
            queue_capacity: sync_config.queue_size,
            failure_policy: sync_config.failure_policy,
        };
        let buffer = Arc::new(TaskBuffer::new(fulltext_coordinator.clone(), batch_config));
        let recovery = Arc::new(RecoveryManager::new(buffer.clone(), data_dir));

        Self {
            fulltext_coordinator,
            vector_coordinator: None,
            buffer,
            mode: Arc::new(RwLock::new(sync_config.mode)),
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            recovery: Some(recovery),
            handle: Mutex::new(None),
        }
    }

    pub async fn on_vertex_change(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
        properties: &[(String, Value)],
        change_type: ChangeType,
    ) -> Result<(), SyncError> {
        let mode = *self.mode.read().await;

        match mode {
            SyncMode::Sync => {
                let props: std::collections::HashMap<_, _> = properties.iter().cloned().collect();
                self.fulltext_coordinator
                    .on_vertex_change(space_id, tag_name, vertex_id, &props, change_type)
                    .await?;

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
            }
            SyncMode::Async => {
                let task = SyncTask::vertex_change(
                    space_id,
                    tag_name,
                    vertex_id,
                    properties.to_vec(),
                    change_type,
                );

                self.buffer.submit(task).await?;
            }
            SyncMode::Off => {}
        }

        Ok(())
    }

    async fn execute_vector_vertex_change_sync(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
        properties: &[(String, Value)],
        change_type: ChangeType,
        vector_coord: &Arc<VectorSyncCoordinator>,
    ) -> Result<(), SyncError> {
        for (field_name, value) in properties {
            if vector_coord.index_exists(space_id, tag_name, field_name) {
                let vector = value.as_vector().unwrap_or_default();
                let mut payload = HashMap::new();
                payload.insert("vertex_id".to_string(), vertex_id.clone());

                let ctx = crate::sync::vector_sync::VectorChangeContext::new(
                    space_id,
                    tag_name,
                    field_name,
                    VectorChangeType::from(change_type),
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

    pub async fn on_vector_change_with_context(
        &self,
        ctx: crate::sync::vector_sync::VectorChangeContext,
    ) -> Result<(), SyncError> {
        if self.vector_coordinator.is_none() {
            return Ok(());
        }

        let mode = *self.mode.read().await;

        match mode {
            SyncMode::Sync => {
                if let Some(ref vector_coord) = self.vector_coordinator {
                    vector_coord
                        .on_vector_change(ctx)
                        .await
                        .map_err(|e| SyncError::VectorError(e.to_string()))?;
                }
            }
            SyncMode::Async => {
                let task = SyncTask::VectorChange {
                    task_id: uuid::Uuid::new_v4().to_string(),
                    space_id: ctx.location.space_id,
                    tag_name: ctx.location.tag_name.clone(),
                    field_name: ctx.location.field_name.clone(),
                    vertex_id: Value::String(ctx.data.id.clone()),
                    vector: Some(ctx.data.vector.clone()),
                    payload: ctx.data.payload.clone(),
                    change_type: ctx.change_type,
                    created_at: chrono::Utc::now(),
                };

                self.buffer.submit(task).await?;
            }
            SyncMode::Off => {}
        }

        Ok(())
    }

    pub async fn on_vector_change(
        &self,
        ctx: crate::sync::vector_sync::VectorChangeContext,
    ) -> Result<(), SyncError> {
        self.on_vector_change_with_context(ctx).await
    }

    pub async fn start(&self) {
        let buffer = self.buffer.clone();
        let vector_coord = self.vector_coordinator.clone();
        let running = self.running.clone();
        let commit_interval = self.buffer.config().commit_interval;
        let batch_size = self.buffer.config().batch_size;

        running.store(true, std::sync::atomic::Ordering::SeqCst);

        let handle = tokio::spawn(async move {
            let mut ticker = interval(commit_interval);

            while running.load(std::sync::atomic::Ordering::SeqCst) {
                ticker.tick().await;

                // Handle bulk commits of full-text index buffers
                let keys = buffer.get_buffer_keys().await;
                for key in keys {
                    if buffer.should_commit(&key).await {
                        if let Err(e) = buffer.commit_batch(key.clone()).await {
                            log::error!("Batch commit failed: {:?}", e);
                        }
                        if let Err(e) = buffer.commit_deletions(key).await {
                            log::error!("Batch deletions commit failed: {:?}", e);
                        }
                    }
                }

                // Processing Vector Batch Tasks
                if let Some(ref vc) = vector_coord {
                    if let Err(e) = Self::process_vector_batch_tasks(vc, &buffer, batch_size).await {
                        log::error!("Vector batch processing failed: {:?}", e);
                    }
                }
            }
        });

        let mut h = self.handle.lock().await;
        *h = Some(handle);
    }

    async fn process_vector_batch_tasks(
        vector_coord: &Arc<VectorSyncCoordinator>,
        buffer: &Arc<TaskBuffer>,
        batch_size: usize,
    ) -> Result<(), SyncError> {
        // Collect vector batch tasks from queues
        let vector_tasks = buffer.drain_vector_tasks(batch_size).await;

        if !vector_tasks.is_empty() {
            // cluster
            let mut upsert_by_collection: HashMap<String, Vec<crate::sync::task::VectorPointData>> = HashMap::new();
            let mut delete_by_collection: HashMap<String, Vec<String>> = HashMap::new();

            for task in vector_tasks {
                match task {
                    SyncTask::VectorBatchUpsert {
                        space_id,
                        tag_name,
                        field_name,
                        points,
                        ..
                    } => {
                        let collection_name = crate::sync::vector_sync::VectorIndexLocation::new(
                            space_id, &tag_name, &field_name
                        ).to_collection_name();

                        upsert_by_collection
                            .entry(collection_name)
                            .or_insert_with(Vec::new)
                            .extend(points);
                    }
                    SyncTask::VectorBatchDelete {
                        space_id,
                        tag_name,
                        field_name,
                        point_ids,
                        ..
                    } => {
                        let collection_name = crate::sync::vector_sync::VectorIndexLocation::new(
                            space_id, &tag_name, &field_name
                        ).to_collection_name();

                        delete_by_collection
                            .entry(collection_name)
                            .or_insert_with(Vec::new)
                            .extend(point_ids);
                    }
                    _ => continue,
                }
            }

            // batch upsert
            for (collection_name, points) in upsert_by_collection {
                if !points.is_empty() {
                    let vector_points: Vec<VectorPoint> = points
                        .into_iter()
                        .map(|p| {
                            let mut payload = HashMap::new();
                            for (k, v) in p.payload {
                                if let Ok(json) = serde_json::to_value(&v) {
                                    payload.insert(k, json);
                                }
                            }
                            VectorPoint::new(p.id, p.vector).with_payload(payload)
                        })
                        .collect();

                    if let Err(e) = vector_coord.upsert_batch(&collection_name, vector_points).await {
                        log::error!("Vector batch upsert failed for {}: {:?}", collection_name, e);
                    } else {
                        log::debug!("Batch upserted {} vectors to {}", points.len(), collection_name);
                    }
                }
            }

            // Batch delete
            for (collection_name, point_ids) in delete_by_collection {
                if !point_ids.is_empty() {
                    let refs: Vec<&str> = point_ids.iter().map(|s| s.as_str()).collect();
                    if let Err(e) = vector_coord.delete_batch(&collection_name, refs).await {
                        log::error!("Vector batch delete failed for {}: {:?}", collection_name, e);
                    } else {
                        log::debug!("Batch deleted {} vectors from {}", point_ids.len(), collection_name);
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn stop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);

        if let Some(handle) = self.handle.lock().await.take() {
            let _ = handle.await;
        }
    }

    pub async fn get_mode(&self) -> SyncMode {
        *self.mode.read().await
    }

    pub async fn set_mode(&self, mode: SyncMode) {
        let mut current = self.mode.write().await;
        *current = mode;
    }

    pub async fn force_commit(&self) -> Result<(), SyncError> {
        // Force commit index buffer
        let results = self.buffer.commit_all().await;

        for (key, result) in results {
            if let Err(e) = result {
                log::error!("Commit failed {:?}: {:?}", key, e);
                return Err(SyncError::CommitError(e.to_string()));
            }
        }

        Ok(())
    }

    pub fn buffer(&self) -> &Arc<TaskBuffer> {
        &self.buffer
    }

    pub fn fulltext_coordinator(&self) -> &Arc<FulltextCoordinator> {
        &self.fulltext_coordinator
    }

    pub fn vector_coordinator(&self) -> Option<&Arc<VectorSyncCoordinator>> {
        self.vector_coordinator.as_ref()
    }

    pub fn recovery(&self) -> Option<&Arc<RecoveryManager>> {
        self.recovery.as_ref()
    }

    // ==================== Two-Phase Commit API ====================

    /// Phase 1: Prepare for submission
    /// Collects pending updates and performs index synchronization asynchronously
    pub async fn prepare_commit(
        &self,
        txn_id: crate::transaction::types::TransactionId,
    ) -> Result<Arc<crate::transaction::SyncHandle>, SyncError> {
        use crate::transaction::sync_handle::SyncHandle;
        use tokio::sync::oneshot;

        // 1. Get pending updates from buffer (async)
        let pending_updates = self.buffer.get_pending_updates(txn_id).await;

        // 2. Creating synchronization handles
        let (tx, rx) = oneshot::channel();
        let handle = Arc::new(SyncHandle::new(txn_id, pending_updates.clone(), tx, rx));

        // 3. Perform index synchronization asynchronously
        let handle_clone = handle.clone();
        let sync_manager = self.clone();

        tokio::spawn(async move {
            handle_clone.set_state(crate::transaction::sync_handle::SyncHandleState::Syncing);

            // Perform the actual index synchronization
            let result = sync_manager.execute_pending_updates(&handle_clone.pending_updates).await;

            match result {
                Ok(()) => {
                    handle_clone.set_state(crate::transaction::sync_handle::SyncHandleState::Synced);
                    let _ = handle_clone.completion_tx.send(Ok(()));
                }
                Err(e) => {
                    handle_clone.set_state(crate::transaction::sync_handle::SyncHandleState::SyncFailed);
                    let _ = handle_clone.completion_tx.send(Err(e));
                }
            }
        });

        Ok(handle)
    }

    /// Perform pending index updates
    async fn execute_pending_updates(
        &self,
        updates: &[crate::sync::pending_update::PendingIndexUpdate],
    ) -> Result<(), SyncError> {
        // Updates grouped by type
        let mut fulltext_upserts = Vec::new();
        let mut fulltext_deletes = Vec::new();
        let mut vector_upserts: HashMap<String, Vec<crate::sync::task::VectorPointData>> =
            HashMap::new();
        let mut vector_deletes: HashMap<String, Vec<String>> = HashMap::new();

        for update in updates {
            match update.change_type {
                ChangeType::Insert | ChangeType::Update => {
                    // full-text index
                    if let Some(ref content) = update.content {
                        fulltext_upserts.push((update.doc_id.clone(), content.clone()));
                    }

                    // Vector index (if any)
                    if let Some(ref vector_coord) = self.vector_coordinator {
                        if vector_coord.index_exists(update.space_id, &update.tag_name, &update.field_name) {
                            // Extract vector data from update
                            if let Some(ref vector) = update.vector_data {
                                let collection_name = crate::sync::vector_sync::VectorIndexLocation::new(
                                    update.space_id,
                                    &update.tag_name,
                                    &update.field_name,
                                ).to_collection_name();

                                let mut payload = HashMap::new();
                                payload.insert("doc_id".to_string(), Value::String(update.doc_id.clone()));
                                payload.insert("tag_name".to_string(), Value::String(update.tag_name.clone()));
                                payload.insert("field_name".to_string(), Value::String(update.field_name.clone()));

                                let point_data = crate::sync::task::VectorPointData {
                                    id: update.doc_id.clone(),
                                    vector: vector.clone(),
                                    payload,
                                };

                                vector_upserts
                                    .entry(collection_name)
                                    .or_insert_with(Vec::new)
                                    .push(point_data);
                            }
                        }
                    }
                }
                ChangeType::Delete => {
                    fulltext_deletes.push(update.doc_id.clone());

                    if let Some(ref vector_coord) = self.vector_coordinator {
                        if vector_coord.index_exists(update.space_id, &update.tag_name, &update.field_name) {
                            let collection_name =
                                crate::sync::vector_sync::VectorIndexLocation::new(
                                    update.space_id,
                                    &update.tag_name,
                                    &update.field_name,
                                )
                                .to_collection_name();
                            vector_deletes
                                .entry(collection_name)
                                .or_insert_with(Vec::new)
                                .push(update.doc_id.clone());
                        }
                    }
                }
            }
        }

        // Batch submission of full-text index
        if !fulltext_upserts.is_empty() {
            let ctx = crate::coordinator::FulltextBatchContext::new(
                fulltext_upserts.into_iter().collect(),
                fulltext_deletes.into_iter().collect(),
            );
            self.fulltext_coordinator.batch_sync(ctx).await?;
        }

        // Batch commit vector index
        if let Some(ref vector_coord) = self.vector_coordinator {
            for (collection_name, points) in vector_upserts {
                vector_coord
                    .upsert_points(&collection_name, points)
                    .await
                    .map_err(|e| SyncError::VectorError(e.to_string()))?;
            }

            for (collection_name, ids) in vector_deletes {
                vector_coord
                    .delete_points(&collection_name, ids)
                    .await
                    .map_err(|e| SyncError::VectorError(e.to_string()))?;
            }
        }

        Ok(())
    }

    /// Phase 2: Confirm submission
    pub fn commit_sync(&self, handle: Arc<crate::transaction::SyncHandle>) -> Result<(), SyncError> {
        // Verification Status
        if handle.state() != crate::transaction::sync_handle::SyncHandleState::Synced {
            return Err(SyncError::InvalidState);
        }

        // Mark as confirmed
        handle.set_state(crate::transaction::sync_handle::SyncHandleState::Confirmed);

        // Clean up resources (the actual indexing has been synchronized in the prepare phase)
        Ok(())
    }

    /// Phase 2: Cancel submission
    pub async fn abort_sync(&self, handle: Arc<crate::transaction::SyncHandle>) -> Result<(), SyncError> {
        // Marked as canceled
        handle.set_state(crate::transaction::sync_handle::SyncHandleState::Cancelled);

        // In FailClosed mode, you need to roll back synchronized indexes
        if self.buffer.config().failure_policy == crate::search::SyncFailurePolicy::FailClosed {
            self.rollback_pending_updates(&handle.pending_updates).await?;
        }

        Ok(())
    }

    /// Rolling back pending updates
    async fn rollback_pending_updates(
        &self,
        updates: &[crate::sync::pending_update::PendingIndexUpdate],
    ) -> Result<(), SyncError> {
        // Perform rollback in reverse order
        for update in updates.iter().rev() {
            match update.change_type {
                ChangeType::Insert => {
                    // Deleting an Inserted Index
                    if let Some(ref _content) = update.content {
                        if let Some(engine) = self.fulltext_coordinator.get_engine(
                            update.space_id,
                            &update.tag_name,
                            &update.field_name,
                        ) {
                            engine
                                .delete(&update.doc_id)
                                .await
                                .map_err(|e| SyncError::CoordinatorError(CoordinatorError::FulltextError(e.into())))?;
                        }
                    }
                }
                ChangeType::Delete => {
                    // Recovering deleted indexes using old_content
                    if let Some(ref old_content) = update.old_content {
                        if let Some(engine) = self.fulltext_coordinator.get_engine(
                            update.space_id,
                            &update.tag_name,
                            &update.field_name,
                        ) {
                            engine
                                .index(&update.doc_id, old_content)
                                .await
                                .map_err(|e| SyncError::CoordinatorError(CoordinatorError::FulltextError(e.into())))?;
                        }
                    }
                }
                ChangeType::Update => {
                    // Rollback to old value using old_content
                    if let Some(ref old_content) = update.old_content {
                        if let Some(engine) = self.fulltext_coordinator.get_engine(
                            update.space_id,
                            &update.tag_name,
                            &update.field_name,
                        ) {
                            engine
                                .index(&update.doc_id, old_content)
                                .await
                                .map_err(|e| SyncError::CoordinatorError(CoordinatorError::FulltextError(e.into())))?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Buffered index updates (within a transaction)
    pub async fn buffer_update(
        &self,
        txn_id: crate::transaction::types::TransactionId,
        update: crate::transaction::PendingIndexUpdate,
    ) -> Result<(), SyncError> {
        self.buffer.add_pending_update(txn_id, update).await
            .map_err(|e| SyncError::BufferError(e))
    }

    /// Get pending updates
    pub async fn get_pending_updates(
        &self,
        txn_id: crate::transaction::types::TransactionId,
    ) -> Vec<crate::transaction::PendingIndexUpdate> {
        self.buffer.take_pending_updates(txn_id).await
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("Queue error: {0}")]
    Queue(#[from] crate::sync::queue::QueueError),
    #[error("Buffer error: {0}")]
    BufferError(#[from] BufferError),
    #[error("Coordinator error: {0}")]
    CoordinatorError(#[from] CoordinatorError),
    #[error("Commit error: {0}")]
    CommitError(String),
    #[error("Recovery error: {0}")]
    RecoveryError(String),
    #[error("Vector error: {0}")]
    VectorError(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Invalid state")]
    InvalidState,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::{FulltextConfig, FulltextIndexManager, SyncConfig};
    use tempfile::TempDir;

    async fn create_test_sync_manager() -> (Arc<FulltextCoordinator>, SyncManager, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = FulltextConfig {
            enabled: true,
            index_path: temp_dir.path().to_path_buf(),
            default_engine: crate::search::EngineType::Bm25,
            sync: SyncConfig::default(),
            bm25: Default::default(),
            inversearch: Default::default(),
            cache_size: 100,
            max_result_cache: 1000,
            result_cache_ttl_secs: 60,
        };

        let manager =
            Arc::new(FulltextIndexManager::new(config).expect("Failed to create manager"));
        let coordinator = Arc::new(FulltextCoordinator::new(manager));
        let sync_config = SyncConfig {
            mode: SyncMode::Async,
            queue_size: 100,
            commit_interval_ms: 100,
            batch_size: 10,
        };
        let sync_manager = SyncManager::with_sync_config(coordinator.clone(), sync_config);

        (coordinator, sync_manager, temp_dir)
    }

    #[tokio::test]
    async fn test_sync_mode_default() {
        let (_, sync_manager, _temp) = create_test_sync_manager().await;
        let mode = sync_manager.get_mode().await;
        assert_eq!(mode, SyncMode::Async);
    }

    #[tokio::test]
    async fn test_sync_mode_set_and_get() {
        let (_, sync_manager, _temp) = create_test_sync_manager().await;

        sync_manager.set_mode(SyncMode::Sync).await;
        assert_eq!(sync_manager.get_mode().await, SyncMode::Sync);

        sync_manager.set_mode(SyncMode::Off).await;
        assert_eq!(sync_manager.get_mode().await, SyncMode::Off);

        sync_manager.set_mode(SyncMode::Async).await;
        assert_eq!(sync_manager.get_mode().await, SyncMode::Async);
    }

    #[tokio::test]
    async fn test_sync_mode_off_skips_processing() {
        let (_, sync_manager, _temp) = create_test_sync_manager().await;

        sync_manager.set_mode(SyncMode::Off).await;

        let result = sync_manager
            .on_vertex_change(
                1,
                "test_tag",
                &crate::core::Value::Int(1),
                &[(
                    "name".to_string(),
                    crate::core::Value::String("test".to_string()),
                )],
                crate::coordinator::ChangeType::Insert,
            )
            .await;

        assert!(result.is_ok());
    }

    #[test]
    fn test_sync_mode_serde() {
        let mode = SyncMode::Sync;
        let json = serde_json::to_string(&mode).expect("Failed to serialize");
        assert_eq!(json, "\"sync\"");

        let mode = SyncMode::Async;
        let json = serde_json::to_string(&mode).expect("Failed to serialize");
        assert_eq!(json, "\"async\"");

        let mode = SyncMode::Off;
        let json = serde_json::to_string(&mode).expect("Failed to serialize");
        assert_eq!(json, "\"off\"");
    }

    #[test]
    fn test_sync_mode_deserialize() {
        let mode: SyncMode = serde_json::from_str("\"sync\"").expect("Failed to deserialize");
        assert_eq!(mode, SyncMode::Sync);

        let mode: SyncMode = serde_json::from_str("\"async\"").expect("Failed to deserialize");
        assert_eq!(mode, SyncMode::Async);

        let mode: SyncMode = serde_json::from_str("\"off\"").expect("Failed to deserialize");
        assert_eq!(mode, SyncMode::Off);
    }

    #[test]
    fn test_sync_config_with_sync_mode() {
        let sync_config = SyncConfig {
            mode: SyncMode::Sync,
            queue_size: 5000,
            commit_interval_ms: 500,
            batch_size: 50,
        };

        assert_eq!(sync_config.mode, SyncMode::Sync);
        assert_eq!(sync_config.queue_size, 5000);
        assert_eq!(sync_config.commit_interval_ms, 500);
        assert_eq!(sync_config.batch_size, 50);
    }

    #[test]
    fn test_sync_config_default() {
        let config = SyncConfig::default();
        assert_eq!(config.mode, SyncMode::Async);
        assert_eq!(config.queue_size, 10000);
        assert_eq!(config.commit_interval_ms, 1000);
        assert_eq!(config.batch_size, 100);
    }
}

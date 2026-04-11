//! Sync Manager
//!
//! Unified synchronization manager using SyncCoordinator.

use crate::core::error::CoordinatorError;
use crate::core::Value;
use crate::search::SyncConfig;
use crate::sync::batch::BatchConfig;
use crate::sync::coordinator::SyncCoordinator;
use crate::sync::recovery::RecoveryManager;
use crate::sync::vector_sync::VectorSyncCoordinator;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncMode {
    Sync,
    Async,
    Off,
}

pub struct SyncManager {
    sync_coordinator: Arc<SyncCoordinator>,
    vector_coordinator: Option<Arc<VectorSyncCoordinator>>,
    mode: Arc<RwLock<SyncMode>>,
    running: Arc<std::sync::atomic::AtomicBool>,
    recovery: Option<Arc<RecoveryManager>>,
    #[allow(clippy::type_complexity)]
    handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl Clone for SyncManager {
    fn clone(&self) -> Self {
        Self {
            sync_coordinator: self.sync_coordinator.clone(),
            vector_coordinator: self.vector_coordinator.clone(),
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
            .field("sync_coordinator", &self.sync_coordinator)
            .field("vector_coordinator", &self.vector_coordinator)
            .field("mode", &self.mode)
            .field("running", &self.running)
            .field("recovery", &self.recovery)
            .finish_non_exhaustive()
    }
}

impl SyncManager {
    pub fn new(sync_coordinator: Arc<SyncCoordinator>, _config: BatchConfig) -> Self {
        Self {
            sync_coordinator,
            vector_coordinator: None,
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
        sync_coordinator: Arc<SyncCoordinator>,
        sync_config: SyncConfig,
    ) -> Self {
        let batch_config = BatchConfig::from(sync_config);
        Self::new(sync_coordinator, batch_config)
    }

    pub fn with_mode(sync_coordinator: Arc<SyncCoordinator>, mode: SyncMode) -> Self {
        let manager = Self::new(sync_coordinator, BatchConfig::default());
        manager.set_mode(mode);
        manager
    }

    pub fn with_recovery(
        sync_coordinator: Arc<SyncCoordinator>,
        config: BatchConfig,
        data_dir: PathBuf,
    ) -> Self {
        let recovery = Arc::new(RecoveryManager::new(sync_coordinator.clone(), data_dir));

        Self {
            sync_coordinator,
            vector_coordinator: None,
            mode: Arc::new(RwLock::new(SyncMode::Async)),
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            recovery: Some(recovery),
            handle: Mutex::new(None),
        }
    }

    pub fn set_mode(&self, mode: SyncMode) {
        self.sync_coordinator.set_mode(mode);
    }

    pub async fn start(&self) -> Result<(), SyncError> {
        if self.running.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }

        self.running.store(true, std::sync::atomic::Ordering::SeqCst);

        // Start background tasks in the coordinator
        self.sync_coordinator.start_background_tasks().await;

        // Start recovery manager if present
        if let Some(ref recovery) = self.recovery {
            recovery.start().await?;
        }

        Ok(())
    }

    pub async fn stop(&self) {
        self.running.store(false, std::sync::atomic::Ordering::SeqCst);

        // Stop background tasks in the coordinator
        self.sync_coordinator.stop_background_tasks().await;

        // Stop recovery manager if present
        if let Some(ref recovery) = self.recovery {
            recovery.stop().await;
        }

        // Wait for handle to complete
        if let Some(handle) = self.handle.lock().await.take() {
            let _ = handle.await;
        }
    }

    pub async fn on_vertex_change(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
        properties: &[(String, Value)],
        change_type: crate::coordinator::ChangeType,
    ) -> Result<(), SyncError> {
        let mode = *self.mode.read().await;

        match mode {
            SyncMode::Sync => {
                // Direct synchronous processing
                self.sync_coordinator
                    .on_vertex_change(space_id, tag_name, vertex_id, properties, change_type.into())
                    .await?;

                // Also process vector changes if vector coordinator is available
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
                // Async processing through coordinator
                self.sync_coordinator
                    .on_vertex_change(space_id, tag_name, vertex_id, properties, change_type.into())
                    .await?;
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
                // Queue for async processing
                // TODO: Implement async queue if needed
            }
            SyncMode::Off => {}
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
        self.sync_coordinator
            .prepare_transaction(txn_id)
            .await?;
        Ok(())
    }

    pub async fn commit_transaction(
        &self,
        txn_id: crate::transaction::types::TransactionId,
    ) -> Result<(), SyncError> {
        self.sync_coordinator
            .commit_transaction(txn_id)
            .await?;
        Ok(())
    }

    pub async fn rollback_transaction(
        &self,
        txn_id: crate::transaction::types::TransactionId,
    ) -> Result<(), SyncError> {
        self.sync_coordinator
            .rollback_transaction(txn_id)
            .await?;
        Ok(())
    }

    pub fn sync_coordinator(&self) -> &Arc<SyncCoordinator> {
        &self.sync_coordinator
    }

    pub fn vector_coordinator(&self) -> Option<&Arc<VectorSyncCoordinator>> {
        self.vector_coordinator.as_ref()
    }

    pub fn mode(&self) -> SyncMode {
        self.mode.try_read().map(|g| *g).unwrap_or(SyncMode::Off)
    }

    pub fn is_running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::SeqCst)
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

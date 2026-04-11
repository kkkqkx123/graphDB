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
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct SyncManager {
    sync_coordinator: Arc<SyncCoordinator>,
    vector_coordinator: Option<Arc<VectorSyncCoordinator>>,
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
            handle: Mutex::new(None),
        }
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

    /// 顶点变更同步（事务模式）
    /// 注意：此方法现在仅用于非事务场景，事务场景应使用 on_vertex_insert 或 on_vertex_change_with_txn
    pub async fn on_vertex_change(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
        properties: &[(String, Value)],
        change_type: crate::coordinator::ChangeType,
    ) -> Result<(), SyncError> {
        // 直接同步处理
        self.sync_coordinator
            .on_vertex_change(space_id, tag_name, vertex_id, properties, change_type.into())
            .await?;

        // 同时处理向量索引变更（如果有）
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

    /// 带事务的顶点插入（同步缓冲）
    pub fn on_vertex_insert(
        &self,
        _txn_id: crate::transaction::types::TransactionId,
        space_id: u64,
        vertex: &crate::core::Vertex,
    ) -> Result<(), SyncError> {
        // 创建变更上下文
        let change_type = crate::coordinator::ChangeType::Insert;
        let vertex_id = &vertex.vid;

        // 提取所有属性
        let props: Vec<(String, Value)> = vertex
            .tags
            .iter()
            .flat_map(|tag| tag.properties.iter().map(|(k, v)| (k.clone(), v.clone())))
            .collect();

        // 获取第一个 tag 名称（如果有）
        if let Some(first_tag) = vertex.tags.first() {
            let tag_name = &first_tag.name;

            // 缓冲操作（同步调用）
            futures::executor::block_on(async {
                self.sync_coordinator
                    .on_vertex_change(space_id, tag_name, vertex_id, &props, change_type.into())
                    .await
            })
            .map_err(SyncError::from)?;
        }

        Ok(())
    }

    /// 带事务的顶点变更（同步缓冲）
    pub fn on_vertex_change_with_txn(
        &self,
        txn_id: crate::transaction::types::TransactionId,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
        properties: &[(String, Value)],
        change_type: crate::coordinator::ChangeType,
    ) -> Result<(), SyncError> {
        // 对于每个属性创建上下文并缓冲
        for (field_name, value) in properties {
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

        // 直接同步处理
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

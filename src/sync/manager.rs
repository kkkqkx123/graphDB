//! Sync Manager
//!
//! Unified synchronization manager with timer-based batch commit.

use crate::coordinator::{ChangeType, FulltextCoordinator};
use crate::core::Value;
use crate::sync::batch::{BatchConfig, BufferError, TaskBuffer};
use crate::sync::recovery::RecoveryManager;
use crate::sync::task::SyncTask;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{interval, Duration};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncMode {
    Sync,
    Async,
    Off,
}

pub struct SyncManager {
    coordinator: Arc<FulltextCoordinator>,
    buffer: Arc<TaskBuffer>,
    mode: Arc<RwLock<SyncMode>>,
    running: Arc<std::sync::atomic::AtomicBool>,
    recovery: Option<Arc<RecoveryManager>>,
    handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl SyncManager {
    pub fn new(coordinator: Arc<FulltextCoordinator>, config: BatchConfig) -> Self {
        let buffer = Arc::new(TaskBuffer::new(coordinator.clone(), config));

        Self {
            coordinator,
            buffer,
            mode: Arc::new(RwLock::new(SyncMode::Async)),
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            recovery: None,
            handle: Mutex::new(None),
        }
    }

    pub fn with_mode(coordinator: Arc<FulltextCoordinator>, mode: SyncMode) -> Self {
        let buffer = Arc::new(TaskBuffer::new(coordinator.clone(), BatchConfig::default()));

        Self {
            coordinator,
            buffer,
            mode: Arc::new(RwLock::new(mode)),
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            recovery: None,
            handle: Mutex::new(None),
        }
    }

    pub fn with_recovery(
        coordinator: Arc<FulltextCoordinator>,
        config: BatchConfig,
        data_dir: PathBuf,
    ) -> Self {
        let buffer = Arc::new(TaskBuffer::new(coordinator.clone(), config.clone()));
        let recovery = Arc::new(RecoveryManager::new(buffer.clone(), data_dir));

        Self {
            coordinator,
            buffer,
            mode: Arc::new(RwLock::new(SyncMode::Async)),
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
                self.coordinator
                    .on_vertex_change(space_id, tag_name, vertex_id, &props, change_type)
                    .await
                    .map_err(|e| SyncError::CoordinatorError(e.to_string()))?;
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

    pub async fn start(&self) {
        let buffer = self.buffer.clone();
        let running = self.running.clone();
        let commit_interval = self.buffer.config().commit_interval;
        let recovery = self.recovery.clone();

        running.store(true, std::sync::atomic::Ordering::SeqCst);

        let handle = tokio::spawn(async move {
            let mut ticker = interval(commit_interval);

            while running.load(std::sync::atomic::Ordering::SeqCst) {
                ticker.tick().await;

                if let Some(task) = buffer.try_next_task().await {
                    Self::execute_task(&buffer, &task, recovery.as_ref()).await;
                }

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
            }
        });

        let mut h = self.handle.lock().await;
        *h = Some(handle);
    }

    pub async fn stop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);

        if let Some(handle) = self.handle.lock().await.take() {
            let _ = handle.await;
        }
    }

    pub async fn process_queue(&self) {
        loop {
            if !self.running.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }

            match self.buffer.next_task().await {
                Some(task) => {
                    Self::execute_task(&self.buffer, &task, self.recovery.as_ref()).await;
                }
                None => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    async fn execute_task(
        buffer: &TaskBuffer,
        task: &SyncTask,
        recovery: Option<&Arc<RecoveryManager>>,
    ) {
        let coordinator = buffer.coordinator();
        let result = match task {
            SyncTask::VertexChange {
                space_id,
                tag_name,
                vertex_id,
                properties,
                change_type,
                ..
            } => {
                let props: std::collections::HashMap<_, _> = properties.iter().cloned().collect();
                coordinator
                    .on_vertex_change(*space_id, tag_name, vertex_id, &props, *change_type)
                    .await
            }
            SyncTask::BatchIndex {
                space_id,
                tag_name,
                field_name,
                documents,
                ..
            } => {
                if let Some(engine) = coordinator.get_engine(*space_id, tag_name, field_name) {
                    engine.index_batch(documents.clone()).await
                } else {
                    Ok(())
                }
            }
            SyncTask::BatchDelete {
                space_id,
                tag_name,
                field_name,
                doc_ids,
                ..
            } => {
                if let Some(engine) = coordinator.get_engine(*space_id, tag_name, field_name) {
                    let mut last_error = None;
                    for doc_id in doc_ids {
                        if let Err(e) = engine.delete(doc_id).await {
                            last_error = Some(e);
                        }
                    }
                    if let Some(e) = last_error {
                        Err(e)
                    } else {
                        engine.commit().await
                    }
                } else {
                    Ok(())
                }
            }
            SyncTask::CommitIndex {
                space_id,
                tag_name,
                field_name,
                ..
            } => {
                if let Some(engine) = coordinator.get_engine(*space_id, tag_name, field_name) {
                    engine.commit().await
                } else {
                    Ok(())
                }
            }
            SyncTask::RebuildIndex {
                space_id,
                tag_name,
                field_name,
                ..
            } => {
                coordinator
                    .rebuild_index(*space_id, tag_name, field_name)
                    .await
            }
        };

        match result {
            Ok(_) => {
                log::debug!("Task executed successfully: {}", task.task_id());
            }
            Err(e) => {
                log::error!("Task execution failed [{}]: {}", task.task_id(), e);
                if let Some(recovery) = recovery {
                    if let Err(re) = recovery.record_failure(task.clone(), e.to_string()).await {
                        log::error!("Failed to record task failure: {}", re);
                    }
                }
            }
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

    pub fn coordinator(&self) -> &Arc<FulltextCoordinator> {
        &self.coordinator
    }

    pub fn recovery(&self) -> Option<&Arc<RecoveryManager>> {
        self.recovery.as_ref()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("Buffer error: {0}")]
    BufferError(#[from] BufferError),
    #[error("Coordinator error: {0}")]
    CoordinatorError(String),
    #[error("Commit error: {0}")]
    CommitError(String),
    #[error("Recovery error: {0}")]
    RecoveryError(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

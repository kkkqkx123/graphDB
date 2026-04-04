use crate::coordinator::{ChangeType, FulltextCoordinator};
use crate::core::Value;
use crate::sync::batch::{BatchConfig, BatchProcessor};
use crate::sync::queue::{QueueError, SyncTaskQueue};
use crate::sync::task::SyncTask;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncMode {
    Sync,
    Async,
    Off,
}

pub struct SyncManager {
    coordinator: Arc<FulltextCoordinator>,
    queue: Option<SyncTaskQueue>,
    batch_processor: Arc<Mutex<BatchProcessor>>,
    mode: Arc<tokio::sync::RwLock<SyncMode>>,
}

impl SyncManager {
    pub fn new(
        coordinator: Arc<FulltextCoordinator>,
        queue_size: usize,
        batch_config: BatchConfig,
    ) -> Self {
        let queue = SyncTaskQueue::new(queue_size);
        let batch_processor = Arc::new(Mutex::new(BatchProcessor::new(
            coordinator.clone(),
            batch_config,
        )));

        Self {
            coordinator,
            queue: Some(queue),
            batch_processor,
            mode: Arc::new(tokio::sync::RwLock::new(SyncMode::Async)),
        }
    }

    pub fn with_mode(coordinator: Arc<FulltextCoordinator>, mode: SyncMode) -> Self {
        let batch_processor = Arc::new(Mutex::new(BatchProcessor::new(
            coordinator.clone(),
            BatchConfig::default(),
        )));

        Self {
            coordinator,
            queue: None,
            batch_processor,
            mode: Arc::new(tokio::sync::RwLock::new(mode)),
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

                if let Some(queue) = &self.queue {
                    queue.submit(task).await?;
                }
            }
            SyncMode::Off => {}
        }

        Ok(())
    }

    pub async fn process_queue(&self) {
        if let Some(queue) = &self.queue {
            loop {
                match queue.next().await {
                    Some(task) => {
                        self.execute_task(&task).await;
                    }
                    None => {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }
            }
        }
    }

    async fn execute_task(&self, task: &SyncTask) {
        let result = match task {
            SyncTask::VertexChange {
                space_id,
                tag_name,
                vertex_id,
                properties,
                change_type,
                ..
            } => match change_type {
                ChangeType::Insert | ChangeType::Update => {
                    let props: std::collections::HashMap<_, _> =
                        properties.iter().cloned().collect();
                    self.coordinator
                        .on_vertex_change(*space_id, tag_name, vertex_id, &props, *change_type)
                        .await
                }
                ChangeType::Delete => {
                    let props: std::collections::HashMap<_, _> =
                        properties.iter().cloned().collect();
                    self.coordinator
                        .on_vertex_change(*space_id, tag_name, vertex_id, &props, *change_type)
                        .await
                }
            },
            SyncTask::BatchIndex {
                space_id,
                tag_name,
                field_name,
                documents,
                ..
            } => {
                if let Some(engine) = self.coordinator.get_engine(*space_id, tag_name, field_name) {
                    engine.index_batch(documents.clone()).await
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
                if let Some(engine) = self.coordinator.get_engine(*space_id, tag_name, field_name) {
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
                self.coordinator
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
        let mut processor = self.batch_processor.lock().await;
        let results = processor.commit_all().await;

        for (key, result) in results {
            if let Err(e) = result {
                log::error!("Commit failed {:?}: {:?}", key, e);
                return Err(SyncError::CommitError(e.to_string()));
            }
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("Queue error: {0}")]
    QueueError(#[from] QueueError),
    #[error("Coordinator error: {0}")]
    CoordinatorError(String),
    #[error("Commit error: {0}")]
    CommitError(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

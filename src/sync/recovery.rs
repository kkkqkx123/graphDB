//! Sync Recovery
//!
//! Handles recovery of failed sync tasks.

use crate::sync::batch::TaskBuffer;
use crate::sync::persistence::{FailedTask, SyncPersistence};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct RecoveryConfig {
    pub max_retry_count: u32,
    pub retry_delay: Duration,
    pub cleanup_interval: Duration,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_retry_count: 3,
            retry_delay: Duration::from_secs(60),
            cleanup_interval: Duration::from_secs(3600),
        }
    }
}

pub struct RecoveryManager {
    persistence: SyncPersistence,
    buffer: Arc<TaskBuffer>,
    config: RecoveryConfig,
}

impl RecoveryManager {
    pub fn new(buffer: Arc<TaskBuffer>, data_dir: PathBuf) -> Self {
        Self {
            persistence: SyncPersistence::new(data_dir),
            buffer,
            config: RecoveryConfig::default(),
        }
    }

    pub fn with_config(buffer: Arc<TaskBuffer>, data_dir: PathBuf, config: RecoveryConfig) -> Self {
        Self {
            persistence: SyncPersistence::new(data_dir),
            buffer,
            config,
        }
    }

    pub async fn recover_failed_tasks(&self) -> Result<RecoveryResult, RecoveryError> {
        let failed_tasks = self.persistence.load_failed_tasks().await?;

        if failed_tasks.is_empty() {
            return Ok(RecoveryResult {
                total: 0,
                recovered: 0,
                skipped: 0,
                failed: 0,
            });
        }

        let mut result = RecoveryResult {
            total: failed_tasks.len(),
            recovered: 0,
            skipped: 0,
            failed: 0,
        };

        for failed_task in failed_tasks {
            if failed_task.retry_count >= self.config.max_retry_count {
                result.skipped += 1;
                continue;
            }

            match self.buffer.submit(failed_task.task.clone()).await {
                Ok(_) => {
                    self.persistence
                        .remove_failed_task(failed_task.task.task_id())
                        .await?;
                    result.recovered += 1;
                }
                Err(e) => {
                    log::error!(
                        "Failed to recover task {}: {}",
                        failed_task.task.task_id(),
                        e
                    );
                    self.persistence
                        .increment_retry_count(failed_task.task.task_id())
                        .await?;
                    result.failed += 1;
                }
            }
        }

        Ok(result)
    }

    pub async fn record_failure(
        &self,
        task: crate::sync::task::SyncTask,
        error: String,
    ) -> Result<(), RecoveryError> {
        self.persistence.add_failed_task(task, error).await?;
        Ok(())
    }

    pub async fn get_failed_tasks(&self) -> Result<Vec<FailedTask>, RecoveryError> {
        self.persistence
            .load_failed_tasks()
            .await
            .map_err(Into::into)
    }

    pub async fn retry_task(&self, task_id: &str) -> Result<bool, RecoveryError> {
        let failed_tasks = self.persistence.load_failed_tasks().await?;

        if let Some(failed_task) = failed_tasks.iter().find(|ft| ft.task.task_id() == task_id) {
            self.buffer.submit(failed_task.task.clone()).await?;
            self.persistence.remove_failed_task(task_id).await?;
            return Ok(true);
        }

        Ok(false)
    }

    pub async fn clear_all_failed(&self) -> Result<(), RecoveryError> {
        self.persistence.clear_failed_tasks().await?;
        Ok(())
    }

    pub fn start_recovery_loop(&self) -> tokio::task::JoinHandle<()> {
        let persistence = SyncPersistence::new(
            self.persistence
                .state_path
                .parent()
                .expect("state_path should have parent")
                .to_path_buf(),
        );
        let buffer = self.buffer.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.cleanup_interval);

            loop {
                interval.tick().await;

                match persistence.load_failed_tasks().await {
                    Ok(failed_tasks) => {
                        for failed_task in failed_tasks {
                            if failed_task.retry_count < config.max_retry_count {
                                if let Err(e) = buffer.submit(failed_task.task.clone()).await {
                                    log::error!("Recovery submit failed: {}", e);
                                } else if let Err(e) = persistence
                                    .remove_failed_task(failed_task.task.task_id())
                                    .await
                                {
                                    log::error!("Failed to remove recovered task: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to load failed tasks for recovery: {}", e);
                    }
                }
            }
        })
    }
}

#[derive(Debug)]
pub struct RecoveryResult {
    pub total: usize,
    pub recovered: usize,
    pub skipped: usize,
    pub failed: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum RecoveryError {
    #[error("Persistence error: {0}")]
    Persistence(#[from] crate::sync::persistence::PersistenceError),
    #[error("Buffer error: {0}")]
    Buffer(#[from] crate::sync::batch::BufferError),
}

//! Sync Recovery
//!
//! Handles recovery of failed sync tasks.

use crate::sync::persistence::{FailedTask, SyncPersistence};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Debug)]
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
    config: RecoveryConfig,
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl std::fmt::Debug for RecoveryManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecoveryManager")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl RecoveryManager {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            persistence: SyncPersistence::new(data_dir),
            config: RecoveryConfig::default(),
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub fn with_config(data_dir: PathBuf, config: RecoveryConfig) -> Self {
        Self {
            persistence: SyncPersistence::new(data_dir),
            config,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub async fn start(&self) -> Result<(), RecoveryError> {
        if self.running.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }

        self.running
            .store(true, std::sync::atomic::Ordering::SeqCst);

        // Recover failed tasks from persistence
        if let Ok(failed_tasks) = self.persistence.load_failed_tasks().await {
            for task in failed_tasks {
                if task.retry_count < self.config.max_retry_count {
                    // Retry the task
                    if let Err(e) = self.retry_task(&task).await {
                        tracing::warn!("Failed to retry task {:?}: {:?}", task.task, e);
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn stop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }

    async fn retry_task(&self, task: &FailedTask) -> Result<(), RecoveryError> {
        // Wait before retry
        tokio::time::sleep(self.config.retry_delay).await;

        // Retry logic - depends on task type
        // For now, just mark as retryable
        tracing::info!("Retrying task {:?}", task.task);

        Ok(())
    }

    pub async fn persist_failed_task(&self, task: FailedTask) -> Result<(), RecoveryError> {
        self.persistence
            .save_failed_tasks(&[task])
            .await
            .map_err(|e| RecoveryError::PersistenceError(e.to_string()))?;
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RecoveryError {
    #[error("Persistence error: {0}")]
    PersistenceError(String),

    #[error("Sync coordinator error: {0}")]
    SyncCoordinatorError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type RecoveryResult<T> = std::result::Result<T, RecoveryError>;

//! Sync Persistence
//!
//! Handles persistence of sync state and failed tasks.

use crate::sync::task::SyncTask;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncState {
    pub last_processed_task_id: Option<String>,
    pub pending_task_count: usize,
    pub failed_task_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedTask {
    pub task: SyncTask,
    pub error: String,
    pub retry_count: u32,
    pub failed_at: chrono::DateTime<chrono::Utc>,
}

pub struct SyncPersistence {
    pub state_path: PathBuf,
    pub failed_tasks_path: PathBuf,
}

impl SyncPersistence {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            state_path: base_path.join("sync_state.json"),
            failed_tasks_path: base_path.join("failed_tasks.json"),
        }
    }

    pub async fn save_state(&self, state: &SyncState) -> Result<(), PersistenceError> {
        let data = serde_json::to_vec_pretty(state)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;

        let temp_path = self.state_path.with_extension("tmp");
        fs::write(&temp_path, data)
            .await
            .map_err(|e| PersistenceError::Io(e.to_string()))?;

        fs::rename(&temp_path, &self.state_path)
            .await
            .map_err(|e| PersistenceError::Io(e.to_string()))?;

        Ok(())
    }

    pub async fn load_state(&self) -> Result<SyncState, PersistenceError> {
        if !self.state_path.exists() {
            return Ok(SyncState::default());
        }

        let data = fs::read(&self.state_path)
            .await
            .map_err(|e| PersistenceError::Io(e.to_string()))?;

        serde_json::from_slice(&data).map_err(|e| PersistenceError::Deserialization(e.to_string()))
    }

    pub async fn save_failed_tasks(&self, tasks: &[FailedTask]) -> Result<(), PersistenceError> {
        let data = serde_json::to_vec_pretty(tasks)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;

        let temp_path = self.failed_tasks_path.with_extension("tmp");
        fs::write(&temp_path, data)
            .await
            .map_err(|e| PersistenceError::Io(e.to_string()))?;

        fs::rename(&temp_path, &self.failed_tasks_path)
            .await
            .map_err(|e| PersistenceError::Io(e.to_string()))?;

        Ok(())
    }

    pub async fn load_failed_tasks(&self) -> Result<Vec<FailedTask>, PersistenceError> {
        if !self.failed_tasks_path.exists() {
            return Ok(Vec::new());
        }

        let data = fs::read(&self.failed_tasks_path)
            .await
            .map_err(|e| PersistenceError::Io(e.to_string()))?;

        serde_json::from_slice(&data).map_err(|e| PersistenceError::Deserialization(e.to_string()))
    }

    pub async fn add_failed_task(
        &self,
        task: SyncTask,
        error: String,
    ) -> Result<(), PersistenceError> {
        let mut failed_tasks = self.load_failed_tasks().await?;

        failed_tasks.push(FailedTask {
            task,
            error,
            retry_count: 0,
            failed_at: chrono::Utc::now(),
        });

        self.save_failed_tasks(&failed_tasks).await?;

        let mut state = self.load_state().await?;
        state.failed_task_count = failed_tasks.len();
        self.save_state(&state).await
    }

    pub async fn remove_failed_task(&self, task_id: &str) -> Result<(), PersistenceError> {
        let mut failed_tasks = self.load_failed_tasks().await?;
        failed_tasks.retain(|ft| ft.task.task_id() != task_id);
        self.save_failed_tasks(&failed_tasks).await?;

        let mut state = self.load_state().await?;
        state.failed_task_count = failed_tasks.len();
        self.save_state(&state).await
    }

    pub async fn increment_retry_count(&self, task_id: &str) -> Result<(), PersistenceError> {
        let mut failed_tasks = self.load_failed_tasks().await?;

        if let Some(ft) = failed_tasks
            .iter_mut()
            .find(|ft| ft.task.task_id() == task_id)
        {
            ft.retry_count += 1;
        }

        self.save_failed_tasks(&failed_tasks).await
    }

    pub async fn clear_failed_tasks(&self) -> Result<(), PersistenceError> {
        self.save_failed_tasks(&[]).await?;

        let mut state = self.load_state().await?;
        state.failed_task_count = 0;
        self.save_state(&state).await
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    #[error("IO error: {0}")]
    Io(String),
}

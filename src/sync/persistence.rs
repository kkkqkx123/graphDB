use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub last_processed_task_id: Option<String>,
    pub pending_task_count: usize,
    pub failed_task_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedTaskInfo {
    pub task_id: String,
    pub error: String,
    pub retry_count: u32,
}

pub struct SyncPersistence {
    storage_path: PathBuf,
}

impl SyncPersistence {
    pub fn new(storage_path: PathBuf) -> Self {
        Self { storage_path }
    }

    pub async fn save_state(&self, state: &SyncState) -> Result<(), PersistenceError> {
        let data = serde_json::to_vec(state)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;

        let temp_path = self.storage_path.with_extension("tmp");
        tokio::fs::write(&temp_path, data)
            .await
            .map_err(|e| PersistenceError::Io(e.to_string()))?;

        tokio::fs::rename(&temp_path, &self.storage_path)
            .await
            .map_err(|e| PersistenceError::Io(e.to_string()))?;

        Ok(())
    }

    pub async fn load_state(&self) -> Result<SyncState, PersistenceError> {
        if !self.storage_path.exists() {
            return Ok(SyncState {
                last_processed_task_id: None,
                pending_task_count: 0,
                failed_task_count: 0,
            });
        }

        let data = tokio::fs::read(&self.storage_path)
            .await
            .map_err(|e| PersistenceError::Io(e.to_string()))?;

        let state = serde_json::from_slice(&data)
            .map_err(|e| PersistenceError::Deserialization(e.to_string()))?;

        Ok(state)
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

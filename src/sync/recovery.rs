use std::sync::Arc;
use crate::sync::persistence::{SyncPersistence, SyncState};
use crate::sync::queue::SyncTaskQueue;

pub struct RecoveryManager {
    persistence: Arc<SyncPersistence>,
}

impl RecoveryManager {
    pub fn new(persistence: Arc<SyncPersistence>) -> Self {
        Self { persistence }
    }

    pub async fn recover(&self, _queue: &SyncTaskQueue) -> Result<RecoveryResult, RecoveryError> {
        let state = self.persistence.load_state().await
            .map_err(|e| RecoveryError::Persistence(e.to_string()))?;

        Ok(RecoveryResult {
            recovered_tasks: state.pending_task_count,
            failed_tasks: state.failed_task_count,
            last_processed_id: state.last_processed_task_id,
        })
    }

    pub async fn save_failed_task(&self, _task_id: &str, _error: &str) -> Result<(), RecoveryError> {
        let mut state = self.persistence.load_state().await
            .map_err(|e| RecoveryError::Persistence(e.to_string()))?;

        state.failed_task_count += 1;

        self.persistence.save_state(&state).await
            .map_err(|e| RecoveryError::Persistence(e.to_string()))?;

        Ok(())
    }

    pub async fn mark_task_completed(&self, task_id: &str) -> Result<(), RecoveryError> {
        let mut state = self.persistence.load_state().await
            .map_err(|e| RecoveryError::Persistence(e.to_string()))?;

        state.last_processed_task_id = Some(task_id.to_string());
        if state.pending_task_count > 0 {
            state.pending_task_count -= 1;
        }

        self.persistence.save_state(&state).await
            .map_err(|e| RecoveryError::Persistence(e.to_string()))?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RecoveryResult {
    pub recovered_tasks: usize,
    pub failed_tasks: usize,
    pub last_processed_id: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum RecoveryError {
    #[error("Persistence error: {0}")]
    Persistence(String),
    #[error("Recovery failed: {0}")]
    RecoveryFailed(String),
}

/// Core data structures associated with two-phase submission
use crate::coordinator::ChangeType;
use crate::transaction::types::TransactionId;
use crossbeam_utils::atomic::AtomicCell;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};

/// Pending index update operations
#[derive(Debug, Clone)]
pub struct PendingIndexUpdate {
    /// Transaction ID
    pub txn_id: TransactionId,
    /// Space ID
    pub space_id: u64,
    /// Tag Name
    pub tag_name: String,
    /// field name
    pub field_name: String,
    /// Document ID
    pub doc_id: String,
    /// Updated content (None means deleted)
    pub content: Option<String>,
    /// Previous content before update (for rollback)
    pub old_content: Option<String>,
    /// Vector data for vector index synchronization
    pub vector_data: Option<Vec<f32>>,
    /// Type of change
    pub change_type: ChangeType,
}

impl PendingIndexUpdate {
    pub fn new(
        txn_id: TransactionId,
        space_id: u64,
        tag_name: String,
        field_name: String,
        doc_id: String,
        content: Option<String>,
        old_content: Option<String>,
        vector_data: Option<Vec<f32>>,
        change_type: ChangeType,
    ) -> Self {
        Self {
            txn_id,
            space_id,
            tag_name,
            field_name,
            doc_id,
            content,
            old_content,
            vector_data,
            change_type,
        }
    }
}

/// synchronization handle state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncHandleState {
    /// Created, waiting for synchronization
    Created,
    /// synchronous
    Syncing,
    /// Synchronization complete, waiting for confirmation
    Synced,
    /// synchronization failure
    SyncFailed,
    /// Submission confirmed
    Confirmed,
    /// Cancelled
    Cancelled,
}

/// Synchronization operation handles for tracking and controlling index synchronization
pub struct SyncHandle {
    /// Transaction ID
    pub txn_id: TransactionId,
    /// Pending Index Update List
    pub pending_updates: Vec<PendingIndexUpdate>,
    /// Synchronized results channel
    pub completion_tx: Option<oneshot::Sender<Result<(), crate::sync::SyncError>>>,
    pub completion_rx: Arc<Mutex<Option<oneshot::Receiver<Result<(), crate::sync::SyncError>>>>>,
    /// state of affairs
    state: AtomicCell<SyncHandleState>,
}

impl SyncHandle {
    pub fn new(
        txn_id: TransactionId,
        pending_updates: Vec<PendingIndexUpdate>,
        completion_tx: oneshot::Sender<Result<(), crate::sync::SyncError>>,
        completion_rx: oneshot::Receiver<Result<(), crate::sync::SyncError>>,
    ) -> Self {
        Self {
            txn_id,
            pending_updates,
            completion_tx: Some(completion_tx),
            completion_rx: Arc::new(Mutex::new(Some(completion_rx))),
            state: AtomicCell::new(SyncHandleState::Created),
        }
    }

    /// Get current state
    pub fn state(&self) -> SyncHandleState {
        self.state.load()
    }

    /// Setting state
    pub fn set_state(&self, state: SyncHandleState) {
        self.state.store(state);
    }

    /// Waiting for synchronization to complete (blocking)
    pub fn wait_for_completion(&self) -> Result<(), crate::sync::SyncError> {
        let rx = self
            .completion_rx
            .try_lock()
            .expect("Lock poisoned")
            .take()
            .expect("Completion receiver not set");
        futures::executor::block_on(rx)
            .map_err(|_| crate::sync::SyncError::Internal("Channel closed".to_string()))?
    }

    /// Asynchronous waiting for synchronization to complete
    pub async fn wait_for_completion_async(&self) -> Result<(), crate::sync::SyncError> {
        let rx = self
            .completion_rx
            .lock()
            .await
            .take()
            .expect("Completion receiver not set");
        rx.await
            .map_err(|_| crate::sync::SyncError::Internal("Channel closed".to_string()))?
    }
}

/// Index Update Buffer Configuration
#[derive(Debug, Clone)]
pub struct IndexBufferConfig {
    /// Maximum buffer size
    pub max_buffer_size: usize,
    /// timeout
    pub timeout: std::time::Duration,
    /// synchronous mode
    pub sync_mode: crate::sync::SyncMode,
}

impl Default for IndexBufferConfig {
    fn default() -> Self {
        Self {
            max_buffer_size: 1000,
            timeout: std::time::Duration::from_secs(30),
            sync_mode: crate::sync::SyncMode::Async,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pending_index_update_creation() {
        let update = PendingIndexUpdate::new(
            TransactionId::from(1u64),
            1,
            "user".to_string(),
            "name".to_string(),
            "1".to_string(),
            Some("Alice".to_string()),
            None,
            None,
            ChangeType::Insert,
        );

        assert_eq!(update.txn_id, TransactionId::from(1u64));
        assert_eq!(update.space_id, 1);
        assert_eq!(update.tag_name, "user");
        assert_eq!(update.doc_id, "1");
        assert_eq!(update.content, Some("Alice".to_string()));
        assert_eq!(update.old_content, None);
        assert_eq!(update.vector_data, None);
        assert_eq!(update.change_type, ChangeType::Insert);
    }

    #[test]
    fn test_sync_handle_state_transitions() {
        let (tx, rx) = oneshot::channel();
        let handle = SyncHandle::new(TransactionId::from(1u64), vec![], tx, rx);

        assert_eq!(handle.state(), SyncHandleState::Created);

        handle.set_state(SyncHandleState::Syncing);
        assert_eq!(handle.state(), SyncHandleState::Syncing);

        handle.set_state(SyncHandleState::Synced);
        assert_eq!(handle.state(), SyncHandleState::Synced);
    }
}

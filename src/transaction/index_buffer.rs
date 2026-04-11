/// Index update buffer management
use crate::sync::{IndexBufferConfig, PendingIndexUpdate, SyncError, SyncHandle};
use crate::transaction::types::TransactionId;
use dashmap::DashMap;
use std::sync::Arc;

/// index update buffer
pub struct IndexUpdateBuffer {
    /// Pending updates organized by transaction ID
    buffers: DashMap<TransactionId, Vec<PendingIndexUpdate>>,
    /// Handle in synchronization
    syncing_handles: DashMap<TransactionId, Arc<SyncHandle>>,
    /// deployment
    config: IndexBufferConfig,
}

impl IndexUpdateBuffer {
    pub fn new(config: IndexBufferConfig) -> Self {
        Self {
            buffers: DashMap::new(),
            syncing_handles: DashMap::new(),
            config,
        }
    }

    /// obtaining configuration
    pub fn config(&self) -> &IndexBufferConfig {
        &self.config
    }

    /// Add Pending Updates
    pub fn add_update(
        &self,
        txn_id: TransactionId,
        update: PendingIndexUpdate,
    ) -> Result<(), SyncError> {
        let mut buffer = self.buffers.entry(txn_id).or_default();

        // Check buffer size
        if buffer.len() >= self.config.max_buffer_size {
            return Err(SyncError::BufferError(
                format!(
                    "Buffer full for transaction {:?}",
                    txn_id
                ),
            ));
        }

        buffer.push(update);
        Ok(())
    }

    /// Getting and clearing pending updates
    pub fn take_updates(&self, txn_id: TransactionId) -> Vec<PendingIndexUpdate> {
        self.buffers
            .remove(&txn_id)
            .map(|(_, updates)| updates)
            .unwrap_or_default()
    }

    /// Check if updates are pending
    pub fn has_pending_updates(&self, txn_id: TransactionId) -> bool {
        if let Some(buffer) = self.buffers.get(&txn_id) {
            !buffer.is_empty()
        } else {
            false
        }
    }

    /// Register Synchronization Handle
    pub fn register_sync_handle(&self, txn_id: TransactionId, handle: Arc<SyncHandle>) {
        self.syncing_handles.insert(txn_id, handle);
    }

    /// Get synchronization handle
    pub fn get_sync_handle(&self, txn_id: TransactionId) -> Option<Arc<SyncHandle>> {
        self.syncing_handles.get(&txn_id).map(|h| h.clone())
    }

    /// Remove Synchronization Handle
    pub fn remove_sync_handle(&self, txn_id: TransactionId) -> Option<Arc<SyncHandle>> {
        self.syncing_handles
            .remove(&txn_id)
            .map(|(_, handle)| handle)
    }

    /// Get all pending transaction IDs
    pub fn get_all_txn_ids(&self) -> Vec<TransactionId> {
        self.buffers.iter().map(|entry| *entry.key()).collect()
    }

    /// Cleaning up timeout transactions
    pub fn cleanup_timeout(&self) -> Vec<TransactionId> {
        let mut expired_txns = Vec::new();

        // Check syncing handles for timeout transactions
        for entry in self.syncing_handles.iter() {
            let txn_id = entry.key();
            expired_txns.push(*txn_id);
        }

        // Clearing timeout transactions
        for txn_id in &expired_txns {
            self.buffers.remove(txn_id);
            self.syncing_handles.remove(txn_id);
        }

        expired_txns
    }

    /// Get buffer statistics
    pub fn stats(&self) -> BufferStats {
        let total_updates: usize = self.buffers.iter().map(|entry| entry.value().len()).sum();
        let total_handles = self.syncing_handles.len();

        BufferStats {
            active_transactions: self.buffers.len(),
            total_pending_updates: total_updates,
            syncing_handles: total_handles,
        }
    }
}

/// Buffer statistics
#[derive(Debug, Clone)]
pub struct BufferStats {
    pub active_transactions: usize,
    pub total_pending_updates: usize,
    pub syncing_handles: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinator::ChangeType;

    fn create_test_update(txn_id: TransactionId) -> PendingIndexUpdate {
        PendingIndexUpdate::new(
            txn_id,
            1,
            "user".to_string(),
            "name".to_string(),
            "1".to_string(),
            Some("Alice".to_string()),
            None,
            None,
            ChangeType::Insert,
        )
    }

    #[test]
    fn test_add_and_take_updates() {
        let buffer = IndexUpdateBuffer::new(IndexBufferConfig::default());
        let txn_id = TransactionId::from(1u64);

        // add update
        buffer
            .add_update(txn_id, create_test_update(txn_id))
            .unwrap();
        buffer
            .add_update(txn_id, create_test_update(txn_id))
            .unwrap();

        // Inspection pending processing update
        assert!(buffer.has_pending_updates(txn_id));

        // Get and clear updates
        let updates = buffer.take_updates(txn_id);
        assert_eq!(updates.len(), 2);
        assert!(!buffer.has_pending_updates(txn_id));
    }

    #[test]
    fn test_buffer_size_limit() {
        let mut config = IndexBufferConfig::default();
        config.max_buffer_size = 2;
        let buffer = IndexUpdateBuffer::new(config);
        let txn_id = TransactionId::from(1u64);

        // Add 2 updates (limit reached)
        buffer
            .add_update(txn_id, create_test_update(txn_id))
            .unwrap();
        buffer
            .add_update(txn_id, create_test_update(txn_id))
            .unwrap();

        // The third update should fail
        let result = buffer.add_update(txn_id, create_test_update(txn_id));
        assert!(result.is_err());
    }

    #[test]
    fn test_sync_handle_management() {
        let buffer = IndexUpdateBuffer::new(IndexBufferConfig::default());
        let txn_id = TransactionId::from(1u64);

        // Creating a Synchronization Handle
        let (tx, rx) = tokio::sync::oneshot::channel();
        let handle = Arc::new(SyncHandle::new(txn_id, vec![], tx, rx));

        // Registration handle
        buffer.register_sync_handle(txn_id, handle.clone());

        // Get handle
        let retrieved = buffer.get_sync_handle(txn_id);
        assert!(retrieved.is_some());
        assert!(Arc::ptr_eq(&retrieved.unwrap(), &handle));

        // Remove handle
        let removed = buffer.remove_sync_handle(txn_id);
        assert!(removed.is_some());
        assert!(buffer.get_sync_handle(txn_id).is_none());
    }

    #[test]
    fn test_buffer_stats() {
        let buffer = IndexUpdateBuffer::new(IndexBufferConfig::default());
        let txn_id1 = TransactionId::from(1u64);
        let txn_id2 = TransactionId::from(2u64);

        // Initial statistics
        let stats = buffer.stats();
        assert_eq!(stats.active_transactions, 0);
        assert_eq!(stats.total_pending_updates, 0);

        // add update
        buffer
            .add_update(txn_id1, create_test_update(txn_id1))
            .unwrap();
        buffer
            .add_update(txn_id1, create_test_update(txn_id1))
            .unwrap();
        buffer
            .add_update(txn_id2, create_test_update(txn_id2))
            .unwrap();

        // Update Statistics
        let stats = buffer.stats();
        assert_eq!(stats.active_transactions, 2);
        assert_eq!(stats.total_pending_updates, 3);
    }
}

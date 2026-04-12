/// Index update buffer management
use crate::sync::{IndexBufferConfig, PendingIndexUpdate, SyncError};
use crate::transaction::types::TransactionId;
use dashmap::DashMap;

/// index update buffer
pub struct IndexUpdateBuffer {
    /// Pending updates organized by transaction ID
    buffers: DashMap<TransactionId, Vec<PendingIndexUpdate>>,
    /// deployment
    config: IndexBufferConfig,
}

impl IndexUpdateBuffer {
    pub fn new(config: IndexBufferConfig) -> Self {
        Self {
            buffers: DashMap::new(),
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
            return Err(SyncError::BufferError(format!(
                "Buffer full for transaction {:?}",
                txn_id
            )));
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

    /// Get all pending transaction IDs
    pub fn get_all_txn_ids(&self) -> Vec<TransactionId> {
        self.buffers.iter().map(|entry| *entry.key()).collect()
    }

    /// Cleaning up timeout transactions
    pub fn cleanup_timeout(&self) -> Vec<TransactionId> {
        let expired_txns: Vec<TransactionId> =
            self.buffers.iter().map(|entry| *entry.key()).collect();
        for txn_id in &expired_txns {
            self.buffers.remove(txn_id);
        }
        expired_txns
    }

    /// Get buffer statistics
    pub fn stats(&self) -> BufferStats {
        let total_updates: usize = self.buffers.iter().map(|entry| entry.value().len()).sum();

        BufferStats {
            active_transactions: self.buffers.len(),
            total_pending_updates: total_updates,
        }
    }
}

/// Buffer statistics
#[derive(Debug, Clone)]
pub struct BufferStats {
    pub active_transactions: usize,
    pub total_pending_updates: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::coordinator::ChangeType;

    fn create_test_update(txn_id: TransactionId) -> PendingIndexUpdate {
        PendingIndexUpdate::new(
            txn_id,
            1,
            "user".to_string(),
            "name".to_string(),
            "1".to_string(),
        )
        .with_content("Alice".to_string())
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
        let buffer = IndexUpdateBuffer::new(IndexBufferConfig {
            max_buffer_size: 2,
            ..Default::default()
        });
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

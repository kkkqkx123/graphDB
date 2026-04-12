//! Vector Index Transaction Buffer
//!
//! Provides transaction buffering for vector index operations.

use crate::sync::vector_sync::VectorChangeContext;
use crate::transaction::types::TransactionId;
use dashmap::DashMap;

/// Pending vector index update
#[derive(Debug, Clone)]
pub struct PendingVectorUpdate {
    pub txn_id: TransactionId,
    pub context: VectorChangeContext,
}

impl PendingVectorUpdate {
    pub fn new(txn_id: TransactionId, context: VectorChangeContext) -> Self {
        Self { txn_id, context }
    }
}

/// Vector transaction buffer configuration
#[derive(Debug, Clone)]
pub struct VectorTransactionBufferConfig {
    pub max_buffer_size: usize,
    pub flush_timeout_ms: u64,
}

impl Default for VectorTransactionBufferConfig {
    fn default() -> Self {
        Self {
            max_buffer_size: 1000,
            flush_timeout_ms: 100,
        }
    }
}

/// Vector transaction buffer
pub struct VectorTransactionBuffer {
    buffers: DashMap<TransactionId, Vec<PendingVectorUpdate>>,
    config: VectorTransactionBufferConfig,
}

impl std::fmt::Debug for VectorTransactionBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VectorTransactionBuffer")
            .field("buffers", &self.buffers.len())
            .field("config", &self.config)
            .finish()
    }
}

impl VectorTransactionBuffer {
    pub fn new(config: VectorTransactionBufferConfig) -> Self {
        Self {
            buffers: DashMap::new(),
            config,
        }
    }

    pub fn config(&self) -> &VectorTransactionBufferConfig {
        &self.config
    }

    /// Add a pending vector update
    pub fn add_update(
        &self,
        txn_id: TransactionId,
        update: PendingVectorUpdate,
    ) -> Result<(), VectorBufferError> {
        let mut buffer = self.buffers.entry(txn_id).or_default();

        if buffer.len() >= self.config.max_buffer_size {
            return Err(VectorBufferError::BufferFull(format!(
                "Buffer full for transaction {:?}",
                txn_id
            )));
        }

        buffer.push(update);
        Ok(())
    }

    /// Get and clear pending updates for a transaction
    pub fn take_updates(&self, txn_id: TransactionId) -> Vec<PendingVectorUpdate> {
        self.buffers
            .remove(&txn_id)
            .map(|(_, updates)| updates)
            .unwrap_or_default()
    }

    /// Check if there are pending updates
    pub fn has_pending_updates(&self, txn_id: TransactionId) -> bool {
        if let Some(buffer) = self.buffers.get(&txn_id) {
            !buffer.is_empty()
        } else {
            false
        }
    }

    /// Get all transaction IDs with pending updates
    pub fn get_all_txn_ids(&self) -> Vec<TransactionId> {
        self.buffers.iter().map(|entry| *entry.key()).collect()
    }

    /// Get buffer statistics
    pub fn stats(&self) -> VectorBufferStats {
        let total_updates: usize = self.buffers.iter().map(|entry| entry.value().len()).sum();

        VectorBufferStats {
            active_transactions: self.buffers.len(),
            total_pending_updates: total_updates,
        }
    }

    /// Cleanup all buffers (for rollback)
    pub fn cleanup(&self, txn_id: TransactionId) {
        self.buffers.remove(&txn_id);
    }

    /// Cleanup all transaction buffers
    pub fn cleanup_all(&self) {
        self.buffers.clear();
    }
}

/// Vector buffer statistics
#[derive(Debug, Clone)]
pub struct VectorBufferStats {
    pub active_transactions: usize,
    pub total_pending_updates: usize,
}

/// Vector buffer error
#[derive(Debug, thiserror::Error)]
pub enum VectorBufferError {
    #[error("Buffer full: {0}")]
    BufferFull(String),

    #[error("Transaction not found: {0}")]
    TransactionNotFound(TransactionId),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::task::VectorPointData;
    use crate::sync::vector_sync::{VectorChangeContext, VectorChangeType, VectorIndexLocation};

    fn create_test_update(txn_id: TransactionId) -> PendingVectorUpdate {
        let location = VectorIndexLocation::new(1, "test", "vector_field");
        let context = VectorChangeContext::new(
            1,
            "test",
            "vector_field",
            VectorChangeType::Insert,
            VectorPointData {
                id: "test_id".to_string(),
                vector: vec![1.0, 2.0, 3.0],
                payload: std::collections::HashMap::new(),
            },
        );
        PendingVectorUpdate::new(txn_id, context)
    }

    #[test]
    fn test_add_and_take_updates() {
        let buffer = VectorTransactionBuffer::new(VectorTransactionBufferConfig::default());
        let txn_id = TransactionId::from(1u64);

        buffer
            .add_update(txn_id, create_test_update(txn_id))
            .unwrap();
        buffer
            .add_update(txn_id, create_test_update(txn_id))
            .unwrap();

        assert!(buffer.has_pending_updates(txn_id));

        let updates = buffer.take_updates(txn_id);
        assert_eq!(updates.len(), 2);
        assert!(!buffer.has_pending_updates(txn_id));
    }

    #[test]
    fn test_buffer_size_limit() {
        let buffer = VectorTransactionBuffer::new(VectorTransactionBufferConfig {
            max_buffer_size: 2,
            ..Default::default()
        });
        let txn_id = TransactionId::from(1u64);

        buffer
            .add_update(txn_id, create_test_update(txn_id))
            .unwrap();
        buffer
            .add_update(txn_id, create_test_update(txn_id))
            .unwrap();

        let result = buffer.add_update(txn_id, create_test_update(txn_id));
        assert!(result.is_err());
    }

    #[test]
    fn test_cleanup() {
        let buffer = VectorTransactionBuffer::new(VectorTransactionBufferConfig::default());
        let txn_id = TransactionId::from(1u64);

        buffer
            .add_update(txn_id, create_test_update(txn_id))
            .unwrap();
        assert!(buffer.has_pending_updates(txn_id));

        buffer.cleanup(txn_id);
        assert!(!buffer.has_pending_updates(txn_id));
    }
}

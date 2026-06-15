use dashmap::DashMap;

use crate::core::types::TransactionId;
use crate::sync::batch::error::BatchResult;
use crate::sync::types::{IndexOpKey, IndexOperation};

#[derive(Debug, Default)]
pub struct TransactionBufferEntry {
    pub operations: Vec<IndexOperation>,
}

pub struct TransactionBatchBuffer {
    pending: DashMap<TransactionId, DashMap<IndexOpKey, TransactionBufferEntry>>,
}

impl std::fmt::Debug for TransactionBatchBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransactionBatchBuffer")
            .field("pending_count", &self.pending.len())
            .finish_non_exhaustive()
    }
}

impl Default for TransactionBatchBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionBatchBuffer {
    pub fn new() -> Self {
        Self {
            pending: DashMap::new(),
        }
    }

    pub fn take_operations(
        &self,
        txn_id: TransactionId,
    ) -> BatchResult<Vec<(IndexOpKey, Vec<IndexOperation>)>> {
        if let Some((_, txn_buffer)) = self.pending.remove(&txn_id) {
            let mut result = Vec::new();
            for entry in txn_buffer.iter() {
                let key = entry.key().clone();
                let ops = entry.value().operations.clone();
                if !ops.is_empty() {
                    result.push((key, ops));
                }
            }
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }

    pub fn prepare(&self, txn_id: TransactionId, operation: IndexOperation) -> BatchResult<()> {
        let txn_buffer = self.pending.entry(txn_id).or_default();

        let key = match &operation {
            IndexOperation::Insert { key, .. }
            | IndexOperation::Update { key, .. }
            | IndexOperation::Delete { key, .. } => key.clone(),
        };

        let mut entry = txn_buffer.entry(key).or_default();
        entry.operations.push(operation);
        Ok(())
    }

    pub fn peek_operations(
        &self,
        txn_id: TransactionId,
    ) -> BatchResult<Vec<(IndexOpKey, Vec<IndexOperation>)>> {
        if let Some(txn_buffer) = self.pending.get(&txn_id) {
            let mut result = Vec::new();
            for entry in txn_buffer.iter() {
                let key = entry.key().clone();
                let ops = entry.value().operations.clone();
                if !ops.is_empty() {
                    result.push((key, ops));
                }
            }
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }

    pub fn rollback(&self, txn_id: TransactionId) -> BatchResult<()> {
        self.pending.remove(&txn_id);
        Ok(())
    }

    pub fn pending_count(&self, txn_id: TransactionId) -> usize {
        self.pending
            .get(&txn_id)
            .map(|txn_buffer| txn_buffer.iter().map(|e| e.value().operations.len()).sum())
            .unwrap_or(0)
    }
}

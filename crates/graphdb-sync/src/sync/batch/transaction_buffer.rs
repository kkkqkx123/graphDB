use dashmap::DashMap;

use crate::core::types::TransactionId;
use crate::sync::batch::error::BatchResult;
use crate::sync::types::{IndexOpKey, IndexOperation};

#[derive(Debug, Default)]
pub struct TransactionBufferEntry {
    pub operations: Vec<SequencedIndexOperation>,
}

#[derive(Debug, Clone)]
pub struct SequencedIndexOperation {
    pub sequence: u64,
    pub operation: IndexOperation,
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
                let ops = entry
                    .value()
                    .operations
                    .iter()
                    .cloned()
                    .map(|op| op.operation)
                    .collect::<Vec<_>>();
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
        self.prepare_with_sequence(txn_id, 0, operation)
    }

    pub fn prepare_with_sequence(
        &self,
        txn_id: TransactionId,
        sequence: u64,
        operation: IndexOperation,
    ) -> BatchResult<()> {
        let txn_buffer = self.pending.entry(txn_id).or_default();

        let key = match &operation {
            IndexOperation::Insert { key, .. }
            | IndexOperation::Update { key, .. }
            | IndexOperation::Delete { key, .. } => key.clone(),
        };

        let mut entry = txn_buffer.entry(key).or_default();
        entry
            .operations
            .push(SequencedIndexOperation { sequence, operation });
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
                let ops = entry
                    .value()
                    .operations
                    .iter()
                    .cloned()
                    .map(|op| op.operation)
                    .collect::<Vec<_>>();
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

    pub fn truncate_operations(&self, txn_id: TransactionId, sequence: u64) -> BatchResult<()> {
        if let Some(txn_buffer) = self.pending.get_mut(&txn_id) {
            let keys: Vec<IndexOpKey> = txn_buffer.iter().map(|entry| entry.key().clone()).collect();

            for key in keys {
                if let Some(mut entry) = txn_buffer.get_mut(&key) {
                    entry.operations.retain(|op| op.sequence <= sequence);
                }
            }
        }
        Ok(())
    }

    pub fn pending_sequence(&self, txn_id: TransactionId) -> u64 {
        self.pending
            .get(&txn_id)
            .and_then(|txn_buffer| {
                txn_buffer
                    .iter()
                    .flat_map(|entry| {
                        let ops = entry.value().operations.clone();
                        ops.into_iter().map(|op| op.sequence)
                    })
                    .max()
            })
            .unwrap_or(0)
    }

    pub fn pending_count(&self, txn_id: TransactionId) -> usize {
        self.pending
            .get(&txn_id)
            .map(|txn_buffer| {
                txn_buffer
                    .iter()
                    .map(|e| e.value().operations.len())
                    .sum()
            })
            .unwrap_or(0)
    }
}

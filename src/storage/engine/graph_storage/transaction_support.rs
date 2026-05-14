//! Transaction Support for GraphStorage
//!
//! This module provides transaction support including undo log management
//! and automatic rollback on failure.

use crate::core::types::Timestamp;
use crate::core::{StorageError, StorageResult};
use crate::storage::engine::PropertyGraph;
use crate::transaction::undo_log::{UndoLogEntry, UndoLogManager};

/// Transaction support for write operations
pub struct TransactionSupport {
    undo_logs: UndoLogManager,
    in_transaction: bool,
}

impl TransactionSupport {
    pub fn new() -> Self {
        Self {
            undo_logs: UndoLogManager::new(),
            in_transaction: false,
        }
    }

    /// Begin a transaction context
    pub fn begin(&mut self) {
        self.in_transaction = true;
        self.undo_logs.clear();
    }

    /// Commit the transaction (clear undo logs)
    pub fn commit(&mut self) {
        self.in_transaction = false;
        self.undo_logs.clear();
    }

    /// Rollback the transaction using undo logs
    pub fn rollback(&mut self, graph: &mut PropertyGraph, ts: Timestamp) -> StorageResult<()> {
        self.in_transaction = false;
        self.undo_logs
            .execute_undo(graph, ts)
            .map_err(|e| StorageError::db_error(format!("Rollback failed: {}", e)))?;
        Ok(())
    }

    /// Record an undo log entry
    pub fn record_undo(&mut self, entry: UndoLogEntry) {
        if self.in_transaction {
            self.undo_logs.add(entry);
        }
    }

    /// Check if in a transaction
    pub fn is_in_transaction(&self) -> bool {
        self.in_transaction
    }

    /// Get undo log count
    pub fn undo_log_count(&self) -> usize {
        self.undo_logs.len()
    }

    /// Check if there are pending undo logs
    pub fn has_pending_undo(&self) -> bool {
        !self.undo_logs.is_empty()
    }
}

impl Default for TransactionSupport {
    fn default() -> Self {
        Self::new()
    }
}

/// Execute an operation with automatic rollback on failure
pub fn with_rollback<T, F>(
    graph: &mut PropertyGraph,
    txn_support: &mut TransactionSupport,
    ts: Timestamp,
    operation: F,
) -> StorageResult<T>
where
    F: FnOnce(&mut PropertyGraph, &mut TransactionSupport) -> StorageResult<T>,
{
    let result = operation(graph, txn_support);

    match result {
        Ok(value) => Ok(value),
        Err(e) => {
            if txn_support.is_in_transaction() && txn_support.has_pending_undo() {
                log::warn!("Operation failed, attempting rollback: {}", e);
                if let Err(rollback_err) = txn_support.rollback(graph, ts) {
                    log::error!("Rollback also failed: {}", rollback_err);
                }
            }
            Err(e)
        }
    }
}

/// Execute an operation in a transaction context
pub fn execute_in_transaction<T, F>(
    graph: &mut PropertyGraph,
    ts: Timestamp,
    operation: F,
) -> StorageResult<T>
where
    F: FnOnce(&mut PropertyGraph, &mut TransactionSupport) -> StorageResult<T>,
{
    let mut txn_support = TransactionSupport::new();
    txn_support.begin();

    let result = operation(graph, &mut txn_support);

    match result {
        Ok(value) => {
            txn_support.commit();
            Ok(value)
        }
        Err(e) => {
            if txn_support.has_pending_undo() {
                log::warn!("Transaction failed, rolling back: {}", e);
                if let Err(rollback_err) = txn_support.rollback(graph, ts) {
                    log::error!("Rollback failed: {}", rollback_err);
                }
            }
            Err(e)
        }
    }
}

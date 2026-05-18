//! Transaction Support for GraphStorage
//!
//! This module provides utility functions for undo log management
//! and automatic rollback on failure.
//!
//! Note: TransactionSupport struct has been removed. Use TransactionContext
//! from the transaction module for transaction state management.
//! These utility functions remain for simple operations that need
//! basic undo support without a full transaction context.

use crate::core::types::Timestamp;
use crate::core::{StorageResult};
use crate::storage::engine::PropertyGraph;
use crate::transaction::undo_log::UndoLogManager;

/// Execute an operation with automatic rollback on failure
pub fn with_rollback<T, F>(
    graph: &PropertyGraph,
    undo_logs: &mut UndoLogManager,
    ts: Timestamp,
    operation: F,
) -> StorageResult<T>
where
    F: FnOnce() -> StorageResult<T>,
{
    let result = operation();

    match result {
        Ok(value) => Ok(value),
        Err(e) => {
            if !undo_logs.is_empty() {
                log::warn!("Operation failed, attempting rollback: {}", e);
                if let Err(rollback_err) = undo_logs.execute_undo(graph, ts) {
                    log::error!("Rollback also failed: {}", rollback_err);
                }
            }
            Err(e)
        }
    }
}

/// Execute an operation in a transaction context
pub fn execute_in_transaction<T, F>(
    graph: &PropertyGraph,
    ts: Timestamp,
    operation: F,
) -> StorageResult<T>
where
    F: FnOnce() -> StorageResult<T>,
{
    let mut undo_logs = UndoLogManager::new();

    let result = operation();

    match result {
        Ok(value) => {
            undo_logs.clear();
            Ok(value)
        }
        Err(e) => {
            if !undo_logs.is_empty() {
                log::warn!("Transaction failed, rolling back: {}", e);
                if let Err(rollback_err) = undo_logs.execute_undo(graph, ts) {
                    log::error!("Rollback failed: {}", rollback_err);
                }
            }
            Err(e)
        }
    }
}

//! Transaction Cleaner
//!
//! Provides cleanup functionality for expired and stale transactions

use std::sync::Arc;

use dashmap::DashMap;

use crate::sync::SyncManager;
use crate::transaction::context::TransactionContext;
use crate::transaction::error::TransactionError;
use crate::transaction::types::{TransactionId, TransactionState, TransactionStats};

/// Transaction Cleaner
///
/// Responsible for cleaning up expired transactions and releasing their resources.
pub struct TransactionCleaner {
    sync_manager: Option<Arc<SyncManager>>,
    stats: Arc<TransactionStats>,
}

impl TransactionCleaner {
    pub fn new(sync_manager: Option<Arc<SyncManager>>, stats: Arc<TransactionStats>) -> Self {
        Self {
            sync_manager,
            stats,
        }
    }

    /// Cleanup expired transactions
    ///
    /// This method removes all expired transactions and releases their resources.
    /// It should be called periodically or before starting new write transactions
    /// to prevent stale transactions from blocking operations.
    pub fn cleanup_expired_transactions(
        &self,
        active_transactions: &DashMap<TransactionId, Arc<TransactionContext>>,
    ) {
        let expired: Vec<TransactionId> = {
            active_transactions
                .iter()
                .filter(|entry| entry.value().is_expired())
                .map(|entry| *entry.key())
                .collect()
        };

        if expired.is_empty() {
            return;
        }

        log::debug!("Cleaning up {} expired transactions", expired.len());

        for txn_id in expired {
            let context = {
                if let Some((_, ctx)) = active_transactions.remove(&txn_id) {
                    ctx
                } else {
                    continue;
                }
            };

            // Storage context cleanup is no longer needed in the new design
            // Transaction context is managed at the transaction layer

            let _ = self.abort_transaction_internal_without_storage_cleanup(context);
            self.stats.increment_timeout();
        }
    }

    /// Abort transaction without clearing storage context (used during cleanup)
    fn abort_transaction_internal_without_storage_cleanup(
        &self,
        context: Arc<TransactionContext>,
    ) -> Result<(), TransactionError> {
        if !context.state().can_abort() {
            return Err(TransactionError::invalid_state_for_abort(context.state()));
        }

        context.transition_to(TransactionState::Aborting)?;

        let txn_id = context.id;
        if let Some(ref sync_manager) = self.sync_manager {
            if let Err(e) = sync_manager.rollback_transaction_sync(txn_id) {
                log::warn!(
                    "Index sync rollback failed for transaction {:?}: {}",
                    txn_id,
                    e
                );
            }
        }

        self.stats.decrement_active();
        self.stats.increment_aborted();

        Ok(())
    }

    /// Abort transaction by ID (helper for cleanup operations)
    pub fn abort_transaction_by_id(
        &self,
        active_transactions: &DashMap<TransactionId, Arc<TransactionContext>>,
        txn_id: TransactionId,
    ) -> Result<(), TransactionError> {
        let context = active_transactions
            .get(&txn_id)
            .map(|entry| entry.value().clone())
            .ok_or(TransactionError::transaction_not_found(txn_id))?;

        self.abort_transaction_internal(context, active_transactions)
    }

    /// Abort transaction (internal version)
    pub fn abort_transaction_internal(
        &self,
        context: Arc<TransactionContext>,
        active_transactions: &DashMap<TransactionId, Arc<TransactionContext>>,
    ) -> Result<(), TransactionError> {
        if !context.state().can_abort() {
            return Err(TransactionError::invalid_state_for_abort(context.state()));
        }

        context.transition_to(TransactionState::Aborting)?;

        let txn_id = context.id;
        if let Some(ref sync_manager) = self.sync_manager {
            if let Err(e) = sync_manager.rollback_transaction_sync(txn_id) {
                log::warn!(
                    "Index sync rollback failed for transaction {:?}: {}",
                    txn_id,
                    e
                );
            }
        }

        self.stats.decrement_active();
        self.stats.increment_aborted();

        // Storage context cleanup is no longer needed in the new design
        // Transaction context is managed at the transaction layer

        active_transactions.remove(&txn_id);

        Ok(())
    }
}

impl Default for TransactionCleaner {
    fn default() -> Self {
        Self::new(None, Arc::new(TransactionStats::new()))
    }
}

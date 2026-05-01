//! Transaction Cleaner
//!
//! Provides cleanup functionality for expired and stale transactions

use std::sync::Arc;

use dashmap::DashMap;

use crate::storage::shared_state::StorageInner;
use crate::sync::SyncManager;
use crate::transaction::context::TransactionContext;
use crate::transaction::types::{TransactionError, TransactionId, TransactionState, TransactionStats};

/// Transaction Cleaner
///
/// Responsible for cleaning up expired transactions and releasing their resources.
pub struct TransactionCleaner {
    sync_manager: Option<Arc<SyncManager>>,
    storage_inner: Option<Arc<StorageInner>>,
    stats: Arc<TransactionStats>,
}

impl TransactionCleaner {
    pub fn new(
        sync_manager: Option<Arc<SyncManager>>,
        storage_inner: Option<Arc<StorageInner>>,
        stats: Arc<TransactionStats>,
    ) -> Self {
        Self {
            sync_manager,
            storage_inner,
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

            if let Some(ref storage_inner) = self.storage_inner {
                if let Some(current_ctx) = storage_inner.get_transaction_context() {
                    if current_ctx.id == txn_id {
                        storage_inner.set_transaction_context(None);
                    }
                }
            }

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
            return Err(TransactionError::InvalidStateForAbort(context.state()));
        }

        context.transition_to(TransactionState::Aborting)?;

        let txn_id = context.id;
        if let Some(ref sync_manager) = self.sync_manager {
            if let Err(e) = futures::executor::block_on(sync_manager.rollback_transaction(txn_id)) {
                log::warn!(
                    "Index sync rollback failed for transaction {:?}: {}",
                    txn_id,
                    e
                );
            }
        }

        if !context.read_only {
            let _ = context.take_write_txn();
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
            .ok_or(TransactionError::TransactionNotFound(txn_id))?;

        self.abort_transaction_internal(context, active_transactions)
    }

    /// Abort transaction (internal version)
    pub fn abort_transaction_internal(
        &self,
        context: Arc<TransactionContext>,
        active_transactions: &DashMap<TransactionId, Arc<TransactionContext>>,
    ) -> Result<(), TransactionError> {
        if !context.state().can_abort() {
            return Err(TransactionError::InvalidStateForAbort(context.state()));
        }

        context.transition_to(TransactionState::Aborting)?;

        let txn_id = context.id;
        if let Some(ref sync_manager) = self.sync_manager {
            if let Err(e) = futures::executor::block_on(sync_manager.rollback_transaction(txn_id)) {
                log::warn!(
                    "Index sync rollback failed for transaction {:?}: {}",
                    txn_id,
                    e
                );
            }
        }

        if !context.read_only {
            let _ = context.take_write_txn();
        }

        self.stats.decrement_active();
        self.stats.increment_aborted();

        if let Some(ref storage_inner) = self.storage_inner {
            if let Some(current_ctx) = storage_inner.get_transaction_context() {
                if current_ctx.id == txn_id {
                    storage_inner.set_transaction_context(None);
                }
            }
        }

        active_transactions.remove(&txn_id);

        Ok(())
    }
}

impl Default for TransactionCleaner {
    fn default() -> Self {
        Self::new(None, None, Arc::new(TransactionStats::new()))
    }
}

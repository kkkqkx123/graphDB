//! Transaction Manager
//!
//! Manages the lifecycle of all transactions, providing operations such as
//! transaction start, commit, and abort. Uses MVCC version management for
//! snapshot isolation.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;

use super::cleaner::TransactionCleaner;
use super::context::TransactionContext;
use super::monitor::TransactionMonitor;
use super::types::*;
use super::version_manager::{VersionManager, VersionManagerConfig};
use super::wal::writer::WalWriter;
use super::undo_log::UndoTarget;

/// Transaction Manager
///
/// Manages the lifecycle of all transactions using MVCC version management.
/// Supports read, insert, update, and compact transactions.
pub struct TransactionManager {
    /// Version manager for MVCC timestamps
    version_manager: Arc<VersionManager>,
    /// Configuration
    config: TransactionManagerConfig,
    /// Active transactions table
    active_transactions: DashMap<TransactionId, Arc<TransactionContext>>,
    /// Transaction ID generator
    id_generator: AtomicU64,
    /// Statistics
    stats: Arc<TransactionStats>,
    /// Whether shutdown
    shutdown_flag: AtomicU64,
    /// Transaction monitor for metrics collection
    monitor: TransactionMonitor,
    /// Transaction cleaner for expired transaction cleanup
    cleaner: TransactionCleaner,
}

impl TransactionManager {
    /// Create a new transaction manager
    pub fn new(config: TransactionManagerConfig) -> Self {
        let stats = Arc::new(TransactionStats::new());
        let monitor = TransactionMonitor::new(Arc::clone(&stats));
        let cleaner = TransactionCleaner::new(None, None, Arc::clone(&stats));
        let version_manager = Arc::new(VersionManager::new());

        Self {
            version_manager,
            config,
            active_transactions: DashMap::new(),
            id_generator: AtomicU64::new(1),
            stats,
            shutdown_flag: AtomicU64::new(0),
            monitor,
            cleaner,
        }
    }

    /// Create a new transaction manager with version manager config
    pub fn with_version_config(
        config: TransactionManagerConfig,
        vm_config: VersionManagerConfig,
    ) -> Self {
        let stats = Arc::new(TransactionStats::new());
        let monitor = TransactionMonitor::new(Arc::clone(&stats));
        let cleaner = TransactionCleaner::new(None, None, Arc::clone(&stats));
        let version_manager = Arc::new(VersionManager::with_config(vm_config));

        Self {
            version_manager,
            config,
            active_transactions: DashMap::new(),
            id_generator: AtomicU64::new(1),
            stats,
            shutdown_flag: AtomicU64::new(0),
            monitor,
            cleaner,
        }
    }

    /// Get the version manager
    pub fn version_manager(&self) -> &Arc<VersionManager> {
        &self.version_manager
    }

    /// Start a new read transaction
    pub fn begin_read_transaction(
        &self,
        options: TransactionOptions,
    ) -> Result<TransactionId, TransactionError> {
        if self.shutdown_flag.load(Ordering::SeqCst) != 0 {
            return Err(TransactionError::Internal(
                "Transaction manager is shutdown".to_string(),
            ));
        }

        self.cleanup_expired_transactions();

        let active_count = self.active_transactions.len();
        if active_count >= self.config.max_concurrent_transactions {
            return Err(TransactionError::TooManyTransactions);
        }

        let txn_id = self.id_generator.fetch_add(1, Ordering::SeqCst);
        let timestamp = self.version_manager.acquire_read_timestamp();
        let timeout = options.timeout.unwrap_or(self.config.default_timeout);

        let config = TransactionConfig {
            timeout,
            durability: options.durability,
            isolation_level: options.isolation_level,
            query_timeout: options.query_timeout,
            statement_timeout: options.statement_timeout,
            idle_timeout: options.idle_timeout,
            two_phase_commit: options.two_phase_commit,
        };

        let context = Arc::new(TransactionContext::new_readonly(txn_id, timestamp, config));

        self.active_transactions.insert(txn_id, context);
        self.stats.increment_total();
        self.stats.increment_active();

        Ok(txn_id)
    }

    /// Start a new insert transaction
    pub fn begin_insert_transaction(
        &self,
        options: TransactionOptions,
    ) -> Result<TransactionId, TransactionError> {
        if self.shutdown_flag.load(Ordering::SeqCst) != 0 {
            return Err(TransactionError::Internal(
                "Transaction manager is shutdown".to_string(),
            ));
        }

        self.cleanup_expired_transactions();

        let active_count = self.active_transactions.len();
        if active_count >= self.config.max_concurrent_transactions {
            return Err(TransactionError::TooManyTransactions);
        }

        let txn_id = self.id_generator.fetch_add(1, Ordering::SeqCst);
        let timestamp = self.version_manager.acquire_insert_timestamp();
        let timeout = options.timeout.unwrap_or(self.config.default_timeout);

        let config = TransactionConfig {
            timeout,
            durability: options.durability,
            isolation_level: options.isolation_level,
            query_timeout: options.query_timeout,
            statement_timeout: options.statement_timeout,
            idle_timeout: options.idle_timeout,
            two_phase_commit: options.two_phase_commit,
        };

        let context = Arc::new(TransactionContext::new(txn_id, timestamp, config));

        self.active_transactions.insert(txn_id, context);
        self.stats.increment_total();
        self.stats.increment_active();

        Ok(txn_id)
    }

    /// Start a new update transaction
    ///
    /// Update transactions require exclusive access and will block
    /// until all other transactions complete.
    pub fn begin_update_transaction(
        &self,
        options: TransactionOptions,
    ) -> Result<TransactionId, TransactionError> {
        if self.shutdown_flag.load(Ordering::SeqCst) != 0 {
            return Err(TransactionError::Internal(
                "Transaction manager is shutdown".to_string(),
            ));
        }

        let txn_id = self.id_generator.fetch_add(1, Ordering::SeqCst);
        let timestamp = self.version_manager.acquire_update_timestamp()
            .map_err(|e| TransactionError::Internal(e.to_string()))?;
        let timeout = options.timeout.unwrap_or(self.config.default_timeout);

        let config = TransactionConfig {
            timeout,
            durability: options.durability,
            isolation_level: options.isolation_level,
            query_timeout: options.query_timeout,
            statement_timeout: options.statement_timeout,
            idle_timeout: options.idle_timeout,
            two_phase_commit: options.two_phase_commit,
        };

        let context = Arc::new(TransactionContext::new(txn_id, timestamp, config));

        self.active_transactions.insert(txn_id, context);
        self.stats.increment_total();
        self.stats.increment_active();

        Ok(txn_id)
    }

    /// Start a new transaction (legacy API for compatibility)
    pub fn begin_transaction(
        &self,
        options: TransactionOptions,
    ) -> Result<TransactionId, TransactionError> {
        if options.read_only {
            self.begin_read_transaction(options)
        } else {
            self.begin_insert_transaction(options)
        }
    }

    /// Get transaction context
    pub fn get_context(
        &self,
        txn_id: TransactionId,
    ) -> Result<Arc<TransactionContext>, TransactionError> {
        self.active_transactions
            .get(&txn_id)
            .map(|entry| entry.value().clone())
            .ok_or(TransactionError::TransactionNotFound(txn_id))
    }

    /// Check if transaction exists and is active
    pub fn is_transaction_active(&self, txn_id: TransactionId) -> bool {
        self.active_transactions
            .get(&txn_id)
            .map(|entry| entry.value().state().can_execute())
            .unwrap_or(false)
    }

    /// Commit transaction
    pub fn commit_transaction(&self, txn_id: TransactionId) -> Result<(), TransactionError> {
        let context = {
            let entry = self
                .active_transactions
                .get(&txn_id)
                .ok_or(TransactionError::TransactionNotFound(txn_id))?;

            let ctx = entry.value().clone();
            drop(entry);

            if !ctx.state().can_commit() {
                return Err(TransactionError::InvalidStateForCommit(ctx.state()));
            }

            if ctx.is_expired() {
                self.active_transactions.remove(&txn_id);
                self.stats.increment_timeout();
                return Err(TransactionError::TransactionTimeout);
            }

            self.active_transactions.remove(&txn_id);
            ctx
        };

        context.transition_to(TransactionState::Committing)?;

        if context.read_only {
            self.version_manager.release_read_timestamp();
        } else {
            self.version_manager.release_insert_timestamp(context.timestamp());
        }

        context.transition_to(TransactionState::Committed)?;

        self.stats.decrement_active();
        self.stats.increment_committed();

        Ok(())
    }

    /// Commit transaction with undo target (for rollback support)
    pub fn commit_transaction_with_undo(
        &self,
        txn_id: TransactionId,
        _target: &mut dyn UndoTarget,
    ) -> Result<(), TransactionError> {
        self.commit_transaction(txn_id)
    }

    /// Abort transaction
    pub fn abort_transaction(&self, txn_id: TransactionId) -> Result<(), TransactionError> {
        let context = {
            let entry = self
                .active_transactions
                .get(&txn_id)
                .ok_or(TransactionError::TransactionNotFound(txn_id))?;
            let ctx = entry.value().clone();
            drop(entry);

            if !ctx.state().can_abort() {
                return Err(TransactionError::InvalidStateForAbort(ctx.state()));
            }

            self.active_transactions.remove(&txn_id);
            ctx
        };

        self.abort_transaction_internal(&context)
    }

    /// Abort transaction with undo target (for rollback support)
    pub fn abort_transaction_with_undo(
        &self,
        txn_id: TransactionId,
        target: &mut dyn UndoTarget,
    ) -> Result<(), TransactionError> {
        let context = {
            let entry = self
                .active_transactions
                .get(&txn_id)
                .ok_or(TransactionError::TransactionNotFound(txn_id))?;
            let ctx = entry.value().clone();
            drop(entry);

            if !ctx.state().can_abort() {
                return Err(TransactionError::InvalidStateForAbort(ctx.state()));
            }

            self.active_transactions.remove(&txn_id);
            ctx
        };

        context.execute_undo_logs(target)?;
        self.abort_transaction_internal(&context)
    }

    /// Internal abort implementation
    fn abort_transaction_internal(&self, context: &TransactionContext) -> Result<(), TransactionError> {
        context.transition_to(TransactionState::Aborting)?;

        if context.read_only {
            self.version_manager.release_read_timestamp();
        } else {
            self.version_manager.release_insert_timestamp(context.timestamp());
        }

        context.transition_to(TransactionState::Aborted)?;

        self.stats.decrement_active();
        self.stats.increment_aborted();

        Ok(())
    }

    /// Get active transaction list
    pub fn list_active_transactions(&self) -> Vec<TransactionInfo> {
        self.monitor.list_active_transactions(&self.active_transactions)
    }

    /// Get transaction info
    pub fn get_transaction_info(&self, txn_id: TransactionId) -> Option<TransactionInfo> {
        self.monitor.get_transaction_info(&self.active_transactions, txn_id)
    }

    /// Get statistics
    pub fn stats(&self) -> &TransactionStats {
        self.monitor.stats()
    }

    /// Cleanup expired transactions
    pub fn cleanup_expired_transactions(&self) {
        self.cleaner.cleanup_expired_transactions(&self.active_transactions);
    }

    /// Shutdown transaction manager
    pub fn shutdown(&self) {
        self.shutdown_flag.store(1, Ordering::SeqCst);

        let txn_ids: Vec<TransactionId> = {
            self.active_transactions
                .iter()
                .map(|entry| *entry.key())
                .collect()
        };

        for txn_id in txn_ids {
            let _ = self.abort_transaction(txn_id);
        }
    }

    /// Get configuration
    pub fn config(&self) -> TransactionManagerConfig {
        self.config.clone()
    }

    /// Create savepoint
    pub fn create_savepoint(
        &self,
        txn_id: TransactionId,
        name: Option<String>,
    ) -> Result<SavepointId, TransactionError> {
        let context = self.get_context(txn_id)?;
        Ok(context.create_savepoint(name))
    }

    /// Get savepoint info
    pub fn get_savepoint(&self, txn_id: TransactionId, id: SavepointId) -> Option<SavepointInfo> {
        let context = self.get_context(txn_id).ok()?;
        context.get_savepoint(id)
    }

    /// Release savepoint
    pub fn release_savepoint(
        &self,
        txn_id: TransactionId,
        id: SavepointId,
    ) -> Result<(), TransactionError> {
        let context = self.get_context(txn_id)?;
        context.release_savepoint(id)
    }

    /// Rollback to savepoint
    pub fn rollback_to_savepoint(
        &self,
        txn_id: TransactionId,
        id: SavepointId,
        target: &mut dyn UndoTarget,
    ) -> Result<(), TransactionError> {
        let context = self.get_context(txn_id)?;
        context.rollback_to_savepoint(id, target)
    }

    /// Get all active savepoints for transaction
    pub fn get_active_savepoints(&self, txn_id: TransactionId) -> Vec<SavepointInfo> {
        self.get_context(txn_id)
            .map(|ctx| ctx.get_all_savepoints())
            .unwrap_or_default()
    }

    /// Get current write timestamp
    pub fn write_timestamp(&self) -> u32 {
        self.version_manager.write_timestamp()
    }

    /// Get current read timestamp
    pub fn read_timestamp(&self) -> u32 {
        self.version_manager.read_timestamp()
    }

    /// Check if an update transaction is in progress
    pub fn is_update_in_progress(&self) -> bool {
        self.version_manager.is_update_in_progress()
    }

    /// Get pending transaction count
    pub fn pending_count(&self) -> i32 {
        self.version_manager.pending_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_manager_basic() {
        let manager = TransactionManager::new(TransactionManagerConfig::default());

        let txn_id = manager
            .begin_read_transaction(TransactionOptions::default())
            .expect("Failed to begin read transaction");

        assert!(manager.is_transaction_active(txn_id));

        manager.commit_transaction(txn_id).expect("Failed to commit");

        assert!(!manager.is_transaction_active(txn_id));
    }

    #[test]
    fn test_transaction_manager_insert() {
        let manager = TransactionManager::new(TransactionManagerConfig::default());

        let txn_id = manager
            .begin_insert_transaction(TransactionOptions::default())
            .expect("Failed to begin insert transaction");

        assert!(manager.is_transaction_active(txn_id));

        manager.commit_transaction(txn_id).expect("Failed to commit");

        assert!(!manager.is_transaction_active(txn_id));
    }

    #[test]
    fn test_transaction_manager_abort() {
        let manager = TransactionManager::new(TransactionManagerConfig::default());

        let txn_id = manager
            .begin_read_transaction(TransactionOptions::default())
            .expect("Failed to begin read transaction");

        manager.abort_transaction(txn_id).expect("Failed to abort");

        assert!(!manager.is_transaction_active(txn_id));
        assert_eq!(manager.stats().aborted_transactions.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_transaction_manager_savepoint() {
        let manager = TransactionManager::new(TransactionManagerConfig::default());

        let txn_id = manager
            .begin_insert_transaction(TransactionOptions::default())
            .expect("Failed to begin transaction");

        let sp_id = manager.create_savepoint(txn_id, Some("test".to_string())).expect("Failed to create savepoint");

        let sp = manager.get_savepoint(txn_id, sp_id).expect("Failed to get savepoint");
        assert_eq!(sp.name, Some("test".to_string()));

        manager.commit_transaction(txn_id).expect("Failed to commit");
    }

    #[test]
    fn test_transaction_manager_shutdown() {
        let manager = TransactionManager::new(TransactionManagerConfig::default());

        let txn_id = manager
            .begin_read_transaction(TransactionOptions::default())
            .expect("Failed to begin transaction");

        manager.shutdown();

        assert!(!manager.is_transaction_active(txn_id));
    }
}

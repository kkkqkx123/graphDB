//! Transaction Manager
//!
//! Manages the lifecycle of all transactions, providing operations such as transaction start, commit, and abort

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use redb::Database;

use crate::sync::{SyncError, SyncManager};
use crate::transaction::context::TransactionContext;
use crate::transaction::types::*;

/// Transaction Manager
pub struct TransactionManager {
    /// Database instance
    db: Arc<Database>,
    /// Configuration
    config: TransactionManagerConfig,
    /// Active transactions table - Using DashMap instead of RwLock<HashMap> for better concurrent performance
    /// DashMap internally uses Arc for values, no need for extra Arc wrapper
    active_transactions: DashMap<TransactionId, Arc<TransactionContext>>,
    /// Transaction ID generator
    id_generator: AtomicU64,
    /// Statistics
    stats: Arc<TransactionStats>,
    /// Whether shutdown
    shutdown_flag: AtomicU64,
    /// Optional sync manager for fulltext index synchronization
    sync_manager: Option<Arc<SyncManager>>,
}

impl TransactionManager {
    /// Create a new transaction manager
    pub fn new(db: Arc<Database>, config: TransactionManagerConfig) -> Self {
        Self {
            db,
            config,
            active_transactions: DashMap::new(),
            id_generator: AtomicU64::new(1),
            stats: Arc::new(TransactionStats::new()),
            shutdown_flag: AtomicU64::new(0),
            sync_manager: None,
        }
    }

    /// Create a new transaction manager with sync manager
    pub fn with_sync_manager(
        db: Arc<Database>,
        config: TransactionManagerConfig,
        sync_manager: Arc<SyncManager>,
    ) -> Self {
        Self {
            db,
            config,
            active_transactions: DashMap::new(),
            id_generator: AtomicU64::new(1),
            stats: Arc::new(TransactionStats::new()),
            shutdown_flag: AtomicU64::new(0),
            sync_manager: Some(sync_manager),
        }
    }

    /// Set sync manager
    pub fn set_sync_manager(&mut self, sync_manager: Arc<SyncManager>) {
        self.sync_manager = Some(sync_manager);
    }

    /// Start a new transaction
    pub fn begin_transaction(
        &self,
        options: TransactionOptions,
    ) -> Result<TransactionId, TransactionError> {
        // Check if shutdown
        if self.shutdown_flag.load(Ordering::SeqCst) != 0 {
            return Err(TransactionError::Internal(
                "Transaction manager is shutdown".to_string(),
            ));
        }

        // Check concurrent transaction count limit
        let active_count = self.active_transactions.len();
        if active_count >= self.config.max_concurrent_transactions {
            return Err(TransactionError::TooManyTransactions);
        }

        // Check if there is already an active write transaction
        if !options.read_only {
            for entry in self.active_transactions.iter() {
                let context = entry.value();
                if !context.read_only {
                    return Err(TransactionError::WriteTransactionConflict);
                }
            }
        }

        let txn_id = self.id_generator.fetch_add(1, Ordering::SeqCst);
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

        let db = Arc::clone(&self.db);
        let context = if options.read_only {
            let read_txn = self
                .db
                .begin_read()
                .map_err(|e| TransactionError::BeginFailed(e.to_string()))?;

            Arc::new(TransactionContext::new_readonly(
                txn_id,
                config,
                read_txn,
                Some(db),
            ))
        } else {
            let write_txn = self
                .db
                .begin_write()
                .map_err(|e| TransactionError::BeginFailed(e.to_string()))?;

            Arc::new(TransactionContext::new_writable(
                txn_id,
                config,
                write_txn,
                Some(db),
            ))
        };

        self.active_transactions.insert(txn_id, context);
        self.stats.increment_total();
        self.stats.increment_active();

        Ok(txn_id)
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
    pub async fn commit_transaction(&self, txn_id: TransactionId) -> Result<(), TransactionError> {
        // Remove transaction from DashMap and get ownership
        let context = {
            let entry = self
                .active_transactions
                .get(&txn_id)
                .ok_or(TransactionError::TransactionNotFound(txn_id))?;

            let ctx = entry.value().clone();
            drop(entry);

            // Check state
            if !ctx.state().can_commit() {
                return Err(TransactionError::InvalidStateForCommit(ctx.state()));
            }

            // Check timeout
            if ctx.is_expired() {
                // Already expired, remove and abort
                self.active_transactions.remove(&txn_id);
                self.stats.increment_timeout();
                // Abort transaction
                self.abort_transaction_internal(ctx)?;
                return Err(TransactionError::TransactionTimeout);
            }

            // State check passed, remove transaction
            self.active_transactions.remove(&txn_id);
            ctx
        };

        // Execute commit
        context.transition_to(TransactionState::Committing)?;

        // Phase 1: Prepare (if two-phase commit is enabled)
        if context.is_two_phase_enabled() {
            if let Some(ref sync_manager) = self.sync_manager {
                // 1. Prepare index sync
                sync_manager
                    .prepare_transaction(txn_id)
                    .await
                    .map_err(|e: SyncError| TransactionError::SyncFailed(e.to_string()))?;

                log::debug!(
                    "Index sync prepared successfully for transaction {:?}",
                    txn_id
                );
            }
        }

        // Phase 2: Commit
        // Commit redb transaction
        if !context.read_only {
            let mut write_txn = context.take_write_txn()?;

            // Set durability level
            let durability: redb::Durability = context.durability.into();
            write_txn.set_durability(durability);

            // Commit transaction
            write_txn
                .commit()
                .map_err(|e| TransactionError::CommitFailed(e.to_string()))?;
        }

        context.transition_to(TransactionState::Committed)?;

        // Confirm index sync (if two-phase commit was used)
        if context.is_two_phase_enabled() {
            if let Some(ref sync_manager) = self.sync_manager {
                // Confirm the index sync after redb commit
                if let Err(e) = sync_manager.commit_transaction(txn_id).await {
                    log::error!(
                        "Failed to confirm index sync for transaction {:?}: {}",
                        txn_id,
                        e
                    );
                    // Note: redb already committed, can only log error
                }
            }
        } else {
            // Non-two-phase mode: trigger sync after commit
            if let Some(ref sync_manager) = self.sync_manager {
                match sync_manager.commit_all().await {
                    Ok(()) => {
                        log::debug!("Index sync completed successfully after transaction commit");
                    }
                    Err(e) => {
                        log::warn!("Index sync failed but transaction committed: {}", e);
                        // Transaction already committed, just log the error
                    }
                }
            }
        }

        // Cleanup
        self.stats.decrement_active();
        self.stats.increment_committed();

        Ok(())
    }

    /// Abort transaction (internal version, does not operate HashMap)
    fn abort_transaction_internal(
        &self,
        context: Arc<TransactionContext>,
    ) -> Result<(), TransactionError> {
        if !context.state().can_abort() {
            return Err(TransactionError::InvalidStateForAbort(context.state()));
        }

        context.transition_to(TransactionState::Aborting)?;

        // Rollback index sync if sync manager is present
        let txn_id = context.id;
        if let Some(ref sync_manager) = self.sync_manager {
            if let Err(e) = futures::executor::block_on(sync_manager.rollback_transaction(txn_id)) {
                log::warn!("Index sync rollback failed for transaction {:?}: {}", txn_id, e);
                // Continue with storage rollback, don't fail the whole operation
            }
        }

        // Take write transaction, automatic rollback on Drop
        if !context.read_only {
            let _ = context.take_write_txn();
        }

        self.stats.decrement_active();
        self.stats.increment_aborted();

        Ok(())
    }

    /// Abort transaction
    pub fn abort_transaction(&self, txn_id: TransactionId) -> Result<(), TransactionError> {
        // Remove transaction from DashMap and get ownership
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

            // State check passed, remove transaction
            self.active_transactions.remove(&txn_id);
            ctx
        };

        // Execute abort
        self.abort_transaction_internal(context)
    }

    /// Get active transaction list
    pub fn list_active_transactions(&self) -> Vec<TransactionInfo> {
        self.active_transactions
            .iter()
            .map(|entry| entry.value().info())
            .collect()
    }

    /// Get transaction info
    ///
    /// # Arguments
    /// * `txn_id` - Transaction ID
    ///
    /// # Returns
    /// * `Some(TransactionInfo)` - If transaction exists
    /// * `None` - If transaction does not exist
    pub fn get_transaction_info(&self, txn_id: TransactionId) -> Option<TransactionInfo> {
        self.active_transactions
            .get(&txn_id)
            .map(|entry| entry.value().info())
    }

    /// Get statistics
    pub fn stats(&self) -> &TransactionStats {
        &self.stats
    }

    /// Cleanup expired transactions
    pub fn cleanup_expired_transactions(&self) {
        // Collect all expired transaction IDs
        let expired: Vec<TransactionId> = {
            self.active_transactions
                .iter()
                .filter(|entry| entry.value().is_expired())
                .map(|entry| *entry.key())
                .collect()
        };

        for txn_id in expired {
            let _ = self.abort_transaction(txn_id);
            self.stats.increment_timeout();
        }
    }

    /// Shutdown transaction manager
    pub fn shutdown(&self) {
        // Set shutdown flag
        self.shutdown_flag.store(1, Ordering::SeqCst);

        // Abort all active transactions
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
    ///
    /// # Arguments
    /// * `txn_id` - Transaction ID
    /// * `name` - Savepoint name (optional)
    ///
    /// # Returns
    /// * `Ok(SavepointId)` - Savepoint ID
    /// * `Err(TransactionError)` - Error on failure
    pub fn create_savepoint(
        &self,
        txn_id: TransactionId,
        name: Option<String>,
    ) -> Result<SavepointId, TransactionError> {
        let context = self.get_context(txn_id)?;
        Ok(context.create_savepoint(name))
    }

    /// Get savepoint info
    ///
    /// # Arguments
    /// * `txn_id` - Transaction ID
    /// * `id` - Savepoint ID
    ///
    /// # Returns
    /// * `Some(SavepointInfo)` - Savepoint info
    /// * `None` - Savepoint does not exist
    pub fn get_savepoint(&self, txn_id: TransactionId, id: SavepointId) -> Option<SavepointInfo> {
        let context = self.get_context(txn_id).ok()?;
        context.get_savepoint(id)
    }

    /// Release savepoint
    ///
    /// # Arguments
    /// * `txn_id` - Transaction ID
    /// * `id` - Savepoint ID
    ///
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err(TransactionError)` - Error on failure
    pub fn release_savepoint(
        &self,
        txn_id: TransactionId,
        id: SavepointId,
    ) -> Result<(), TransactionError> {
        let context = self.get_context(txn_id)?;
        context.release_savepoint(id)
    }

    /// Rollback to savepoint
    ///
    /// # Arguments
    /// * `txn_id` - Transaction ID
    /// * `id` - Savepoint ID
    ///
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err(TransactionError)` - Error on failure
    ///
    /// # Note
    /// This method removes all savepoints after this savepoint
    /// Actual data rollback needs to be implemented with the storage layer
    pub fn rollback_to_savepoint(
        &self,
        txn_id: TransactionId,
        id: SavepointId,
    ) -> Result<(), TransactionError> {
        let context = self.get_context(txn_id)?;
        context.rollback_to_savepoint(id)
    }

    /// Get all active savepoints for transaction
    ///
    /// # Arguments
    /// * `txn_id` - Transaction ID
    ///
    /// # Returns
    /// * `Vec<SavepointInfo>` - Savepoint info list
    pub fn get_active_savepoints(&self, txn_id: TransactionId) -> Vec<SavepointInfo> {
        let context = match self.get_context(txn_id) {
            Ok(ctx) => ctx,
            Err(_) => return Vec::new(),
        };
        context.get_all_savepoints()
    }

    /// Find savepoint by name
    ///
    /// # Arguments
    /// * `txn_id` - Transaction ID
    /// * `name` - Savepoint name
    ///
    /// # Returns
    /// * `Some(SavepointInfo)` - Savepoint info
    /// * `None` - Savepoint does not exist
    pub fn find_savepoint_by_name(
        &self,
        txn_id: TransactionId,
        name: &str,
    ) -> Option<SavepointInfo> {
        let context = self.get_context(txn_id).ok()?;
        context.find_savepoint_by_name(name)
    }

    /// Execute operation with retry mechanism
    ///
    /// # Arguments
    /// * `options` - Transaction options
    /// * `retry_config` - Retry configuration
    /// * `f` - Operation to execute
    ///
    /// # Returns
    /// * `Ok(R)` - Operation result
    /// * `Err(TransactionError)` - Error on failure
    ///
    /// # Note
    /// Only retryable errors will be retried (WriteTransactionConflict, TransactionTimeout)
    pub async fn execute_with_retry<F, R>(
        &self,
        options: TransactionOptions,
        retry_config: RetryConfig,
        f: F,
    ) -> Result<R, TransactionError>
    where
        F: Fn(TransactionId) -> Result<R, TransactionError>,
    {
        let mut last_error = None;
        let mut delay = retry_config.initial_delay;

        for attempt in 0..=retry_config.max_retries {
            let txn_id = self.begin_transaction(options.clone())?;

            match f(txn_id) {
                Ok(result) => {
                    self.commit_transaction(txn_id).await?;
                    return Ok(result);
                }
                Err(e) => {
                    self.abort_transaction(txn_id)?;
                    last_error = Some(e.clone());

                    // Check if error is retryable
                    let is_retryable = matches!(
                        e,
                        TransactionError::WriteTransactionConflict
                            | TransactionError::TransactionTimeout
                    );

                    if !is_retryable || attempt == retry_config.max_retries {
                        return Err(e);
                    }

                    // Wait before retry
                    std::thread::sleep(delay);

                    // Exponential backoff
                    delay = std::cmp::min(
                        Duration::from_secs_f64(
                            delay.as_secs_f64() * retry_config.backoff_multiplier,
                        ),
                        retry_config.max_delay,
                    );
                }
            }
        }

        Err(last_error.unwrap_or(TransactionError::Internal("Retry failed".to_string())))
    }

    /// Commit multiple transactions in batch
    ///
    /// # Arguments
    /// * `txn_ids` - Transaction IDs to commit
    ///
    /// # Returns
    /// * `Ok(())` - All transactions committed successfully
    /// * `Err(TransactionError)` - Error on failure (all transactions will be rolled back)
    ///
    /// # Note
    /// If any transaction fails to commit, all previously committed transactions will be rolled back
    pub async fn commit_batch(&self, txn_ids: Vec<TransactionId>) -> Result<(), TransactionError> {
        let mut committed = Vec::new();

        for txn_id in txn_ids {
            match self.commit_transaction(txn_id).await {
                Ok(()) => committed.push(txn_id),
                Err(e) => {
                    // Rollback all previously committed transactions
                    for committed_id in committed {
                        let _ = self.abort_transaction(committed_id);
                    }
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    /// Get transaction metrics
    ///
    /// # Returns
    /// * `TransactionMetrics` - Transaction metrics
    pub fn get_metrics(&self) -> TransactionMetrics {
        let mut metrics = TransactionMetrics::new();

        // Collect transaction durations
        let durations: Vec<Duration> = self
            .active_transactions
            .iter()
            .map(|entry| entry.value().start_time.elapsed())
            .collect();

        if durations.is_empty() {
            return metrics;
        }

        // Calculate percentiles
        let mut sorted_durations = durations.clone();
        sorted_durations.sort();

        metrics.p50_duration = sorted_durations[sorted_durations.len() * 50 / 100];
        metrics.p95_duration = sorted_durations[sorted_durations.len() * 95 / 100];
        metrics.p99_duration = sorted_durations[sorted_durations.len() * 99 / 100];

        // Calculate average
        let total: Duration = durations.iter().sum();
        metrics.avg_duration = total / durations.len() as u32;

        // Collect long transactions (duration > 10s)
        metrics.long_transactions = self
            .active_transactions
            .iter()
            .filter(|entry| entry.value().start_time.elapsed() > Duration::from_secs(10))
            .map(|entry| entry.value().info())
            .collect();

        metrics.total_count = self.stats.total_transactions.load(Ordering::Relaxed);

        metrics
    }

    /// Get all active transactions info
    ///
    /// # Returns
    /// * `Vec<TransactionInfo>` - Active transactions info
    pub fn get_active_transactions(&self) -> Vec<TransactionInfo> {
        self.active_transactions
            .iter()
            .map(|entry| entry.value().info())
            .collect()
    }

    /// Get long transactions (duration > 10s)
    ///
    /// # Returns
    /// * `Vec<TransactionInfo>` - Long transactions info
    pub fn get_long_transactions(&self) -> Vec<TransactionInfo> {
        self.active_transactions
            .iter()
            .filter(|entry| entry.value().start_time.elapsed() > Duration::from_secs(10))
            .map(|entry| entry.value().info())
            .collect()
    }
}

impl Drop for TransactionManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn create_test_manager() -> (TransactionManager, Arc<Database>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let db = Arc::new(
            Database::create(temp_dir.path().join("test.db"))
                .expect("Failed to create test database"),
        );
        let config = TransactionManagerConfig::default();
        let manager = TransactionManager::new(db.clone(), config);
        (manager, db, temp_dir)
    }

    #[tokio::test]
    async fn test_begin_and_commit_transaction() {
        let (manager, _db, _temp) = create_test_manager();

        let txn_id = manager
            .begin_transaction(TransactionOptions::default())
            .expect("Failed to begin transaction");

        assert!(manager.is_transaction_active(txn_id));

        manager
            .commit_transaction(txn_id)
            .await
            .expect("Failed to commit transaction");

        assert!(!manager.is_transaction_active(txn_id));
        assert_eq!(
            manager
                .stats()
                .committed_transactions
                .load(Ordering::Relaxed),
            1
        );
    }

    #[tokio::test]
    async fn test_begin_and_abort_transaction() {
        let (manager, _db, _temp) = create_test_manager();

        let txn_id = manager
            .begin_transaction(TransactionOptions::default())
            .expect("Failed to begin transaction");

        manager
            .abort_transaction(txn_id)
            .expect("Failed to abort transaction");

        assert!(!manager.is_transaction_active(txn_id));
        assert_eq!(
            manager.stats().aborted_transactions.load(Ordering::Relaxed),
            1
        );
    }

    #[tokio::test]
    async fn test_readonly_transaction() {
        let (manager, _db, _temp) = create_test_manager();

        let options = TransactionOptions::new().read_only();
        let txn_id = manager
            .begin_transaction(options)
            .expect("Failed to begin readonly transaction");

        let context = manager
            .get_context(txn_id)
            .expect("Failed to get transaction context");
        assert!(context.read_only);

        manager
            .commit_transaction(txn_id)
            .await
            .expect("Failed to commit readonly transaction");
    }

    #[test]
    fn test_transaction_not_found() {
        let (manager, _db, _temp) = create_test_manager();

        let result = manager.get_context(9999);
        assert!(matches!(
            result,
            Err(TransactionError::TransactionNotFound(9999))
        ));
    }

    #[tokio::test]
    async fn test_invalid_state_transition() {
        let (manager, _db, _temp) = create_test_manager();

        let txn_id = manager
            .begin_transaction(TransactionOptions::default())
            .expect("Failed to begin transaction");

        // Commit transaction
        manager
            .commit_transaction(txn_id)
            .await
            .expect("Failed to commit transaction");

        // Second commit should fail
        let result = manager.commit_transaction(txn_id).await;
        assert!(matches!(
            result,
            Err(TransactionError::TransactionNotFound(_))
        ));
    }

    #[tokio::test]
    async fn test_concurrent_transactions() {
        let (manager, _db, _temp) = create_test_manager();

        // Due to redb's single-writer restriction, we can only execute transactions sequentially
        // First transaction
        let txn1 = manager
            .begin_transaction(TransactionOptions::default())
            .expect("Failed to begin transaction");
        assert!(manager.is_transaction_active(txn1));
        manager
            .commit_transaction(txn1)
            .await
            .expect("Failed to commit transaction");
        assert!(!manager.is_transaction_active(txn1));

        // Second transaction
        let txn2 = manager
            .begin_transaction(TransactionOptions::default())
            .expect("Failed to begin transaction");
        assert!(manager.is_transaction_active(txn2));
        manager
            .abort_transaction(txn2)
            .expect("Failed to abort transaction");
        assert!(!manager.is_transaction_active(txn2));

        // Third transaction
        let txn3 = manager
            .begin_transaction(TransactionOptions::default())
            .expect("Failed to begin transaction");
        assert!(manager.is_transaction_active(txn3));
        manager
            .commit_transaction(txn3)
            .await
            .expect("Failed to commit transaction");
        assert!(!manager.is_transaction_active(txn3));

        assert_eq!(
            manager
                .stats()
                .committed_transactions
                .load(Ordering::Relaxed),
            2
        );
        assert_eq!(
            manager.stats().aborted_transactions.load(Ordering::Relaxed),
            1
        );
    }

    #[tokio::test]
    async fn test_multiple_readonly_transactions() {
        let (manager, _db, _temp) = create_test_manager();

        // Read-only transactions can be concurrent
        let options = TransactionOptions::new().read_only();
        let txn1 = manager
            .begin_transaction(options.clone())
            .expect("Failed to begin transaction");
        let txn2 = manager
            .begin_transaction(options.clone())
            .expect("Failed to begin transaction");
        let txn3 = manager
            .begin_transaction(options)
            .expect("Failed to begin transaction");

        assert!(manager.is_transaction_active(txn1));
        assert!(manager.is_transaction_active(txn2));
        assert!(manager.is_transaction_active(txn3));

        manager
            .commit_transaction(txn1)
            .await
            .expect("Failed to commit transaction");
        manager
            .commit_transaction(txn2)
            .await
            .expect("Failed to commit transaction");
        manager
            .commit_transaction(txn3)
            .await
            .expect("Failed to commit transaction");
    }
}

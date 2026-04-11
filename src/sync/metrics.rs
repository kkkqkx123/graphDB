//! Metrics for sync system
//!
//! Provides monitoring and statistics for transaction and index synchronization.

use std::fmt;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;

/// Sync system metrics
#[derive(Debug, Default)]
pub struct SyncMetrics {
    /// Total number of transactions committed
    pub transactions_committed: AtomicU64,
    /// Total number of transactions rolled back
    pub transactions_rolled_back: AtomicU64,
    /// Total number of index operations
    pub index_operations_total: AtomicU64,
    /// Number of index operations by type (insert, update, delete)
    pub index_operations_insert: AtomicU64,
    pub index_operations_update: AtomicU64,
    pub index_operations_delete: AtomicU64,
    /// Number of retry attempts
    pub retry_attempts_total: AtomicU64,
    /// Number of successful retries
    pub retry_successes: AtomicU64,
    /// Number of failed retries (exhausted)
    pub retry_failures: AtomicU64,
    /// Number of operations in dead letter queue
    pub dead_letter_queue_size: AtomicUsize,
    /// Current number of active transactions
    pub active_transactions: AtomicUsize,
    /// Total processing time (in milliseconds)
    pub total_processing_time_ms: AtomicU64,
    /// Last error message
    last_error: parking_lot::Mutex<Option<String>>,
}

impl SyncMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_transaction_commit(&self) {
        self.transactions_committed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_transaction_rollback(&self) {
        self.transactions_rolled_back
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_index_operation(&self, operation_type: &str) {
        self.index_operations_total.fetch_add(1, Ordering::Relaxed);
        match operation_type {
            "insert" => self.index_operations_insert.fetch_add(1, Ordering::Relaxed),
            "update" => self.index_operations_update.fetch_add(1, Ordering::Relaxed),
            "delete" => self.index_operations_delete.fetch_add(1, Ordering::Relaxed),
            _ => return,
        };
    }

    pub fn record_retry_attempt(&self) {
        self.retry_attempts_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_retry_success(&self) {
        self.retry_successes.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_retry_failure(&self) {
        self.retry_failures.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_dead_letter(&self) {
        self.dead_letter_queue_size.fetch_add(1, Ordering::Relaxed);
    }

    pub fn remove_dead_letter(&self) {
        self.dead_letter_queue_size.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn record_active_transaction_start(&self) {
        self.active_transactions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_active_transaction_end(&self) {
        self.active_transactions.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn record_processing_time(&self, duration: Duration) {
        let ms = duration.as_millis() as u64;
        self.total_processing_time_ms
            .fetch_add(ms, Ordering::Relaxed);
    }

    pub fn record_error(&self, error: impl Into<String>) {
        *self.last_error.lock() = Some(error.into());
    }

    pub fn get_last_error(&self) -> Option<String> {
        self.last_error.lock().clone()
    }

    pub fn get_stats(&self) -> SyncStats {
        SyncStats {
            transactions_committed: self.transactions_committed.load(Ordering::Relaxed),
            transactions_rolled_back: self.transactions_rolled_back.load(Ordering::Relaxed),
            index_operations_total: self.index_operations_total.load(Ordering::Relaxed),
            index_operations_insert: self.index_operations_insert.load(Ordering::Relaxed),
            index_operations_update: self.index_operations_update.load(Ordering::Relaxed),
            index_operations_delete: self.index_operations_delete.load(Ordering::Relaxed),
            retry_attempts_total: self.retry_attempts_total.load(Ordering::Relaxed),
            retry_successes: self.retry_successes.load(Ordering::Relaxed),
            retry_failures: self.retry_failures.load(Ordering::Relaxed),
            dead_letter_queue_size: self.dead_letter_queue_size.load(Ordering::Relaxed),
            active_transactions: self.active_transactions.load(Ordering::Relaxed),
            total_processing_time_ms: self.total_processing_time_ms.load(Ordering::Relaxed),
        }
    }

    pub fn reset(&self) {
        self.transactions_committed.store(0, Ordering::Relaxed);
        self.transactions_rolled_back.store(0, Ordering::Relaxed);
        self.index_operations_total.store(0, Ordering::Relaxed);
        self.index_operations_insert.store(0, Ordering::Relaxed);
        self.index_operations_update.store(0, Ordering::Relaxed);
        self.index_operations_delete.store(0, Ordering::Relaxed);
        self.retry_attempts_total.store(0, Ordering::Relaxed);
        self.retry_successes.store(0, Ordering::Relaxed);
        self.retry_failures.store(0, Ordering::Relaxed);
        self.dead_letter_queue_size.store(0, Ordering::Relaxed);
        self.active_transactions.store(0, Ordering::Relaxed);
        self.total_processing_time_ms.store(0, Ordering::Relaxed);
        *self.last_error.lock() = None;
    }
}

/// Statistics snapshot
#[derive(Debug, Clone, Default)]
pub struct SyncStats {
    pub transactions_committed: u64,
    pub transactions_rolled_back: u64,
    pub index_operations_total: u64,
    pub index_operations_insert: u64,
    pub index_operations_update: u64,
    pub index_operations_delete: u64,
    pub retry_attempts_total: u64,
    pub retry_successes: u64,
    pub retry_failures: u64,
    pub dead_letter_queue_size: usize,
    pub active_transactions: usize,
    pub total_processing_time_ms: u64,
}

impl SyncStats {
    pub fn success_rate(&self) -> f64 {
        if self.retry_attempts_total == 0 {
            return 1.0;
        }
        self.retry_successes as f64 / self.retry_attempts_total as f64
    }

    pub fn avg_processing_time_ms(&self) -> f64 {
        let total_ops = self.transactions_committed + self.transactions_rolled_back;
        if total_ops == 0 {
            return 0.0;
        }
        self.total_processing_time_ms as f64 / total_ops as f64
    }
}

impl fmt::Display for SyncStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SyncStats {{ \
             committed: {}, rolled_back: {}, \
             index_ops: {} (insert: {}, update: {}, delete: {}), \
             retries: {} (success: {}, failures: {}), \
             dlq_size: {}, active_txns: {}, \
             avg_time_ms: {:.2} }}",
            self.transactions_committed,
            self.transactions_rolled_back,
            self.index_operations_total,
            self.index_operations_insert,
            self.index_operations_update,
            self.index_operations_delete,
            self.retry_attempts_total,
            self.retry_successes,
            self.retry_failures,
            self.dead_letter_queue_size,
            self.active_transactions,
            self.avg_processing_time_ms()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_recording() {
        let metrics = SyncMetrics::new();

        metrics.record_transaction_commit();
        metrics.record_transaction_commit();
        metrics.record_transaction_rollback();

        metrics.record_index_operation("insert");
        metrics.record_index_operation("update");
        metrics.record_index_operation("delete");

        metrics.record_retry_attempt();
        metrics.record_retry_success();

        let stats = metrics.get_stats();

        assert_eq!(stats.transactions_committed, 2);
        assert_eq!(stats.transactions_rolled_back, 1);
        assert_eq!(stats.index_operations_insert, 1);
        assert_eq!(stats.index_operations_update, 1);
        assert_eq!(stats.index_operations_delete, 1);
        assert_eq!(stats.retry_successes, 1);
        assert_eq!(stats.success_rate(), 1.0);
    }

    #[test]
    fn test_metrics_reset() {
        let metrics = SyncMetrics::new();

        metrics.record_transaction_commit();
        metrics.record_index_operation("insert");
        metrics.record_retry_attempt();

        metrics.reset();

        let stats = metrics.get_stats();
        assert_eq!(stats.transactions_committed, 0);
        assert_eq!(stats.index_operations_total, 0);
        assert_eq!(stats.retry_attempts_total, 0);
    }
}

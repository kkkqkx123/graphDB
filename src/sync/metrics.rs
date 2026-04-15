//! Metrics for sync system
//!
//! Provides monitoring and statistics for transaction and index synchronization.

use std::fmt;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;

/// Sync system metrics using metrics crate
#[derive(Debug, Default)]
pub struct SyncMetrics {
    // Internal counters maintained for get_stats() compatibility
    // The actual metrics are recorded via global metrics crate
    #[allow(dead_code)]
    internal_counters: SyncInternalCounters,
}

#[derive(Debug, Default)]
struct SyncInternalCounters {
    transactions_committed: AtomicU64,
    transactions_rolled_back: AtomicU64,
    index_operations_total: AtomicU64,
    index_operations_insert: AtomicU64,
    index_operations_update: AtomicU64,
    index_operations_delete: AtomicU64,
    retry_attempts_total: AtomicU64,
    retry_successes: AtomicU64,
    retry_failures: AtomicU64,
    dead_letter_queue_size: AtomicUsize,
    active_transactions: AtomicUsize,
    total_processing_time_ms: AtomicU64,
    compensation_attempts_total: AtomicU64,
    compensation_successes: AtomicU64,
    compensation_failures: AtomicU64,
}

impl SyncMetrics {
    pub fn new() -> Self {
        Self {
            internal_counters: SyncInternalCounters::default(),
        }
    }

    pub fn record_transaction_commit(&self) {
        metrics::counter!("graphdb_sync_transactions_committed_total").increment(1);
        self.internal_counters
            .transactions_committed
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_transaction_rollback(&self) {
        metrics::counter!("graphdb_sync_transactions_rolled_back_total").increment(1);
        self.internal_counters
            .transactions_rolled_back
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_index_operation(&self, operation_type: &str) {
        metrics::counter!("graphdb_sync_index_operations_total").increment(1);
        self.internal_counters
            .index_operations_total
            .fetch_add(1, Ordering::Relaxed);
        match operation_type {
            "insert" => {
                metrics::counter!("graphdb_sync_index_operations_insert_total").increment(1);
                self.internal_counters
                    .index_operations_insert
                    .fetch_add(1, Ordering::Relaxed);
            }
            "update" => {
                metrics::counter!("graphdb_sync_index_operations_update_total").increment(1);
                self.internal_counters
                    .index_operations_update
                    .fetch_add(1, Ordering::Relaxed);
            }
            "delete" => {
                metrics::counter!("graphdb_sync_index_operations_delete_total").increment(1);
                self.internal_counters
                    .index_operations_delete
                    .fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        };
    }

    pub fn record_retry_attempt(&self) {
        metrics::counter!("graphdb_sync_retry_attempts_total").increment(1);
        self.internal_counters
            .retry_attempts_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_retry_success(&self) {
        metrics::counter!("graphdb_sync_retry_successes_total").increment(1);
        self.internal_counters
            .retry_successes
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_retry_failure(&self) {
        metrics::counter!("graphdb_sync_retry_failures_total").increment(1);
        self.internal_counters
            .retry_failures
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_dead_letter(&self) {
        metrics::gauge!("graphdb_sync_dead_letter_queue_size").increment(1.0);
        self.internal_counters
            .dead_letter_queue_size
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn remove_dead_letter(&self) {
        metrics::gauge!("graphdb_sync_dead_letter_queue_size").decrement(1.0);
        self.internal_counters
            .dead_letter_queue_size
            .fetch_sub(1, Ordering::Relaxed);
    }

    pub fn record_compensation_attempt(&self, count: usize) {
        metrics::counter!("graphdb_sync_compensation_attempts_total").increment(count as u64);
        self.internal_counters
            .compensation_attempts_total
            .fetch_add(count as u64, Ordering::Relaxed);
    }

    pub fn record_compensation_success(&self, count: usize) {
        metrics::counter!("graphdb_sync_compensation_successes_total").increment(count as u64);
        self.internal_counters
            .compensation_successes
            .fetch_add(count as u64, Ordering::Relaxed);
    }

    pub fn record_compensation_failure(&self, count: usize) {
        metrics::counter!("graphdb_sync_compensation_failures_total").increment(count as u64);
        self.internal_counters
            .compensation_failures
            .fetch_add(count as u64, Ordering::Relaxed);
    }

    pub fn record_active_transaction_start(&self) {
        metrics::gauge!("graphdb_sync_active_transactions").increment(1.0);
        self.internal_counters
            .active_transactions
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_active_transaction_end(&self) {
        metrics::gauge!("graphdb_sync_active_transactions").decrement(1.0);
        self.internal_counters
            .active_transactions
            .fetch_sub(1, Ordering::Relaxed);
    }

    pub fn record_processing_time(&self, duration: Duration) {
        let ms = duration.as_millis() as u64;
        metrics::histogram!("graphdb_sync_processing_time_ms").record(ms as f64);
        self.internal_counters
            .total_processing_time_ms
            .fetch_add(ms, Ordering::Relaxed);
    }

    pub fn record_error(&self, _error: impl Into<String>) {
        metrics::counter!("graphdb_sync_errors_total").increment(1);
    }

    pub fn get_stats(&self) -> SyncStats {
        SyncStats {
            transactions_committed: self
                .internal_counters
                .transactions_committed
                .load(Ordering::Relaxed),
            transactions_rolled_back: self
                .internal_counters
                .transactions_rolled_back
                .load(Ordering::Relaxed),
            index_operations_total: self
                .internal_counters
                .index_operations_total
                .load(Ordering::Relaxed),
            index_operations_insert: self
                .internal_counters
                .index_operations_insert
                .load(Ordering::Relaxed),
            index_operations_update: self
                .internal_counters
                .index_operations_update
                .load(Ordering::Relaxed),
            index_operations_delete: self
                .internal_counters
                .index_operations_delete
                .load(Ordering::Relaxed),
            retry_attempts_total: self
                .internal_counters
                .retry_attempts_total
                .load(Ordering::Relaxed),
            retry_successes: self
                .internal_counters
                .retry_successes
                .load(Ordering::Relaxed),
            retry_failures: self
                .internal_counters
                .retry_failures
                .load(Ordering::Relaxed),
            dead_letter_queue_size: self
                .internal_counters
                .dead_letter_queue_size
                .load(Ordering::Relaxed),
            active_transactions: self
                .internal_counters
                .active_transactions
                .load(Ordering::Relaxed),
            total_processing_time_ms: self
                .internal_counters
                .total_processing_time_ms
                .load(Ordering::Relaxed),
            compensation_attempts_total: self
                .internal_counters
                .compensation_attempts_total
                .load(Ordering::Relaxed),
            compensation_successes: self
                .internal_counters
                .compensation_successes
                .load(Ordering::Relaxed),
            compensation_failures: self
                .internal_counters
                .compensation_failures
                .load(Ordering::Relaxed),
        }
    }

    pub fn reset(&self) {
        self.internal_counters
            .transactions_committed
            .store(0, Ordering::Relaxed);
        self.internal_counters
            .transactions_rolled_back
            .store(0, Ordering::Relaxed);
        self.internal_counters
            .index_operations_total
            .store(0, Ordering::Relaxed);
        self.internal_counters
            .index_operations_insert
            .store(0, Ordering::Relaxed);
        self.internal_counters
            .index_operations_update
            .store(0, Ordering::Relaxed);
        self.internal_counters
            .index_operations_delete
            .store(0, Ordering::Relaxed);
        self.internal_counters
            .retry_attempts_total
            .store(0, Ordering::Relaxed);
        self.internal_counters
            .retry_successes
            .store(0, Ordering::Relaxed);
        self.internal_counters
            .retry_failures
            .store(0, Ordering::Relaxed);
        self.internal_counters
            .dead_letter_queue_size
            .store(0, Ordering::Relaxed);
        self.internal_counters
            .active_transactions
            .store(0, Ordering::Relaxed);
        self.internal_counters
            .total_processing_time_ms
            .store(0, Ordering::Relaxed);
        self.internal_counters
            .compensation_attempts_total
            .store(0, Ordering::Relaxed);
        self.internal_counters
            .compensation_successes
            .store(0, Ordering::Relaxed);
        self.internal_counters
            .compensation_failures
            .store(0, Ordering::Relaxed);
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
    pub compensation_attempts_total: u64,
    pub compensation_successes: u64,
    pub compensation_failures: u64,
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
             avg_time_ms: {:.2}, \
             compensation: {} (success: {}, failures: {}) }}",
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
            self.avg_processing_time_ms(),
            self.compensation_attempts_total,
            self.compensation_successes,
            self.compensation_failures
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

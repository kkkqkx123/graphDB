//! Metrics for sync system
//!
//! Provides monitoring and statistics for transaction and index synchronization.

use std::time::Duration;

use crate::core::stats::CacheStats;

/// Sync system metrics using metrics crate
#[derive(Debug, Default)]
pub struct SyncMetrics {
    #[allow(dead_code)]
    cache_stats: CacheStats,
}

impl SyncMetrics {
    pub fn new() -> Self {
        Self {
            cache_stats: CacheStats::new(),
        }
    }

    pub fn record_transaction_commit(&self) {
        metrics::counter!("graphdb_sync_transactions_committed_total").increment(1);
    }

    pub fn record_transaction_rollback(&self) {
        metrics::counter!("graphdb_sync_transactions_rolled_back_total").increment(1);
    }

    pub fn record_index_operation(&self, operation_type: &str) {
        metrics::counter!("graphdb_sync_index_operations_total").increment(1);
        match operation_type {
            "insert" => {
                metrics::counter!("graphdb_sync_index_operations_insert_total").increment(1);
            }
            "update" => {
                metrics::counter!("graphdb_sync_index_operations_update_total").increment(1);
            }
            "delete" => {
                metrics::counter!("graphdb_sync_index_operations_delete_total").increment(1);
            }
            _ => {}
        };
    }

    pub fn record_retry_attempt(&self) {
        metrics::counter!("graphdb_sync_retry_attempts_total").increment(1);
    }

    pub fn record_retry_success(&self) {
        metrics::counter!("graphdb_sync_retry_successes_total").increment(1);
    }

    pub fn record_retry_failure(&self) {
        metrics::counter!("graphdb_sync_retry_failures_total").increment(1);
    }

    pub fn record_dead_letter(&self) {
        metrics::gauge!("graphdb_sync_dead_letter_queue_size").increment(1.0);
    }

    pub fn remove_dead_letter(&self) {
        metrics::gauge!("graphdb_sync_dead_letter_queue_size").decrement(1.0);
    }

    pub fn record_compensation_attempt(&self, count: usize) {
        metrics::counter!("graphdb_sync_compensation_attempts_total").increment(count as u64);
    }

    pub fn record_compensation_success(&self, count: usize) {
        metrics::counter!("graphdb_sync_compensation_successes_total").increment(count as u64);
    }

    pub fn record_compensation_failure(&self, count: usize) {
        metrics::counter!("graphdb_sync_compensation_failures_total").increment(count as u64);
    }

    pub fn record_active_transaction_start(&self) {
        metrics::gauge!("graphdb_sync_active_transactions").increment(1.0);
    }

    pub fn record_active_transaction_end(&self) {
        metrics::gauge!("graphdb_sync_active_transactions").decrement(1.0);
    }

    pub fn record_processing_time(&self, duration: Duration) {
        let ms = duration.as_millis() as u64;
        metrics::histogram!("graphdb_sync_processing_time_ms").record(ms as f64);
    }

    pub fn record_error(&self, _error: impl Into<String>) {
        metrics::counter!("graphdb_sync_errors_total").increment(1);
    }

    pub fn record_cache_hit(&self) {
        self.cache_stats.record_hit();
    }

    pub fn record_cache_miss(&self) {
        self.cache_stats.record_miss();
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

        // Metrics are now recorded via metrics crate only
        // No internal counters to verify
    }

    #[test]
    fn test_cache_metrics() {
        let metrics = SyncMetrics::new();
        metrics.record_cache_hit();
        metrics.record_cache_miss();
        // Cache stats recorded via CacheStats
    }
}

//! Global metrics using Prometheus-style metrics crate
//!
//! Provides a unified metrics interface using the `metrics` crate for Prometheus-compatible monitoring.

use metrics::{counter, gauge, histogram, Counter, Gauge, Histogram};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Duration;

/// Global metrics instance
static GLOBAL_METRICS: OnceLock<GlobalMetrics> = OnceLock::new();

/// Global metrics for the graph database
///
/// Provides Prometheus-style metrics for monitoring:
/// - Query execution metrics
/// - Storage operation metrics
/// - Executor performance metrics
pub struct GlobalMetrics {
    query_total: Counter,
    query_total_count: AtomicU64,
    query_duration: Histogram,
    query_active: Gauge,

    query_match_total: Counter,
    query_match_count: AtomicU64,
    query_create_total: Counter,
    query_create_count: AtomicU64,
    query_update_total: Counter,
    query_update_count: AtomicU64,
    query_delete_total: Counter,
    query_delete_count: AtomicU64,
    query_insert_total: Counter,
    query_insert_count: AtomicU64,
    query_go_total: Counter,
    query_go_count: AtomicU64,
    query_fetch_total: Counter,
    query_fetch_count: AtomicU64,
    query_lookup_total: Counter,
    query_lookup_count: AtomicU64,
    query_show_total: Counter,
    query_show_count: AtomicU64,

    storage_scan_total: Counter,
    storage_scan_count: AtomicU64,
    storage_scan_duration: Histogram,
    storage_cache_hits: Counter,
    storage_cache_hit_count: AtomicU64,
    storage_cache_misses: Counter,
    storage_cache_miss_count: AtomicU64,

    executor_rows_processed: Counter,
    executor_rows_count: AtomicU64,
    executor_memory_used: Gauge,

    error_total: Counter,
    error_count: AtomicU64,
    error_by_type: Counter,
    error_by_phase: Counter,

    slow_query_total: Counter,
    slow_query_count: AtomicU64,
    slow_query_duration: Histogram,
    slow_query_active: Gauge,
    slow_query_errors: Counter,
    slow_query_error_count: AtomicU64,
}

impl GlobalMetrics {
    /// Create a new global metrics instance
    pub fn new() -> Self {
        Self {
            query_total: counter!("graphdb_query_total"),
            query_total_count: AtomicU64::new(0),
            query_duration: histogram!("graphdb_query_duration_seconds"),
            query_active: gauge!("graphdb_query_active"),

            query_match_total: counter!("graphdb_query_match_total"),
            query_match_count: AtomicU64::new(0),
            query_create_total: counter!("graphdb_query_create_total"),
            query_create_count: AtomicU64::new(0),
            query_update_total: counter!("graphdb_query_update_total"),
            query_update_count: AtomicU64::new(0),
            query_delete_total: counter!("graphdb_query_delete_total"),
            query_delete_count: AtomicU64::new(0),
            query_insert_total: counter!("graphdb_query_insert_total"),
            query_insert_count: AtomicU64::new(0),
            query_go_total: counter!("graphdb_query_go_total"),
            query_go_count: AtomicU64::new(0),
            query_fetch_total: counter!("graphdb_query_fetch_total"),
            query_fetch_count: AtomicU64::new(0),
            query_lookup_total: counter!("graphdb_query_lookup_total"),
            query_lookup_count: AtomicU64::new(0),
            query_show_total: counter!("graphdb_query_show_total"),
            query_show_count: AtomicU64::new(0),

            storage_scan_total: counter!("graphdb_storage_scan_total"),
            storage_scan_count: AtomicU64::new(0),
            storage_scan_duration: histogram!("graphdb_storage_scan_duration_seconds"),
            storage_cache_hits: counter!("graphdb_storage_cache_hits_total"),
            storage_cache_hit_count: AtomicU64::new(0),
            storage_cache_misses: counter!("graphdb_storage_cache_misses_total"),
            storage_cache_miss_count: AtomicU64::new(0),

            executor_rows_processed: counter!("graphdb_executor_rows_processed_total"),
            executor_rows_count: AtomicU64::new(0),
            executor_memory_used: gauge!("graphdb_executor_memory_used_bytes"),

            error_total: counter!("graphdb_error_total"),
            error_count: AtomicU64::new(0),
            error_by_type: counter!("graphdb_error_by_type_total"),
            error_by_phase: counter!("graphdb_error_by_phase_total"),

            slow_query_total: counter!("graphdb_slow_query_total"),
            slow_query_count: AtomicU64::new(0),
            slow_query_duration: histogram!("graphdb_slow_query_duration_seconds"),
            slow_query_active: gauge!("graphdb_slow_query_active"),
            slow_query_errors: counter!("graphdb_slow_query_errors_total"),
            slow_query_error_count: AtomicU64::new(0),
        }
    }

    /// Get the global metrics instance
    pub fn global() -> &'static GlobalMetrics {
        GLOBAL_METRICS.get_or_init(Self::new)
    }

    /// Record a query execution
    pub fn record_query(&self, duration: Duration) {
        self.query_total.increment(1);
        self.query_total_count.fetch_add(1, Ordering::Relaxed);
        self.query_duration.record(duration.as_secs_f64());
    }

    /// Increment active query count
    pub fn query_started(&self) {
        self.query_active.increment(1.0);
    }

    /// Decrement active query count
    pub fn query_completed(&self) {
        self.query_active.decrement(1.0);
    }

    /// Record query by type
    pub fn record_query_type(&self, query_type: &str) {
        match query_type.to_lowercase().as_str() {
            "match" => {
                self.query_match_total.increment(1);
                self.query_match_count.fetch_add(1, Ordering::Relaxed);
            }
            "create" => {
                self.query_create_total.increment(1);
                self.query_create_count.fetch_add(1, Ordering::Relaxed);
            }
            "update" => {
                self.query_update_total.increment(1);
                self.query_update_count.fetch_add(1, Ordering::Relaxed);
            }
            "delete" => {
                self.query_delete_total.increment(1);
                self.query_delete_count.fetch_add(1, Ordering::Relaxed);
            }
            "insert" => {
                self.query_insert_total.increment(1);
                self.query_insert_count.fetch_add(1, Ordering::Relaxed);
            }
            "go" => {
                self.query_go_total.increment(1);
                self.query_go_count.fetch_add(1, Ordering::Relaxed);
            }
            "fetch" => {
                self.query_fetch_total.increment(1);
                self.query_fetch_count.fetch_add(1, Ordering::Relaxed);
            }
            "lookup" => {
                self.query_lookup_total.increment(1);
                self.query_lookup_count.fetch_add(1, Ordering::Relaxed);
            }
            "show" => {
                self.query_show_total.increment(1);
                self.query_show_count.fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }
    }

    /// Record storage scan operation
    pub fn record_storage_scan(&self, duration: Duration) {
        self.storage_scan_total.increment(1);
        self.storage_scan_count.fetch_add(1, Ordering::Relaxed);
        self.storage_scan_duration.record(duration.as_secs_f64());
    }

    /// Record cache hit
    pub fn record_cache_hit(&self) {
        self.storage_cache_hits.increment(1);
        self.storage_cache_hit_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Record cache miss
    pub fn record_cache_miss(&self) {
        self.storage_cache_misses.increment(1);
        self.storage_cache_miss_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Record rows processed by executor
    pub fn record_rows_processed(&self, count: u64) {
        self.executor_rows_processed.increment(count);
        self.executor_rows_count.fetch_add(count, Ordering::Relaxed);
    }

    /// Update executor memory usage
    pub fn update_memory_used(&self, bytes: i64) {
        if bytes >= 0 {
            self.executor_memory_used.increment(bytes as f64);
        } else {
            self.executor_memory_used.decrement(bytes.abs() as f64);
        }
    }

    /// Record an error
    pub fn record_error(&self, error_type: &str, error_phase: &str) {
        self.error_total.increment(1);
        self.error_count.fetch_add(1, Ordering::Relaxed);
        self.error_by_type.increment(1);
        self.error_by_phase.increment(1);

        // Record with labels
        metrics::counter!("graphdb_error_by_type_total", "type" => error_type.to_string())
            .increment(1);
        metrics::counter!("graphdb_error_by_phase_total", "phase" => error_phase.to_string())
            .increment(1);
    }

    /// Record a slow query error
    pub fn record_slow_query_error(&self, error_type: &str, error_phase: &str) {
        self.slow_query_errors.increment(1);
        self.slow_query_error_count.fetch_add(1, Ordering::Relaxed);
        metrics::counter!("graphdb_slow_query_error_total",
            "type" => error_type.to_string(),
            "phase" => error_phase.to_string()
        )
        .increment(1);
    }

    /// Record a slow query
    pub fn record_slow_query(&self, duration_secs: f64) {
        self.slow_query_total.increment(1);
        self.slow_query_count.fetch_add(1, Ordering::Relaxed);
        self.slow_query_duration.record(duration_secs);
    }

    /// Increment slow query active count
    pub fn slow_query_started(&self) {
        self.slow_query_active.increment(1.0);
    }

    /// Decrement slow query active count
    pub fn slow_query_completed(&self) {
        self.slow_query_active.decrement(1.0);
    }

    /// Get total query count
    pub fn get_query_count(&self) -> u64 {
        self.query_total_count.load(Ordering::Relaxed)
    }

    /// Get query count by type
    pub fn get_query_count_by_type(&self, query_type: &str) -> u64 {
        match query_type.to_lowercase().as_str() {
            "match" => self.query_match_count.load(Ordering::Relaxed),
            "create" => self.query_create_count.load(Ordering::Relaxed),
            "update" => self.query_update_count.load(Ordering::Relaxed),
            "delete" => self.query_delete_count.load(Ordering::Relaxed),
            "insert" => self.query_insert_count.load(Ordering::Relaxed),
            "go" => self.query_go_count.load(Ordering::Relaxed),
            "fetch" => self.query_fetch_count.load(Ordering::Relaxed),
            "lookup" => self.query_lookup_count.load(Ordering::Relaxed),
            "show" => self.query_show_count.load(Ordering::Relaxed),
            _ => 0,
        }
    }

    /// Get storage scan count
    pub fn get_storage_scan_count(&self) -> u64 {
        self.storage_scan_count.load(Ordering::Relaxed)
    }

    /// Get cache hit count
    pub fn get_cache_hit_count(&self) -> u64 {
        self.storage_cache_hit_count.load(Ordering::Relaxed)
    }

    /// Get cache miss count
    pub fn get_cache_miss_count(&self) -> u64 {
        self.storage_cache_miss_count.load(Ordering::Relaxed)
    }

    /// Get cache hit rate
    pub fn get_cache_hit_rate(&self) -> f64 {
        let hits = self.storage_cache_hit_count.load(Ordering::Relaxed);
        let misses = self.storage_cache_miss_count.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// Get rows processed count
    pub fn get_rows_processed_count(&self) -> u64 {
        self.executor_rows_count.load(Ordering::Relaxed)
    }

    /// Get error count
    pub fn get_error_count(&self) -> u64 {
        self.error_count.load(Ordering::Relaxed)
    }

    /// Get slow query count
    pub fn get_slow_query_count(&self) -> u64 {
        self.slow_query_count.load(Ordering::Relaxed)
    }

    /// Get slow query error count
    pub fn get_slow_query_error_count(&self) -> u64 {
        self.slow_query_error_count.load(Ordering::Relaxed)
    }
}

impl Default for GlobalMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to get global metrics
pub fn metrics() -> &'static GlobalMetrics {
    GlobalMetrics::global()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_metrics_creation() {
        let _metrics = GlobalMetrics::new();
        // Just verify it doesn't panic
    }

    #[test]
    fn test_global_metrics_singleton() {
        let m1 = GlobalMetrics::global();
        let m2 = GlobalMetrics::global();
        // Both should point to the same instance
        assert!(std::ptr::eq(m1, m2));
    }

    #[test]
    fn test_record_query() {
        let metrics = GlobalMetrics::new();
        metrics.record_query(Duration::from_millis(100));
        // Verify no panic
    }

    #[test]
    fn test_query_lifecycle() {
        let metrics = GlobalMetrics::new();
        metrics.query_started();
        metrics.record_query(Duration::from_millis(50));
        metrics.query_completed();
        // Verify no panic
    }

    #[test]
    fn test_record_query_types() {
        let metrics = GlobalMetrics::new();
        metrics.record_query_type("MATCH");
        metrics.record_query_type("create");
        metrics.record_query_type("UPDATE");
        metrics.record_query_type("unknown");
        // Verify no panic
    }

    #[test]
    fn test_storage_metrics() {
        let metrics = GlobalMetrics::new();
        metrics.record_storage_scan(Duration::from_micros(100));
        metrics.record_cache_hit();
        metrics.record_cache_miss();
        // Verify no panic
    }

    #[test]
    fn test_executor_metrics() {
        let metrics = GlobalMetrics::new();
        metrics.record_rows_processed(100);
        metrics.update_memory_used(1024);
        metrics.update_memory_used(-512);
        // Verify no panic
    }

    #[test]
    fn test_error_metrics() {
        let metrics = GlobalMetrics::new();
        metrics.record_error("parse_error", "parse");
        metrics.record_error("execution_error", "execute");
        // Verify no panic
    }
}

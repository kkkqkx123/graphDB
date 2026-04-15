//! Global metrics using Prometheus-style metrics crate
//!
//! Provides a unified metrics interface using the `metrics` crate for Prometheus-compatible monitoring.

use metrics::{counter, gauge, histogram, Counter, Gauge, Histogram};
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
    // Query metrics
    query_total: Counter,
    query_duration: Histogram,
    query_active: Gauge,

    // Query type counters
    query_match_total: Counter,
    query_create_total: Counter,
    query_update_total: Counter,
    query_delete_total: Counter,
    query_insert_total: Counter,
    query_go_total: Counter,
    query_fetch_total: Counter,
    query_lookup_total: Counter,
    query_show_total: Counter,

    // Storage metrics
    storage_scan_total: Counter,
    storage_scan_duration: Histogram,
    storage_cache_hits: Counter,
    storage_cache_misses: Counter,

    // Executor metrics
    executor_rows_processed: Counter,
    executor_memory_used: Gauge,

    // Error metrics
    error_total: Counter,
}

impl GlobalMetrics {
    /// Create a new global metrics instance
    pub fn new() -> Self {
        Self {
            query_total: counter!("graphdb_query_total"),
            query_duration: histogram!("graphdb_query_duration_seconds"),
            query_active: gauge!("graphdb_query_active"),

            query_match_total: counter!("graphdb_query_match_total"),
            query_create_total: counter!("graphdb_query_create_total"),
            query_update_total: counter!("graphdb_query_update_total"),
            query_delete_total: counter!("graphdb_query_delete_total"),
            query_insert_total: counter!("graphdb_query_insert_total"),
            query_go_total: counter!("graphdb_query_go_total"),
            query_fetch_total: counter!("graphdb_query_fetch_total"),
            query_lookup_total: counter!("graphdb_query_lookup_total"),
            query_show_total: counter!("graphdb_query_show_total"),

            storage_scan_total: counter!("graphdb_storage_scan_total"),
            storage_scan_duration: histogram!("graphdb_storage_scan_duration_seconds"),
            storage_cache_hits: counter!("graphdb_storage_cache_hits_total"),
            storage_cache_misses: counter!("graphdb_storage_cache_misses_total"),

            executor_rows_processed: counter!("graphdb_executor_rows_processed_total"),
            executor_memory_used: gauge!("graphdb_executor_memory_used_bytes"),

            error_total: counter!("graphdb_error_total"),
        }
    }

    /// Get the global metrics instance
    pub fn global() -> &'static GlobalMetrics {
        GLOBAL_METRICS.get_or_init(Self::new)
    }

    /// Record a query execution
    pub fn record_query(&self, duration: Duration) {
        self.query_total.increment(1);
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
            "match" => self.query_match_total.increment(1),
            "create" => self.query_create_total.increment(1),
            "update" => self.query_update_total.increment(1),
            "delete" => self.query_delete_total.increment(1),
            "insert" => self.query_insert_total.increment(1),
            "go" => self.query_go_total.increment(1),
            "fetch" => self.query_fetch_total.increment(1),
            "lookup" => self.query_lookup_total.increment(1),
            "show" => self.query_show_total.increment(1),
            _ => {}
        }
    }

    /// Record storage scan operation
    pub fn record_storage_scan(&self, duration: Duration) {
        self.storage_scan_total.increment(1);
        self.storage_scan_duration.record(duration.as_secs_f64());
    }

    /// Record cache hit
    pub fn record_cache_hit(&self) {
        self.storage_cache_hits.increment(1);
    }

    /// Record cache miss
    pub fn record_cache_miss(&self) {
        self.storage_cache_misses.increment(1);
    }

    /// Record rows processed by executor
    pub fn record_rows_processed(&self, count: u64) {
        self.executor_rows_processed.increment(count);
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
    pub fn record_error(&self, error_type: &str) {
        self.error_total.increment(1);
        // Also increment a labeled counter for error type
        metrics::counter!("graphdb_error_by_type_total", "type" => error_type.to_string())
            .increment(1);
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
        metrics.record_error("parse_error");
        metrics.record_error("execution_error");
        // Verify no panic
    }
}

//! Storage Metrics Collector
//!
//! Collect performance metrics of the storage engine, including iterator statistics, cache hit rates, and more.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::core::stats::CacheMetrics;

/// Storage metric snapshot
#[derive(Debug, Clone, Default)]
pub struct StorageMetricsSnapshot {
    /// Number of items scanned
    pub items_scanned: u64,
    /// Number of items returned
    pub items_returned: u64,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of cache misses
    pub cache_misses: u64,
    /// Count of each type of operation
    pub operation_counts: HashMap<String, u64>,
}

impl CacheMetrics for StorageMetricsSnapshot {
    fn cache_hits(&self) -> u64 {
        self.cache_hits
    }

    fn cache_misses(&self) -> u64 {
        self.cache_misses
    }
}

impl StorageMetricsSnapshot {
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate the scanning efficiency (number of results returned / number of scans performed)
    pub fn scan_efficiency(&self) -> f64 {
        if self.items_scanned > 0 {
            self.items_returned as f64 / self.items_scanned as f64
        } else {
            0.0
        }
    }
}

/// Storage Metrics Collector
#[derive(Debug)]
pub struct StorageMetricsCollector {
    /// Number of items scanned
    items_scanned: AtomicU64,
    /// Number of items returned
    items_returned: AtomicU64,
    /// Number of cache hits
    cache_hits: AtomicU64,
    /// Number of cache misses
    cache_misses: AtomicU64,
    /// Count of each operation type
    operation_counts: dashmap::DashMap<String, AtomicU64>,
}

impl StorageMetricsCollector {
    pub fn new() -> Self {
        Self {
            items_scanned: AtomicU64::new(0),
            items_returned: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            operation_counts: dashmap::DashMap::new(),
        }
    }

    /// Record the scanning operations
    pub fn record_scan(&self, count: u64) {
        self.items_scanned.fetch_add(count, Ordering::Relaxed);
    }

    /// Record the return operation
    pub fn record_return(&self, count: u64) {
        self.items_returned.fetch_add(count, Ordering::Relaxed);
    }

    /// Record of cache hits
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record of a cache miss.
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record the operation
    pub fn record_operation(&self, operation: &str) {
        let counter = self
            .operation_counts
            .entry(operation.to_string())
            .or_insert_with(|| AtomicU64::new(0));
        counter.fetch_add(1, Ordering::Relaxed);
    }

    /// Obtain a snapshot of the current metrics.
    pub fn snapshot(&self) -> StorageMetricsSnapshot {
        let mut operation_counts = HashMap::new();
        for entry in self.operation_counts.iter() {
            operation_counts.insert(entry.key().clone(), entry.value().load(Ordering::Relaxed));
        }

        StorageMetricsSnapshot {
            items_scanned: self.items_scanned.load(Ordering::Relaxed),
            items_returned: self.items_returned.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            operation_counts,
        }
    }

    /// Reset all indicators
    pub fn reset(&self) {
        self.items_scanned.store(0, Ordering::Relaxed);
        self.items_returned.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
        self.operation_counts.clear();
    }
}

impl Default for StorageMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_metrics_collector() {
        let collector = StorageMetricsCollector::new();

        collector.record_scan(100);
        collector.record_return(50);
        collector.record_cache_hit();
        collector.record_cache_hit();
        collector.record_cache_miss();
        collector.record_operation("scan_vertices");
        collector.record_operation("scan_vertices");

        let snapshot = collector.snapshot();

        assert_eq!(snapshot.items_scanned, 100);
        assert_eq!(snapshot.items_returned, 50);
        assert_eq!(snapshot.cache_hits, 2);
        assert_eq!(snapshot.cache_misses, 1);
        assert_eq!(snapshot.operation_counts.get("scan_vertices"), Some(&2));
        assert!((snapshot.cache_hit_rate() - 0.666).abs() < 0.01);
        assert_eq!(snapshot.scan_efficiency(), 0.5);
    }

    #[test]
    fn test_reset() {
        let collector = StorageMetricsCollector::new();

        collector.record_scan(100);
        collector.record_cache_hit();

        collector.reset();

        let snapshot = collector.snapshot();
        assert_eq!(snapshot.items_scanned, 0);
        assert_eq!(snapshot.cache_hits, 0);
    }
}

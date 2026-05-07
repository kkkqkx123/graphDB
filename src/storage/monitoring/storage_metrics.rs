//! Storage Metrics Collector
//!
//! Collect performance metrics of the storage engine, including iterator statistics, cache hit rates, and more.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::core::stats::CacheStats;
use crate::storage::cache::RecordCacheStats;

/// Storage metric snapshot
#[derive(Debug, Clone, Default)]
pub struct StorageMetricsSnapshot {
    /// Number of items scanned
    pub items_scanned: u64,
    /// Number of items returned
    pub items_returned: u64,
    /// Cache hit rate (calculated)
    pub cache_hit_rate: f64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Vertex cache count
    pub vertex_count: u64,
    /// Edge cache count
    pub edge_count: u64,
    /// Edge query cache count
    pub edge_query_count: u64,
    /// ID index cache count
    pub id_index_count: u64,
    /// Cache memory usage
    pub cache_memory_usage: usize,
    /// Cache max memory
    pub cache_max_memory: usize,
    /// Count of each type of operation
    pub operation_counts: HashMap<String, u64>,
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
    /// Cache statistics using unified CacheStats
    cache_stats: CacheStats,
    /// Count of each operation type
    operation_counts: dashmap::DashMap<String, AtomicU64>,
}

impl StorageMetricsCollector {
    pub fn new() -> Self {
        Self {
            items_scanned: AtomicU64::new(0),
            items_returned: AtomicU64::new(0),
            cache_stats: CacheStats::new(),
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
        self.cache_stats.record_hit();
    }

    /// Record of a cache miss.
    pub fn record_cache_miss(&self) {
        self.cache_stats.record_miss();
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
            cache_hit_rate: self.cache_stats.hit_rate(),
            cache_hits: self.cache_stats.hits(),
            cache_misses: self.cache_stats.misses(),
            vertex_count: 0,
            edge_count: 0,
            edge_query_count: 0,
            id_index_count: 0,
            cache_memory_usage: 0,
            cache_max_memory: 0,
            operation_counts,
        }
    }

    /// Obtain a snapshot with RecordCache statistics.
    pub fn snapshot_with_cache_stats(&self, cache_stats: &RecordCacheStats) -> StorageMetricsSnapshot {
        let mut snapshot = self.snapshot();
        snapshot.cache_hit_rate = cache_stats.hit_rate;
        snapshot.cache_hits = cache_stats.total_hits;
        snapshot.cache_misses = cache_stats.total_misses;
        snapshot.vertex_count = cache_stats.vertex.count;
        snapshot.edge_query_count = cache_stats.edge_query.count;
        snapshot.id_index_count = cache_stats.id_index.count;
        snapshot.cache_memory_usage = cache_stats.memory_usage;
        snapshot.cache_max_memory = cache_stats.max_memory;
        snapshot
    }

    /// Sync statistics from RecordCacheStats.
    pub fn sync_from_record_cache(&self, cache_stats: &RecordCacheStats) {
        self.cache_stats.record_hits(cache_stats.total_hits);
        self.cache_stats.record_misses(cache_stats.total_misses);
    }

    /// Reset all indicators
    pub fn reset(&self) {
        self.items_scanned.store(0, Ordering::Relaxed);
        self.items_returned.store(0, Ordering::Relaxed);
        self.cache_stats.reset();
        self.operation_counts.clear();
    }

    /// Get cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        self.cache_stats.hit_rate()
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
        assert!((snapshot.cache_hit_rate - 0.666).abs() < 0.01);
        assert_eq!(snapshot.operation_counts.get("scan_vertices"), Some(&2));
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
        assert_eq!(snapshot.cache_hit_rate, 0.0);
    }
}

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use crate::core::stats::CacheMetrics;

/// Fulltext search metrics
pub struct FulltextMetrics {
    /// Index operation counter
    index_ops: AtomicU64,
    /// Search operation counter
    search_ops: AtomicU64,
    /// Total search latency in microseconds
    search_latency_us: AtomicU64,
    /// Indexed document count
    indexed_docs: AtomicU64,
    /// Queue size
    queue_size: AtomicU64,
    /// Cache hit counter
    cache_hits: AtomicU64,
    /// Cache miss counter
    cache_misses: AtomicU64,
}

impl CacheMetrics for FulltextMetrics {
    fn cache_hits(&self) -> u64 {
        self.cache_hits.load(Ordering::Relaxed)
    }

    fn cache_misses(&self) -> u64 {
        self.cache_misses.load(Ordering::Relaxed)
    }
}

impl FulltextMetrics {
    pub fn new() -> Self {
        Self {
            index_ops: AtomicU64::new(0),
            search_ops: AtomicU64::new(0),
            search_latency_us: AtomicU64::new(0),
            indexed_docs: AtomicU64::new(0),
            queue_size: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
        }
    }

    /// Record index operation
    pub fn record_index(&self, count: usize) {
        self.index_ops.fetch_add(count as u64, Ordering::Relaxed);
        self.indexed_docs.fetch_add(count as u64, Ordering::Relaxed);
    }

    /// Record search operation
    pub fn record_search(&self, latency: Duration) {
        self.search_ops.fetch_add(1, Ordering::Relaxed);
        self.search_latency_us
            .fetch_add(latency.as_micros() as u64, Ordering::Relaxed);
    }

    /// Set queue size
    pub fn set_queue_size(&self, size: usize) {
        self.queue_size.store(size as u64, Ordering::Relaxed);
    }

    /// Record cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Get index operation count
    pub fn index_ops(&self) -> u64 {
        self.index_ops.load(Ordering::Relaxed)
    }

    /// Get search operation count
    pub fn search_ops(&self) -> u64 {
        self.search_ops.load(Ordering::Relaxed)
    }

    /// Get average search latency in milliseconds
    pub fn avg_search_latency_ms(&self) -> f64 {
        let ops = self.search_ops.load(Ordering::Relaxed);
        if ops == 0 {
            0.0
        } else {
            let total_us = self.search_latency_us.load(Ordering::Relaxed);
            (total_us as f64 / ops as f64) / 1000.0
        }
    }

    /// Get indexed document count
    pub fn indexed_docs(&self) -> u64 {
        self.indexed_docs.load(Ordering::Relaxed)
    }

    /// Get current queue size
    pub fn queue_size(&self) -> u64 {
        self.queue_size.load(Ordering::Relaxed)
    }

    /// Get all metrics as a formatted string
    pub fn report(&self) -> String {
        format!(
            "Fulltext Metrics:\n\
             - Index operations: {}\n\
             - Search operations: {}\n\
             - Avg search latency: {:.2} ms\n\
             - Indexed documents: {}\n\
             - Queue size: {}\n\
             - Cache hit rate: {:.2}%",
            self.index_ops(),
            self.search_ops(),
            self.avg_search_latency_ms(),
            self.indexed_docs(),
            self.queue_size(),
            self.cache_hit_rate() * 100.0
        )
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.index_ops.store(0, Ordering::Relaxed);
        self.search_ops.store(0, Ordering::Relaxed);
        self.search_latency_us.store(0, Ordering::Relaxed);
        self.indexed_docs.store(0, Ordering::Relaxed);
        self.queue_size.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
    }
}

impl Default for FulltextMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_basic_operations() {
        let metrics = FulltextMetrics::new();

        // Initially all zero
        assert_eq!(metrics.index_ops(), 0);
        assert_eq!(metrics.search_ops(), 0);
        assert_eq!(metrics.indexed_docs(), 0);

        // Record operations
        metrics.record_index(10);
        assert_eq!(metrics.index_ops(), 10);
        assert_eq!(metrics.indexed_docs(), 10);

        metrics.record_search(Duration::from_millis(5));
        metrics.record_search(Duration::from_millis(15));
        assert_eq!(metrics.search_ops(), 2);
        assert_eq!(metrics.avg_search_latency_ms(), 10.0);
    }

    #[test]
    fn test_metrics_cache_stats() {
        let metrics = FulltextMetrics::new();

        // Initially zero hit rate
        assert_eq!(metrics.cache_hit_rate(), 0.0);

        // Record hits and misses
        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_miss();

        assert_eq!(metrics.cache_hits(), 2);
        assert_eq!(metrics.cache_misses(), 1);
        assert!((metrics.cache_hit_rate() - 0.6667).abs() < 0.01);
    }

    #[test]
    fn test_metrics_queue_size() {
        let metrics = FulltextMetrics::new();

        metrics.set_queue_size(100);
        assert_eq!(metrics.queue_size(), 100);

        metrics.set_queue_size(50);
        assert_eq!(metrics.queue_size(), 50);
    }

    #[test]
    fn test_metrics_reset() {
        let metrics = FulltextMetrics::new();

        metrics.record_index(100);
        metrics.record_search(Duration::from_millis(10));
        metrics.record_cache_hit();
        metrics.set_queue_size(50);

        metrics.reset();

        assert_eq!(metrics.index_ops(), 0);
        assert_eq!(metrics.search_ops(), 0);
        assert_eq!(metrics.indexed_docs(), 0);
        assert_eq!(metrics.cache_hits(), 0);
        assert_eq!(metrics.queue_size(), 0);
    }

    #[test]
    fn test_metrics_report() {
        let metrics = FulltextMetrics::new();

        metrics.record_index(100);
        metrics.record_search(Duration::from_millis(10));
        metrics.record_cache_hit();

        let report = metrics.report();
        assert!(report.contains("Index operations: 100"));
        assert!(report.contains("Search operations: 1"));
        assert!(report.contains("Cache hit rate:"));
    }
}

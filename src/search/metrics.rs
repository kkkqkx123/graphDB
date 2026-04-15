use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use crate::core::stats::CacheMetrics;

/// Internal counters for FulltextMetrics (maintained for backward compatibility)
#[derive(Debug, Default)]
struct InternalCounters {
    index_ops: AtomicU64,
    search_ops: AtomicU64,
    search_latency_us: AtomicU64,
    indexed_docs: AtomicU64,
    queue_size: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
}

/// Fulltext search metrics using metrics crate
pub struct FulltextMetrics {
    counters: InternalCounters,
}

impl CacheMetrics for FulltextMetrics {
    fn cache_hits(&self) -> u64 {
        self.counters.cache_hits.load(Ordering::Relaxed)
    }

    fn cache_misses(&self) -> u64 {
        self.counters.cache_misses.load(Ordering::Relaxed)
    }
}

impl FulltextMetrics {
    pub fn new() -> Self {
        Self {
            counters: InternalCounters::default(),
        }
    }

    pub fn record_index(&self, count: usize) {
        metrics::counter!("graphdb_fulltext_index_ops_total").increment(count as u64);
        metrics::counter!("graphdb_fulltext_indexed_docs_total").increment(count as u64);
        self.counters
            .index_ops
            .fetch_add(count as u64, Ordering::Relaxed);
        self.counters
            .indexed_docs
            .fetch_add(count as u64, Ordering::Relaxed);
    }

    pub fn record_search(&self, latency: Duration) {
        metrics::counter!("graphdb_fulltext_search_ops_total").increment(1);
        metrics::histogram!("graphdb_fulltext_search_duration_seconds")
            .record(latency.as_secs_f64());
        self.counters.search_ops.fetch_add(1, Ordering::Relaxed);
        self.counters
            .search_latency_us
            .fetch_add(latency.as_micros() as u64, Ordering::Relaxed);
    }

    pub fn set_queue_size(&self, size: usize) {
        metrics::gauge!("graphdb_fulltext_queue_size").set(size as f64);
        self.counters
            .queue_size
            .store(size as u64, Ordering::Relaxed);
    }

    pub fn record_cache_hit(&self) {
        metrics::counter!("graphdb_fulltext_cache_hits_total").increment(1);
        self.counters.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_cache_miss(&self) {
        metrics::counter!("graphdb_fulltext_cache_misses_total").increment(1);
        self.counters.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn index_ops(&self) -> u64 {
        self.counters.index_ops.load(Ordering::Relaxed)
    }

    pub fn search_ops(&self) -> u64 {
        self.counters.search_ops.load(Ordering::Relaxed)
    }

    pub fn avg_search_latency_ms(&self) -> f64 {
        let ops = self.counters.search_ops.load(Ordering::Relaxed);
        if ops == 0 {
            0.0
        } else {
            let total_us = self.counters.search_latency_us.load(Ordering::Relaxed);
            (total_us as f64 / ops as f64) / 1000.0
        }
    }

    pub fn indexed_docs(&self) -> u64 {
        self.counters.indexed_docs.load(Ordering::Relaxed)
    }

    pub fn queue_size(&self) -> u64 {
        self.counters.queue_size.load(Ordering::Relaxed)
    }

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

    pub fn reset(&self) {
        self.counters.index_ops.store(0, Ordering::Relaxed);
        self.counters.search_ops.store(0, Ordering::Relaxed);
        self.counters.search_latency_us.store(0, Ordering::Relaxed);
        self.counters.indexed_docs.store(0, Ordering::Relaxed);
        self.counters.queue_size.store(0, Ordering::Relaxed);
        self.counters.cache_hits.store(0, Ordering::Relaxed);
        self.counters.cache_misses.store(0, Ordering::Relaxed);
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

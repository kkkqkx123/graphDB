use std::time::Duration;

use crate::core::stats::CacheStats;

/// Fulltext search metrics using metrics crate
pub struct FulltextMetrics {
    cache_stats: CacheStats,
}

impl FulltextMetrics {
    pub fn new() -> Self {
        Self {
            cache_stats: CacheStats::new(),
        }
    }

    pub fn record_index(&self, count: usize) {
        metrics::counter!("graphdb_fulltext_index_ops_total").increment(count as u64);
        metrics::counter!("graphdb_fulltext_indexed_docs_total").increment(count as u64);
    }

    pub fn record_search(&self, latency: Duration) {
        metrics::counter!("graphdb_fulltext_search_ops_total").increment(1);
        metrics::histogram!("graphdb_fulltext_search_duration_seconds")
            .record(latency.as_secs_f64());
    }

    pub fn set_queue_size(&self, size: usize) {
        metrics::gauge!("graphdb_fulltext_queue_size").set(size as f64);
    }

    pub fn record_cache_hit(&self) {
        metrics::counter!("graphdb_fulltext_cache_hits_total").increment(1);
        self.cache_stats.record_hit();
    }

    pub fn record_cache_miss(&self) {
        metrics::counter!("graphdb_fulltext_cache_misses_total").increment(1);
        self.cache_stats.record_miss();
    }

    pub fn cache_hit_rate(&self) -> f64 {
        self.cache_stats.hit_rate()
    }

    pub fn reset(&self) {
        self.cache_stats.reset();
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

        // Record operations - metrics are recorded via metrics crate
        metrics.record_index(10);
        metrics.record_search(Duration::from_millis(5));
        metrics.record_search(Duration::from_millis(15));
        // No internal counters to verify
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

        assert!((metrics.cache_hit_rate() - 0.6667).abs() < 0.01);
    }

    #[test]
    fn test_metrics_queue_size() {
        let metrics = FulltextMetrics::new();

        metrics.set_queue_size(100);
        metrics.set_queue_size(50);
        // Queue size only recorded via metrics crate
    }

    #[test]
    fn test_metrics_reset() {
        let metrics = FulltextMetrics::new();

        metrics.record_index(100);
        metrics.record_search(Duration::from_millis(10));
        metrics.record_cache_hit();
        metrics.set_queue_size(50);

        metrics.reset();

        assert_eq!(metrics.cache_hit_rate(), 0.0);
    }
}

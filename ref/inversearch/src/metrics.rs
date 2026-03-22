use metrics::{counter, histogram, Counter, Histogram};

pub struct Metrics {
    pub document_add_total: Counter,
    pub document_update_total: Counter,
    pub document_remove_total: Counter,
    pub search_total: Counter,
    pub search_duration: Histogram,
    pub index_size: Histogram,
    pub cache_hits: Counter,
    pub cache_misses: Counter,
}

impl Metrics {
    pub fn new() -> Self {
        Metrics {
            document_add_total: counter!("inversearch_document_add_total"),
            document_update_total: counter!("inversearch_document_update_total"),
            document_remove_total: counter!("inversearch_document_remove_total"),
            search_total: counter!("inversearch_search_total"),
            search_duration: histogram!("inversearch_search_duration_seconds"),
            index_size: histogram!("inversearch_index_size_bytes"),
            cache_hits: counter!("inversearch_cache_hits_total"),
            cache_misses: counter!("inversearch_cache_misses_total"),
        }
    }

    pub fn record_document_add(&self) {
        self.document_add_total.increment(1);
    }

    pub fn record_document_update(&self) {
        self.document_update_total.increment(1);
    }

    pub fn record_document_remove(&self) {
        self.document_remove_total.increment(1);
    }

    pub fn record_search(&self, duration: f64) {
        self.search_total.increment(1);
        self.search_duration.record(duration);
    }

    pub fn record_index_size(&self, size: f64) {
        self.index_size.record(size);
    }

    pub fn record_cache_hit(&self) {
        self.cache_hits.increment(1);
    }

    pub fn record_cache_miss(&self) {
        self.cache_misses.increment(1);
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_new() {
        let metrics = Metrics::new();
        metrics.record_document_add();
        metrics.record_document_update();
        metrics.record_document_remove();
        metrics.record_search(0.1);
        metrics.record_index_size(1024.0);
        metrics.record_cache_hit();
        metrics.record_cache_miss();
    }
}

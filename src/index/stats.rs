use std::time::Duration;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    Exact,
    Prefix,
    Range,
}

#[derive(Debug)]
pub struct IndexQueryStats {
    exact_queries: AtomicU64,
    prefix_queries: AtomicU64,
    range_queries: AtomicU64,
    exact_hits: AtomicU64,
    prefix_hits: AtomicU64,
    range_hits: AtomicU64,
    total_query_time_us: AtomicU64,
}

impl Clone for IndexQueryStats {
    fn clone(&self) -> Self {
        Self {
            exact_queries: AtomicU64::new(self.exact_queries.load(Ordering::Relaxed)),
            prefix_queries: AtomicU64::new(self.prefix_queries.load(Ordering::Relaxed)),
            range_queries: AtomicU64::new(self.range_queries.load(Ordering::Relaxed)),
            exact_hits: AtomicU64::new(self.exact_hits.load(Ordering::Relaxed)),
            prefix_hits: AtomicU64::new(self.prefix_hits.load(Ordering::Relaxed)),
            range_hits: AtomicU64::new(self.range_hits.load(Ordering::Relaxed)),
            total_query_time_us: AtomicU64::new(self.total_query_time_us.load(Ordering::Relaxed)),
        }
    }
}

impl IndexQueryStats {
    pub fn new() -> Self {
        Self {
            exact_queries: AtomicU64::new(0),
            prefix_queries: AtomicU64::new(0),
            range_queries: AtomicU64::new(0),
            exact_hits: AtomicU64::new(0),
            prefix_hits: AtomicU64::new(0),
            range_hits: AtomicU64::new(0),
            total_query_time_us: AtomicU64::new(0),
        }
    }

    pub fn reset(&self) {
        self.exact_queries.store(0, Ordering::Relaxed);
        self.prefix_queries.store(0, Ordering::Relaxed);
        self.range_queries.store(0, Ordering::Relaxed);
        self.exact_hits.store(0, Ordering::Relaxed);
        self.prefix_hits.store(0, Ordering::Relaxed);
        self.range_hits.store(0, Ordering::Relaxed);
        self.total_query_time_us.store(0, Ordering::Relaxed);
    }

    pub fn record_query(&self, found: bool, duration: Duration, query_type: QueryType) {
        let duration_us = duration.as_micros() as u64;
        self.total_query_time_us.fetch_add(duration_us, Ordering::Relaxed);

        match query_type {
            QueryType::Exact => {
                self.exact_queries.fetch_add(1, Ordering::Relaxed);
                if found {
                    self.exact_hits.fetch_add(1, Ordering::Relaxed);
                }
            }
            QueryType::Prefix => {
                self.prefix_queries.fetch_add(1, Ordering::Relaxed);
                if found {
                    self.prefix_hits.fetch_add(1, Ordering::Relaxed);
                }
            }
            QueryType::Range => {
                self.range_queries.fetch_add(1, Ordering::Relaxed);
                if found {
                    self.range_hits.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    }

    pub fn get_exact_queries(&self) -> u64 {
        self.exact_queries.load(Ordering::Relaxed)
    }

    pub fn get_prefix_queries(&self) -> u64 {
        self.prefix_queries.load(Ordering::Relaxed)
    }

    pub fn get_range_queries(&self) -> u64 {
        self.range_queries.load(Ordering::Relaxed)
    }

    pub fn get_exact_hits(&self) -> u64 {
        self.exact_hits.load(Ordering::Relaxed)
    }

    pub fn get_prefix_hits(&self) -> u64 {
        self.prefix_hits.load(Ordering::Relaxed)
    }

    pub fn get_range_hits(&self) -> u64 {
        self.range_hits.load(Ordering::Relaxed)
    }

    pub fn get_total_queries(&self) -> u64 {
        self.exact_queries.load(Ordering::Relaxed) +
        self.prefix_queries.load(Ordering::Relaxed) +
        self.range_queries.load(Ordering::Relaxed)
    }

    pub fn get_total_hits(&self) -> u64 {
        self.exact_hits.load(Ordering::Relaxed) +
        self.prefix_hits.load(Ordering::Relaxed) +
        self.range_hits.load(Ordering::Relaxed)
    }

    pub fn get_total_query_time_us(&self) -> u64 {
        self.total_query_time_us.load(Ordering::Relaxed)
    }

    pub fn get_hit_rate(&self) -> f64 {
        let total_queries = self.get_total_queries();
        if total_queries == 0 {
            0.0
        } else {
            self.get_total_hits() as f64 / total_queries as f64
        }
    }

    pub fn get_average_query_time_us(&self) -> f64 {
        let total_queries = self.get_total_queries();
        if total_queries == 0 {
            0.0
        } else {
            self.get_total_query_time_us() as f64 / total_queries as f64
        }
    }
}

impl Default for IndexQueryStats {
    fn default() -> Self {
        Self::new()
    }
}

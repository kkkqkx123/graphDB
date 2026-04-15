//! Actuator statistics
//!
//! Used to record a variety of statistical information during actuator execution, including the number of lines processed, execution time, and so on.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::core::stats::CacheMetrics;

/// Actuator statistics
///
/// Record statistics during actuator execution for performance analysis and query optimization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutorStats {
    /// Number of rows processed
    pub num_rows: usize,
    /// Execution time (microseconds)
    pub exec_time_us: u64,
    /// Total time (microseconds)
    pub total_time_us: u64,
    /// Peak memory usage (bytes)
    pub memory_peak: usize,
    /// Memory use current value (bytes)
    pub memory_current: usize,
    /// Number of batch operations
    pub batch_count: usize,
    /// Cache hits
    pub cache_hits: usize,
    /// Number of cache misses
    pub cache_misses: usize,
    /// Other statistical information
    pub other_stats: HashMap<String, String>,
}

impl CacheMetrics for ExecutorStats {
    fn cache_hits(&self) -> u64 {
        self.cache_hits as u64
    }

    fn cache_misses(&self) -> u64 {
        self.cache_misses as u64
    }
}

impl ExecutorStats {
    /// Creating a New Statistical Information Instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Increase the number of rows processed
    pub fn add_row(&mut self, count: usize) {
        self.num_rows += count;
    }

    /// Increase in execution time
    pub fn add_exec_time(&mut self, duration: Duration) {
        self.exec_time_us += duration.as_micros() as u64;
    }

    /// Increase in total time
    pub fn add_total_time(&mut self, duration: Duration) {
        self.total_time_us += duration.as_micros() as u64;
    }

    /// Setting peak memory usage
    pub fn set_memory_peak(&mut self, peak: usize) {
        if peak > self.memory_peak {
            self.memory_peak = peak;
        }
    }

    /// Update memory current usage
    pub fn update_memory_current(&mut self, current: usize) {
        self.memory_current = current;
    }

    /// Increase the number of batch operations
    pub fn add_batch(&mut self, count: usize) {
        self.batch_count += count;
    }

    /// Logging Cache Hits
    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    /// Record cache misses
    pub fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
    }

    /// Calculating Cache Hit Ratio
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total > 0 {
            self.cache_hits as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Adding customized statistics
    pub fn add_stat(&mut self, key: String, value: String) {
        self.other_stats.insert(key, value);
    }

    /// Getting customized statistics
    pub fn get_stat(&self, key: &str) -> Option<&String> {
        self.other_stats.get(key)
    }

    /// Get throughput (lines/sec)
    pub fn throughput_rows_per_sec(&self) -> f64 {
        if self.total_time_us > 0 {
            self.num_rows as f64 * 1_000_000.0 / self.total_time_us as f64
        } else {
            0.0
        }
    }

    /// Get execution efficiency (lines/microseconds)
    pub fn efficiency_rows_per_us(&self) -> f64 {
        if self.exec_time_us > 0 {
            self.num_rows as f64 / self.exec_time_us as f64
        } else {
            0.0
        }
    }

    /// Export as JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Import from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Export as Formatted String
    pub fn to_formatted_string(&self) -> String {
        format!(
            "rows: {}, exec_time: {}us, total_time: {}us, memory_peak: {}B, \
             memory_current: {}B, batches: {}, cache_hits: {}, cache_misses: {}, \
             cache_hit_rate: {:.2}%, throughput: {:.2} rows/sec",
            self.num_rows,
            self.exec_time_us,
            self.total_time_us,
            self.memory_peak,
            self.memory_current,
            self.batch_count,
            self.cache_hits,
            self.cache_misses,
            self.cache_hit_rate() * 100.0,
            self.throughput_rows_per_sec()
        )
    }
}

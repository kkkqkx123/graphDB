//! Attribute Statistics Information Module
//!
//! Provide statistical information at the attribute level, which is used by the query optimizer to estimate selectivity.

use std::time::Instant;

use super::histogram::Histogram;

/// Property combination statistics
///
/// 轻量级属性组合统计，用于GROUP BY基数估计
#[derive(Debug, Clone)]
pub struct PropertyCombinationStats {
    /// 属性组合键（如 "tag.prop1.prop2"）
    pub key: String,
    /// 关联的标签（如果有）
    pub tag_name: Option<String>,
    /// 属性列表
    pub properties: Vec<String>,
    /// 联合不同值数量
    pub combined_distinct_values: u64,
    /// 样本数量
    pub sample_count: u64,
    /// 最后更新时间
    pub last_updated: Instant,
}

impl PropertyCombinationStats {
    /// Create new property combination statistics.
    pub fn new(key: String, tag_name: Option<String>, properties: Vec<String>) -> Self {
        Self {
            key,
            tag_name,
            properties,
            combined_distinct_values: 0,
            sample_count: 0,
            last_updated: Instant::now(),
        }
    }

    /// Update statistics with new sample data.
    pub fn update(&mut self, distinct_values: u64, sample_count: u64) {
        // Use exponential moving average for stability
        if self.sample_count == 0 {
            self.combined_distinct_values = distinct_values;
            self.sample_count = sample_count;
        } else {
            let alpha = 0.3; // Smoothing factor
            self.combined_distinct_values = ((1.0 - alpha) * self.combined_distinct_values as f64
                + alpha * distinct_values as f64) as u64;
            self.sample_count = self.sample_count.saturating_add(sample_count);
        }
        self.last_updated = Instant::now();
    }

    /// Check if statistics are stale (older than 1 hour).
    pub fn is_stale(&self) -> bool {
        self.last_updated.elapsed().as_secs() > 3600
    }

    /// Get the estimated cardinality.
    pub fn estimated_cardinality(&self) -> u64 {
        self.combined_distinct_values.max(1)
    }
}

/// Attribute statistics information
#[derive(Debug, Clone)]
pub struct PropertyStatistics {
    /// Attribute name
    pub property_name: String,
    /// Associated Tags (optional)
    pub tag_name: Option<String>,
    /// Number of different values
    pub distinct_values: u64,
    /// Optional histograms (enabled for attributes with a high cardinality)
    pub histogram: Option<Histogram>,
    /// Is it appropriate to use a histogram? (Histograms are not necessary for attributes with a low cardinality.)
    pub use_histogram: bool,
}

impl PropertyStatistics {
    /// Create new attribute statistics information.
    pub fn new(property_name: String, tag_name: Option<String>) -> Self {
        Self {
            property_name,
            tag_name,
            distinct_values: 0,
            histogram: None,
            use_histogram: false,
        }
    }

    /// Setting up a histogram
    pub fn with_histogram(mut self, histogram: Histogram) -> Self {
        self.histogram = Some(histogram);
        self.use_histogram = true;
        self
    }

    /// Determine whether to use a histogram.
    pub fn should_use_histogram(&self) -> bool {
        self.use_histogram && self.histogram.is_some()
    }
}

impl Default for PropertyStatistics {
    fn default() -> Self {
        Self::new(String::new(), None)
    }
}

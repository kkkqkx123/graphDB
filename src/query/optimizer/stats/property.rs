//! Attribute Statistics Information Module
//!
//! Provide statistical information at the attribute level, which is used by the query optimizer to estimate selectivity.

use super::histogram::Histogram;

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

//! Embedded Telemetry API
//!
//! Provides access to telemetry metrics data for embedded mode.
//! This allows embedded applications to retrieve metrics without HTTP server.
//!
//! ## Example
//!
//! ```rust
//! use graphdb::api::embedded::telemetry::EmbeddedTelemetry;
//!
//! # fn example() {
//! // Get current metrics snapshot
//! let snapshot = EmbeddedTelemetry::get_metrics();
//! println!("Total queries: {:?}", snapshot.counters.iter().find(|(k, _)| k == "graphdb_query_total"));
//! # }
//! ```

use crate::api::core::telemetry::{global_recorder, HistogramData, MetricsSnapshot};
use crate::core::stats::GlobalMetrics;

/// Embedded telemetry accessor
///
/// Provides programmatic access to metrics data for embedded applications.
pub struct EmbeddedTelemetry;

impl EmbeddedTelemetry {
    /// Get a snapshot of current metrics
    ///
    /// Returns all recorded metrics (counters, gauges, histograms) at the current moment.
    pub fn get_metrics() -> MetricsSnapshot {
        global_recorder()
            .map(|r| r.get_snapshot())
            .unwrap_or_default()
    }

    /// Get metrics filtered by name prefix
    ///
    /// # Arguments
    ///
    /// * `prefix` - The prefix to filter metric names
    pub fn get_metrics_filtered(prefix: &str) -> MetricsSnapshot {
        let snapshot = Self::get_metrics();
        snapshot.filter_by_prefix(prefix)
    }

    /// Get a specific counter value
    ///
    /// # Arguments
    ///
    /// * `name` - The counter name
    pub fn get_counter(name: &str) -> Option<u64> {
        global_recorder().and_then(|r| r.get_counter(name))
    }

    /// Get a specific gauge value
    ///
    /// # Arguments
    ///
    /// * `name` - The gauge name
    pub fn get_gauge(name: &str) -> Option<f64> {
        global_recorder().and_then(|r| r.get_gauge(name))
    }

    /// Get histogram data
    ///
    /// # Arguments
    ///
    /// * `name` - The histogram name
    pub fn get_histogram(name: &str) -> Option<HistogramData> {
        global_recorder().and_then(|r| r.get_histogram(name))
    }

    /// Get global metrics instance
    ///
    /// Provides access to high-level business metrics such as query counts, storage stats, etc.
    pub fn get_global_metrics() -> &'static GlobalMetrics {
        GlobalMetrics::global()
    }

    /// Export metrics to JSON string
    ///
    /// Returns metrics in JSON format for easy serialization.
    pub fn export_to_json() -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&Self::get_metrics())
    }

    /// Export metrics to Prometheus text format
    ///
    /// Returns metrics in Prometheus-compatible text format.
    pub fn export_to_text() -> String {
        Self::get_metrics().to_text_format()
    }

    /// Check if telemetry recorder is initialized
    pub fn is_initialized() -> bool {
        global_recorder().is_some()
    }
}

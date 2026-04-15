//! Core Telemetry Module
//!
//! Provides core telemetry functionality for metrics collection and exposure.
//! This module is transport-layer agnostic and can be used by HTTP server,
//! embedded API, or C API.
//!
//! ## Features
//!
//! - Custom metrics recorder implementing `metrics::Recorder` trait
//! - Support for counters, gauges, and histograms
//! - Efficient storage using DashMap for minimal lock contention
//! - Prometheus-style metrics export
//!
//! ## Usage
//!
//! ```rust
//! use graphdb::api::core::telemetry::{TelemetryRecorder, init_global_recorder};
//!
//! // Initialize global recorder
//! let recorder = init_global_recorder();
//!
//! // Record metrics
//! metrics::counter!("my_counter").increment(1);
//!
//! // Get snapshot
//! let snapshot = recorder.get_snapshot();
//! ```

use dashmap::DashMap;
use metrics::{
    Counter, CounterFn, Gauge, GaugeFn, Histogram, HistogramFn, Key, KeyName, Metadata, Recorder,
    SharedString, Unit,
};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Histogram data with statistical calculations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HistogramData {
    pub count: usize,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
}

impl HistogramData {
    /// Create histogram data from a slice of values
    pub fn from_values(values: &[f64]) -> Self {
        if values.is_empty() {
            return Self::default();
        }

        let count = values.len();
        let sum: f64 = values.iter().sum();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let p50 = Self::percentile(&sorted, 0.50);
        let p95 = Self::percentile(&sorted, 0.95);
        let p99 = Self::percentile(&sorted, 0.99);

        Self {
            count,
            sum,
            min,
            max,
            p50,
            p95,
            p99,
        }
    }

    /// Calculate percentile from sorted values
    fn percentile(sorted: &[f64], p: f64) -> f64 {
        if sorted.is_empty() {
            return 0.0;
        }
        let index = (p * (sorted.len() - 1) as f64).round() as usize;
        sorted[index.min(sorted.len() - 1)]
    }
}

/// Metrics snapshot for safe data transfer
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub counters: Vec<(String, u64)>,
    pub gauges: Vec<(String, f64)>,
    pub histograms: Vec<(String, HistogramData)>,
    pub timestamp: u64,
}

impl MetricsSnapshot {
    /// Create a new metrics snapshot with current timestamp
    pub fn new() -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            ..Default::default()
        }
    }

    /// Convert to Prometheus-style text format
    pub fn to_text_format(&self) -> String {
        let mut output = String::new();

        // Counters
        for (name, value) in &self.counters {
            output.push_str(&format!("# TYPE {} counter\n", name));
            output.push_str(&format!("{} {}\n", name, value));
        }

        // Gauges
        for (name, value) in &self.gauges {
            output.push_str(&format!("# TYPE {} gauge\n", name));
            output.push_str(&format!("{} {}\n", name, value));
        }

        // Histograms
        for (name, data) in &self.histograms {
            output.push_str(&format!("# TYPE {} histogram\n", name));
            output.push_str(&format!("{}_count {}\n", name, data.count));
            output.push_str(&format!("{}_sum {}\n", name, data.sum));
            output.push_str(&format!("{}_min {}\n", name, data.min));
            output.push_str(&format!("{}_max {}\n", name, data.max));
            output.push_str(&format!("{}_p50 {}\n", name, data.p50));
            output.push_str(&format!("{}_p95 {}\n", name, data.p95));
            output.push_str(&format!("{}_p99 {}\n", name, data.p99));
        }

        output
    }

    /// Filter metrics by name prefix
    pub fn filter_by_prefix(&self, prefix: &str) -> Self {
        Self {
            counters: self
                .counters
                .iter()
                .filter(|(name, _)| name.starts_with(prefix))
                .cloned()
                .collect(),
            gauges: self
                .gauges
                .iter()
                .filter(|(name, _)| name.starts_with(prefix))
                .cloned()
                .collect(),
            histograms: self
                .histograms
                .iter()
                .filter(|(name, _)| name.starts_with(prefix))
                .cloned()
                .collect(),
            timestamp: self.timestamp,
        }
    }
}

/// Inner store for metrics data
#[derive(Debug, Default)]
struct MetricsStore {
    counters: DashMap<String, Arc<TelemetryCounter>>,
    gauges: DashMap<String, Arc<TelemetryGauge>>,
    histograms: DashMap<String, Arc<TelemetryHistogram>>,
}

impl MetricsStore {
    /// Create a snapshot of current metrics
    fn snapshot(&self) -> MetricsSnapshot {
        let mut snapshot = MetricsSnapshot::new();

        // Collect counters
        for entry in self.counters.iter() {
            snapshot
                .counters
                .push((entry.key().clone(), entry.value().get()));
        }

        // Collect gauges
        for entry in self.gauges.iter() {
            snapshot
                .gauges
                .push((entry.key().clone(), entry.value().get()));
        }

        // Collect histograms
        for entry in self.histograms.iter() {
            let data = entry.value().get_data();
            snapshot.histograms.push((entry.key().clone(), data));
        }

        snapshot
    }

    /// Cleanup old histogram entries to prevent memory growth
    fn cleanup_histograms(&self, max_entries: usize) {
        for entry in self.histograms.iter() {
            entry.value().cleanup(max_entries);
        }
    }
}

/// Custom counter implementation
#[derive(Debug)]
struct TelemetryCounter {
    value: AtomicU64,
}

impl TelemetryCounter {
    fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
        }
    }

    fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

impl CounterFn for TelemetryCounter {
    fn increment(&self, value: u64) {
        self.value.fetch_add(value, Ordering::Relaxed);
    }

    fn absolute(&self, value: u64) {
        self.value.store(value, Ordering::Relaxed);
    }
}

/// Custom gauge implementation
#[derive(Debug)]
struct TelemetryGauge {
    value: AtomicU64,
}

impl TelemetryGauge {
    fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
        }
    }

    fn get(&self) -> f64 {
        f64::from_bits(self.value.load(Ordering::Relaxed))
    }
}

impl GaugeFn for TelemetryGauge {
    fn increment(&self, value: f64) {
        let current = f64::from_bits(self.value.load(Ordering::Relaxed));
        let new_value = current + value;
        self.value.store(new_value.to_bits(), Ordering::Relaxed);
    }

    fn decrement(&self, value: f64) {
        let current = f64::from_bits(self.value.load(Ordering::Relaxed));
        let new_value = current - value;
        self.value.store(new_value.to_bits(), Ordering::Relaxed);
    }

    fn set(&self, value: f64) {
        self.value.store(value.to_bits(), Ordering::Relaxed);
    }
}

/// Custom histogram implementation
#[derive(Debug)]
struct TelemetryHistogram {
    values: parking_lot::Mutex<Vec<f64>>,
}

impl TelemetryHistogram {
    fn new() -> Self {
        Self {
            values: parking_lot::Mutex::new(Vec::new()),
        }
    }

    fn get_data(&self) -> HistogramData {
        let values = self.values.lock();
        HistogramData::from_values(&values)
    }

    fn cleanup(&self, max_entries: usize) {
        let mut values = self.values.lock();
        if values.len() > max_entries {
            let start = values.len() - max_entries;
            *values = values[start..].to_vec();
        }
    }
}

impl HistogramFn for TelemetryHistogram {
    fn record(&self, value: f64) {
        self.values.lock().push(value);
    }
}

/// Custom telemetry recorder implementing metrics::Recorder
#[derive(Debug, Clone)]
pub struct TelemetryRecorder {
    store: Arc<MetricsStore>,
}

impl Default for TelemetryRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetryRecorder {
    /// Create a new telemetry recorder
    pub fn new() -> Self {
        Self {
            store: Arc::new(MetricsStore::default()),
        }
    }

    /// Get a snapshot of current metrics
    pub fn get_snapshot(&self) -> MetricsSnapshot {
        self.store.snapshot()
    }

    /// Cleanup old histogram data
    pub fn cleanup_histograms(&self, max_entries: usize) {
        self.store.cleanup_histograms(max_entries);
    }

    /// Get counter value by name
    pub fn get_counter(&self, name: &str) -> Option<u64> {
        self.store.counters.get(name).map(|v| v.get())
    }

    /// Get gauge value by name
    pub fn get_gauge(&self, name: &str) -> Option<f64> {
        self.store.gauges.get(name).map(|v| v.get())
    }

    /// Get histogram data by name
    pub fn get_histogram(&self, name: &str) -> Option<HistogramData> {
        self.store.histograms.get(name).map(|v| v.get_data())
    }
}

impl Recorder for TelemetryRecorder {
    fn describe_counter(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {
        // Descriptions are not stored; metadata can be added if needed
    }

    fn describe_gauge(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {
        // Descriptions are not stored; metadata can be added if needed
    }

    fn describe_histogram(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {
        // Descriptions are not stored; metadata can be added if needed
    }

    fn register_counter(&self, key: &Key, _metadata: &Metadata<'_>) -> Counter {
        let name = key.name().to_string();
        let counter = self
            .store
            .counters
            .entry(name)
            .or_insert_with(|| Arc::new(TelemetryCounter::new()));
        Counter::from_arc(counter.clone())
    }

    fn register_gauge(&self, key: &Key, _metadata: &Metadata<'_>) -> Gauge {
        let name = key.name().to_string();
        let gauge = self
            .store
            .gauges
            .entry(name)
            .or_insert_with(|| Arc::new(TelemetryGauge::new()));
        Gauge::from_arc(gauge.clone())
    }

    fn register_histogram(&self, key: &Key, _metadata: &Metadata<'_>) -> Histogram {
        let name = key.name().to_string();
        let histogram = self
            .store
            .histograms
            .entry(name)
            .or_insert_with(|| Arc::new(TelemetryHistogram::new()));
        Histogram::from_arc(histogram.clone())
    }
}

/// Global telemetry recorder instance
static GLOBAL_RECORDER: std::sync::OnceLock<TelemetryRecorder> = std::sync::OnceLock::new();

/// Initialize the global telemetry recorder
pub fn init_global_recorder() -> &'static TelemetryRecorder {
    GLOBAL_RECORDER.get_or_init(TelemetryRecorder::new)
}

/// Get the global telemetry recorder
pub fn global_recorder() -> Option<&'static TelemetryRecorder> {
    GLOBAL_RECORDER.get()
}

/// Set the global telemetry recorder and install it as the metrics recorder
pub fn set_global_recorder(
    recorder: TelemetryRecorder,
) -> Result<(), metrics::SetRecorderError<TelemetryRecorder>> {
    let recorder_clone = recorder.clone();
    GLOBAL_RECORDER
        .set(recorder)
        .map_err(|_| metrics::SetRecorderError(recorder_clone.clone()))?;
    metrics::set_global_recorder(recorder_clone)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_histogram_data_calculation() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let data = HistogramData::from_values(&values);

        assert_eq!(data.count, 10);
        assert_eq!(data.sum, 55.0);
        assert_eq!(data.min, 1.0);
        assert_eq!(data.max, 10.0);
        // p50 for 10 elements: index = round(0.5 * 9) = 5, value at index 5 is 6.0
        assert_eq!(data.p50, 6.0);
    }

    #[test]
    fn test_telemetry_counter() {
        let counter = TelemetryCounter::new();
        counter.increment(5);
        counter.increment(3);
        assert_eq!(counter.get(), 8);

        counter.absolute(10);
        assert_eq!(counter.get(), 10);
    }

    #[test]
    fn test_telemetry_gauge() {
        let gauge = TelemetryGauge::new();
        gauge.set(42.0);
        assert_eq!(gauge.get(), 42.0);

        gauge.increment(10.0);
        assert_eq!(gauge.get(), 52.0);

        gauge.decrement(20.0);
        assert_eq!(gauge.get(), 32.0);
    }

    #[test]
    fn test_telemetry_recorder() {
        use metrics::Level;
        let recorder = TelemetryRecorder::new();
        let key = Key::from_name("test_metric");
        let counter = recorder.register_counter(&key, &Metadata::new("test", Level::INFO, None));
        counter.increment(1);

        assert_eq!(recorder.get_counter("test_metric"), Some(1));
    }

    #[test]
    fn test_metrics_snapshot() {
        use metrics::Level;
        let recorder = TelemetryRecorder::new();
        let key = Key::from_name("test_counter");
        let counter = recorder.register_counter(&key, &Metadata::new("test", Level::INFO, None));
        counter.increment(100);

        let snapshot = recorder.get_snapshot();
        assert!(snapshot.counters.iter().any(|(k, v)| k == "test_counter" && *v == 100));
    }

    #[test]
    fn test_snapshot_filtering() {
        let mut snapshot = MetricsSnapshot::new();
        snapshot.counters = vec![
            ("graphdb_query_total".to_string(), 100),
            ("graphdb_error_total".to_string(), 5),
            ("other_metric".to_string(), 10),
        ];

        let filtered = snapshot.filter_by_prefix("graphdb_");
        assert_eq!(filtered.counters.len(), 2);
        assert!(filtered.counters.iter().all(|(k, _)| k.starts_with("graphdb_")));
    }

    #[test]
    fn test_snapshot_to_text() {
        let mut snapshot = MetricsSnapshot::new();
        snapshot.counters = vec![("test_counter".to_string(), 42)];

        let text = snapshot.to_text_format();
        assert!(text.contains("# TYPE test_counter counter"));
        assert!(text.contains("test_counter 42"));
    }
}

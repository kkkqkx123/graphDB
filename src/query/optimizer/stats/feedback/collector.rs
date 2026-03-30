//! Feedback Collection Module
//!
//! Provide a lightweight mechanism for collecting execution feedback, used to gather actual statistical information about the execution of queries.

use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Feedback collection tool
///
/// A lightweight collector used for collecting actual statistical information about the execution of queries.
/// Use atomic operations to ensure thread safety.
///
/// # Example
/// ```
/// use graphdb::query::optimizer::stats::feedback::collector::ExecutionFeedbackCollector;
///
/// let collector = ExecutionFeedbackCollector::new();
/// collector.start();
/// collector.record_rows(100);
/// let time_us = collector.finish();
/// assert_eq!(collector.get_actual_rows(), 100);
/// ```
#[derive(Debug)]
pub struct ExecutionFeedbackCollector {
    /// Actual number of output lines (atomic counter)
    actual_rows: AtomicU64,
    /// Execution time (in microseconds)
    execution_time_us: AtomicU64,
    /// Start time
    start_time: RwLock<Option<Instant>>,
}

impl ExecutionFeedbackCollector {
    /// Create a new feedback collector.
    pub fn new() -> Self {
        Self {
            actual_rows: AtomicU64::new(0),
            execution_time_us: AtomicU64::new(0),
            start_time: RwLock::new(None),
        }
    }

    /// Start collecting
    ///
    /// Record the current time as the start time.
    pub fn start(&self) {
        *self.start_time.write() = Some(Instant::now());
    }

    /// Record the number of output lines.
    ///
    /// Increment the count of output lines on an atomic basis (i.e., without any intermediate updates or delays).
    pub fn record_rows(&self, rows: u64) {
        self.actual_rows.fetch_add(rows, Ordering::Relaxed);
    }

    /// End the data collection process and return the execution time (in microseconds).
    ///
    /// Calculate the elapsed time from the start to the current moment, and store the execution time.
    pub fn finish(&self) -> u64 {
        let elapsed = self
            .start_time
            .read()
            .map(|start| start.elapsed().as_micros() as u64)
            .unwrap_or(0);
        self.execution_time_us.store(elapsed, Ordering::Relaxed);
        elapsed
    }

    /// Get the actual number of output lines.
    pub fn get_actual_rows(&self) -> u64 {
        self.actual_rows.load(Ordering::Relaxed)
    }

    /// Obtain the execution time (in microseconds).
    pub fn get_execution_time_us(&self) -> u64 {
        self.execution_time_us.load(Ordering::Relaxed)
    }

    /// Reset the collector.
    ///
    /// Clear all collected data and restore the system to its initial state.
    pub fn reset(&self) {
        self.actual_rows.store(0, Ordering::Relaxed);
        self.execution_time_us.store(0, Ordering::Relaxed);
        *self.start_time.write() = None;
    }
}

impl Default for ExecutionFeedbackCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple feedback collector for cardinality estimation
///
/// Lightweight feedback collector for base estimation correction
#[derive(Debug)]
pub struct SimpleFeedbackCollector {
    /// Map from query pattern to feedback
    feedback_map: RwLock<std::collections::HashMap<String, SimpleExecutionFeedback>>,
    /// Maximum number of patterns to store
    max_patterns: usize,
}

/// Simple execution feedback record
#[derive(Debug, Clone)]
pub struct SimpleExecutionFeedback {
    /// Query pattern fingerprint (simplified)
    pub query_pattern: String,
    /// Estimated rows
    pub estimated_rows: u64,
    /// Actual rows
    pub actual_rows: u64,
    /// Estimation error ratio (|actual - estimated| / actual)
    pub estimation_error: f64,
    /// Execution count
    pub execution_count: u64,
    /// Last execution time
    pub last_executed: Instant,
}

impl SimpleExecutionFeedback {
    /// Create new feedback record
    pub fn new(query_pattern: String, estimated: u64, actual: u64) -> Self {
        Self {
            query_pattern,
            estimated_rows: estimated,
            actual_rows: actual,
            estimation_error: Self::calculate_error(estimated, actual),
            execution_count: 1,
            last_executed: Instant::now(),
        }
    }

    /// Update with new execution data
    pub fn update(&mut self, estimated: u64, actual: u64) {
        const ALPHA: f64 = 0.3;

        self.estimated_rows = Self::ema(self.estimated_rows, estimated, ALPHA);
        self.actual_rows = Self::ema(self.actual_rows, actual, ALPHA);
        self.estimation_error = Self::calculate_error(estimated, actual);
        self.execution_count += 1;
        self.last_executed = Instant::now();
    }

    /// Check if feedback is stale (older than 1 hour)
    pub fn is_stale(&self) -> bool {
        self.last_executed.elapsed().as_secs() > 3600
    }

    /// Calculate estimation error: |actual - estimated| / actual
    fn calculate_error(estimated: u64, actual: u64) -> f64 {
        if actual > 0 {
            (actual as f64 - estimated as f64).abs() / actual as f64
        } else {
            0.0
        }
    }

    /// Exponential moving average update
    fn ema(current: u64, new_value: u64, alpha: f64) -> u64 {
        ((1.0 - alpha) * current as f64 + alpha * new_value as f64) as u64
    }
}

impl SimpleFeedbackCollector {
    /// Create new feedback collector
    pub fn new() -> Self {
        Self {
            feedback_map: RwLock::new(std::collections::HashMap::new()),
            max_patterns: 1000,
        }
    }

    /// Create with custom max patterns
    pub fn with_max_patterns(max_patterns: usize) -> Self {
        let mut collector = Self::new();
        collector.max_patterns = max_patterns;
        collector
    }

    /// Record execution feedback
    pub fn record_feedback(&self, pattern: &str, estimated: u64, actual: u64) {
        let mut map = self.feedback_map.write();

        // Ensure capacity by removing stale entries
        if map.len() >= self.max_patterns {
            self.evict_stale_entries(&mut map);
        }

        // If still full, remove oldest entry
        if map.len() >= self.max_patterns {
            self.evict_oldest(&mut map);
        }

        // Update or insert
        match map.get_mut(pattern) {
            Some(feedback) => feedback.update(estimated, actual),
            None => {
                let feedback = SimpleExecutionFeedback::new(pattern.to_string(), estimated, actual);
                map.insert(pattern.to_string(), feedback);
            }
        }
    }

    /// Evict stale entries (older than 1 hour)
    fn evict_stale_entries(
        &self,
        map: &mut std::collections::HashMap<String, SimpleExecutionFeedback>,
    ) {
        let stale_keys: Vec<String> = map
            .iter()
            .filter(|(_, v)| v.is_stale())
            .map(|(k, _)| k.clone())
            .collect();
        for key in stale_keys {
            map.remove(&key);
        }
    }

    /// Evict the oldest entry
    fn evict_oldest(&self, map: &mut std::collections::HashMap<String, SimpleExecutionFeedback>) {
        if let Some(oldest_key) = map
            .iter()
            .min_by_key(|(_, v)| v.last_executed)
            .map(|(k, _)| k.clone())
        {
            map.remove(&oldest_key);
        }
    }

    /// Get feedback for a query pattern
    pub fn get_feedback(&self, pattern: &str) -> Option<SimpleExecutionFeedback> {
        self.feedback_map.read().get(pattern).cloned()
    }

    /// Get average estimation error
    pub fn get_avg_estimation_error(&self) -> f64 {
        let map = self.feedback_map.read();
        if map.is_empty() {
            return 0.0;
        }
        map.values().map(|f| f.estimation_error).sum::<f64>() / map.len() as f64
    }

    /// Get feedback count
    pub fn feedback_count(&self) -> usize {
        self.feedback_map.read().len()
    }

    /// Get correction factor for a pattern: actual / estimated
    /// Returns 1.0 if no feedback available
    /// Usage: corrected_estimate = estimate * correction_factor
    pub fn get_correction_factor(&self, pattern: &str) -> f64 {
        self.get_feedback(pattern)
            .map(|f| {
                if f.estimated_rows > 0 {
                    f.actual_rows as f64 / f.estimated_rows as f64
                } else {
                    1.0
                }
            })
            .unwrap_or(1.0)
    }

    /// Clear all feedback
    pub fn clear(&self) {
        self.feedback_map.write().clear();
    }
}

impl Default for SimpleFeedbackCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_feedback_collector() {
        let collector = ExecutionFeedbackCollector::new();
        collector.start();
        collector.record_rows(100);
        collector.record_rows(50);

        let time = collector.finish();
        assert_eq!(collector.get_actual_rows(), 150);
        assert_eq!(collector.get_execution_time_us(), time);
    }

    #[test]
    fn test_collector_reset() {
        let collector = ExecutionFeedbackCollector::new();
        collector.start();
        collector.record_rows(100);
        collector.finish();

        collector.reset();
        assert_eq!(collector.get_actual_rows(), 0);
        assert_eq!(collector.get_execution_time_us(), 0);
    }

    #[test]
    fn test_collector_without_start() {
        let collector = ExecutionFeedbackCollector::new();
        // Finish directly without calling “start”.
        let time = collector.finish();
        assert_eq!(time, 0);
        assert_eq!(collector.get_execution_time_us(), 0);
    }
}

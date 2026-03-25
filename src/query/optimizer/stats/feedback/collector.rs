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
        assert!(time > 0);
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

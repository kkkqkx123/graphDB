//! Storage Metrics Module
//!
//! Provides thread-safe metrics collection for storage operations.
//! All metrics are based on `std::sync::atomic` with zero external dependencies.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Error type enumeration for precise error classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    /// Connection-related errors (pool exhaustion, network failures)
    Connection,
    /// Serialization errors (encoding, format issues)
    Serialization,
    /// Deserialization errors (decoding, parsing issues)
    Deserialization,
    /// Timeout errors (operation exceeded time limit)
    Timeout,
    /// Other unspecified errors
    Other,
}

/// Storage metrics collector
///
/// Thread-safe collector that tracks operation counts, latencies, and errors.
/// Uses atomic operations for zero-lock concurrent access.
///
/// # Example
///
/// ```rust
/// use bm25::storage::common::metrics::{StorageMetricsCollector, ErrorType};
/// use std::time::Instant;
///
/// let collector = StorageMetricsCollector::new();
///
/// // Record a successful operation
/// let start = Instant::now();
/// // ... perform operation ...
/// collector.record_operation(start);
///
/// // Record an error
/// collector.record_error(ErrorType::Connection);
///
/// // Get aggregated metrics
/// let metrics = collector.get_metrics(1024); // 1KB memory usage
/// println!("Operations: {}", metrics.operation_count);
/// ```
#[derive(Debug, Default)]
pub struct StorageMetricsCollector {
    /// Total number of operations
    operation_count: AtomicU64,
    /// Cumulative latency in microseconds
    total_latency: AtomicU64,
    /// Total number of errors
    error_count: AtomicU64,
    /// Connection error count
    connection_errors: AtomicU64,
    /// Serialization error count
    serialization_errors: AtomicU64,
    /// Deserialization error count
    deserialization_errors: AtomicU64,
    /// Timeout error count
    timeout_errors: AtomicU64,
    /// Other error count
    other_errors: AtomicU64,
}

impl StorageMetricsCollector {
    /// Creates a new metrics collector
    pub fn new() -> Self {
        Self::default()
    }

    /// Records the completion of an operation
    ///
    /// Calculates the elapsed time from the start instant and updates
    /// operation count and total latency counters.
    ///
    /// # Arguments
    ///
    /// * `start` - The instant when the operation started
    ///
    /// # Example
    ///
    /// ```rust
    /// let collector = StorageMetricsCollector::new();
    /// let start = Instant::now();
    /// // ... perform operation ...
    /// collector.record_operation(start);
    /// ```
    pub fn record_operation(&self, start: Instant) {
        let latency = start.elapsed().as_micros() as u64;
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        self.total_latency.fetch_add(latency, Ordering::Relaxed);
    }

    /// Records an error of the specified type
    ///
    /// Increments both the total error count and the specific error type counter.
    ///
    /// # Arguments
    ///
    /// * `error_type` - The type of error that occurred
    ///
    /// # Example
    ///
    /// ```rust
    /// use bm25::storage::common::metrics::{StorageMetricsCollector, ErrorType};
    ///
    /// let collector = StorageMetricsCollector::new();
    /// collector.record_error(ErrorType::Connection);
    /// assert_eq!(collector.get_error_count(), 1);
    /// ```
    pub fn record_error(&self, error_type: ErrorType) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
        match error_type {
            ErrorType::Connection => {
                self.connection_errors.fetch_add(1, Ordering::Relaxed);
            }
            ErrorType::Serialization => {
                self.serialization_errors.fetch_add(1, Ordering::Relaxed);
            }
            ErrorType::Deserialization => {
                self.deserialization_errors.fetch_add(1, Ordering::Relaxed);
            }
            ErrorType::Timeout => {
                self.timeout_errors.fetch_add(1, Ordering::Relaxed);
            }
            ErrorType::Other => {
                self.other_errors.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Gets the total number of operations recorded
    pub fn get_operation_count(&self) -> u64 {
        self.operation_count.load(Ordering::Relaxed)
    }

    /// Gets the total latency in microseconds
    pub fn get_total_latency(&self) -> u64 {
        self.total_latency.load(Ordering::Relaxed)
    }

    /// Gets the average latency in microseconds
    ///
    /// Returns 0 if no operations have been recorded.
    pub fn get_average_latency(&self) -> u64 {
        let count = self.get_operation_count();
        if count > 0 {
            self.total_latency.load(Ordering::Relaxed) / count
        } else {
            0
        }
    }

    /// Gets the total number of errors
    pub fn get_error_count(&self) -> u64 {
        self.error_count.load(Ordering::Relaxed)
    }

    /// Gets the connection error count
    pub fn get_connection_errors(&self) -> u64 {
        self.connection_errors.load(Ordering::Relaxed)
    }

    /// Gets the serialization error count
    pub fn get_serialization_errors(&self) -> u64 {
        self.serialization_errors.load(Ordering::Relaxed)
    }

    /// Gets the deserialization error count
    pub fn get_deserialization_errors(&self) -> u64 {
        self.deserialization_errors.load(Ordering::Relaxed)
    }

    /// Gets the timeout error count
    pub fn get_timeout_errors(&self) -> u64 {
        self.timeout_errors.load(Ordering::Relaxed)
    }

    /// Gets the other error count
    pub fn get_other_errors(&self) -> u64 {
        self.other_errors.load(Ordering::Relaxed)
    }

    /// Resets all metrics to zero
    pub fn reset(&self) {
        self.operation_count.store(0, Ordering::Relaxed);
        self.total_latency.store(0, Ordering::Relaxed);
        self.error_count.store(0, Ordering::Relaxed);
        self.connection_errors.store(0, Ordering::Relaxed);
        self.serialization_errors.store(0, Ordering::Relaxed);
        self.deserialization_errors.store(0, Ordering::Relaxed);
        self.timeout_errors.store(0, Ordering::Relaxed);
        self.other_errors.store(0, Ordering::Relaxed);
    }

    /// Gets aggregated storage metrics
    ///
    /// # Arguments
    ///
    /// * `memory_usage` - Current memory usage in bytes
    ///
    /// # Returns
    ///
    /// A `StorageMetrics` struct containing all current metrics
    pub fn get_metrics(&self, memory_usage: u64) -> StorageMetrics {
        StorageMetrics {
            operation_count: self.get_operation_count(),
            average_latency: self.get_average_latency(),
            memory_usage,
            error_count: self.get_error_count(),
            connection_errors: self.connection_errors.load(Ordering::Relaxed),
            serialization_errors: self.serialization_errors.load(Ordering::Relaxed),
            deserialization_errors: self.deserialization_errors.load(Ordering::Relaxed),
        }
    }

    /// Starts a timer for recording operation latency
    ///
    /// Convenience method that returns an `Instant` to be used with `record_operation()`.
    ///
    /// # Example
    ///
    /// ```rust
    /// let collector = StorageMetricsCollector::new();
    /// let start = collector.start_timer();
    /// // ... perform operation ...
    /// collector.record_operation(start);
    /// ```
    pub fn start_timer(&self) -> Instant {
        Instant::now()
    }
}

impl Clone for StorageMetricsCollector {
    fn clone(&self) -> Self {
        // Clone creates a new instance with current values
        Self {
            operation_count: AtomicU64::new(self.get_operation_count()),
            total_latency: AtomicU64::new(self.get_total_latency()),
            error_count: AtomicU64::new(self.get_error_count()),
            connection_errors: AtomicU64::new(self.get_connection_errors()),
            serialization_errors: AtomicU64::new(self.get_serialization_errors()),
            deserialization_errors: AtomicU64::new(self.get_deserialization_errors()),
            timeout_errors: AtomicU64::new(self.get_timeout_errors()),
            other_errors: AtomicU64::new(self.get_other_errors()),
        }
    }
}

/// Storage performance metrics (read-only snapshot)
///
/// This struct provides a point-in-time snapshot of storage performance metrics.
/// It is typically created from a `StorageMetricsCollector` via `get_metrics()`.
#[derive(Debug, Clone, Default)]
pub struct StorageMetrics {
    /// Total number of operations
    pub operation_count: u64,
    /// Average latency in microseconds
    pub average_latency: u64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Total number of errors
    pub error_count: u64,
    /// Connection error count
    pub connection_errors: u64,
    /// Serialization error count
    pub serialization_errors: u64,
    /// Deserialization error count
    pub deserialization_errors: u64,
}

impl StorageMetrics {
    /// Creates a new empty metrics struct
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets all metrics to zero
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Calculates the error rate as a percentage
    ///
    /// # Returns
    ///
    /// A value between 0.0 and 1.0 representing the error rate
    ///
    /// # Example
    ///
    /// ```rust
    /// use bm25::storage::common::metrics::StorageMetrics;
    ///
    /// let mut metrics = StorageMetrics::new();
    /// metrics.operation_count = 1000;
    /// metrics.error_count = 10;
    /// assert_eq!(metrics.error_rate(), 0.01);
    /// ```
    pub fn error_rate(&self) -> f64 {
        if self.operation_count == 0 {
            0.0
        } else {
            self.error_count as f64 / self.operation_count as f64
        }
    }

    /// Gets the success count
    pub fn success_count(&self) -> u64 {
        self.operation_count.saturating_sub(self.error_count)
    }

    /// Gets the success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        1.0 - self.error_rate()
    }
}

/// RAII-style operation timer
///
/// Automatically records operation latency when dropped.
///
/// # Example
///
/// ```rust
/// use bm25::storage::common::metrics::{StorageMetricsCollector, OperationTimer};
///
/// let collector = StorageMetricsCollector::new();
/// {
///     let _timer = OperationTimer::new(&collector);
///     // ... perform operation ...
/// } // Timer automatically records latency here
/// ```
pub struct OperationTimer<'a> {
    start: Instant,
    collector: &'a StorageMetricsCollector,
}

impl<'a> OperationTimer<'a> {
    /// Creates a new operation timer
    ///
    /// The timer starts immediately upon creation.
    pub fn new(collector: &'a StorageMetricsCollector) -> Self {
        Self {
            start: Instant::now(),
     collector,
        }
    }

    /// Stops the timer without recording
    ///
    /// Useful if you want to manually record the operation later.
    pub fn stop(self) -> u64 {
        self.start.elapsed().as_micros() as u64
    }
}

impl<'a> Drop for OperationTimer<'a> {
    fn drop(&mut self) {
        self.collector.record_operation(self.start);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_storage_metrics_collector_basic() {
        let collector = StorageMetricsCollector::new();

        let start = collector.start_timer();
        thread::sleep(Duration::from_millis(1));
        collector.record_operation(start);

        assert_eq!(collector.get_operation_count(), 1);
        assert!(collector.get_total_latency() > 0);
        assert!(collector.get_average_latency() > 0);
    }

    #[test]
    fn test_error_recording() {
        let collector = StorageMetricsCollector::new();

        collector.record_error(ErrorType::Connection);
        collector.record_error(ErrorType::Serialization);
        collector.record_error(ErrorType::Deserialization);

        assert_eq!(collector.get_error_count(), 3);
        assert_eq!(collector.get_connection_errors(), 1);
        assert_eq!(collector.get_serialization_errors(), 1);
        assert_eq!(collector.get_deserialization_errors(), 1);
    }

    #[test]
    fn test_reset() {
        let collector = StorageMetricsCollector::new();

        let start = collector.start_timer();
        thread::sleep(Duration::from_millis(1));
        collector.record_operation(start);
        collector.record_error(ErrorType::Connection);

        collector.reset();

        assert_eq!(collector.get_operation_count(), 0);
        assert_eq!(collector.get_error_count(), 0);
        assert_eq!(collector.get_average_latency(), 0);
    }

    #[test]
    fn test_get_metrics() {
        let collector = StorageMetricsCollector::new();

        let start = collector.start_timer();
        thread::sleep(Duration::from_millis(1));
        collector.record_operation(start);
        collector.record_error(ErrorType::Connection);

        let metrics = collector.get_metrics(1024);

        assert_eq!(metrics.operation_count, 1);
        assert!(metrics.average_latency > 0);
        assert_eq!(metrics.memory_usage, 1024);
        assert_eq!(metrics.error_count, 1);
        assert_eq!(metrics.connection_errors, 1);
    }

    #[test]
    fn test_error_rate() {
        let mut metrics = StorageMetrics::new();
        metrics.operation_count = 1000;
        metrics.error_count = 10;

        assert!((metrics.error_rate() - 0.01).abs() < f64::EPSILON);
        assert!((metrics.success_rate() - 0.99).abs() < f64::EPSILON);
    }

    #[test]
    fn test_success_count() {
        let mut metrics = StorageMetrics::new();
        metrics.operation_count = 1000;
        metrics.error_count = 10;

        assert_eq!(metrics.success_count(), 990);
    }

    #[test]
    fn test_concurrent_access() {
        let collector = std::sync::Arc::new(StorageMetricsCollector::new());
        let mut handles = vec![];

        for i in 0..10 {
            let collector_clone = collector.clone();
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    let start = collector_clone.start_timer();
                    if j % 10 == 0 {
                        collector_clone.record_error(ErrorType::Other);
                    }
                    thread::sleep(Duration::from_micros(10));
                    collector_clone.record_operation(start);
                }
                println!("Thread {} completed", i);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(collector.get_operation_count(), 1000);
        assert_eq!(collector.get_error_count(), 100);
    }

    #[test]
    fn test_operation_timer_raii() {
        let collector = StorageMetricsCollector::new();

        {
            let _timer = OperationTimer::new(&collector);
            thread::sleep(Duration::from_millis(1));
        } // Timer drops here and records

        assert_eq!(collector.get_operation_count(), 1);
        assert!(collector.get_average_latency() > 0);
    }

    #[test]
    fn test_operation_timer_manual_stop() {
        let collector = StorageMetricsCollector::new();

        let timer = OperationTimer::new(&collector);
        thread::sleep(Duration::from_millis(1));
        let elapsed = timer.stop();

        assert_eq!(collector.get_operation_count(), 0); // Not recorded yet
        assert!(elapsed > 0);
    }

    #[test]
    fn test_zero_operations_average_latency() {
        let collector = StorageMetricsCollector::new();
        assert_eq!(collector.get_average_latency(), 0);
    }

    #[test]
    fn test_clone() {
        let collector = StorageMetricsCollector::new();

        let start = collector.start_timer();
        thread::sleep(Duration::from_millis(1));
        collector.record_operation(start);
        collector.record_error(ErrorType::Connection);

        let cloned = collector.clone();

        assert_eq!(cloned.get_operation_count(), 1);
        assert_eq!(cloned.get_error_count(), 1);
    }
}

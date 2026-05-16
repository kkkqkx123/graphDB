//! Storage Metrics Module
//!
//! Provides metrics collection for storage backends in inversearch.
//! Similar to bm25's metrics module for consistency across crates.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorType {
    Connection,
    Serialization,
    Deserialization,
    Timeout,
    Other,
}

#[derive(Debug, Default)]
pub struct StorageMetricsCollector {
    operation_count: AtomicU64,
    total_latency: AtomicU64,
    error_count: AtomicU64,
    connection_errors: AtomicU64,
    serialization_errors: AtomicU64,
    deserialization_errors: AtomicU64,
    timeout_errors: AtomicU64,
    other_errors: AtomicU64,
}

impl StorageMetricsCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_operation(&self, start: Instant) {
        let latency = start.elapsed().as_micros() as u64;
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        self.total_latency.fetch_add(latency, Ordering::Relaxed);
    }

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

    pub fn get_operation_count(&self) -> u64 {
        self.operation_count.load(Ordering::Relaxed)
    }

    pub fn get_average_latency(&self) -> u64 {
        let count = self.get_operation_count();
        if count > 0 {
            self.total_latency.load(Ordering::Relaxed) / count
        } else {
            0
        }
    }

    pub fn get_error_count(&self) -> u64 {
        self.error_count.load(Ordering::Relaxed)
    }

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
}

#[derive(Debug, Clone, Default)]
pub struct StorageMetrics {
    pub operation_count: u64,
    pub average_latency: u64,
    pub memory_usage: u64,
    pub error_count: u64,
    pub connection_errors: u64,
    pub serialization_errors: u64,
    pub deserialization_errors: u64,
}

impl StorageMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn error_rate(&self) -> f64 {
        if self.operation_count == 0 {
            0.0
        } else {
            self.error_count as f64 / self.operation_count as f64
        }
    }
}

pub struct OperationTimer<'a> {
    start: Instant,
    collector: &'a StorageMetricsCollector,
}

impl<'a> OperationTimer<'a> {
    pub fn new(collector: &'a StorageMetricsCollector) -> Self {
        Self {
            start: Instant::now(),
            collector,
        }
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
    fn test_storage_metrics_collector() {
        let collector = StorageMetricsCollector::new();

        let start = Instant::now();
        thread::sleep(Duration::from_millis(1));
        collector.record_operation(start);

        assert_eq!(collector.get_operation_count(), 1);
        assert!(collector.get_average_latency() > 0);
    }

    #[test]
    fn test_error_recording() {
        let collector = StorageMetricsCollector::new();

        collector.record_error(ErrorType::Connection);
        collector.record_error(ErrorType::Serialization);

        assert_eq!(collector.get_error_count(), 2);
    }

    #[test]
    fn test_concurrent_access() {
        let collector = Arc::new(StorageMetricsCollector::new());
        let mut handles = vec![];

        for _ in 0..10 {
            let collector_clone = collector.clone();
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    let start = Instant::now();
                    thread::sleep(Duration::from_micros(10));
                    collector_clone.record_operation(start);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(collector.get_operation_count(), 1000);
    }

    #[test]
    fn test_operation_timer() {
        let collector = StorageMetricsCollector::new();

        {
            let _timer = OperationTimer::new(&collector);
            thread::sleep(Duration::from_millis(1));
        }

        assert_eq!(collector.get_operation_count(), 1);
    }

    #[test]
    fn test_storage_metrics_error_rate() {
        let mut metrics = StorageMetrics::new();
        metrics.operation_count = 100;
        metrics.error_count = 5;

        assert!((metrics.error_rate() - 0.05).abs() < 0.0001);
    }
}

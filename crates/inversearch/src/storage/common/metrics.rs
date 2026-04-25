//! Storage performance metrics
//!
//! Provides storage operation performance statistics

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

#[derive(Debug, Clone, Default)]
pub struct StorageMetrics {
    pub operation_count: usize,
    pub average_latency: usize,
    pub memory_usage: usize,
    pub error_count: usize,
}

impl StorageMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.operation_count = 0;
        self.average_latency = 0;
        self.memory_usage = 0;
        self.error_count = 0;
    }
}

pub struct OperationTimer {
    start: Instant,
    operation_count: AtomicUsize,
    total_latency: AtomicUsize,
}

impl OperationTimer {
    pub fn new(operation_count: &AtomicUsize, total_latency: &AtomicUsize) -> Self {
        Self {
            start: Instant::now(),
            operation_count: AtomicUsize::new(operation_count.load(Ordering::Relaxed)),
            total_latency: AtomicUsize::new(total_latency.load(Ordering::Relaxed)),
        }
    }

    pub fn record_completion(&self) {
        let latency = self.start.elapsed().as_micros() as usize;
        metrics::counter!("inversearch_storage_operations_total").increment(1);
        metrics::histogram!("inversearch_storage_latency_micros").record(latency as f64);
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        self.total_latency.fetch_add(latency, Ordering::Relaxed);
    }

    pub fn elapsed_micros(&self) -> u128 {
        self.start.elapsed().as_micros()
    }
}

#[derive(Debug)]
pub struct MetricsCollector {
    operation_count: AtomicUsize,
    total_latency: AtomicUsize,
    error_count: AtomicUsize,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            operation_count: AtomicUsize::new(0),
            total_latency: AtomicUsize::new(0),
            error_count: AtomicUsize::new(0),
        }
    }

    pub fn start_timer(&self) -> Instant {
        Instant::now()
    }

    pub fn record_operation(&self, start: Instant) {
        let latency = start.elapsed().as_micros() as usize;
        metrics::counter!("inversearch_storage_operations_total").increment(1);
        metrics::histogram!("inversearch_storage_latency_micros").record(latency as f64);
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        self.total_latency.fetch_add(latency, Ordering::Relaxed);
    }

    pub fn record_error(&self) {
        metrics::counter!("inversearch_storage_errors_total").increment(1);
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_operation_count(&self) -> usize {
        self.operation_count.load(Ordering::Relaxed)
    }

    pub fn get_total_latency(&self) -> usize {
        self.total_latency.load(Ordering::Relaxed)
    }

    pub fn get_average_latency(&self) -> usize {
        let count = self.get_operation_count();
        if count > 0 {
            self.get_total_latency() / count
        } else {
            0
        }
    }

    pub fn get_error_count(&self) -> usize {
        self.error_count.load(Ordering::Relaxed)
    }

    pub fn get_metrics(&self, memory_usage: usize) -> StorageMetrics {
        metrics::gauge!("inversearch_storage_memory_usage").set(memory_usage as f64);
        StorageMetrics {
            operation_count: self.get_operation_count(),
            average_latency: self.get_average_latency(),
            memory_usage,
            error_count: self.get_error_count(),
        }
    }

    pub fn reset(&self) {
        self.operation_count.store(0, Ordering::Relaxed);
        self.total_latency.store(0, Ordering::Relaxed);
        self.error_count.store(0, Ordering::Relaxed);
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::new();

        // 模拟操作
        let start = collector.start_timer();
        thread::sleep(Duration::from_millis(1));
        collector.record_operation(start);

        assert_eq!(collector.get_operation_count(), 1);
        assert!(collector.get_total_latency() > 0);
        assert!(collector.get_average_latency() > 0);
    }

    #[test]
    fn test_metrics_collector_error() {
        let collector = MetricsCollector::new();

        collector.record_error();
        collector.record_error();

        assert_eq!(collector.get_error_count(), 2);
    }

    #[test]
    fn test_metrics_collector_reset() {
        let collector = MetricsCollector::new();

        let start = collector.start_timer();
        collector.record_operation(start);
        collector.record_error();

        collector.reset();

        assert_eq!(collector.get_operation_count(), 0);
        assert_eq!(collector.get_total_latency(), 0);
        assert_eq!(collector.get_error_count(), 0);
    }
}

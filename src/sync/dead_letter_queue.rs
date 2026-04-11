//! Dead Letter Queue for failed index operations
//!
//! Stores operations that failed after all retry attempts for later analysis and recovery.

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use crate::sync::external_index::IndexOperation;

/// Dead letter queue entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterEntry {
    /// The failed operation
    pub operation: IndexOperation,
    /// Error message
    pub error: String,
    /// Number of retry attempts
    pub retry_attempts: u32,
    /// First failure timestamp
    pub first_failure: SystemTime,
    /// Last failure timestamp
    pub last_failure: SystemTime,
    /// Whether this entry has been processed for recovery
    pub recovered: bool,
}

impl DeadLetterEntry {
    pub fn new(operation: IndexOperation, error: String, retry_attempts: u32) -> Self {
        let now = SystemTime::now();
        Self {
            operation,
            error,
            retry_attempts,
            first_failure: now,
            last_failure: now,
            recovered: false,
        }
    }

    pub fn update_failure(&mut self, error: String) {
        self.error = error;
        self.last_failure = SystemTime::now();
    }

    pub fn age(&self) -> Duration {
        self.first_failure
            .elapsed()
            .unwrap_or(Duration::from_secs(0))
    }

    pub fn mark_recovered(&mut self) {
        self.recovered = true;
    }
}

/// Dead letter queue configuration
#[derive(Debug, Clone)]
pub struct DeadLetterQueueConfig {
    /// Maximum number of entries in the queue
    pub max_size: usize,
    /// Maximum age of entries before automatic cleanup
    pub max_age: Duration,
    /// Whether to enable automatic cleanup
    pub auto_cleanup_enabled: bool,
}

impl DeadLetterQueueConfig {
    pub fn is_auto_cleanup_enabled(&self) -> bool {
        self.auto_cleanup_enabled
    }

    pub fn get_cleanup_interval(&self) -> Duration {
        self.max_age / 2
    }
}

impl Default for DeadLetterQueueConfig {
    fn default() -> Self {
        Self {
            max_size: 10_000,
            max_age: Duration::from_secs(3600), // 1 hour
            auto_cleanup_enabled: true,
        }
    }
}

/// Dead Letter Queue
#[derive(Debug)]
pub struct DeadLetterQueue {
    entries: Mutex<Vec<DeadLetterEntry>>,
    config: DeadLetterQueueConfig,
    metrics: Option<Arc<crate::sync::metrics::SyncMetrics>>,
}

impl DeadLetterQueue {
    pub fn new(config: DeadLetterQueueConfig) -> Self {
        Self {
            entries: Mutex::new(Vec::with_capacity(config.max_size)),
            config,
            metrics: None,
        }
    }

    pub fn is_auto_cleanup_enabled(&self) -> bool {
        self.config.auto_cleanup_enabled
    }

    pub fn get_cleanup_interval(&self) -> Duration {
        self.config.max_age / 2
    }

    pub fn with_metrics(mut self, metrics: Arc<crate::sync::metrics::SyncMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Add an entry to the dead letter queue
    pub fn add(&self, entry: DeadLetterEntry) {
        let mut entries = self.entries.lock();
        
        // Check if queue is full
        if entries.len() >= self.config.max_size {
            // Remove oldest entry if full
            if !entries.is_empty() {
                entries.remove(0);
                log::warn!("Dead letter queue is full, removed oldest entry");
            }
        }

        entries.push(entry);

        if let Some(metrics) = &self.metrics {
            metrics.record_dead_letter();
        }

        log::warn!(
            "Added entry to dead letter queue (size: {})",
            entries.len()
        );
    }

    /// Get all entries
    pub fn get_all(&self) -> Vec<DeadLetterEntry> {
        self.entries.lock().clone()
    }

    /// Get entries that haven't been recovered
    pub fn get_unrecovered(&self) -> Vec<DeadLetterEntry> {
        self.entries
            .lock()
            .iter()
            .filter(|e| !e.recovered)
            .cloned()
            .collect()
    }

    /// Get entries older than specified duration
    pub fn get_old_entries(&self, age: Duration) -> Vec<DeadLetterEntry> {
        self.entries
            .lock()
            .iter()
            .filter(|e| e.age() > age)
            .cloned()
            .collect()
    }

    /// Remove an entry by index
    pub fn remove(&self, index: usize) -> Option<DeadLetterEntry> {
        let mut entries = self.entries.lock();
        if index < entries.len() {
            let entry = entries.remove(index);
            if let Some(metrics) = &self.metrics {
                metrics.remove_dead_letter();
            }
            Some(entry)
        } else {
            None
        }
    }

    /// Mark an entry as recovered
    pub fn mark_recovered(&self, index: usize) -> bool {
        let mut entries = self.entries.lock();
        if index < entries.len() {
            entries[index].mark_recovered();
            true
        } else {
            false
        }
    }

    /// Cleanup old entries
    pub fn cleanup(&self) -> usize {
        let mut entries = self.entries.lock();
        let initial_len = entries.len();
        
        entries.retain(|e| e.age() <= self.config.max_age);
        
        let removed = initial_len - entries.len();
        
        if removed > 0 {
            log::info!("Cleaned up {} old dead letter entries", removed);
            
            // Update metrics
            if let Some(metrics) = &self.metrics {
                for _ in 0..removed {
                    metrics.remove_dead_letter();
                }
            }
        }
        
        removed
    }

    /// Get queue size
    pub fn len(&self) -> usize {
        self.entries.lock().len()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.entries.lock().is_empty()
    }

    /// Clear all entries
    pub fn clear(&self) {
        let mut entries = self.entries.lock();
        let count = entries.len();
        entries.clear();
        
        if let Some(metrics) = &self.metrics {
            for _ in 0..count {
                metrics.remove_dead_letter();
            }
        }
        
        log::info!("Cleared {} entries from dead letter queue", count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::external_index::IndexData;
    use std::collections::HashMap;

    fn create_test_operation() -> IndexOperation {
        IndexOperation::Insert {
            key: crate::sync::external_index::IndexKey {
                space_id: 1,
                tag_name: "test_tag".to_string(),
                field_name: "test_field".to_string(),
            },
            id: "test_id".to_string(),
            data: IndexData::Fulltext("test".to_string()),
            payload: HashMap::new(),
        }
    }

    #[test]
    fn test_dead_letter_queue_add() {
        let config = DeadLetterQueueConfig::default();
        let dlq = DeadLetterQueue::new(config);

        let entry = DeadLetterEntry::new(
            create_test_operation(),
            "test error".to_string(),
            3,
        );

        dlq.add(entry);
        assert_eq!(dlq.len(), 1);
    }

    #[test]
    fn test_dead_letter_queue_max_size() {
        let config = DeadLetterQueueConfig {
            max_size: 3,
            ..DeadLetterQueueConfig::default()
        };
        let dlq = DeadLetterQueue::new(config);

        for i in 0..5 {
            let entry = DeadLetterEntry::new(
                create_test_operation(),
                format!("error {}", i),
                3,
            );
            dlq.add(entry);
        }

        assert_eq!(dlq.len(), 3);
    }

    #[test]
    fn test_dead_letter_queue_cleanup() {
        let config = DeadLetterQueueConfig {
            max_age: Duration::from_millis(100),
            ..DeadLetterQueueConfig::default()
        };
        let dlq = DeadLetterQueue::new(config);

        let entry = DeadLetterEntry::new(
            create_test_operation(),
            "test error".to_string(),
            3,
        );
        dlq.add(entry);

        // Wait for entry to become old
        std::thread::sleep(Duration::from_millis(150));

        let removed = dlq.cleanup();
        assert_eq!(removed, 1);
        assert_eq!(dlq.len(), 0);
    }

    #[test]
    fn test_dead_letter_queue_recovery() {
        let config = DeadLetterQueueConfig::default();
        let dlq = DeadLetterQueue::new(config);

        let entry = DeadLetterEntry::new(
            create_test_operation(),
            "test error".to_string(),
            3,
        );
        dlq.add(entry);

        let unrecovered = dlq.get_unrecovered();
        assert_eq!(unrecovered.len(), 1);
        assert!(!unrecovered[0].recovered);

        dlq.mark_recovered(0);

        let unrecovered = dlq.get_unrecovered();
        assert_eq!(unrecovered.len(), 0);
    }
}

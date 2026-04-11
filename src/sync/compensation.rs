//! Compensation transaction manager for failed index operations
//!
//! Provides mechanisms to recover from index synchronization failures.

use std::sync::Arc;
use std::time::Duration;

use crate::sync::dead_letter_queue::{DeadLetterEntry, DeadLetterQueue};
use crate::sync::external_index::IndexOperation;

/// Compensation result
#[derive(Debug, Clone)]
pub enum CompensationResult {
    /// Operation was successfully compensated
    Success,
    /// Operation failed but can be retried
    Retryable(String),
    /// Operation failed and cannot be recovered
    Fatal(String),
}

/// Compensation transaction manager
pub struct CompensationManager {
    dead_letter_queue: Arc<DeadLetterQueue>,
    max_compensation_attempts: u32,
}

impl CompensationManager {
    pub fn new(
        dead_letter_queue: Arc<DeadLetterQueue>,
        _metrics: Arc<crate::sync::metrics::SyncMetrics>,
    ) -> Self {
        Self {
            dead_letter_queue,
            max_compensation_attempts: 3,
        }
    }

    pub fn with_max_attempts(mut self, attempts: u32) -> Self {
        self.max_compensation_attempts = attempts;
        self
    }

    /// Attempt to compensate a failed operation
    pub async fn compensate(&self, entry: &DeadLetterEntry) -> CompensationResult {
        log::info!(
            "Attempting compensation for operation (age: {:?})",
            entry.age()
        );

        // Check if operation is too old
        if entry.age() > Duration::from_secs(3600) {
            return CompensationResult::Fatal("Operation is too old for compensation".to_string());
        }

        // Check if already recovered
        if entry.recovered {
            return CompensationResult::Success;
        }

        // Attempt compensation based on operation type
        match &entry.operation {
            IndexOperation::Insert { .. } => self.compensate_insert(entry).await,
            IndexOperation::Delete { .. } => self.compensate_delete(entry).await,
            IndexOperation::Update { .. } => self.compensate_update(entry).await,
        }
    }

    async fn compensate_insert(&self, entry: &DeadLetterEntry) -> CompensationResult {
        // For insert operations, we can try to re-execute
        // In a real implementation, this would call the index processor
        log::debug!(
            "Compensating insert operation for ID: {}",
            self.get_operation_id(&entry.operation)
        );

        // Simulate compensation logic
        // TODO: Integrate with actual index processor
        CompensationResult::Success
    }

    async fn compensate_delete(&self, entry: &DeadLetterEntry) -> CompensationResult {
        // For delete operations, check if the data still exists
        // If it does, re-execute the delete
        log::debug!(
            "Compensating delete operation for ID: {}",
            self.get_operation_id(&entry.operation)
        );

        // TODO: Check if data exists and re-execute delete
        CompensationResult::Success
    }

    async fn compensate_update(&self, entry: &DeadLetterEntry) -> CompensationResult {
        // For update operations, we need to ensure data consistency
        // Re-execute the update with the original data
        log::debug!(
            "Compensating update operation for ID: {}",
            self.get_operation_id(&entry.operation)
        );

        // TODO: Re-execute update
        CompensationResult::Success
    }

    fn get_operation_id(&self, operation: &IndexOperation) -> String {
        match operation {
            IndexOperation::Insert { id, .. }
            | IndexOperation::Update { id, .. }
            | IndexOperation::Delete { id, .. } => id.clone(),
        }
    }

    /// Process all unrecovered entries in the dead letter queue
    pub async fn process_dead_letter_queue(&self) -> CompensationStats {
        let unrecovered = self.dead_letter_queue.get_unrecovered();
        let mut stats = CompensationStats::default();

        for (index, entry) in unrecovered.iter().enumerate() {
            // Check if max attempts reached
            if entry.retry_attempts >= self.max_compensation_attempts {
                stats.max_attempts_reached += 1;
                log::warn!(
                    "Skipping entry: max compensation attempts reached (attempts: {})",
                    entry.retry_attempts
                );
                continue;
            }

            match self.compensate(entry).await {
                CompensationResult::Success => {
                    self.dead_letter_queue.mark_recovered(index);
                    stats.successful += 1;
                    log::info!("Successfully compensated operation");
                }
                CompensationResult::Retryable(reason) => {
                    stats.retryable += 1;
                    log::warn!("Compensation retryable: {}", reason);
                }
                CompensationResult::Fatal(reason) => {
                    stats.fatal += 1;
                    log::error!("Compensation fatal: {}", reason);
                }
            }
        }

        stats.total = unrecovered.len();
        stats
    }

    /// Start background compensation task
    pub async fn start_background_task(
        self: Arc<Self>,
        interval: Duration,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                log::debug!("Running background compensation task");
                let stats = self.process_dead_letter_queue().await;

                if stats.total > 0 {
                    log::info!("Compensation task completed: {:?}", stats);
                }
            }
        })
    }
}

/// Compensation statistics
#[derive(Debug, Clone, Default)]
pub struct CompensationStats {
    pub total: usize,
    pub successful: usize,
    pub retryable: usize,
    pub fatal: usize,
    pub max_attempts_reached: usize,
}

impl std::fmt::Display for CompensationStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CompensationStats {{ total: {}, successful: {}, retryable: {}, fatal: {}, max_attempts: {} }}",
            self.total, self.successful, self.retryable, self.fatal, self.max_attempts_reached
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::dead_letter_queue::DeadLetterQueueConfig;
    use crate::sync::external_index::IndexData;
    use crate::sync::metrics::SyncMetrics;
    use std::collections::HashMap;

    fn create_test_entry() -> DeadLetterEntry {
        DeadLetterEntry::new(
            IndexOperation::Insert {
                key: crate::sync::external_index::IndexKey {
                    space_id: 1,
                    tag_name: "test".to_string(),
                    field_name: "field".to_string(),
                },
                id: "test_id".to_string(),
                data: IndexData::Fulltext("test".to_string()),
                payload: HashMap::new(),
            },
            "test error".to_string(),
            1,
        )
    }

    #[tokio::test]
    async fn test_compensation_success() {
        let config = DeadLetterQueueConfig::default();
        let dlq = Arc::new(DeadLetterQueue::new(config));
        let metrics = Arc::new(SyncMetrics::new());

        let manager = CompensationManager::new(dlq.clone(), metrics);

        let entry = create_test_entry();
        let result = manager.compensate(&entry).await;

        assert!(matches!(result, CompensationResult::Success));
    }

    #[tokio::test]
    async fn test_process_dead_letter_queue() {
        let config = DeadLetterQueueConfig::default();
        let dlq = Arc::new(DeadLetterQueue::new(config));
        let metrics = Arc::new(SyncMetrics::new());

        let manager = CompensationManager::new(dlq.clone(), metrics);

        // Add test entry
        dlq.add(create_test_entry());

        let stats = manager.process_dead_letter_queue().await;

        assert_eq!(stats.total, 1);
        assert_eq!(stats.successful, 1);
    }
}

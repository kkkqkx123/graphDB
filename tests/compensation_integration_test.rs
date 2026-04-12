//! Compensation Manager Integration Tests

use std::sync::Arc;
use std::time::Duration;

use graphdb::sync::compensation::{CompensationManager, CompensationResult};
use graphdb::sync::dead_letter_queue::{DeadLetterEntry, DeadLetterQueue, DeadLetterQueueConfig};
use graphdb::sync::external_index::{IndexData, IndexKey, IndexOperation};
use graphdb::sync::metrics::SyncMetrics;

#[tokio::test]
async fn test_compensation_manager_auto_trigger() {
    // Setup
    let config = DeadLetterQueueConfig::default();
    let dlq = Arc::new(DeadLetterQueue::new(config));
    let metrics = Arc::new(SyncMetrics::new());
    
    let manager = Arc::new(
        CompensationManager::new(dlq.clone(), metrics.clone())
            .with_max_attempts(3)
    );
    
    // Add a failed operation to DLQ
    let entry = DeadLetterEntry::new(
        IndexOperation::Insert {
            key: IndexKey {
                space_id: 1,
                tag_name: "test".to_string(),
                field_name: "field".to_string(),
            },
            id: "test_id".to_string(),
            data: IndexData::Fulltext("test".to_string()),
            payload: std::collections::HashMap::new(),
        },
        "test error".to_string(),
        1,
    );
    
    dlq.add(entry);
    
    // Verify entry is in DLQ
    assert_eq!(dlq.get_unrecovered().len(), 1);
    
    // Trigger compensation
    let stats = manager.process_dead_letter_queue().await;
    
    // Verify compensation succeeded
    assert_eq!(stats.total, 1);
    assert_eq!(stats.successful, 1);
    assert_eq!(stats.retryable, 0);
    assert_eq!(stats.fatal, 0);
    
    // Verify entry is marked as recovered
    let unrecovered = dlq.get_unrecovered();
    assert_eq!(unrecovered.len(), 0);
}

#[tokio::test]
async fn test_compensation_manager_background_task() {
    // Setup
    let config = DeadLetterQueueConfig::default();
    let dlq = Arc::new(DeadLetterQueue::new(config));
    let metrics = Arc::new(SyncMetrics::new());
    
    let manager = Arc::new(
        CompensationManager::new(dlq.clone(), metrics.clone())
            .with_max_attempts(3)
    );
    
    // Add multiple failed operations
    for i in 0..3 {
        let entry = DeadLetterEntry::new(
            IndexOperation::Insert {
                key: IndexKey {
                    space_id: 1,
                    tag_name: "test".to_string(),
                    field_name: "field".to_string(),
                },
                id: format!("test_id_{}", i),
                data: IndexData::Fulltext(format!("test_{}", i)),
                payload: std::collections::HashMap::new(),
            },
            "test error".to_string(),
            1,
        );
        dlq.add(entry);
    }
    
    // Start background task
    let handle = manager.clone().start_background_task(Duration::from_millis(100)).await;
    
    // Wait for background task to process
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Stop the manager
    manager.stop();
    
    // Wait for handle to complete
    let _ = handle.await;
    
    // Verify all entries were processed
    let unrecovered = dlq.get_unrecovered();
    assert_eq!(unrecovered.len(), 0);
}

#[tokio::test]
async fn test_compensation_retryable_failure() {
    // Setup
    let config = DeadLetterQueueConfig::default();
    let dlq = Arc::new(DeadLetterQueue::new(config));
    let metrics = Arc::new(SyncMetrics::new());
    
    let manager = CompensationManager::new(dlq.clone(), metrics.clone())
        .with_max_attempts(3);
    
    // Add an entry with max attempts already reached
    let mut entry = DeadLetterEntry::new(
        IndexOperation::Insert {
            key: IndexKey {
                space_id: 1,
                tag_name: "test".to_string(),
                field_name: "field".to_string(),
            },
            id: "test_id".to_string(),
            data: IndexData::Fulltext("test".to_string()),
            payload: std::collections::HashMap::new(),
        },
        "test error".to_string(),
        3, // Already at max attempts
    );
    
    dlq.add(entry.clone());
    
    // Process DLQ
    let stats = manager.process_dead_letter_queue().await;
    
    // Verify entry was skipped due to max attempts
    assert_eq!(stats.max_attempts_reached, 1);
    assert_eq!(stats.total, 1);
}

#[tokio::test]
async fn test_compensation_metrics() {
    // Setup
    let config = DeadLetterQueueConfig::default();
    let dlq = Arc::new(DeadLetterQueue::new(config));
    let metrics = Arc::new(SyncMetrics::new());
    
    let manager = Arc::new(
        CompensationManager::new(dlq.clone(), metrics.clone())
            .with_max_attempts(3)
    );
    
    // Add failed operations
    for i in 0..5 {
        let entry = DeadLetterEntry::new(
            IndexOperation::Insert {
                key: IndexKey {
                    space_id: 1,
                    tag_name: "test".to_string(),
                    field_name: "field".to_string(),
                },
                id: format!("test_id_{}", i),
                data: IndexData::Fulltext(format!("test_{}", i)),
                payload: std::collections::HashMap::new(),
            },
            "test error".to_string(),
            1,
        );
        dlq.add(entry);
    }
    
    // Process DLQ
    let stats = manager.process_dead_letter_queue().await;
    
    // Verify metrics were recorded
    let sync_stats = metrics.get_stats();
    assert_eq!(sync_stats.compensation_attempts_total, 5);
    assert_eq!(sync_stats.compensation_successes, 5);
    assert_eq!(sync_stats.compensation_failures, 0);
}

#[tokio::test]
async fn test_dlq_management_api() {
    // Setup
    let config = DeadLetterQueueConfig::default();
    let dlq = Arc::new(DeadLetterQueue::new(config));
    
    // Add entries
    for i in 0..3 {
        let entry = DeadLetterEntry::new(
            IndexOperation::Insert {
                key: IndexKey {
                    space_id: 1,
                    tag_name: "test".to_string(),
                    field_name: "field".to_string(),
                },
                id: format!("test_id_{}", i),
                data: IndexData::Fulltext(format!("test_{}", i)),
                payload: std::collections::HashMap::new(),
            },
            "test error".to_string(),
            1,
        );
        dlq.add(entry);
    }
    
    // Test get_all
    let all_entries = dlq.get_all();
    assert_eq!(all_entries.len(), 3);
    
    // Test get_unrecovered
    let unrecovered = dlq.get_unrecovered();
    assert_eq!(unrecovered.len(), 3);
    
    // Test remove
    let removed = dlq.remove(0);
    assert!(removed.is_some());
    
    // Verify removal
    let all_entries = dlq.get_all();
    assert_eq!(all_entries.len(), 2);
    
    // Test mark_recovered
    dlq.mark_recovered(0);
    let unrecovered = dlq.get_unrecovered();
    assert_eq!(unrecovered.len(), 1);
}

#[tokio::test]
async fn test_compensation_old_entry() {
    // Setup
    let config = DeadLetterQueueConfig::default();
    let dlq = Arc::new(DeadLetterQueue::new(config));
    let metrics = Arc::new(SyncMetrics::new());
    
    let manager = CompensationManager::new(dlq.clone(), metrics.clone())
        .with_max_attempts(3);
    
    // Create an old entry manually
    let mut entry = DeadLetterEntry::new(
        IndexOperation::Insert {
            key: IndexKey {
                space_id: 1,
                tag_name: "test".to_string(),
                field_name: "field".to_string(),
            },
            id: "old_test_id".to_string(),
            data: IndexData::Fulltext("test".to_string()),
            payload: std::collections::HashMap::new(),
        },
        "old error".to_string(),
        1,
    );
    
    // Simulate old entry by modifying first_failure
    use std::time::{SystemTime, UNIX_EPOCH};
    entry.first_failure = SystemTime::now() - Duration::from_secs(3601); // Older than 1 hour
    
    dlq.add(entry);
    
    // Process DLQ
    let stats = manager.process_dead_letter_queue().await;
    
    // Verify old entry was marked as fatal
    assert_eq!(stats.fatal, 1);
    assert_eq!(stats.total, 1);
}

//! Fulltext Integration Tests - Dead Letter Queue
//!
//! Test scope:
//! - Dead letter queue basic operations
//! - Failed operation entry creation
//! - Recovery from dead letter queue
//! - Queue cleanup and expiration
//! - Statistics and monitoring
//!
//! Test cases: TC-FT-DLQ-001 ~ TC-FT-DLQ-010

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use graphdb::sync::dead_letter_queue::{
    DeadLetterEntry, DeadLetterQueue, DeadLetterQueueConfig, DeadLetterQueueStats,
};
use graphdb::sync::external_index::{IndexData, IndexKey, IndexOperation};

fn create_test_index_key() -> IndexKey {
    IndexKey {
        space_id: 1,
        tag_name: "Article".to_string(),
        field_name: "content".to_string(),
    }
}

fn create_test_insert_operation(id: &str, text: &str) -> IndexOperation {
    IndexOperation::Insert {
        key: create_test_index_key(),
        id: id.to_string(),
        data: IndexData::Fulltext(text.to_string()),
        payload: HashMap::new(),
    }
}

fn create_test_delete_operation(id: &str) -> IndexOperation {
    IndexOperation::Delete {
        key: create_test_index_key(),
        id: id.to_string(),
    }
}

fn create_test_entry(error: &str, retry_attempts: u32) -> DeadLetterEntry {
    DeadLetterEntry::new(
        create_test_insert_operation("test_doc", "test content"),
        error.to_string(),
        retry_attempts,
    )
}

/// TC-FT-DLQ-001: Basic Dead Letter Queue Creation
#[test]
fn test_dead_letter_queue_creation() {
    let config = DeadLetterQueueConfig::default();
    let dlq = DeadLetterQueue::new(config);

    assert!(dlq.is_empty(), "New DLQ should be empty");
    assert_eq!(dlq.len(), 0, "New DLQ should have 0 entries");
}

/// TC-FT-DLQ-002: Add Entry to Dead Letter Queue
#[test]
fn test_add_entry_to_dlq() {
    let config = DeadLetterQueueConfig::default();
    let dlq = DeadLetterQueue::new(config);

    let entry = create_test_entry("Connection failed", 3);
    dlq.add(entry);

    assert!(!dlq.is_empty(), "DLQ should not be empty after adding entry");
    assert_eq!(dlq.len(), 1, "DLQ should have 1 entry");
}

/// TC-FT-DLQ-003: Multiple Entries in Dead Letter Queue
#[test]
fn test_multiple_entries_in_dlq() {
    let config = DeadLetterQueueConfig::default();
    let dlq = DeadLetterQueue::new(config);

    for i in 0..5 {
        let entry = DeadLetterEntry::new(
            create_test_insert_operation(&format!("doc_{}", i), &format!("content {}", i)),
            format!("Error {}", i),
            3,
        );
        dlq.add(entry);
    }

    assert_eq!(dlq.len(), 5, "DLQ should have 5 entries");

    let all_entries = dlq.get_all();
    assert_eq!(all_entries.len(), 5, "Should retrieve all 5 entries");
}

/// TC-FT-DLQ-004: Dead Letter Queue Max Size Limit
#[test]
fn test_dlq_max_size_limit() {
    let config = DeadLetterQueueConfig {
        max_size: 3,
        ..DeadLetterQueueConfig::default()
    };
    let dlq = DeadLetterQueue::new(config);

    for i in 0..10 {
        let entry = create_test_entry(&format!("Error {}", i), 3);
        dlq.add(entry);
    }

    assert_eq!(dlq.len(), 3, "DLQ should respect max_size limit");
}

/// TC-FT-DLQ-005: Get Unrecovered Entries
#[test]
fn test_get_unrecovered_entries() {
    let config = DeadLetterQueueConfig::default();
    let dlq = DeadLetterQueue::new(config);

    for i in 0..3 {
        let entry = create_test_entry(&format!("Error {}", i), 3);
        dlq.add(entry);
    }

    let unrecovered = dlq.get_unrecovered();
    assert_eq!(unrecovered.len(), 3, "All entries should be unrecovered initially");

    dlq.mark_recovered(1);

    let unrecovered_after = dlq.get_unrecovered();
    assert_eq!(
        unrecovered_after.len(),
        2,
        "Should have 2 unrecovered after marking one as recovered"
    );
}

/// TC-FT-DLQ-006: Mark Entry as Recovered
#[test]
fn test_mark_entry_recovered() {
    let config = DeadLetterQueueConfig::default();
    let dlq = DeadLetterQueue::new(config);

    let entry = create_test_entry("Test error", 3);
    dlq.add(entry);

    let result = dlq.mark_recovered(0);
    assert!(result, "mark_recovered should return true for valid index");

    let all_entries = dlq.get_all();
    assert!(
        all_entries[0].recovered,
        "Entry should be marked as recovered"
    );
}

/// TC-FT-DLQ-007: Remove Entry from Dead Letter Queue
#[test]
fn test_remove_entry_from_dlq() {
    let config = DeadLetterQueueConfig::default();
    let dlq = DeadLetterQueue::new(config);

    for i in 0..3 {
        let entry = create_test_entry(&format!("Error {}", i), 3);
        dlq.add(entry);
    }

    let removed = dlq.remove(1);
    assert!(removed.is_some(), "remove should return the removed entry");
    assert_eq!(dlq.len(), 2, "DLQ should have 2 entries after removal");

    let invalid_remove = dlq.remove(10);
    assert!(
        invalid_remove.is_none(),
        "remove should return None for invalid index"
    );
}

/// TC-FT-DLQ-008: Dead Letter Queue Cleanup
#[test]
fn test_dlq_cleanup() {
    let config = DeadLetterQueueConfig {
        max_age: Duration::from_millis(50),
        ..DeadLetterQueueConfig::default()
    };
    let dlq = DeadLetterQueue::new(config);

    let entry = create_test_entry("Test error", 3);
    dlq.add(entry);

    assert_eq!(dlq.len(), 1, "Should have 1 entry before cleanup");

    std::thread::sleep(Duration::from_millis(100));

    let removed = dlq.cleanup();
    assert_eq!(removed, 1, "Should remove 1 old entry");
    assert_eq!(dlq.len(), 0, "Should have 0 entries after cleanup");
}

/// TC-FT-DLQ-009: Dead Letter Queue Statistics
#[test]
fn test_dlq_statistics() {
    let config = DeadLetterQueueConfig::default();
    let dlq = DeadLetterQueue::new(config);

    for i in 0..5 {
        let entry = create_test_entry(&format!("Error {}", i), 3);
        dlq.add(entry);
    }

    dlq.mark_recovered(0);
    dlq.mark_recovered(2);

    let stats: DeadLetterQueueStats = dlq.get_stats();

    assert_eq!(stats.total_entries, 5, "Should have 5 total entries");
    assert_eq!(stats.recovered_entries, 2, "Should have 2 recovered entries");
    assert_eq!(stats.unrecovered_entries, 3, "Should have 3 unrecovered entries");
}

/// TC-FT-DLQ-010: Clear Dead Letter Queue
#[test]
fn test_clear_dlq() {
    let config = DeadLetterQueueConfig::default();
    let dlq = DeadLetterQueue::new(config);

    for i in 0..5 {
        let entry = create_test_entry(&format!("Error {}", i), 3);
        dlq.add(entry);
    }

    assert_eq!(dlq.len(), 5, "Should have 5 entries before clear");

    dlq.clear();

    assert!(dlq.is_empty(), "DLQ should be empty after clear");
    assert_eq!(dlq.len(), 0, "DLQ should have 0 entries after clear");
}

/// TC-FT-DLQ-011: Entry Age Calculation
#[test]
fn test_entry_age_calculation() {
    let entry = create_test_entry("Test error", 3);

    std::thread::sleep(Duration::from_millis(100));

    let age = entry.age();
    assert!(
        age >= Duration::from_millis(100),
        "Entry age should be at least 100ms"
    );
}

/// TC-FT-DLQ-012: Entry Update Failure
#[test]
fn test_entry_update_failure() {
    let mut entry = create_test_entry("Original error", 3);

    entry.update_failure("Updated error message".to_string());

    assert_eq!(
        entry.error, "Updated error message",
        "Error message should be updated"
    );
    assert!(
        entry.last_failure > entry.first_failure,
        "Last failure time should be updated"
    );
}

/// TC-FT-DLQ-013: Get Old Entries
#[test]
fn test_get_old_entries() {
    let config = DeadLetterQueueConfig::default();
    let dlq = DeadLetterQueue::new(config);

    for i in 0..3 {
        let entry = create_test_entry(&format!("Error {}", i), 3);
        dlq.add(entry);
    }

    std::thread::sleep(Duration::from_millis(100));

    let old_entries = dlq.get_old_entries(Duration::from_millis(50));
    assert_eq!(old_entries.len(), 3, "All entries should be old enough");
}

/// TC-FT-DLQ-014: Different Operation Types in DLQ
#[test]
fn test_different_operation_types() {
    let config = DeadLetterQueueConfig::default();
    let dlq = DeadLetterQueue::new(config);

    let insert_entry = DeadLetterEntry::new(
        create_test_insert_operation("doc_1", "insert content"),
        "Insert failed".to_string(),
        3,
    );
    dlq.add(insert_entry);

    let delete_entry = DeadLetterEntry::new(
        create_test_delete_operation("doc_2"),
        "Delete failed".to_string(),
        3,
    );
    dlq.add(delete_entry);

    let all_entries = dlq.get_all();
    assert_eq!(all_entries.len(), 2, "Should have 2 entries of different types");

    match &all_entries[0].operation {
        IndexOperation::Insert { .. } => {}
        _ => panic!("First entry should be an Insert operation"),
    }

    match &all_entries[1].operation {
        IndexOperation::Delete { .. } => {}
        _ => panic!("Second entry should be a Delete operation"),
    }
}

/// TC-FT-DLQ-015: DLQ Config Default Values
#[test]
fn test_dlq_config_defaults() {
    let config = DeadLetterQueueConfig::default();

    assert_eq!(config.max_size, 10_000, "Default max_size should be 10000");
    assert_eq!(
        config.max_age,
        Duration::from_secs(3600),
        "Default max_age should be 1 hour"
    );
    assert!(
        config.auto_cleanup_enabled,
        "Auto cleanup should be enabled by default"
    );
}

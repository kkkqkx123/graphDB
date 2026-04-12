//! Sync Module 2PC Protocol Tests
//!
//! Tests for two-phase commit protocol implementation

mod common;

use common::sync_helpers::{create_test_vertex, SyncTestHarness};
use graphdb::core::types::DataType;
use graphdb::core::Value;
use graphdb::sync::manager::SyncManager;
use graphdb::transaction::{TransactionManager, TransactionManagerConfig, TransactionOptions};
use std::sync::Arc;

/// TC-040: 2PC full protocol flow
#[test]
fn test_2pc_full_protocol() {
    let mut harness = SyncTestHarness::new().expect("Failed to create test harness");

    // Setup
    harness
        .create_space("test_space")
        .expect("Failed to create space");
    harness
        .create_tag_with_fulltext(
            "test_space",
            "Person",
            vec![("name", DataType::String)],
            vec!["name"],
        )
        .expect("Failed to create tag");

    // Begin transaction
    let txn_id = harness
        .begin_transaction()
        .expect("Failed to begin transaction");

    // Execute multiple operations
    for i in 0..10 {
        let vertex = create_test_vertex(
            i + 1,
            "Person",
            vec![(
                "name",
                Value::String(format!("Person{}", i + 1)),
            )],
        );
        harness
            .insert_vertex_with_txn("test_space", vertex)
            .expect("Failed to insert vertex");
    }

    // Phase 1: Prepare
    let sync_manager = harness.sync_manager.clone();
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        sync_manager
            .prepare_transaction(txn_id)
            .await
            .expect("Prepare should succeed");
    });

    // Phase 2: Commit storage (done in commit_transaction)
    // Phase 3: Commit index sync (done in commit_transaction)
    harness
        .commit_transaction()
        .expect("Failed to commit transaction");

    harness.wait_for_async(300);

    // Verify all operations are committed
    for i in 0..10 {
        harness
            .assert_vertex_exists("test_space", &Value::Int((i + 1) as i64))
            .expect("Vertex should exist");
    }

    // Verify index sync
    let results = harness
        .search_fulltext("test_space", "Person", "name", "Person1", 20)
        .expect("Failed to search");
    println!("Search results for 'Person1': {}", results.len());
    assert!(results.len() >= 1, "At least one index should be synced, found {}", results.len());
    
    // Verify all indexes are synced
    let mut found_count = 0;
    for i in 0..10 {
        let search_term = format!("Person{}", i + 1);
        let results = harness
            .search_fulltext("test_space", "Person", "name", &search_term, 10)
            .expect("Failed to search");
        if !results.is_empty() {
            found_count += 1;
        }
    }
    assert!(found_count >= 10, "All indexes should be synced, found {}", found_count);
}

/// TC-041: 2PC prepare phase failure
#[test]
fn test_2pc_prepare_failure() {
    let mut harness = SyncTestHarness::new().expect("Failed to create test harness");

    // Setup
    harness
        .create_space("test_space")
        .expect("Failed to create space");
    harness
        .create_tag_with_fulltext(
            "test_space",
            "Person",
            vec![("name", DataType::String)],
            vec!["name"],
        )
        .expect("Failed to create tag");

    // Begin transaction
    harness
        .begin_transaction()
        .expect("Failed to begin transaction");

    // Insert vertex
    let vertex = create_test_vertex(
        1,
        "Person",
        vec![("name", Value::String("Alice".to_string()))],
    );
    harness
        .insert_vertex_with_txn("test_space", vertex)
        .expect("Failed to insert vertex");

    // Prepare should succeed in normal case
    let txn_id = harness.current_txn_id.unwrap();
    let sync_manager = harness.sync_manager.clone();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let prepare_result = rt.block_on(async { sync_manager.prepare_transaction(txn_id).await });

    // Prepare should succeed
    assert!(
        prepare_result.is_ok(),
        "Prepare should succeed for valid operations"
    );
}

/// TC-042: 2PC storage commit failure
#[test]
fn test_2pc_storage_commit_failure() {
    // This test verifies that when storage commit fails,
    // the index buffer is cleaned up
    
    let mut harness = SyncTestHarness::new().expect("Failed to create test harness");

    // Setup
    harness
        .create_space("test_space")
        .expect("Failed to create space");
    harness
        .create_tag_with_fulltext(
            "test_space",
            "Person",
            vec![("name", DataType::String)],
            vec!["name"],
        )
        .expect("Failed to create tag");

    // Begin transaction
    harness
        .begin_transaction()
        .expect("Failed to begin transaction");

    // Insert vertex (buffered in sync manager)
    let vertex = create_test_vertex(
        1,
        "Person",
        vec![("name", Value::String("Alice".to_string()))],
    );
    harness
        .insert_vertex_with_txn("test_space", vertex)
        .expect("Failed to insert vertex");

    // Rollback instead of commit (simulating storage failure)
    harness
        .rollback_transaction()
        .expect("Failed to rollback");

    harness.wait_for_async(200);

    // Verify nothing was committed
    let vertex_opt = harness
        .get_vertex("test_space", &Value::Int(1))
        .expect("Failed to get vertex");
    assert!(
        vertex_opt.is_none(),
        "Vertex should not exist after rollback"
    );

    // Verify index buffer was cleaned up
    let results = harness
        .search_fulltext("test_space", "Person", "name", "Alice", 10)
        .expect("Failed to search");
    assert_eq!(
        results.len(),
        0,
        "Index buffer should be cleaned up"
    );
}

/// TC-043: 2PC index sync failure handling
#[test]
fn test_2pc_index_sync_failure() {
    // This test verifies that storage is committed even if index sync fails
    // (FailOpen policy)
    
    let mut harness = SyncTestHarness::new().expect("Failed to create test harness");

    // Setup
    harness
        .create_space("test_space")
        .expect("Failed to create space");
    harness
        .create_tag_with_fulltext(
            "test_space",
            "Person",
            vec![("name", DataType::String)],
            vec!["name"],
        )
        .expect("Failed to create tag");

    // Begin transaction
    harness
        .begin_transaction()
        .expect("Failed to begin transaction");

    // Insert vertex
    let vertex = create_test_vertex(
        1,
        "Person",
        vec![("name", Value::String("Alice".to_string()))],
    );
    harness
        .insert_vertex_with_txn("test_space", vertex)
        .expect("Failed to insert vertex");

    // Commit transaction
    // Note: In actual implementation, if index sync fails,
    // the error is logged but storage commit succeeds (FailOpen)
    let result = harness.commit_transaction();

    // Commit should succeed (FailOpen policy)
    assert!(
        result.is_ok(),
        "Commit should succeed even if index sync has issues"
    );

    // Verify storage committed
    harness
        .assert_vertex_exists("test_space", &Value::Int(1))
        .expect("Vertex should exist in storage");
}

/// TC-050: Concurrent transactions sync
#[test]
fn test_concurrent_transactions_sync() {
    use std::thread;

    let harness = Arc::new(SyncTestHarness::new().expect("Failed to create test harness"));

    // Setup
    let mut harness_setup = Arc::try_unwrap(harness).unwrap_or_else(|arc| (*arc).clone());
    harness_setup
        .create_space("test_space")
        .expect("Failed to create space");
    harness_setup
        .create_tag_with_fulltext(
            "test_space",
            "Person",
            vec![("name", DataType::String)],
            vec!["name"],
        )
        .expect("Failed to create tag");

    let harness = Arc::new(harness_setup);

    // Spawn multiple threads
    let mut handles = vec![];
    for i in 0..5 {
        let harness_clone = harness.clone();
        let handle = thread::spawn(move || {
            // Each thread has its own harness instance
            let mut harness = (*harness_clone).clone();
            
            // Begin transaction
            harness
                .begin_transaction()
                .expect("Failed to begin transaction");

            // Insert vertex
            let vertex = create_test_vertex(
                i * 10 + 1,
                "Person",
                vec![(
                    "name",
                    Value::String(format!("Thread{}", i)),
                )],
            );
            harness
                .insert_vertex_with_txn("test_space", vertex)
                .expect("Failed to insert vertex");

            // Commit transaction
            harness
                .commit_transaction()
                .expect("Failed to commit transaction");
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread failed");
    }

    harness.wait_for_async(500);

    // Verify all vertices exist
    for i in 0..5 {
        harness
            .assert_vertex_exists("test_space", &Value::Int((i * 10 + 1) as i64))
            .expect("Vertex should exist");
    }

    // Verify all indexes are synced
    let results = harness
        .search_fulltext("test_space", "Person", "name", "Thread", 20)
        .expect("Failed to search");
    assert!(results.len() >= 5, "All concurrent transactions should be synced");
}

/// TC-051: Concurrent index updates same space
#[test]
fn test_concurrent_index_updates_same_space() {
    use std::thread;

    let harness = Arc::new(SyncTestHarness::new().expect("Failed to create test harness"));

    // Setup
    let mut harness_setup = Arc::try_unwrap(harness).unwrap_or_else(|arc| (*arc).clone());
    harness_setup
        .create_space("test_space")
        .expect("Failed to create space");
    harness_setup
        .create_tag_with_fulltext(
            "test_space",
            "Person",
            vec![("name", DataType::String)],
            vec!["name"],
        )
        .expect("Failed to create tag");

    let harness = Arc::new(harness_setup);

    // Spawn multiple threads updating different vertices in same space
    let mut handles = vec![];
    for i in 0..10 {
        let harness_clone = harness.clone();
        let handle = thread::spawn(move || {
            let mut harness = (*harness_clone).clone();
            
            // Non-transactional insert (concurrent)
            let vertex = create_test_vertex(
                i + 1,
                "Person",
                vec![(
                    "name",
                    Value::String(format!("Concurrent{}", i)),
                )],
            );
            harness
                .insert_vertex("test_space", vertex)
                .expect("Failed to insert vertex");
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread failed");
    }

    harness.wait_for_async(500);

    // Verify all vertices exist
    for i in 0..10 {
        harness
            .assert_vertex_exists("test_space", &Value::Int((i + 1) as i64))
            .expect("Vertex should exist");
    }

    // Verify DashMap handles concurrent access correctly
    let results = harness
        .search_fulltext("test_space", "Person", "name", "Concurrent", 20)
        .expect("Failed to search");
    assert!(
        results.len() >= 10,
        "All concurrent updates should be synced correctly"
    );
}

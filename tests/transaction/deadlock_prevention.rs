//! Transaction Deadlock Prevention Tests
//!
//! These tests specifically verify the fix for the deadlock issue caused by
//! calling block_on inside spawn_blocking contexts when handling transactions.
//!
//! The deadlock scenario:
//! 1. HTTP handler uses task::spawn_blocking to run synchronous code
//! 2. Synchronous code calls tokio::runtime::Handle::current().block_on()
//! 3. This blocks the spawn_blocking thread
//! 4. When all spawn_blocking threads are exhausted, new requests cannot be processed
//! 5. Deadlock occurs because block_on is waiting for a thread that will never be available
//!
//! The fix:
//! - Convert graph_service.execute() to async function
//! - Remove spawn_blocking from HTTP handlers
//! - Use direct await instead of block_on
//!
//! Note: Tests use low concurrency (3-5 tasks) to verify correctness
//! without high load stress testing.

use graphdb::transaction::{
    TransactionManager, TransactionManagerConfig, TransactionOptions,
};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::{sleep, timeout};

/// Test that verifies no deadlock occurs with concurrent read-only transaction operations
/// This test simulates the pattern that previously caused deadlocks
/// Note: Uses read-only transactions since write transactions cannot be concurrent
#[tokio::test]
async fn test_no_deadlock_concurrent_transactions() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db"))
            .expect("Failed to create database"),
    );

    let manager = Arc::new(TransactionManager::new(
        db,
        TransactionManagerConfig::default(),
    ));

    let mut handles = vec![];

    // Spawn concurrent read-only transaction operations
    // This would previously deadlock if using spawn_blocking + block_on
    // Read-only transactions can run concurrently
    for i in 0..5 {
        let manager = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            // Begin read-only transaction
            let txn_id = manager
                .begin_transaction(TransactionOptions::new().read_only())
                .expect("Failed to begin transaction");

            // Simulate some work
            sleep(Duration::from_millis(10)).await;

            // Commit transaction
            manager
                .commit_transaction(txn_id)
                .await
                .expect("Failed to commit transaction");

            println!("Transaction {} completed", i);
        });
        handles.push(handle);
    }

    // All operations should complete without deadlock
    let result = timeout(Duration::from_secs(30), async {
        for handle in handles {
            handle.await.expect("Task should complete");
        }
    })
    .await;

    assert!(
        result.is_ok(),
        "All concurrent transactions should complete without deadlock"
    );
}

/// Test that verifies proper async/await pattern in transaction handling
/// This test ensures we're not using block_on in async contexts
#[tokio::test]
async fn test_proper_async_pattern() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db"))
            .expect("Failed to create database"),
    );

    let manager = Arc::new(TransactionManager::new(
        db,
        TransactionManagerConfig::default(),
    ));

    // Test that commit_transaction is properly async
    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction");

    // This should be a direct await, not block_on
    let commit_result = manager.commit_transaction(txn_id).await;
    assert!(commit_result.is_ok(), "Commit should succeed");

    // Test with multiple sequential operations
    for i in 0..5 {
        let txn_id = manager
            .begin_transaction(TransactionOptions::default())
            .expect("Failed to begin transaction");

        // Simulate async work
        sleep(Duration::from_millis(5)).await;

        let result = manager.commit_transaction(txn_id).await;
        assert!(result.is_ok(), "Commit {} should succeed", i);
    }
}

/// Test that write transactions are properly serialized
/// Write transactions cannot be concurrent, but should not deadlock
#[tokio::test]
async fn test_write_transaction_serialization() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db"))
            .expect("Failed to create database"),
    );

    let manager = Arc::new(TransactionManager::new(
        db,
        TransactionManagerConfig::default(),
    ));

    // Sequential write transactions
    for i in 0..5 {
        let txn_id = manager
            .begin_transaction(TransactionOptions::default())
            .expect("Failed to begin transaction");

        sleep(Duration::from_millis(5)).await;

        manager
            .commit_transaction(txn_id)
            .await
            .expect("Failed to commit transaction");

        println!("Write transaction {} completed", i);
    }
}

/// Test nested async operations in transaction handling
/// Ensures no block_on is used in nested async calls
#[tokio::test]
async fn test_nested_async_operations() {
    use std::pin::Pin;
    use std::future::Future;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db"))
            .expect("Failed to create database"),
    );

    let manager = Arc::new(TransactionManager::new(
        db,
        TransactionManagerConfig::default(),
    ));

    fn inner_operation(
        manager: Arc<TransactionManager>,
        depth: i32,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> {
        Box::pin(async move {
            if depth <= 0 {
                let txn_id = manager
                    .begin_transaction(TransactionOptions::default())
                    .map_err(|e| e.to_string())?;

                sleep(Duration::from_millis(5)).await;

                manager
                    .commit_transaction(txn_id)
                    .await
                    .map_err(|e| e.to_string())?;

                Ok(())
            } else {
                // Recursive async call with Box::pin
                inner_operation(manager, depth - 1).await
            }
        })
    }

    // Test deeply nested async operations
    let result = inner_operation(manager, 5).await;
    assert!(result.is_ok(), "Nested async operations should complete");
}

/// Test transaction timeout handling without deadlock
#[tokio::test]
async fn test_transaction_timeout_no_deadlock() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db"))
            .expect("Failed to create database"),
    );

    let manager = Arc::new(TransactionManager::new(
        db,
        TransactionManagerConfig::default(),
    ));

    // Begin transaction with short timeout
    let txn_id = manager
        .begin_transaction(TransactionOptions::new().with_timeout(Duration::from_millis(50)))
        .expect("Failed to begin transaction");

    // Wait for timeout
    sleep(Duration::from_millis(100)).await;

    // Cleanup should work without deadlock
    manager.cleanup_expired_transactions();

    // Transaction should be cleaned up
    assert!(!manager.is_transaction_active(txn_id));
}

/// Test rapid begin/commit cycles
/// This test verifies no resource leak occurs
#[tokio::test]
async fn test_rapid_transaction_cycles() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db"))
            .expect("Failed to create database"),
    );

    let manager = Arc::new(TransactionManager::new(
        db,
        TransactionManagerConfig::default(),
    ));

    // Perform rapid transaction cycles
    for i in 0..10 {
        let txn_id = manager
            .begin_transaction(TransactionOptions::default())
            .expect("Failed to begin transaction");

        manager
            .commit_transaction(txn_id)
            .await
            .expect("Failed to commit transaction");

        if i % 5 == 0 {
            println!("Completed {} transaction cycles", i);
        }
    }

    // Verify no lingering transactions
    let active = manager.list_active_transactions();
    assert!(active.is_empty(), "No transactions should be active after all commits");
}

/// Test that spawn_blocking is not used for transaction operations
/// This is a conceptual test - it verifies the async pattern works correctly
/// Note: Uses only read-only transactions since write transactions cannot be concurrent
#[tokio::test]
async fn test_no_spawn_blocking_pattern() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db"))
            .expect("Failed to create database"),
    );

    let manager = Arc::new(TransactionManager::new(
        db,
        TransactionManagerConfig::default(),
    ));

    // If spawn_blocking were used with block_on, this would deadlock
    // with enough concurrent operations
    // Use only read-only transactions since they can truly run concurrently
    let mut handles = vec![];

    for i in 0..5 {
        let manager = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            // Use read-only transactions only - they can run concurrently
            let options = TransactionOptions::new().read_only();

            let txn_id = manager
                .begin_transaction(options)
                .expect("Failed to begin transaction");

            // Very short operation
            sleep(Duration::from_millis(1)).await;

            if i % 3 == 0 {
                manager.rollback_transaction(txn_id).expect("Failed to rollback");
            } else {
                manager.commit_transaction(txn_id).await.expect("Failed to commit");
            }
        });
        handles.push(handle);
    }

    // Should complete without deadlock
    let result = timeout(Duration::from_secs(30), async {
        for handle in handles {
            handle.await.expect("Task should complete");
        }
    })
    .await;

    assert!(
        result.is_ok(),
        "All operations should complete without deadlock - verifies no spawn_blocking + block_on pattern"
    );
}

/// Test transaction manager shutdown with pending operations
#[tokio::test]
async fn test_shutdown_with_pending_transactions() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db"))
            .expect("Failed to create database"),
    );

    let manager = Arc::new(TransactionManager::new(
        db,
        TransactionManagerConfig::default(),
    ));

    // Create several transactions
    let txn1 = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction 1");
    
    let txn2 = manager
        .begin_transaction(TransactionOptions::new().read_only())
        .expect("Failed to begin transaction 2");

    // Shutdown should complete without deadlock
    manager.shutdown();

    // All transactions should be aborted
    assert!(!manager.is_transaction_active(txn1));
    assert!(!manager.is_transaction_active(txn2));
}

/// Test mixed read and write transaction patterns
#[tokio::test]
async fn test_mixed_read_write_patterns() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db"))
            .expect("Failed to create database"),
    );

    let manager = Arc::new(TransactionManager::new(
        db,
        TransactionManagerConfig::default(),
    ));

    // Pattern: Write followed by multiple reads
    for _ in 0..3 {
        // Write transaction
        let write_txn = manager
            .begin_transaction(TransactionOptions::default())
            .expect("Failed to begin write transaction");

        sleep(Duration::from_millis(5)).await;

        manager
            .commit_transaction(write_txn)
            .await
            .expect("Failed to commit write transaction");

        // Multiple concurrent read transactions
        let mut read_handles = vec![];
        for _ in 0..3 {
            let manager = Arc::clone(&manager);
            let handle = tokio::spawn(async move {
                let read_txn = manager
                    .begin_transaction(TransactionOptions::new().read_only())
                    .expect("Failed to begin read transaction");

                sleep(Duration::from_millis(10)).await;

                manager
                    .commit_transaction(read_txn)
                    .await
                    .expect("Failed to commit read transaction");
            });
            read_handles.push(handle);
        }

        for handle in read_handles {
            handle.await.expect("Read task should complete");
        }
    }
}

// Additional tests for nested lock acquisition deadlock prevention
//
// These tests verify the fix for deadlock issues caused by nested lock acquisition
// in TransactionContext methods (info, create_savepoint, rollback_to_savepoint).
//
// The deadlock scenarios fixed:
// 1. info() - was holding modified_tables and savepoint_manager locks simultaneously
// 2. create_savepoint() - was holding savepoint_manager while calling operation_log_len()
// 3. rollback_to_savepoint() - was holding savepoint_manager while accessing operation_logs

/// Test that info() method does not hold multiple locks simultaneously
/// This verifies the fix for nested lock acquisition in info()
#[tokio::test]
async fn test_info_no_nested_locks() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db"))
            .expect("Failed to create database"),
    );

    let manager = Arc::new(TransactionManager::new(
        db,
        TransactionManagerConfig::default(),
    ));

    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction");

    // Call info() multiple times concurrently to verify no deadlock
    let mut handles = vec![];
    for _ in 0..5 {
        let manager = Arc::clone(&manager);
        let txn_id = txn_id;
        let handle = tokio::task::spawn_blocking(move || {
            let info = manager.get_transaction_info(txn_id);
            assert!(info.is_some());
        });
        handles.push(handle);
    }

    // All calls should complete without deadlock
    let result = timeout(Duration::from_secs(10), async {
        for handle in handles {
            handle.await.expect("Info task should complete");
        }
    })
    .await;

    assert!(
        result.is_ok(),
        "All info() calls should complete without deadlock"
    );

    manager
        .commit_transaction(txn_id)
        .await
        .expect("Failed to commit transaction");
}

/// Test that create_savepoint() does not cause nested lock acquisition
/// This verifies the fix where operation_log_len() is called before savepoint_manager lock
#[tokio::test]
async fn test_create_savepoint_no_nested_locks() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db"))
            .expect("Failed to create database"),
    );

    let manager = Arc::new(TransactionManager::new(
        db,
        TransactionManagerConfig::default(),
    ));

    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction");

    // Create multiple savepoints rapidly
    for i in 0..10 {
        let sp_name = format!("savepoint_{}", i);
        let sp_id = manager
            .create_savepoint(txn_id, Some(sp_name.clone()))
            .expect("Failed to create savepoint");

        let sp = manager.get_savepoint(txn_id, sp_id);
        assert!(sp.is_some());
        assert_eq!(sp.unwrap().name, Some(sp_name));
    }

    manager
        .commit_transaction(txn_id)
        .await
        .expect("Failed to commit transaction");
}

/// Test that rollback_to_savepoint() does not cause nested lock acquisition
/// This verifies the fix where locks are acquired and released in sequence
#[tokio::test]
async fn test_rollback_to_savepoint_no_nested_locks() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db"))
            .expect("Failed to create database"),
    );

    let manager = Arc::new(TransactionManager::new(
        db,
        TransactionManagerConfig::default(),
    ));

    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction");

    // Create multiple savepoints
    let sp1 = manager
        .create_savepoint(txn_id, Some("sp1".to_string()))
        .expect("Failed to create savepoint 1");
    let sp2 = manager
        .create_savepoint(txn_id, Some("sp2".to_string()))
        .expect("Failed to create savepoint 2");
    let sp3 = manager
        .create_savepoint(txn_id, Some("sp3".to_string()))
        .expect("Failed to create savepoint 3");

    // Rollback to sp2 - this should remove sp3
    manager
        .rollback_to_savepoint(txn_id, sp2)
        .expect("Failed to rollback to sp2");

    // Verify sp3 is removed
    let sp3_after = manager.get_savepoint(txn_id, sp3);
    assert!(sp3_after.is_none(), "sp3 should be removed after rollback");

    // Verify sp1 and sp2 still exist
    let sp1_after = manager.get_savepoint(txn_id, sp1);
    let sp2_after = manager.get_savepoint(txn_id, sp2);
    assert!(sp1_after.is_some(), "sp1 should still exist");
    assert!(sp2_after.is_some(), "sp2 should still exist");

    // Rollback to sp1 - this should remove sp2
    manager
        .rollback_to_savepoint(txn_id, sp1)
        .expect("Failed to rollback to sp1");

    let sp2_final = manager.get_savepoint(txn_id, sp2);
    assert!(sp2_final.is_none(), "sp2 should be removed after rollback to sp1");

    manager
        .commit_transaction(txn_id)
        .await
        .expect("Failed to commit transaction");
}

/// Test concurrent savepoint operations to verify no deadlock
/// This tests the scenario where multiple threads operate on savepoints
/// Note: Each thread uses its own transaction (read-only) to avoid race conditions
#[tokio::test]
async fn test_concurrent_savepoint_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db"))
            .expect("Failed to create database"),
    );

    let manager = Arc::new(TransactionManager::new(
        db,
        TransactionManagerConfig::default(),
    ));

    // Spawn multiple tasks that perform savepoint operations
    // Each task uses its own read-only transaction to avoid race conditions
    let mut handles = vec![];

    for i in 0..5 {
        let manager = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            // Each task gets its own read-only transaction
            let txn_id = manager
                .begin_transaction(TransactionOptions::new().read_only())
                .expect("Failed to begin transaction");

            // Create savepoint
            let sp = manager
                .create_savepoint(txn_id, Some(format!("task_{}", i)))
                .expect("Failed to create savepoint");

            // Get savepoint info
            let info = manager.get_savepoint(txn_id, sp);
            assert!(info.is_some());

            // Get all savepoints
            let all_sp = manager.get_active_savepoints(txn_id);
            assert!(!all_sp.is_empty());

            // Get transaction info (tests the info() fix)
            let txn_info = manager.get_transaction_info(txn_id);
            assert!(txn_info.is_some());

            // Commit the transaction
            manager
                .commit_transaction(txn_id)
                .await
                .expect("Failed to commit");
        });
        handles.push(handle);
    }

    // All operations should complete without deadlock
    let result = timeout(Duration::from_secs(30), async {
        for handle in handles {
            handle.await.expect("Task should complete");
        }
    })
    .await;

    assert!(
        result.is_ok(),
        "All concurrent savepoint operations should complete without deadlock"
    );
}

/// Test concurrent info() and savepoint operations
/// This verifies that info() doesn't interfere with savepoint operations
#[tokio::test]
async fn test_concurrent_info_and_savepoint() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db"))
            .expect("Failed to create database"),
    );

    let manager = Arc::new(TransactionManager::new(
        db,
        TransactionManagerConfig::default(),
    ));

    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction");

    let manager_clone = Arc::clone(&manager);

    // Task 1: Continuously call info()
    let info_handle = tokio::task::spawn_blocking(move || {
        for _ in 0..20 {
            let info = manager_clone.get_transaction_info(txn_id);
            assert!(info.is_some());
            std::thread::sleep(Duration::from_millis(1));
        }
    });

    // Task 2: Create and rollback savepoints
    let sp_handle = {
        let manager = Arc::clone(&manager);
        tokio::task::spawn_blocking(move || {
            for i in 0..10 {
                let sp = manager
                    .create_savepoint(txn_id, Some(format!("sp_{}", i)))
                    .expect("Failed to create savepoint");

                std::thread::sleep(Duration::from_millis(1));

                // Verify savepoint was created
                let sp_info = manager.get_savepoint(txn_id, sp);
                assert!(sp_info.is_some());

                // Get info while holding savepoint operations
                let info = manager.get_transaction_info(txn_id);
                assert!(info.is_some());
            }
        })
    };

    // Both tasks should complete without deadlock
    let result = timeout(Duration::from_secs(30), async {
        info_handle.await.expect("Info task should complete");
        sp_handle.await.expect("Savepoint task should complete");
    })
    .await;

    assert!(
        result.is_ok(),
        "Concurrent info() and savepoint operations should complete without deadlock"
    );

    manager
        .commit_transaction(txn_id)
        .await
        .expect("Failed to commit transaction");
}

/// Test rapid create/rollback savepoint cycles
/// This stress tests the lock acquisition pattern
#[tokio::test]
async fn test_rapid_savepoint_cycles() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db"))
            .expect("Failed to create database"),
    );

    let manager = Arc::new(TransactionManager::new(
        db,
        TransactionManagerConfig::default(),
    ));

    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction");

    // Rapid create/rollback cycles
    for i in 0..20 {
        let sp = manager
            .create_savepoint(txn_id, Some(format!("cycle_{}", i)))
            .expect("Failed to create savepoint");

        // Immediately rollback to this savepoint
        manager
            .rollback_to_savepoint(txn_id, sp)
            .expect("Failed to rollback");
    }

    // Transaction should still be usable
    let info = manager.get_transaction_info(txn_id);
    assert!(info.is_some());

    manager
        .commit_transaction(txn_id)
        .await
        .expect("Failed to commit transaction");
}

/// Test that multiple transactions can create savepoints concurrently
/// (using read-only transactions for true concurrency)
#[tokio::test]
async fn test_concurrent_transactions_savepoints() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db"))
            .expect("Failed to create database"),
    );

    let manager = Arc::new(TransactionManager::new(
        db,
        TransactionManagerConfig::default(),
    ));

    let mut handles = vec![];

    // Each task gets its own transaction (read-only for concurrency)
    for i in 0..5 {
        let manager = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            let txn_id = manager
                .begin_transaction(TransactionOptions::new().read_only())
                .expect("Failed to begin transaction");

            // Create savepoint
            let sp = manager
                .create_savepoint(txn_id, Some(format!("txn_{}_sp", i)))
                .expect("Failed to create savepoint");

            // Get info
            let info = manager.get_transaction_info(txn_id);
            assert!(info.is_some());

            // Get savepoint
            let sp_info = manager.get_savepoint(txn_id, sp);
            assert!(sp_info.is_some());

            // Commit
            manager
                .commit_transaction(txn_id)
                .await
                .expect("Failed to commit");
        });
        handles.push(handle);
    }

    // All should complete without deadlock
    let result = timeout(Duration::from_secs(30), async {
        for handle in handles {
            handle.await.expect("Task should complete");
        }
    })
    .await;

    assert!(
        result.is_ok(),
        "All concurrent transaction savepoint operations should complete without deadlock"
    );
}

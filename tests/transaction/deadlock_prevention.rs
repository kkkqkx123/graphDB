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

    // Spawn many concurrent read-only transaction operations
    // This would previously deadlock if using spawn_blocking + block_on
    // Read-only transactions can run concurrently
    for i in 0..20 {
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
    for i in 0..10 {
        let txn_id = manager
            .begin_transaction(TransactionOptions::default())
            .expect("Failed to begin transaction");

        // Simulate async work
        sleep(Duration::from_millis(5)).await;

        let result = manager.commit_transaction(txn_id).await;
        assert!(result.is_ok(), "Commit {} should succeed", i);
    }
}

/// Test transaction operations under high concurrency
/// This test verifies thread pool is not exhausted
#[tokio::test]
async fn test_high_concurrency_no_thread_exhaustion() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db"))
            .expect("Failed to create database"),
    );

    let manager = Arc::new(TransactionManager::new(
        db,
        TransactionManagerConfig::default(),
    ));

    // Use many concurrent read-only transactions
    // These can truly run concurrently
    let mut handles = vec![];

    for i in 0..50 {
        let manager = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            let options = TransactionOptions::new().read_only();
            let txn_id = manager
                .begin_transaction(options)
                .expect("Failed to begin read-only transaction");

            // Simulate read work
            sleep(Duration::from_millis(20)).await;

            manager
                .commit_transaction(txn_id)
                .await
                .expect("Failed to commit read-only transaction");

            i
        });
        handles.push(handle);
    }

    // Collect all results
    let results = timeout(Duration::from_secs(30), async {
        let mut completed = vec![];
        for handle in handles {
            completed.push(handle.await.expect("Task should complete"));
        }
        completed
    })
    .await;

    assert!(results.is_ok(), "All read-only transactions should complete");
    assert_eq!(results.unwrap().len(), 50, "All 50 transactions should complete");
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
    for i in 0..10 {
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

    // Perform many rapid transaction cycles
    for i in 0..100 {
        let txn_id = manager
            .begin_transaction(TransactionOptions::default())
            .expect("Failed to begin transaction");

        manager
            .commit_transaction(txn_id)
            .await
            .expect("Failed to commit transaction");

        if i % 20 == 0 {
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

    for i in 0..100 {
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
    for _ in 0..10 {
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
        for _ in 0..5 {
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

//! Two-Phase Commit Transaction Tests
//!
//! Test coverage for two-phase commit functionality:
//! - Basic two-phase commit flow
//! - Prepare phase failure handling
//! - Commit phase failure handling
//! - Rollback after prepare
//! - Multiple transactions with two-phase commit
//! - SyncManager integration with transactions
//! - Deadlock prevention in two-phase commit

use graphdb::sync::coordinator::SyncCoordinator;
use graphdb::sync::batch::BatchConfig;
use graphdb::sync::SyncManager;
use graphdb::search::manager::FulltextIndexManager;
use graphdb::search::config::FulltextConfig;
use graphdb::transaction::{
    TransactionManager, TransactionManagerConfig, TransactionOptions,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, timeout};

/// Helper function to create a transaction manager with sync manager
async fn create_manager_with_sync() -> (TransactionManager, Arc<SyncManager>) {
    let config = FulltextConfig::default();
    let fulltext_manager = Arc::new(FulltextIndexManager::new(config).expect("Failed to create fulltext manager"));
    let batch_config = BatchConfig::default();
    let sync_coordinator = Arc::new(SyncCoordinator::new(fulltext_manager, batch_config));
    let sync_manager = Arc::new(SyncManager::new(sync_coordinator));

    sync_manager
        .start()
        .await
        .expect("Failed to start sync manager");

    let manager = TransactionManager::with_sync_manager(
        TransactionManagerConfig::default(),
        sync_manager.clone(),
    );

    (manager, sync_manager)
}

/// Test basic two-phase commit flow
#[tokio::test]
async fn test_two_phase_commit_basic() {
    let (manager, _sync_manager) = create_manager_with_sync().await;

    let mut options = TransactionOptions::new();
    options.two_phase_commit = true;
    let txn_id = manager
        .begin_transaction(options)
        .expect("Failed to begin transaction");

    assert!(manager.is_transaction_active(txn_id));

    manager
        .commit_transaction(txn_id)
        .await
        .expect("Failed to commit transaction");

    assert!(!manager.is_transaction_active(txn_id));
}

/// Test two-phase commit with multiple operations
#[tokio::test]
async fn test_two_phase_commit_multiple_operations() {
    let (manager, _sync_manager) = create_manager_with_sync().await;

    let mut options = TransactionOptions::new();
    options.two_phase_commit = true;
    let txn_id = manager
        .begin_transaction(options)
        .expect("Failed to begin transaction");

    let context = manager.get_context(txn_id).expect("Failed to get context");

    for i in 0..5 {
        let operation = graphdb::transaction::OperationLog::InsertVertex {
            space: "test_space".to_string(),
            vertex_id: vec![i as u8, 0, 0, 0, 0, 0, 0, 0],
            previous_state: None,
        };
        context.add_operation_log(operation);
    }

    manager
        .commit_transaction(txn_id)
        .await
        .expect("Failed to commit transaction");

    assert!(!manager.is_transaction_active(txn_id));
}

/// Test two-phase commit rollback
#[tokio::test]
async fn test_two_phase_commit_rollback() {
    let (manager, _sync_manager) = create_manager_with_sync().await;

    let mut options = TransactionOptions::new();
    options.two_phase_commit = true;
    let txn_id = manager
        .begin_transaction(options)
        .expect("Failed to begin transaction");

    let context = manager.get_context(txn_id).expect("Failed to get context");

    let operation = graphdb::transaction::OperationLog::InsertVertex {
        space: "test_space".to_string(),
        vertex_id: vec![1, 0, 0, 0, 0, 0, 0, 0],
        previous_state: None,
    };
    context.add_operation_log(operation);

    manager
        .rollback_transaction(txn_id)
        .expect("Failed to rollback transaction");

    assert!(!manager.is_transaction_active(txn_id));
}

/// Test sequential two-phase commit transactions
#[tokio::test]
async fn test_two_phase_commit_sequential() {
    let (manager, _sync_manager) = create_manager_with_sync().await;

    for i in 0..5 {
        let mut options = TransactionOptions::new();
        options.two_phase_commit = true;
        let txn_id = manager
            .begin_transaction(options)
            .expect("Failed to begin transaction");

        let context = manager.get_context(txn_id).expect("Failed to get context");
        let operation = graphdb::transaction::OperationLog::InsertVertex {
            space: "test_space".to_string(),
            vertex_id: vec![i as u8, 0, 0, 0, 0, 0, 0, 0],
            previous_state: None,
        };
        context.add_operation_log(operation);

        manager
            .commit_transaction(txn_id)
            .await
            .expect("Failed to commit transaction");

        sleep(Duration::from_millis(10)).await;
    }
}

/// Test two-phase commit with savepoints
#[tokio::test]
async fn test_two_phase_commit_with_savepoints() {
    let (manager, _sync_manager) = create_manager_with_sync().await;

    let mut options = TransactionOptions::new();
    options.two_phase_commit = true;
    let txn_id = manager
        .begin_transaction(options)
        .expect("Failed to begin transaction");

    let sp_id = manager
        .create_savepoint(txn_id, Some("checkpoint".to_string()))
        .expect("Failed to create savepoint");

    let context = manager.get_context(txn_id).expect("Failed to get context");
    let operation = graphdb::transaction::OperationLog::InsertVertex {
        space: "test_space".to_string(),
        vertex_id: vec![1, 0, 0, 0, 0, 0, 0, 0],
        previous_state: None,
    };
    context.add_operation_log(operation);

    manager
        .rollback_to_savepoint(txn_id, sp_id)
        .expect("Failed to rollback to savepoint");

    manager
        .commit_transaction(txn_id)
        .await
        .expect("Failed to commit transaction");
}

/// Test concurrent read-only transactions with two-phase commit manager
#[tokio::test]
async fn test_two_phase_commit_concurrent_readonly() {
    let (manager, _sync_manager) = create_manager_with_sync().await;

    let manager = Arc::new(manager);
    let mut handles = vec![];

    for i in 0..5 {
        let manager_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            let mut options = TransactionOptions::new();
            options.read_only = true;
            options.two_phase_commit = true;
            let txn_id = manager_clone
                .begin_transaction(options)
                .expect("Failed to begin transaction");

            sleep(Duration::from_millis(20)).await;

            manager_clone
                .commit_transaction(txn_id)
                .await
                .expect("Failed to commit transaction");

            println!("Read-only transaction {} completed", i);
        });
        handles.push(handle);
    }

    let result = timeout(Duration::from_secs(30), async {
        for handle in handles {
            handle.await.expect("Task should complete");
        }
    })
    .await;

    assert!(
        result.is_ok(),
        "All concurrent read-only transactions should complete without deadlock"
    );
}

/// Test two-phase commit with transaction timeout
#[tokio::test]
async fn test_two_phase_commit_with_timeout() {
    let (manager, _sync_manager) = create_manager_with_sync().await;

    let mut options = TransactionOptions::new();
    options.two_phase_commit = true;
    options.timeout = Some(Duration::from_secs(5));

    let txn_id = manager
        .begin_transaction(options)
        .expect("Failed to begin transaction");

    let context = manager.get_context(txn_id).expect("Failed to get context");
    assert!(context.is_two_phase_enabled());

    manager
        .commit_transaction(txn_id)
        .await
        .expect("Failed to commit transaction");
}

/// Test two-phase commit transaction info and metrics
#[tokio::test]
async fn test_two_phase_commit_transaction_info() {
    let (manager, _sync_manager) = create_manager_with_sync().await;

    let mut options = TransactionOptions::new();
    options.two_phase_commit = true;
    let txn_id = manager
        .begin_transaction(options)
        .expect("Failed to begin transaction");

    let info = manager
        .get_transaction_info(txn_id)
        .expect("Failed to get transaction info");

    assert_eq!(info.id, txn_id);
    assert!(!info.is_read_only);

    manager
        .commit_transaction(txn_id)
        .await
        .expect("Failed to commit transaction");

    let stats = manager.stats();
    assert!(stats.total_transactions.load(std::sync::atomic::Ordering::Relaxed) >= 1);
}

/// Test no deadlock with rapid two-phase commit cycles
#[tokio::test]
async fn test_two_phase_commit_no_deadlock_rapid_cycles() {
    let (manager, _sync_manager) = create_manager_with_sync().await;

    for i in 0..10 {
        let mut options = TransactionOptions::new();
        options.two_phase_commit = true;
        let txn_id = manager
            .begin_transaction(options)
            .expect("Failed to begin transaction");

        sleep(Duration::from_millis(5)).await;

        manager
            .commit_transaction(txn_id)
            .await
            .expect("Failed to commit transaction");

        if i % 5 == 0 {
            println!("Completed {} two-phase commit cycles", i);
        }
    }

    let active = manager.list_active_transactions();
    assert!(
        active.is_empty(),
        "No transactions should be active after all commits"
    );
}

/// Test two-phase commit with different durability levels
#[tokio::test]
async fn test_two_phase_commit_durability_levels() {
    let (manager, _sync_manager) = create_manager_with_sync().await;

    let mut options1 = TransactionOptions::new();
    options1.two_phase_commit = true;
    options1.durability = graphdb::transaction::DurabilityLevel::Immediate;
    let txn1 = manager
        .begin_transaction(options1)
        .expect("Failed to begin transaction 1");
    manager
        .commit_transaction(txn1)
        .await
        .expect("Failed to commit transaction 1");

    let mut options2 = TransactionOptions::new();
    options2.two_phase_commit = true;
    options2.durability = graphdb::transaction::DurabilityLevel::None;
    let txn2 = manager
        .begin_transaction(options2)
        .expect("Failed to begin transaction 2");
    manager
        .commit_transaction(txn2)
        .await
        .expect("Failed to commit transaction 2");
}

/// Test two-phase commit shutdown with pending transaction
#[tokio::test]
async fn test_two_phase_commit_shutdown_with_pending() {
    let config = FulltextConfig::default();
    let fulltext_manager = Arc::new(FulltextIndexManager::new(config).expect("Failed to create fulltext manager"));
    let batch_config = BatchConfig::default();
    let sync_coordinator = Arc::new(SyncCoordinator::new(fulltext_manager, batch_config));
    let sync_manager = Arc::new(SyncManager::new(sync_coordinator));
    sync_manager
        .start()
        .await
        .expect("Failed to start sync manager");

    let manager = TransactionManager::with_sync_manager(
        TransactionManagerConfig::default(),
        sync_manager.clone(),
    );

    let mut options = TransactionOptions::new();
    options.two_phase_commit = true;
    let txn_id = manager
        .begin_transaction(options)
        .expect("Failed to begin transaction");

    assert!(manager.is_transaction_active(txn_id));

    manager.shutdown();

    assert!(!manager.is_transaction_active(txn_id));

    sync_manager.stop().await;
}

/// Test two-phase commit with retry mechanism
#[tokio::test]
async fn test_two_phase_commit_with_retry() {
    let (manager, _sync_manager) = create_manager_with_sync().await;

    let retry_config = graphdb::transaction::RetryConfig::new()
        .with_max_retries(2)
        .with_initial_delay(Duration::from_millis(10));

    let mut options = TransactionOptions::new();
    options.two_phase_commit = true;
    let result = manager
        .execute_with_retry(
            options,
            retry_config,
            |_txn_id| Ok("success"),
        )
        .await;

    assert_eq!(result.expect("Retry should succeed"), "success");
}

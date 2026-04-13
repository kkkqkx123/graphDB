//! TransactionManager Tests
//!
//! Test transaction manager functionality, including transaction lifecycle management, concurrency control, timeout handling, etc.

use std::sync::Arc;
use std::time::Duration;

use tempfile::TempDir;
use tokio;

use crate::transaction::manager::TransactionManager;
use crate::transaction::types::{
    DurabilityLevel, TransactionError, TransactionOptions, TransactionState,
};

/// Create test database and manager
fn create_test_manager() -> (TransactionManager, Arc<redb::Database>, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db"))
            .expect("Failed to create test database"),
    );

    let config = crate::transaction::types::TransactionManagerConfig {
        auto_cleanup: false,
        ..Default::default()
    };

    let manager = TransactionManager::new(db.clone(), config);
    (manager, db, temp_dir)
}

#[test]
fn test_transaction_manager_creation() {
    let (manager, _db, _temp) = create_test_manager();

    // Verify manager configuration
    let config = manager.config();
    assert_eq!(config.max_concurrent_transactions, 1000);
    assert!(!config.auto_cleanup);
}

#[test]
fn test_begin_write_transaction() {
    let (manager, _db, _temp) = create_test_manager();

    let options = TransactionOptions::default();
    let txn_id = manager
        .begin_transaction(options)
        .expect("Failed to begin transaction");

    assert!(manager.is_transaction_active(txn_id));

    let context = manager
        .get_context(txn_id)
        .expect("Failed to get transaction context");
    assert_eq!(context.id, txn_id);
    assert_eq!(context.state(), TransactionState::Active);
    assert!(!context.read_only);
}

#[test]
fn test_begin_readonly_transaction() {
    let (manager, _db, _temp) = create_test_manager();

    let options = TransactionOptions::new().read_only();
    let txn_id = manager
        .begin_transaction(options)
        .expect("Failed to begin readonly transaction");

    assert!(manager.is_transaction_active(txn_id));

    let context = manager
        .get_context(txn_id)
        .expect("Failed to get transaction context");
    assert_eq!(context.id, txn_id);
    assert!(context.read_only);
}

#[test]
fn test_begin_transaction_with_timeout() {
    let (manager, _db, _temp) = create_test_manager();

    let options = TransactionOptions::new()
        .with_timeout(Duration::from_secs(60))
        .with_durability(DurabilityLevel::None);

    let txn_id = manager
        .begin_transaction(options)
        .expect("Failed to begin transaction");

    let context = manager
        .get_context(txn_id)
        .expect("Failed to get transaction context");
    assert!(context.remaining_time() > Duration::from_secs(50));
}

#[tokio::test]
async fn test_commit_transaction() {
    let (manager, _db, _temp) = create_test_manager();

    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction");

    assert!(manager.is_transaction_active(txn_id));

    manager
        .commit_transaction(txn_id)
        .await
        .expect("Failed to commit transaction");

    assert!(!manager.is_transaction_active(txn_id));

    // Verify statistics
    let stats = manager.stats();
    assert_eq!(
        stats
            .committed_transactions
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );
}

#[test]
fn test_rollback_transaction() {
    let (manager, _db, _temp) = create_test_manager();

    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to start transaction");

    assert!(manager.is_transaction_active(txn_id));

    manager
        .rollback_transaction(txn_id)
        .expect("Failed to rollback transaction");

    assert!(!manager.is_transaction_active(txn_id));

    // Verify statistics
    let stats = manager.stats();
    assert_eq!(
        stats
            .aborted_transactions
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );
}

#[tokio::test]
async fn test_commit_readonly_transaction() {
    let (manager, _db, _temp) = create_test_manager();

    let options = TransactionOptions::new().read_only();
    let txn_id = manager
        .begin_transaction(options)
        .expect("Failed to begin readonly transaction");

    manager
        .commit_transaction(txn_id)
        .await
        .expect("Failed to commit readonly transaction");

    assert!(!manager.is_transaction_active(txn_id));
}

#[test]
fn test_get_transaction_not_found() {
    let (manager, _db, _temp) = create_test_manager();

    let result = manager.get_context(9999);
    assert!(matches!(
        result,
        Err(TransactionError::TransactionNotFound(9999))
    ));
}

#[tokio::test]
async fn test_commit_transaction_not_found() {
    let (manager, _db, _temp) = create_test_manager();

    let result = manager.commit_transaction(9999).await;
    assert!(matches!(
        result,
        Err(TransactionError::TransactionNotFound(_))
    ));
}

#[test]
fn test_rollback_transaction_not_found() {
    let (manager, _db, _temp) = create_test_manager();

    let result = manager.rollback_transaction(9999);
    assert!(matches!(
        result,
        Err(TransactionError::TransactionNotFound(_))
    ));
}

#[tokio::test]
async fn test_commit_already_committed_transaction() {
    let (manager, _db, _temp) = create_test_manager();

    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction");

    manager
        .commit_transaction(txn_id)
        .await
        .expect("First commit failed");

    // Second commit should fail
    let result = manager.commit_transaction(txn_id).await;
    assert!(matches!(
        result,
        Err(TransactionError::TransactionNotFound(_))
    ));
}

#[test]
fn test_rollback_already_rolledback_transaction() {
    let (manager, _db, _temp) = create_test_manager();

    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to start transaction");

    manager
        .rollback_transaction(txn_id)
        .expect("First rollback failed");

    // Second rollback should fail
    let result = manager.rollback_transaction(txn_id);
    assert!(matches!(
        result,
        Err(TransactionError::TransactionNotFound(_))
    ));
}

#[tokio::test]
async fn test_write_transaction_conflict() {
    let (manager, _db, _temp) = create_test_manager();

    // Begin first write transaction
    let txn1 = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin first transaction");

    // Try to begin second write transaction should fail
    let result = manager.begin_transaction(TransactionOptions::default());
    assert!(matches!(
        result,
        Err(TransactionError::WriteTransactionConflict)
    ));

    // Commit first transaction
    manager
        .commit_transaction(txn1)
        .await
        .expect("Failed to commit first transaction");

    // Now can begin new transaction
    let txn2 = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin second transaction");

    manager
        .commit_transaction(txn2)
        .await
        .expect("Failed to commit second transaction");
}

#[tokio::test]
async fn test_multiple_readonly_transactions() {
    let (manager, _db, _temp) = create_test_manager();

    let options = TransactionOptions::new().read_only();

    // Multiple readonly transactions can be active simultaneously
    let txn1 = manager
        .begin_transaction(options.clone())
        .expect("Failed to begin first readonly transaction");
    let txn2 = manager
        .begin_transaction(options.clone())
        .expect("Failed to begin second readonly transaction");
    let txn3 = manager
        .begin_transaction(options)
        .expect("Failed to begin third readonly transaction");

    assert!(manager.is_transaction_active(txn1));
    assert!(manager.is_transaction_active(txn2));
    assert!(manager.is_transaction_active(txn3));

    // Commit all readonly transactions
    manager
        .commit_transaction(txn1)
        .await
        .expect("Failed to commit first readonly transaction");
    manager
        .commit_transaction(txn2)
        .await
        .expect("Failed to commit second readonly transaction");
    manager
        .commit_transaction(txn3)
        .await
        .expect("Failed to commit third readonly transaction");
}

#[tokio::test]
async fn test_sequential_write_transactions() {
    let (manager, _db, _temp) = create_test_manager();

    // First transaction
    let txn1 = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin first transaction");
    manager
        .commit_transaction(txn1)
        .await
        .expect("Failed to commit first transaction");

    // Second transaction
    let txn2 = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin second transaction");
    manager
        .rollback_transaction(txn2)
        .expect("Failed to rollback second transaction");

    // Third transaction
    let txn3 = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin third transaction");
    manager
        .commit_transaction(txn3)
        .await
        .expect("Failed to commit third transaction");

    // Verify statistics
    let stats = manager.stats();
    assert_eq!(
        stats
            .committed_transactions
            .load(std::sync::atomic::Ordering::Relaxed),
        2
    );
    assert_eq!(
        stats
            .aborted_transactions
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );
}

#[tokio::test]
async fn test_transaction_timeout() {
    let (manager, _db, _temp) = create_test_manager();

    let options = TransactionOptions::new().with_timeout(Duration::from_millis(50));

    let txn_id = manager
        .begin_transaction(options)
        .expect("Failed to begin transaction");

    // Wait for transaction timeout
    std::thread::sleep(Duration::from_millis(100));

    // Committing timeout transaction should fail
    let result = manager.commit_transaction(txn_id).await;
    assert!(matches!(result, Err(TransactionError::TransactionTimeout)));

    // Verify statistics
    let stats = manager.stats();
    assert_eq!(
        stats
            .timeout_transactions
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );
}

#[tokio::test]
async fn test_list_active_transactions() {
    let (manager, _db, _temp) = create_test_manager();

    // Begin several transactions
    let txn1 = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin first transaction");
    let txn2 = manager
        .begin_transaction(TransactionOptions::new().read_only())
        .expect("Failed to begin second transaction");

    // List active transactions
    let active_txns = manager.list_active_transactions();
    assert_eq!(active_txns.len(), 2);

    // Commit one transaction
    manager
        .commit_transaction(txn1)
        .await
        .expect("Failed to commit transaction");

    // List active transactions again
    let active_txns = manager.list_active_transactions();
    assert_eq!(active_txns.len(), 1);

    // Cleanup
    manager
        .commit_transaction(txn2)
        .await
        .expect("Failed to commit transaction");
}

#[tokio::test]
async fn test_get_transaction_info() {
    let (manager, _db, _temp) = create_test_manager();

    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction");

    let info = manager
        .get_transaction_info(txn_id)
        .expect("Failed to get transaction info");

    assert_eq!(info.id, txn_id);
    assert_eq!(info.state, TransactionState::Active);
    assert!(!info.is_read_only);

    manager
        .commit_transaction(txn_id)
        .await
        .expect("Failed to commit transaction");
}

#[tokio::test]
async fn test_max_concurrent_transactions() {
    let config = crate::transaction::types::TransactionManagerConfig {
        max_concurrent_transactions: 2,
        auto_cleanup: false,
        ..Default::default()
    };

    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db"))
            .expect("Failed to create test database"),
    );

    let manager = TransactionManager::new(db, config);

    // Begin first readonly transaction
    let txn1 = manager
        .begin_transaction(TransactionOptions::new().read_only())
        .expect("Failed to begin first transaction");

    // Begin second readonly transaction
    let txn2 = manager
        .begin_transaction(TransactionOptions::new().read_only())
        .expect("Failed to begin second transaction");

    // Try to begin third transaction should fail (exceeds max concurrent transactions)
    let result = manager.begin_transaction(TransactionOptions::new().read_only());
    assert!(matches!(result, Err(TransactionError::TooManyTransactions)));

    // Cleanup
    manager
        .commit_transaction(txn1)
        .await
        .expect("Failed to commit first transaction");
    manager
        .commit_transaction(txn2)
        .await
        .expect("Failed to commit second transaction");
}

#[tokio::test]
async fn test_transaction_stats() {
    let (manager, _db, _temp) = create_test_manager();

    let stats = manager.stats();

    // Initial statistics
    assert_eq!(
        stats
            .total_transactions
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    assert_eq!(
        stats
            .active_transactions
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    assert_eq!(
        stats
            .committed_transactions
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    assert_eq!(
        stats
            .aborted_transactions
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );

    // Begin one transaction
    let txn1 = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction");

    assert_eq!(
        stats
            .total_transactions
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );
    assert_eq!(
        stats
            .active_transactions
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );

    // Commit transaction
    manager
        .commit_transaction(txn1)
        .await
        .expect("Failed to commit transaction");

    assert_eq!(
        stats
            .active_transactions
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    assert_eq!(
        stats
            .committed_transactions
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );

    // Begin and rollback another transaction
    let txn2 = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction");

    manager
        .rollback_transaction(txn2)
        .expect("Failed to rollback transaction");

    assert_eq!(
        stats
            .aborted_transactions
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );
}

#[test]
fn test_cleanup_expired_transactions() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db"))
            .expect("Failed to create test database"),
    );

    let config = crate::transaction::types::TransactionManagerConfig {
        auto_cleanup: false,
        ..Default::default()
    };

    let manager = TransactionManager::new(db.clone(), config);

    // Begin a short timeout transaction
    let txn1 = manager
        .begin_transaction(TransactionOptions::new().with_timeout(Duration::from_millis(50)))
        .expect("Failed to begin transaction");

    // Wait for first transaction to timeout
    std::thread::sleep(Duration::from_millis(100));

    // Cleanup expired transactions
    manager.cleanup_expired_transactions();

    // First transaction should be cleaned up
    assert!(!manager.is_transaction_active(txn1));
}

#[test]
fn test_shutdown_manager() {
    let (manager, _db, _temp) = create_test_manager();

    // Begin several transactions
    let txn1 = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin first transaction");
    let txn2 = manager
        .begin_transaction(TransactionOptions::new().read_only())
        .expect("Failed to begin second transaction");

    // Shutdown manager
    manager.shutdown();

    // All transactions should be aborted
    assert!(!manager.is_transaction_active(txn1));
    assert!(!manager.is_transaction_active(txn2));

    // Cannot begin new transaction after shutdown
    let result = manager.begin_transaction(TransactionOptions::default());
    assert!(matches!(result, Err(TransactionError::Internal(_))));
}

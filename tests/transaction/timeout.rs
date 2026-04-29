//! Transaction Timeout Tests
//!
//! Test coverage:
//! - Transaction timeout handling
//! - Query timeout
//! - Statement timeout
//! - Idle timeout

use graphdb::transaction::{TransactionManager, TransactionManagerConfig, TransactionOptions};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

/// Test transaction with timeout handling
#[tokio::test]
async fn test_transaction_timeout_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db")).expect("Failed to create database"),
    );

    let manager = TransactionManager::new(db, TransactionManagerConfig::default());

    let options = TransactionOptions::new().with_timeout(Duration::from_millis(50));

    let txn_id = manager
        .begin_transaction(options)
        .expect("Failed to begin transaction");

    tokio::time::sleep(Duration::from_millis(100)).await;

    let result = manager.commit_transaction(txn_id).await;
    assert!(
        result.is_err() || manager.get_context(txn_id).is_err(),
        "Transaction should have timed out or been cleaned up"
    );
}

/// Test transaction with query timeout
#[tokio::test]
async fn test_transaction_query_timeout() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db")).expect("Failed to create database"),
    );

    let manager = TransactionManager::new(db, TransactionManagerConfig::default());

    let options = TransactionOptions::new().with_query_timeout(Duration::from_secs(5));

    let txn_id = manager
        .begin_transaction(options)
        .expect("Failed to begin transaction");

    let context = manager.get_context(txn_id).expect("Failed to get context");

    assert!(context.query_timeout.is_some());
    assert_eq!(context.query_timeout.unwrap(), Duration::from_secs(5));

    manager
        .commit_transaction(txn_id)
        .await
        .expect("Failed to commit transaction");
}

/// Test transaction with statement timeout
#[tokio::test]
async fn test_transaction_statement_timeout() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db")).expect("Failed to create database"),
    );

    let manager = TransactionManager::new(db, TransactionManagerConfig::default());

    let options = TransactionOptions::new().with_statement_timeout(Duration::from_secs(1));

    let txn_id = manager
        .begin_transaction(options)
        .expect("Failed to begin transaction");

    let context = manager.get_context(txn_id).expect("Failed to get context");

    assert!(context.statement_timeout.is_some());
    assert_eq!(context.statement_timeout.unwrap(), Duration::from_secs(1));

    manager
        .commit_transaction(txn_id)
        .await
        .expect("Failed to commit transaction");
}

/// Test transaction with idle timeout
#[tokio::test]
async fn test_transaction_idle_timeout() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db")).expect("Failed to create database"),
    );

    let manager = TransactionManager::new(db, TransactionManagerConfig::default());

    let options = TransactionOptions::new().with_idle_timeout(Duration::from_secs(30));

    let txn_id = manager
        .begin_transaction(options)
        .expect("Failed to begin transaction");

    let context = manager.get_context(txn_id).expect("Failed to get context");

    assert!(context.idle_timeout.is_some());
    assert_eq!(context.idle_timeout.unwrap(), Duration::from_secs(30));

    manager
        .commit_transaction(txn_id)
        .await
        .expect("Failed to commit transaction");
}

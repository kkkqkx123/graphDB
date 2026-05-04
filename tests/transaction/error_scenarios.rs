//! Transaction Error Scenarios Tests
//!
//! Test coverage for various error conditions and edge cases:
//! - Transaction not found errors
//! - Invalid state transitions
//! - Savepoint not found errors
//! - Concurrent write transaction conflicts
//! - Too many transactions error
//! - Transaction timeout scenarios
//! - Read-only transaction write attempts
//! - Invalid operations on committed/aborted transactions
//! - Double commit/rollback attempts
//! - Shutdown errors

use graphdb::transaction::{
    TransactionError, TransactionManager, TransactionManagerConfig, TransactionOptions,
    TransactionState,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Test transaction not found error
#[tokio::test]
async fn test_error_transaction_not_found() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    let result = manager.get_context(99999);
    assert!(
        matches!(result, Err(TransactionError::TransactionNotFound(99999))),
        "Expected TransactionNotFound error"
    );

    let result = manager.commit_transaction(99999).await;
    assert!(
        matches!(result, Err(TransactionError::TransactionNotFound(99999))),
        "Expected TransactionNotFound error on commit"
    );

    let result = manager.rollback_transaction(99999);
    assert!(
        matches!(result, Err(TransactionError::TransactionNotFound(99999))),
        "Expected TransactionNotFound error on rollback"
    );
}

/// Test invalid state transitions
#[tokio::test]
async fn test_error_invalid_state_transition() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction");

    let context = manager.get_context(txn_id).expect("Failed to get context");

    context
        .transition_to(TransactionState::Committing)
        .expect("Failed to transition to Committing");

    let result = context.transition_to(TransactionState::Aborting);
    assert!(
        matches!(
            result,
            Err(TransactionError::InvalidStateTransition {
                from: TransactionState::Committing,
                to: TransactionState::Aborting
            })
        ),
        "Expected InvalidStateTransition error"
    );

    context
        .transition_to(TransactionState::Committed)
        .expect("Failed to transition to Committed");

    let result = context.transition_to(TransactionState::Active);
    assert!(
        matches!(
            result,
            Err(TransactionError::InvalidStateTransition {
                from: TransactionState::Committed,
                to: TransactionState::Active
            })
        ),
        "Expected InvalidStateTransition error from terminal state"
    );
}

/// Test invalid state for commit/abort
#[tokio::test]
async fn test_error_invalid_state_for_commit_abort() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction");

    let context = manager.get_context(txn_id).expect("Failed to get context");

    context
        .transition_to(TransactionState::Committing)
        .expect("Failed to transition");

    let result = context.can_execute();
    assert!(
        matches!(result, Err(TransactionError::InvalidStateForCommit(_))),
        "Expected InvalidStateForCommit error"
    );
}

/// Test savepoint not found error
#[tokio::test]
async fn test_error_savepoint_not_found() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction");

    let result = manager.rollback_to_savepoint(txn_id, 99999);
    assert!(
        matches!(result, Err(TransactionError::SavepointNotFound(99999))),
        "Expected SavepointNotFound error"
    );

    let result = manager.release_savepoint(txn_id, 99999);
    assert!(
        matches!(result, Err(TransactionError::SavepointNotFound(99999))),
        "Expected SavepointNotFound error on release"
    );

    manager
        .commit_transaction(txn_id)
        .await
        .expect("Failed to commit transaction");
}

/// Test write transaction conflict
#[tokio::test]
async fn test_error_write_transaction_conflict() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    let txn1 = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin first transaction");

    let result = manager.begin_transaction(TransactionOptions::default());
    assert!(
        matches!(result, Err(TransactionError::WriteTransactionConflict)),
        "Expected WriteTransactionConflict error"
    );

    manager
        .commit_transaction(txn1)
        .await
        .expect("Failed to commit transaction");

    let txn2 = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Should be able to begin transaction after commit");

    manager
        .commit_transaction(txn2)
        .await
        .expect("Failed to commit second transaction");
}

/// Test too many transactions error
#[test]
fn test_error_too_many_transactions() {
    let config = TransactionManagerConfig {
        max_concurrent_transactions: 2,
        ..Default::default()
    };

    let manager = TransactionManager::new(config);

    let txn1 = manager
        .begin_transaction(TransactionOptions::new().read_only())
        .expect("Failed to begin transaction 1");
    let txn2 = manager
        .begin_transaction(TransactionOptions::new().read_only())
        .expect("Failed to begin transaction 2");

    let result = manager.begin_transaction(TransactionOptions::new().read_only());
    assert!(
        matches!(result, Err(TransactionError::TooManyTransactions)),
        "Expected TooManyTransactions error"
    );

    manager
        .rollback_transaction(txn1)
        .expect("Failed to rollback txn1");
    manager
        .rollback_transaction(txn2)
        .expect("Failed to rollback txn2");
}

/// Test transaction timeout error
#[tokio::test]
async fn test_error_transaction_timeout() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    let options = TransactionOptions::new().with_timeout(Duration::from_millis(50));
    let txn_id = manager
        .begin_transaction(options)
        .expect("Failed to begin transaction");

    sleep(Duration::from_millis(100)).await;

    let result = manager.commit_transaction(txn_id).await;
    assert!(
        matches!(result, Err(TransactionError::TransactionTimeout)),
        "Expected TransactionTimeout error, got {:?}",
        result
    );
}

/// Test transaction expired error on operations
#[tokio::test]
async fn test_error_transaction_expired() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    let options = TransactionOptions::new().with_timeout(Duration::from_millis(50));
    let txn_id = manager
        .begin_transaction(options)
        .expect("Failed to begin transaction");

    let context = manager.get_context(txn_id).expect("Failed to get context");

    sleep(Duration::from_millis(100)).await;

    let result = context.can_execute();
    assert!(
        matches!(result, Err(TransactionError::TransactionExpired)),
        "Expected TransactionExpired error"
    );
}

/// Test read-only transaction errors
#[tokio::test]
async fn test_error_readonly_transaction() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    let options = TransactionOptions::new().read_only();
    let txn_id = manager
        .begin_transaction(options)
        .expect("Failed to begin read-only transaction");

    let context = manager.get_context(txn_id).expect("Failed to get context");

    assert!(context.read_only);

    manager
        .commit_transaction(txn_id)
        .await
        .expect("Failed to commit read-only transaction");
}

/// Test double commit attempt
#[tokio::test]
async fn test_error_double_commit() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction");

    manager
        .commit_transaction(txn_id)
        .await
        .expect("First commit should succeed");

    let result = manager.commit_transaction(txn_id).await;
    assert!(
        matches!(result, Err(TransactionError::TransactionNotFound(_))),
        "Expected TransactionNotFound on double commit"
    );
}

/// Test double rollback attempt
#[tokio::test]
async fn test_error_double_rollback() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction");

    manager
        .rollback_transaction(txn_id)
        .expect("First rollback should succeed");

    let result = manager.rollback_transaction(txn_id);
    assert!(
        matches!(result, Err(TransactionError::TransactionNotFound(_))),
        "Expected TransactionNotFound on double rollback"
    );
}

/// Test commit after rollback attempt
#[tokio::test]
async fn test_error_commit_after_rollback() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction");

    manager
        .rollback_transaction(txn_id)
        .expect("Rollback should succeed");

    let result = manager.commit_transaction(txn_id).await;
    assert!(
        matches!(result, Err(TransactionError::TransactionNotFound(_))),
        "Expected TransactionNotFound on commit after rollback"
    );
}

/// Test shutdown error
#[test]
fn test_error_shutdown() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    manager.shutdown();

    let result = manager.begin_transaction(TransactionOptions::default());
    assert!(
        matches!(result, Err(TransactionError::Internal(_))),
        "Expected Internal error after shutdown"
    );
}

/// Test no savepoints in transaction error
#[tokio::test]
async fn test_error_no_savepoints() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction");

    let savepoints = manager.get_active_savepoints(txn_id);
    assert!(savepoints.is_empty());

    let found = manager.find_savepoint_by_name(txn_id, "non_existent");
    assert!(found.is_none());

    manager
        .commit_transaction(txn_id)
        .await
        .expect("Failed to commit");
}

/// Test transaction state display
#[test]
fn test_transaction_state_display() {
    assert_eq!(format!("{}", TransactionState::Active), "Active");
    assert_eq!(format!("{}", TransactionState::Committing), "Committing");
    assert_eq!(format!("{}", TransactionState::Committed), "Committed");
    assert_eq!(format!("{}", TransactionState::Aborting), "Aborting");
    assert_eq!(format!("{}", TransactionState::Aborted), "Aborted");
}

/// Test transaction state helpers
#[test]
fn test_transaction_state_helpers() {
    assert!(TransactionState::Active.can_execute());
    assert!(TransactionState::Active.can_commit());
    assert!(TransactionState::Active.can_abort());
    assert!(!TransactionState::Active.is_terminal());

    assert!(!TransactionState::Committed.can_execute());
    assert!(!TransactionState::Committed.can_commit());
    assert!(!TransactionState::Committed.can_abort());
    assert!(TransactionState::Committed.is_terminal());

    assert!(!TransactionState::Aborted.can_execute());
    assert!(!TransactionState::Aborted.can_commit());
    assert!(!TransactionState::Aborted.can_abort());
    assert!(TransactionState::Aborted.is_terminal());
}

/// Test error formatting
#[test]
fn test_error_formatting() {
    let error = TransactionError::TransactionNotFound(123);
    assert!(format!("{}", error).contains("123"));

    let error = TransactionError::TooManyTransactions;
    assert!(format!("{}", error).contains("Too many"));

    let error = TransactionError::TransactionTimeout;
    assert!(format!("{}", error).contains("timeout"));

    let error = TransactionError::Internal("test error".to_string());
    assert!(format!("{}", error).contains("test error"));
}

/// Test retry with non-retryable error
#[tokio::test]
async fn test_retry_non_retryable_error() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    let retry_config = graphdb::transaction::RetryConfig::new()
        .with_max_retries(3)
        .with_initial_delay(Duration::from_millis(10));

    let result: Result<&str, _> = manager
        .execute_with_retry(
            TransactionOptions::default(),
            retry_config,
            |_txn_id| Err(TransactionError::Internal("non-retryable".to_string())),
        )
        .await;

    assert!(
        matches!(result, Err(TransactionError::Internal(_))),
        "Expected immediate failure for non-retryable error"
    );
}

/// Test retry with retryable error
#[tokio::test]
async fn test_retry_retryable_error() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    let retry_config = graphdb::transaction::RetryConfig::new()
        .with_max_retries(2)
        .with_initial_delay(Duration::from_millis(10));

    let attempts = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let attempts_clone = attempts.clone();

    let result: Result<&str, _> = manager
        .execute_with_retry(
            TransactionOptions::default(),
            retry_config,
            move |_txn_id| {
                let count = attempts_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if count < 1 {
                    Err(TransactionError::WriteTransactionConflict)
                } else {
                    Ok("success")
                }
            },
        )
        .await;

    assert_eq!(result.expect("Should succeed after retry"), "success");
    assert_eq!(
        attempts.load(std::sync::atomic::Ordering::SeqCst),
        2,
        "Should have attempted twice"
    );
}

/// Test batch commit with invalid transaction
#[tokio::test]
async fn test_batch_commit_invalid_transaction() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    let result = manager.commit_batch(vec![99999, 99998]).await;
    assert!(
        result.is_err(),
        "Batch commit with invalid transactions should fail"
    );
}

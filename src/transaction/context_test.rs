//! TransactionContext Tests
//!
//! Test transaction context functionality, including state management, timeout checking, operation logs, etc.

use std::sync::Arc;
use std::time::Duration;

use tempfile::TempDir;

use crate::transaction::context::TransactionContext;
use crate::transaction::types::{
    DurabilityLevel, OperationLog, TransactionConfig, TransactionError, TransactionId,
    TransactionState,
};

/// Create test database
fn create_test_db() -> (Arc<redb::Database>, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db"))
            .expect("Failed to create test database"),
    );
    (db, temp_dir)
}

/// Create default transaction config
fn create_default_config(timeout: Duration) -> TransactionConfig {
    TransactionConfig {
        timeout,
        durability: DurabilityLevel::Immediate,
        isolation_level: crate::transaction::types::IsolationLevel::default(),
        query_timeout: None,
        statement_timeout: None,
        idle_timeout: None,
        two_phase_commit: false,
    }
}

#[test]
fn test_transaction_context_writable_creation() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);
    let durability = DurabilityLevel::Immediate;

    let config = create_default_config(timeout);
    let config = TransactionConfig {
        durability,
        ..config
    };

    let write_txn = db
        .begin_write()
        .expect("Failed to create write transaction");

    let ctx = TransactionContext::new_writable(txn_id, config, write_txn, None);

    assert_eq!(ctx.id, txn_id);
    assert_eq!(ctx.state(), TransactionState::Active);
    assert!(!ctx.read_only);
    assert_eq!(ctx.durability, durability);
}

#[test]
fn test_transaction_context_readonly_creation() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let config = create_default_config(timeout);

    let read_txn = db.begin_read().expect("Failed to create read transaction");

    let ctx = TransactionContext::new_readonly(txn_id, config, read_txn, None);

    assert_eq!(ctx.id, txn_id);
    assert_eq!(ctx.state(), TransactionState::Active);
    assert!(ctx.read_only);
}

#[test]
fn test_transaction_context_state_transitions() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let config = create_default_config(timeout);

    let write_txn = db
        .begin_write()
        .expect("Failed to create write transaction");

    let ctx = TransactionContext::new_writable(txn_id, config, write_txn, None);

    // Active -> Committing
    assert!(ctx.transition_to(TransactionState::Committing).is_ok());
    assert_eq!(ctx.state(), TransactionState::Committing);

    // Committing -> Committed
    assert!(ctx.transition_to(TransactionState::Committed).is_ok());
    assert_eq!(ctx.state(), TransactionState::Committed);
}

#[test]
fn test_transaction_context_invalid_state_transition() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let config = create_default_config(timeout);

    let write_txn = db
        .begin_write()
        .expect("Failed to create write transaction");

    let ctx = TransactionContext::new_writable(txn_id, config, write_txn, None);

    // Active -> Committed (invalid transition)
    let result = ctx.transition_to(TransactionState::Committed);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TransactionError::InvalidStateTransition { .. }
    ));

    // Correct state transition path: Active -> Committing -> Committed
    assert!(ctx.transition_to(TransactionState::Committing).is_ok());
    assert_eq!(ctx.state(), TransactionState::Committing);

    assert!(ctx.transition_to(TransactionState::Committed).is_ok());
    assert_eq!(ctx.state(), TransactionState::Committed);
}

#[test]
fn test_transaction_context_timeout() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_millis(100);

    let config = create_default_config(timeout);

    let write_txn = db
        .begin_write()
        .expect("Failed to create write transaction");

    let ctx = TransactionContext::new_writable(txn_id, config, write_txn, None);

    // Initially should not be expired
    assert!(!ctx.is_expired());

    // Wait for timeout
    std::thread::sleep(Duration::from_millis(150));

    // Now should be expired
    assert!(ctx.is_expired());
}

#[test]
fn test_transaction_context_remaining_time() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_millis(200);

    let config = create_default_config(timeout);

    let write_txn = db
        .begin_write()
        .expect("Failed to create write transaction");

    let ctx = TransactionContext::new_writable(txn_id, config, write_txn, None);

    // Initial remaining time should be close to timeout
    let remaining = ctx.remaining_time();
    assert!(remaining > Duration::from_millis(150));

    // Wait for a while
    std::thread::sleep(Duration::from_millis(100));

    // Remaining time should decrease
    let remaining = ctx.remaining_time();
    assert!(remaining < Duration::from_millis(150));
    assert!(remaining > Duration::from_millis(50));
}

#[test]
fn test_transaction_context_modified_tables() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let config = create_default_config(timeout);

    let write_txn = db
        .begin_write()
        .expect("Failed to create write transaction");

    let ctx = TransactionContext::new_writable(txn_id, config, write_txn, None);

    // Record table modifications
    ctx.record_table_modification("vertices");
    ctx.record_table_modification("edges");
    ctx.record_table_modification("vertices"); // Duplicate record

    let modified = ctx.get_modified_tables();
    assert_eq!(modified.len(), 2);
    assert!(modified.contains(&"vertices".to_string()));
    assert!(modified.contains(&"edges".to_string()));
}

#[test]
fn test_transaction_context_operation_log() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let config = create_default_config(timeout);

    let write_txn = db
        .begin_write()
        .expect("Failed to create write transaction");

    let ctx = TransactionContext::new_writable(txn_id, config, write_txn, None);

    // Initial operation log is empty
    assert_eq!(ctx.operation_log_len(), 0);

    // Add operation log
    ctx.add_operation_log(OperationLog::InsertVertex {
        space: "test".to_string(),
        vertex_id: vec![1, 2, 3],
        previous_state: None,
    });

    assert_eq!(ctx.operation_log_len(), 1);

    ctx.add_operation_log(OperationLog::UpdateVertex {
        space: "test".to_string(),
        vertex_id: vec![1, 2, 3],
        previous_data: vec![4, 5, 6],
    });

    assert_eq!(ctx.operation_log_len(), 2);

    // Truncate operation log
    ctx.truncate_operation_log(1);
    assert_eq!(ctx.operation_log_len(), 1);
}

#[test]
fn test_transaction_context_can_execute() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let config = create_default_config(timeout);

    let write_txn = db
        .begin_write()
        .expect("Failed to create write transaction");

    let ctx = TransactionContext::new_writable(txn_id, config, write_txn, None);

    // Active state can execute
    assert!(ctx.can_execute().is_ok());

    // Transition to Committing state
    ctx.transition_to(TransactionState::Committing)
        .expect("State transition failed");

    // Committing state cannot execute
    assert!(ctx.can_execute().is_err());
}

#[test]
fn test_transaction_context_can_execute_expired() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_millis(50);

    let config = create_default_config(timeout);

    let write_txn = db
        .begin_write()
        .expect("Failed to create write transaction");

    let ctx = TransactionContext::new_writable(txn_id, config, write_txn, None);

    // Wait for timeout
    std::thread::sleep(Duration::from_millis(100));

    // Cannot execute after timeout
    let result = ctx.can_execute();
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TransactionError::TransactionExpired
    ));
}

#[test]
fn test_transaction_context_info() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let config = create_default_config(timeout);

    let write_txn = db
        .begin_write()
        .expect("Failed to create write transaction");

    let ctx = TransactionContext::new_writable(txn_id, config, write_txn, None);

    // Record some modifications
    ctx.record_table_modification("vertices");

    let info = ctx.info();
    assert_eq!(info.id, txn_id);
    assert_eq!(info.state, TransactionState::Active);
    assert!(!info.is_read_only);
    assert_eq!(info.modified_tables.len(), 1);
    assert!(info.modified_tables.contains(&"vertices".to_string()));
}

#[test]
fn test_transaction_context_take_write_txn() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let config = create_default_config(timeout);

    let write_txn = db
        .begin_write()
        .expect("Failed to create write transaction");

    let ctx = TransactionContext::new_writable(txn_id, config, write_txn, None);

    // Take write transaction
    let taken_txn = ctx.take_write_txn();
    assert!(taken_txn.is_ok());

    // Second take should fail
    let result = ctx.take_write_txn();
    assert!(result.is_err());
}

#[test]
fn test_transaction_context_readonly_take_write_txn() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let config = create_default_config(timeout);

    let read_txn = db.begin_read().expect("Failed to create read transaction");

    let ctx = TransactionContext::new_readonly(txn_id, config, read_txn, None);

    // Read-only transaction cannot take write transaction
    let result = ctx.take_write_txn();
    assert!(result.is_err());
}

#[test]
fn test_transaction_context_with_write_txn() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let config = create_default_config(timeout);

    let write_txn = db
        .begin_write()
        .expect("Failed to create write transaction");

    let ctx = TransactionContext::new_writable(txn_id, config, write_txn, None);

    // Execute with write transaction
    let result = ctx.with_write_txn(|_txn| Ok::<(), crate::core::StorageError>(()));

    assert!(result.is_ok());
}

#[test]
fn test_transaction_context_readonly_with_write_txn() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let config = create_default_config(timeout);

    let read_txn = db.begin_read().expect("Failed to create read transaction");

    let ctx = TransactionContext::new_readonly(txn_id, config, read_txn, None);

    // Read-only transaction cannot use with_write_txn
    let result = ctx.with_write_txn(|_txn| Ok::<(), crate::core::StorageError>(()));

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TransactionError::ReadOnlyTransaction
    ));
}

#[test]
fn test_transaction_context_with_read_txn() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let config = create_default_config(timeout);

    let read_txn = db.begin_read().expect("Failed to create read transaction");

    let ctx = TransactionContext::new_readonly(txn_id, config, read_txn, None);

    // Execute with read transaction
    let result = ctx.with_read_txn(|_txn| Ok::<(), crate::core::StorageError>(()));

    assert!(result.is_ok());
}

#[test]
fn test_transaction_context_writable_with_read_txn() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let config = create_default_config(timeout);

    let write_txn = db
        .begin_write()
        .expect("Failed to create write transaction");

    let ctx = TransactionContext::new_writable(txn_id, config, write_txn, None);

    // Write transaction cannot directly use with_read_txn
    let result = ctx.with_read_txn(|_txn| Ok::<(), crate::core::StorageError>(()));

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), TransactionError::Internal(_)));
}

#[test]
fn test_transaction_context_with_write_txn_expired() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_millis(50);

    let config = create_default_config(timeout);

    let write_txn = db
        .begin_write()
        .expect("Failed to create write transaction");

    let ctx = TransactionContext::new_writable(txn_id, config, write_txn, None);

    // Wait for timeout
    std::thread::sleep(Duration::from_millis(100));

    // Cannot execute after timeout
    let result = ctx.with_write_txn(|_txn| Ok::<(), crate::core::StorageError>(()));

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TransactionError::TransactionExpired
    ));
}

#[test]
fn test_transaction_context_with_write_txn_invalid_state() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let config = create_default_config(timeout);

    let write_txn = db
        .begin_write()
        .expect("Failed to create write transaction");

    let ctx = TransactionContext::new_writable(txn_id, config, write_txn, None);

    // Correct state transition path: Active -> Committing -> Committed
    ctx.transition_to(TransactionState::Committing)
        .expect("State transition failed");
    ctx.transition_to(TransactionState::Committed)
        .expect("State transition failed");

    // Committed state cannot execute
    let result = ctx.with_write_txn(|_txn| Ok::<(), crate::core::StorageError>(()));

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TransactionError::InvalidStateForCommit(_)
    ));
}

#[test]
fn test_savepoint_creation() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let config = create_default_config(timeout);

    let write_txn = db
        .begin_write()
        .expect("Failed to create write transaction");

    let ctx = TransactionContext::new_writable(txn_id, config, write_txn, None);

    // Create savepoint
    let savepoint_id = ctx.create_savepoint(Some("sp1".to_string()));
    assert_eq!(savepoint_id, 1);

    // Get savepoint info
    let savepoint_info = ctx.get_savepoint(savepoint_id);
    assert!(savepoint_info.is_some());
    let info = savepoint_info.expect("savepoint info should exist");
    assert_eq!(info.name, Some("sp1".to_string()));
    assert_eq!(info.operation_log_index, 0);
}

#[test]
fn test_multiple_savepoints() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let config = create_default_config(timeout);

    let write_txn = db
        .begin_write()
        .expect("Failed to create write transaction");

    let ctx = TransactionContext::new_writable(txn_id, config, write_txn, None);

    // Create multiple savepoints
    let sp1 = ctx.create_savepoint(Some("sp1".to_string()));
    let sp2 = ctx.create_savepoint(Some("sp2".to_string()));
    let sp3 = ctx.create_savepoint(Some("sp3".to_string()));

    assert_eq!(sp1, 1);
    assert_eq!(sp2, 2);
    assert_eq!(sp3, 3);

    // Verify savepoint info
    assert!(ctx.get_savepoint(sp1).is_some());
    assert!(ctx.get_savepoint(sp2).is_some());
    assert!(ctx.get_savepoint(sp3).is_some());
}

#[test]
fn test_rollback_to_savepoint() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let config = create_default_config(timeout);

    let write_txn = db
        .begin_write()
        .expect("Failed to create write transaction");

    let ctx = TransactionContext::new_writable(txn_id, config, write_txn, None);

    // Create savepoint
    let savepoint_id = ctx.create_savepoint(Some("sp1".to_string()));

    // Rollback to savepoint
    let result = ctx.rollback_to_savepoint(savepoint_id);
    assert!(result.is_ok());

    // Verify operation log has been truncated
    assert_eq!(ctx.operation_log_len(), 0);
}

#[test]
fn test_rollback_to_nonexistent_savepoint() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let config = create_default_config(timeout);

    let write_txn = db
        .begin_write()
        .expect("Failed to create write transaction");

    let ctx = TransactionContext::new_writable(txn_id, config, write_txn, None);

    // Try to rollback to nonexistent savepoint
    let result = ctx.rollback_to_savepoint(999);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TransactionError::SavepointNotFound(_)
    ));
}

#[test]
fn test_release_savepoint() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let config = create_default_config(timeout);

    let write_txn = db
        .begin_write()
        .expect("Failed to create write transaction");

    let ctx = TransactionContext::new_writable(txn_id, config, write_txn, None);

    // Create savepoint
    let savepoint_id = ctx.create_savepoint(Some("sp1".to_string()));

    // Release savepoint
    let result = ctx.release_savepoint(savepoint_id);
    assert!(result.is_ok());

    // Verify savepoint has been released
    let savepoint_info = ctx.get_savepoint(savepoint_id);
    assert!(savepoint_info.is_none());
}

#[test]
fn test_release_nonexistent_savepoint() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let config = create_default_config(timeout);

    let write_txn = db
        .begin_write()
        .expect("Failed to create write transaction");

    let ctx = TransactionContext::new_writable(txn_id, config, write_txn, None);

    // Try to release nonexistent savepoint
    let result = ctx.release_savepoint(999);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TransactionError::SavepointNotFound(_)
    ));
}

#[test]
fn test_savepoint_with_operations() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let config = create_default_config(timeout);

    let write_txn = db
        .begin_write()
        .expect("Failed to create write transaction");

    let ctx = TransactionContext::new_writable(txn_id, config, write_txn, None);

    // Create first savepoint
    let sp1 = ctx.create_savepoint(Some("sp1".to_string()));

    // Add some operation logs
    let log1 = OperationLog::InsertVertex {
        space: "test".to_string(),
        vertex_id: vec![1u8, 2u8, 3u8],
        previous_state: None,
    };
    ctx.add_operation_log(log1);

    let log2 = OperationLog::InsertVertex {
        space: "test".to_string(),
        vertex_id: vec![4u8, 5u8, 6u8],
        previous_state: None,
    };
    ctx.add_operation_log(log2);

    // Create second savepoint
    let _sp2 = ctx.create_savepoint(Some("sp2".to_string()));

    // Verify operation log count
    assert_eq!(ctx.operation_log_len(), 2);

    // Rollback to first savepoint
    let result = ctx.rollback_to_savepoint(sp1);
    assert!(result.is_ok());

    // Verify operation log has been truncated to first savepoint
    assert_eq!(ctx.operation_log_len(), 0);
}

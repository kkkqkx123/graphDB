//! Transaction Function Integration Testing
//!
//! Test the core functions of the transaction manager, including:
//! Transaction lifecycle management ( initiation, commitment, cancellation )
//! Transaction isolation
//! Concurrency transaction processing
//! Integration of the transaction and storage layers
//! Save point management

mod common;

use std::sync::Arc;
use std::time::Duration;

use graphdb::storage::RedbStorage;
use graphdb::transaction::{
    TransactionError, TransactionManager, TransactionManagerConfig, TransactionOptions,
    TransactionState,
};

/// Create a test transaction manager.
fn create_test_transaction_manager() -> Arc<TransactionManager> {
    use redb::Database;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("test_txn.db");
    let db = Arc::new(Database::create(db_path).expect("创建数据库失败"));

    let config = TransactionManagerConfig {
        default_timeout: Duration::from_secs(30),
        max_concurrent_transactions: 1000,
        auto_cleanup: true,
    };

    Arc::new(TransactionManager::new(db, config))
}

/// Testing the transaction lifecycle
#[tokio::test]
async fn test_transaction_lifecycle() {
    let txn_manager = create_test_transaction_manager();

    // Start a transaction
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // Check the transaction status.
    let txn_info = txn_manager
        .get_transaction_info(txn_id)
        .expect("获取事务失败");
    assert_eq!(txn_info.state, TransactionState::Active);

    // Commit a transaction
    txn_manager
        .commit_transaction(txn_id)
        .await
        .expect("提交事务失败");

    // The transaction has been committed and is no longer listed in the table of active transactions.
    let txn_info = txn_manager.get_transaction_info(txn_id);
    assert!(txn_info.is_none());

    // Verify statistical information
    let stats = txn_manager.stats();
    assert_eq!(
        stats
            .committed_transactions
            .load(std::sync::atomic::Ordering::SeqCst),
        1
    );
}

/// Testing transaction rollback
#[tokio::test]
async fn test_transaction_rollback() {
    let txn_manager = create_test_transaction_manager();

    // Start a transaction
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // Roll back a transaction
    txn_manager.abort_transaction(txn_id).expect("回滚事务失败");

    // The transaction has been aborted and is no longer listed in the active transactions table.
    let txn_info = txn_manager.get_transaction_info(txn_id);
    assert!(txn_info.is_none());

    // Verify statistical information
    let stats = txn_manager.stats();
    assert_eq!(
        stats
            .aborted_transactions
            .load(std::sync::atomic::Ordering::SeqCst),
        1
    );
}

/// Testing read-only transactions
#[tokio::test]
async fn test_read_only_transaction() {
    let txn_manager = create_test_transaction_manager();

    // Start a read-only transaction
    let options = TransactionOptions {
        read_only: true,
        timeout: Some(Duration::from_secs(30)),
        durability: graphdb::transaction::DurabilityLevel::None,
        isolation_level: graphdb::transaction::IsolationLevel::default(),
        query_timeout: None,
        statement_timeout: None,
        idle_timeout: None,
        two_phase_commit: false,
    };
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // Check whether the transaction is read-only.
    let txn_info = txn_manager
        .get_transaction_info(txn_id)
        .expect("获取事务失败");
    assert!(txn_info.is_read_only);

    // Commit a transaction
    txn_manager
        .commit_transaction(txn_id)
        .await
        .expect("提交事务失败");
}

/// Testing the rollback of saved points and data recovery
#[tokio::test]
async fn test_savepoint_rollback_with_data_recovery() {
    use graphdb::core::Value;
    use graphdb::core::Vertex;
    use std::collections::HashMap;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("test_rollback_data.db");

    // Create storage
    let storage = RedbStorage::new_with_path(db_path).expect("创建存储失败");
    let db = storage.get_db().clone();

    // Create a transaction manager
    let config = TransactionManagerConfig::default();
    let txn_manager = Arc::new(TransactionManager::new(db, config));

    // Start a transaction
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // Obtaining the transaction context
    let context = txn_manager.get_context(txn_id).expect("获取事务上下文失败");

    // Create the first vertex.
    let _vertex1 = Vertex {
        vid: Box::new(Value::Int(1)),
        id: 1,
        tags: vec![],
        properties: HashMap::new(),
    };

    context.add_operation_log(graphdb::transaction::types::OperationLog::InsertVertex {
        space: "test".to_string(),
        vertex_id: vec![1u8],
        previous_state: None,
    });

    // Create the first save point.
    let savepoint_id1 = txn_manager
        .create_savepoint(txn_id, Some("checkpoint1".to_string()))
        .expect("创建保存点失败");

    // Create a second vertex.
    let _vertex2 = Vertex {
        vid: Box::new(Value::Int(2)),
        id: 2,
        tags: vec![],
        properties: HashMap::new(),
    };

    context.add_operation_log(graphdb::transaction::types::OperationLog::InsertVertex {
        space: "test".to_string(),
        vertex_id: vec![2u8],
        previous_state: None,
    });

    // Create a second save point.
    let _savepoint_id2 = txn_manager
        .create_savepoint(txn_id, Some("checkpoint2".to_string()))
        .expect("创建保存点失败");

    // Verify the number of operation logs.
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 2);

    // Roll back to the first save point.
    txn_manager
        .rollback_to_savepoint(txn_id, savepoint_id1)
        .expect("回滚到保存点失败");

    // The verification operation logs have been truncated to the position of the first save point (there should be 1 log in total).
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 1);

    // Verify that the second save point has been removed.
    let savepoints = txn_manager.get_active_savepoints(txn_id);
    assert_eq!(savepoints.len(), 1);
    assert_eq!(savepoints[0].id, savepoint_id1);

    // Commit a transaction
    txn_manager
        .commit_transaction(txn_id)
        .await
        .expect("提交事务失败");
}

/// Continue with the operations after rolling back to the test save point.
#[tokio::test]
async fn test_continue_operations_after_savepoint_rollback() {
    let txn_manager = create_test_transaction_manager();

    // Start a transaction
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // Obtaining the transaction context
    let context = txn_manager.get_context(txn_id).expect("获取事务上下文失败");

    // Add some operation logs.
    use graphdb::transaction::types::OperationLog;
    context.add_operation_log(OperationLog::InsertVertex {
        space: "test".to_string(),
        vertex_id: vec![1u8],
        previous_state: None,
    });

    // Create a save point (there is 1 operation log at this time).
    let savepoint_id = txn_manager
        .create_savepoint(txn_id, Some("sp1".to_string()))
        .expect("创建保存点失败");

    // Add more operation logs.
    context.add_operation_log(OperationLog::InsertVertex {
        space: "test".to_string(),
        vertex_id: vec![2u8],
        previous_state: None,
    });

    // Roll back to the saved point.
    txn_manager
        .rollback_to_savepoint(txn_id, savepoint_id)
        .expect("回滚到保存点失败");

    // The verification operation logs have been truncated to the point where the files were saved (there should be 1 log in total).
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 1);

    // Continue adding operation logs after the rollback.
    context.add_operation_log(OperationLog::InsertVertex {
        space: "test".to_string(),
        vertex_id: vec![3u8],
        previous_state: None,
    });

    // Verify the number of operation logs (there should be 2 logs)
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 2);

    // Submission of transactions
    txn_manager
        .commit_transaction(txn_id)
        .await
        .expect("提交事务失败");
}

/// Test Transaction Timeout
#[test]
fn test_transaction_timeout() {
    let txn_manager = create_test_transaction_manager();

    // Starting a transaction with a short timeout
    let options = TransactionOptions {
        read_only: false,
        timeout: Some(Duration::from_millis(100)),
        durability: graphdb::transaction::DurabilityLevel::None,
        isolation_level: graphdb::transaction::IsolationLevel::default(),
        query_timeout: None,
        statement_timeout: None,
        idle_timeout: None,
        two_phase_commit: false,
    };
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // Wait for timeout
    std::thread::sleep(Duration::from_millis(150));

    // Clearance of obsolete services
    txn_manager.cleanup_expired_transactions();

    // Check transaction status (should be automatically aborted)
    let txn_info = txn_manager.get_transaction_info(txn_id);
    assert!(txn_info.is_none());

    // Authentication timeout statistics
    let stats = txn_manager.stats();
    assert_eq!(
        stats
            .timeout_transactions
            .load(std::sync::atomic::Ordering::SeqCst),
        1
    );
}

/// Testing concurrent transactions
#[tokio::test]
async fn test_concurrent_transactions() {
    let txn_manager = create_test_transaction_manager();

    // Starting multiple read-only transactions
    let mut txn_ids = Vec::new();
    for _ in 0..10 {
        let options = TransactionOptions {
            read_only: true,
            timeout: Some(Duration::from_secs(30)),
            durability: graphdb::transaction::DurabilityLevel::None,
            isolation_level: graphdb::transaction::IsolationLevel::default(),
            query_timeout: None,
            statement_timeout: None,
            idle_timeout: None,
            two_phase_commit: false,
        };
        let txn_id = txn_manager
            .begin_transaction(options)
            .expect("开始事务失败");
        txn_ids.push(txn_id);
    }

    // Submission of all transactions
    for txn_id in txn_ids {
        txn_manager
            .commit_transaction(txn_id)
            .await
            .expect("提交事务失败");
    }

    // Verify that all transactions are committed
    let stats = txn_manager.stats();
    assert_eq!(
        stats
            .committed_transactions
            .load(std::sync::atomic::Ordering::SeqCst),
        10
    );
}

/// Testing the integration of transactions with the storage layer
#[tokio::test]
async fn test_transaction_with_storage() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("test_storage.db");

    // Creating Storage
    let storage = RedbStorage::new_with_path(db_path).expect("创建存储失败");
    let db = storage.get_db().clone();

    // Creating a Transaction Manager
    let config = TransactionManagerConfig::default();
    let txn_manager = Arc::new(TransactionManager::new(db, config));

    // Commencement of business
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // Submission of transactions
    txn_manager
        .commit_transaction(txn_id)
        .await
        .expect("提交事务失败");
}

/// Test transaction statistics
#[tokio::test]
async fn test_transaction_stats() {
    let txn_manager = create_test_transaction_manager();

    // Start and commit a transaction
    let options = TransactionOptions::default();
    let txn_id1 = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");
    txn_manager
        .commit_transaction(txn_id1)
        .await
        .expect("提交事务失败");

    // Starting and rolling back a transaction
    let options = TransactionOptions::default();
    let txn_id2 = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");
    txn_manager
        .abort_transaction(txn_id2)
        .expect("回滚事务失败");

    // Checking statistical information
    let stats = txn_manager.stats();
    assert_eq!(
        stats
            .committed_transactions
            .load(std::sync::atomic::Ordering::SeqCst),
        1
    );
    assert_eq!(
        stats
            .aborted_transactions
            .load(std::sync::atomic::Ordering::SeqCst),
        1
    );
}

/// Test creation of savepoints
#[tokio::test]
async fn test_create_savepoint() {
    let txn_manager = create_test_transaction_manager();

    // Commencement of business
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // Creating a save point
    let savepoint_id1 = txn_manager
        .create_savepoint(txn_id, Some("sp1".to_string()))
        .expect("创建保存点失败");

    // Creating a second save point
    let savepoint_id2 = txn_manager
        .create_savepoint(txn_id, Some("sp2".to_string()))
        .expect("创建保存点失败");

    // Verify savepoint ID incrementing
    assert!(savepoint_id2 > savepoint_id1);

    // Get a list of savepoints
    let savepoints = txn_manager.get_active_savepoints(txn_id);
    assert_eq!(savepoints.len(), 2);

    // Submission of transactions
    txn_manager
        .commit_transaction(txn_id)
        .await
        .expect("提交事务失败");
}

/// Test finding savepoints by name
#[tokio::test]
async fn test_find_savepoint_by_name() {
    let txn_manager = create_test_transaction_manager();

    // Commencement of business
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // Creating Named Savepoints
    let savepoint_id = txn_manager
        .create_savepoint(txn_id, Some("checkpoint".to_string()))
        .expect("创建保存点失败");

    // Find savepoints by name
    let found = txn_manager.find_savepoint_by_name(txn_id, "checkpoint");
    assert!(found.is_some());
    assert_eq!(found.expect("保存点应存在").id, savepoint_id);

    // Find non-existent savepoints
    let not_found = txn_manager.find_savepoint_by_name(txn_id, "nonexistent");
    assert!(not_found.is_none());

    // Submission of transactions
    txn_manager
        .commit_transaction(txn_id)
        .await
        .expect("提交事务失败");
}

/// Test release save point
#[tokio::test]
async fn test_release_savepoint() {
    let txn_manager = create_test_transaction_manager();

    // Commencement of business
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // Creating a save point
    let savepoint_id1 = txn_manager
        .create_savepoint(txn_id, Some("sp1".to_string()))
        .expect("创建保存点失败");

    let savepoint_id2 = txn_manager
        .create_savepoint(txn_id, Some("sp2".to_string()))
        .expect("创建保存点失败");

    // Release the first save point
    txn_manager
        .release_savepoint(txn_id, savepoint_id1)
        .expect("释放保存点失败");

    // Verify that the savepoint has been released
    let savepoints = txn_manager.get_active_savepoints(txn_id);
    assert_eq!(savepoints.len(), 1);
    assert_eq!(savepoints[0].id, savepoint_id2);

    // Submission of transactions
    txn_manager
        .commit_transaction(txn_id)
        .await
        .expect("提交事务失败");
}

/// Test rollback to save point
#[tokio::test]
async fn test_rollback_to_savepoint() {
    let txn_manager = create_test_transaction_manager();

    // Commencement of business
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // Creating the first save point
    let savepoint_id1 = txn_manager
        .create_savepoint(txn_id, Some("sp1".to_string()))
        .expect("创建保存点失败");

    // Creating a second save point
    let _savepoint_id2 = txn_manager
        .create_savepoint(txn_id, Some("sp2".to_string()))
        .expect("创建保存点失败");

    // Validation has two save points
    let savepoints = txn_manager.get_active_savepoints(txn_id);
    assert_eq!(savepoints.len(), 2);

    // Rollback to the first save point
    txn_manager
        .rollback_to_savepoint(txn_id, savepoint_id1)
        .expect("回滚到保存点失败");

    // Verify that the second savepoint has been removed
    let savepoints = txn_manager.get_active_savepoints(txn_id);
    assert_eq!(savepoints.len(), 1);
    assert_eq!(savepoints[0].id, savepoint_id1);

    // Submission of transactions
    txn_manager
        .commit_transaction(txn_id)
        .await
        .expect("提交事务失败");
}

/// Test rollback to a non-existent savepoint
#[tokio::test]
async fn test_rollback_to_nonexistent_savepoint() {
    let txn_manager = create_test_transaction_manager();

    // Start a transaction
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // Trying to roll back to a non-existent save point.
    let result = txn_manager.rollback_to_savepoint(txn_id, 999);
    assert!(matches!(
        result,
        Err(TransactionError::SavepointNotFound(_))
    ));

    // Verify that the transaction is still in an active state.
    let txn_info = txn_manager
        .get_transaction_info(txn_id)
        .expect("获取事务失败");
    assert_eq!(txn_info.state, TransactionState::Active);

    // Submit the transaction
    txn_manager
        .commit_transaction(txn_id)
        .await
        .expect("提交事务失败");
}

/// Testing the release of a non-existent save point
#[tokio::test]
async fn test_release_nonexistent_savepoint() {
    let txn_manager = create_test_transaction_manager();

    // Start a transaction
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // Trying to release a non-existent save point…
    let result = txn_manager.release_savepoint(txn_id, 999);
    assert!(matches!(
        result,
        Err(TransactionError::SavepointNotFound(_))
    ));

    // Submit the transaction
    txn_manager
        .commit_transaction(txn_id)
        .await
        .expect("提交事务失败");
}

/// Testing savepoints and transaction commits
#[tokio::test]
async fn test_savepoint_with_transaction_commit() {
    let txn_manager = create_test_transaction_manager();

    // Start a transaction
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // Create a save point.
    let _savepoint_id = txn_manager
        .create_savepoint(txn_id, Some("checkpoint".to_string()))
        .expect("创建保存点失败");

    // Commit a transaction
    txn_manager
        .commit_transaction(txn_id)
        .await
        .expect("提交事务失败");

    // The transaction has been committed and is no longer listed in the table of active transactions.
    let txn_info = txn_manager.get_transaction_info(txn_id);
    assert!(txn_info.is_none());
}

/// Testing savepoints and transaction rollback
#[tokio::test]
async fn test_savepoint_with_transaction_rollback() {
    let txn_manager = create_test_transaction_manager();

    // Start a transaction
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // Create a save point.
    let _savepoint_id = txn_manager
        .create_savepoint(txn_id, Some("checkpoint".to_string()))
        .expect("创建保存点失败");

    // Roll back a transaction
    txn_manager.abort_transaction(txn_id).expect("回滚事务失败");

    // The transaction has been aborted and is no longer listed in the active transactions table.
    let txn_info = txn_manager.get_transaction_info(txn_id);
    assert!(txn_info.is_none());
}

/// Testing the retrieval of savepoint information
#[tokio::test]
async fn test_get_savepoint_info() {
    let txn_manager = create_test_transaction_manager();

    // Start a transaction
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // Create a save point.
    let savepoint_id = txn_manager
        .create_savepoint(txn_id, Some("test_sp".to_string()))
        .expect("创建保存点失败");

    // Obtain information about the save points.
    let savepoint_info = txn_manager.get_savepoint(txn_id, savepoint_id);
    assert!(savepoint_info.is_some());
    assert_eq!(savepoint_info.expect("保存点信息应存在").id, savepoint_id);

    // Trying to obtain information about a non-existent save point.
    let nonexistent = txn_manager.get_savepoint(txn_id, 999);
    assert!(nonexistent.is_none());

    // Submit the transaction
    txn_manager
        .commit_transaction(txn_id)
        .await
        .expect("提交事务失败");
}

/// Testing the management of multiple savepoints
#[tokio::test]
async fn test_multiple_savepoints() {
    let txn_manager = create_test_transaction_manager();

    // Start a transaction
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // Create multiple save points.
    let mut savepoint_ids = Vec::new();
    for i in 0..5 {
        let id = txn_manager
            .create_savepoint(txn_id, Some(format!("sp{}", i)))
            .expect("创建保存点失败");
        savepoint_ids.push(id);
    }

    // Verify that all savepoints have been created.
    let savepoints = txn_manager.get_active_savepoints(txn_id);
    assert_eq!(savepoints.len(), 5);

    // Roll back to the third save point.
    txn_manager
        .rollback_to_savepoint(txn_id, savepoint_ids[2])
        .expect("回滚到保存点失败");

    // Only the first three save points remain for verification.
    let savepoints = txn_manager.get_active_savepoints(txn_id);
    assert_eq!(savepoints.len(), 3);

    // Submit the transaction
    txn_manager
        .commit_transaction(txn_id)
        .await
        .expect("提交事务失败");
}

/// The test transaction manager has been shut down.
#[tokio::test]
async fn test_transaction_manager_shutdown() {
    let txn_manager = create_test_transaction_manager();

    // Start a transaction
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // Close the Transaction Manager.
    txn_manager.shutdown();

    // The validation transaction has been aborted; it is no longer listed in the active transactions table.
    let txn_info = txn_manager.get_transaction_info(txn_id);
    assert!(txn_info.is_none());

    // Once it is disabled, no new transactions can be initiated.
    let result = txn_manager.begin_transaction(TransactionOptions::default());
    assert!(matches!(result, Err(TransactionError::Internal(_))));
}

/// Testing operation log recording and rollback
#[tokio::test]
async fn test_operation_log_recording_and_rollback() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("test_operation_log.db");

    // Create storage
    let storage = RedbStorage::new_with_path(db_path).expect("创建存储失败");
    let db = storage.get_db().clone();

    // Create a transaction manager
    let config = TransactionManagerConfig::default();
    let txn_manager = Arc::new(TransactionManager::new(db, config));

    // Start a transaction
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // Obtaining the transaction context
    let context = txn_manager.get_context(txn_id).expect("获取事务上下文失败");

    // Add operation logs
    use graphdb::transaction::types::OperationLog;
    context.add_operation_log(OperationLog::InsertVertex {
        space: "test_space".to_string(),
        vertex_id: vec![1],
        previous_state: None,
    });

    context.add_operation_log(OperationLog::InsertVertex {
        space: "test_space".to_string(),
        vertex_id: vec![2],
        previous_state: None,
    });

    // The verification operation logs have been recorded.
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 2);

    // Create a save point.
    let savepoint_id = txn_manager
        .create_savepoint(txn_id, Some("sp1".to_string()))
        .expect("创建保存点失败");

    // Add more operation logs.
    context.add_operation_log(OperationLog::InsertVertex {
        space: "test_space".to_string(),
        vertex_id: vec![3],
        previous_state: None,
    });

    // The number of verification operation logs has increased.
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 3);

    // Roll back to the saved point.
    txn_manager
        .rollback_to_savepoint(txn_id, savepoint_id)
        .expect("回滚到保存点失败");

    // The verification operation logs have been truncated.
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 2);

    // Submission of transactions
    txn_manager
        .commit_transaction(txn_id)
        .await
        .expect("提交事务失败");
}

/// Test batch operation operation logging
#[tokio::test]
async fn test_batch_operation_log_recording() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("test_batch_log.db");

    // Creating Storage
    let storage = RedbStorage::new_with_path(db_path).expect("创建存储失败");
    let db = storage.get_db().clone();

    // Creating a Transaction Manager
    let config = TransactionManagerConfig::default();
    let txn_manager = Arc::new(TransactionManager::new(db, config));

    // Commencement of business
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // Getting the Transaction Context
    let context = txn_manager.get_context(txn_id).expect("获取事务上下文失败");

    // Batch add operation log
    use graphdb::transaction::types::OperationLog;
    let batch_logs = vec![
        OperationLog::InsertVertex {
            space: "test_space".to_string(),
            vertex_id: vec![1],
            previous_state: None,
        },
        OperationLog::InsertVertex {
            space: "test_space".to_string(),
            vertex_id: vec![2],
            previous_state: None,
        },
        OperationLog::InsertVertex {
            space: "test_space".to_string(),
            vertex_id: vec![3],
            previous_state: None,
        },
    ];

    context.add_operation_logs(batch_logs);

    // Verify that all operation logs are recorded
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 3);

    // Creating a save point
    let savepoint_id = txn_manager
        .create_savepoint(txn_id, Some("sp1".to_string()))
        .expect("创建保存点失败");

    // Batch add more operation logs
    let more_logs = vec![
        OperationLog::InsertVertex {
            space: "test_space".to_string(),
            vertex_id: vec![4],
            previous_state: None,
        },
        OperationLog::InsertVertex {
            space: "test_space".to_string(),
            vertex_id: vec![5],
            previous_state: None,
        },
    ];

    context.add_operation_logs(more_logs);

    // Increase in the number of validation operation logs
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 5);

    // Rollback to save point
    txn_manager
        .rollback_to_savepoint(txn_id, savepoint_id)
        .expect("回滚到保存点失败");

    // Verify that the operation log has been truncated to the savepoint location
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 3);

    // Submission of transactions
    txn_manager
        .commit_transaction(txn_id)
        .await
        .expect("提交事务失败");
}

/// Test savepoint resource auto-cleaning
#[tokio::test]
async fn test_savepoint_resource_cleanup() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("test_cleanup.db");

    // Creating Storage
    let storage = RedbStorage::new_with_path(db_path).expect("创建存储失败");
    let db = storage.get_db().clone();

    // Creating a Transaction Manager
    let config = TransactionManagerConfig::default();
    let txn_manager = Arc::new(TransactionManager::new(db, config));

    // Commencement of business
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // Creating multiple save points
    let _savepoint_id1 = txn_manager
        .create_savepoint(txn_id, Some("sp1".to_string()))
        .expect("创建保存点失败");
    let _savepoint_id2 = txn_manager
        .create_savepoint(txn_id, Some("sp2".to_string()))
        .expect("创建保存点失败");
    let _savepoint_id3 = txn_manager
        .create_savepoint(txn_id, Some("sp3".to_string()))
        .expect("创建保存点失败");

    // Verify that the savepoint has been created
    let savepoints = txn_manager.get_active_savepoints(txn_id);
    assert_eq!(savepoints.len(), 3);

    // Submission of transactions
    txn_manager
        .commit_transaction(txn_id)
        .await
        .expect("提交事务失败");

    // Transaction has been committed and is no longer in the active transaction table
    let txn_info = txn_manager.get_transaction_info(txn_id);
    assert!(txn_info.is_none());

    // Verify that savepoint resources have been cleaned up (verify by restarting the transaction)
    let options2 = TransactionOptions::default();
    let txn_id2 = txn_manager
        .begin_transaction(options2)
        .expect("开始事务失败");
    let savepoints = txn_manager.get_active_savepoints(txn_id2);
    assert_eq!(savepoints.len(), 0);

    txn_manager
        .commit_transaction(txn_id2)
        .await
        .expect("提交事务失败");
}

/// Test savepoint resources are automatically cleaned up when transactions are rolled back
#[tokio::test]
async fn test_savepoint_cleanup_on_rollback() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("test_cleanup_rollback.db");

    // Creating Storage
    let storage = RedbStorage::new_with_path(db_path).expect("创建存储失败");
    let db = storage.get_db().clone();

    // Creating a Transaction Manager
    let config = TransactionManagerConfig::default();
    let txn_manager = Arc::new(TransactionManager::new(db, config));

    // Commencement of business
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // Creating multiple save points
    let _savepoint_id1 = txn_manager
        .create_savepoint(txn_id, Some("sp1".to_string()))
        .expect("创建保存点失败");
    let _savepoint_id2 = txn_manager
        .create_savepoint(txn_id, Some("sp2".to_string()))
        .expect("创建保存点失败");

    // Verify that the savepoint has been created
    let savepoints = txn_manager.get_active_savepoints(txn_id);
    assert_eq!(savepoints.len(), 2);

    // Rolling back transactions
    txn_manager.abort_transaction(txn_id).expect("回滚事务失败");

    // The transaction has been aborted and is no longer in the active transaction table
    let txn_info = txn_manager.get_transaction_info(txn_id);
    assert!(txn_info.is_none());
}

/// Test Rollback Failure Error Handling
#[tokio::test]
async fn test_rollback_failure_error_handling() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("test_rollback_error.db");

    // Creating Storage
    let storage = RedbStorage::new_with_path(db_path).expect("创建存储失败");
    let db = storage.get_db().clone();

    // Creating a Transaction Manager
    let config = TransactionManagerConfig::default();
    let txn_manager = Arc::new(TransactionManager::new(db, config));

    // Commencement of business
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // Getting the Transaction Context
    let context = txn_manager.get_context(txn_id).expect("获取事务上下文失败");

    // Add operation log
    use graphdb::transaction::types::OperationLog;
    context.add_operation_log(OperationLog::InsertVertex {
        space: "test_space".to_string(),
        vertex_id: vec![1],
        previous_state: None,
    });

    // Creating a save point
    let savepoint_id = txn_manager
        .create_savepoint(txn_id, Some("sp1".to_string()))
        .expect("创建保存点失败");

    // Add more operation logs
    context.add_operation_log(OperationLog::InsertVertex {
        space: "test_space".to_string(),
        vertex_id: vec![2],
        previous_state: None,
    });

    // Rollback to savepoint (should succeed because TransactionContext handles rollback internally)
    let result = txn_manager.rollback_to_savepoint(txn_id, savepoint_id);

    // Verify rollback results (should be successful)
    assert!(result.is_ok());

    // Verify that the operation log has been truncated to the save point location (there should be 1 log)
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 1);

    // Submission of transactions
    txn_manager
        .commit_transaction(txn_id)
        .await
        .expect("提交事务失败");
}

/// Test concurrent access to operation logs
#[tokio::test]
async fn test_concurrent_operation_log_access() {
    use std::thread;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("test_concurrent.db");

    // Creating Storage
    let storage = RedbStorage::new_with_path(db_path).expect("创建存储失败");
    let db = storage.get_db().clone();

    // Creating a Transaction Manager
    let config = TransactionManagerConfig::default();
    let txn_manager = Arc::new(TransactionManager::new(db, config));

    // Commencement of business
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // Getting the Transaction Context
    let context = txn_manager.get_context(txn_id).expect("获取事务上下文失败");

    // Create multiple threads to access the operation log concurrently
    let mut handles = vec![];

    for i in 0..5 {
        let context_clone = context.clone();
        let handle = thread::spawn(move || {
            use graphdb::transaction::types::OperationLog;

            // Add operation logs.
            context_clone.add_operation_log(OperationLog::InsertVertex {
                space: "test_space".to_string(),
                vertex_id: vec![i as u8],
                previous_state: None,
            });

            // Reading the operation log
            let logs = context_clone.get_operation_logs();
            logs.len()
        });
        handles.push(handle);
    }

    // Wait for all threads to complete.
    for handle in handles {
        handle.join().expect("线程执行失败");
    }

    // Verify that all operation logs have been recorded.
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 5);

    // Commit a transaction
    txn_manager
        .commit_transaction(txn_id)
        .await
        .expect("提交事务失败");
}

/// Testing the log truncation feature
#[tokio::test]
async fn test_operation_log_truncation() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("test_truncation.db");

    // Create storage
    let storage = RedbStorage::new_with_path(db_path).expect("创建存储失败");
    let db = storage.get_db().clone();

    // Create a transaction manager
    let config = TransactionManagerConfig::default();
    let txn_manager = Arc::new(TransactionManager::new(db, config));

    // Start a transaction
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // Obtaining the transaction context
    let context = txn_manager.get_context(txn_id).expect("获取事务上下文失败");

    // Add multiple operation logs
    use graphdb::transaction::types::OperationLog;
    for i in 0..10 {
        context.add_operation_log(OperationLog::InsertVertex {
            space: "test_space".to_string(),
            vertex_id: vec![i as u8],
            previous_state: None,
        });
    }

    // Verify the number of operation logs.
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 10);

    // Truncate the operation log to index 5.
    context.truncate_operation_log(5);

    // The verification operation log has been truncated.
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 5);

    // Submit the transaction
    txn_manager
        .commit_transaction(txn_id)
        .await
        .expect("提交事务失败");
}

/// Cleaning of subsequent savepoints after rolling back to a test savepoint
#[tokio::test]
async fn test_savepoint_cleanup_after_rollback() {
    let txn_manager = create_test_transaction_manager();

    // Start a transaction
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // Create multiple save points.
    let savepoint_id1 = txn_manager
        .create_savepoint(txn_id, Some("sp1".to_string()))
        .expect("创建保存点失败");
    let savepoint_id2 = txn_manager
        .create_savepoint(txn_id, Some("sp2".to_string()))
        .expect("创建保存点失败");
    let savepoint_id3 = txn_manager
        .create_savepoint(txn_id, Some("sp3".to_string()))
        .expect("创建保存点失败");

    // Verify that all savepoints have been created.
    let savepoints = txn_manager.get_active_savepoints(txn_id);
    assert_eq!(savepoints.len(), 3);

    // Roll back to the second save point.
    txn_manager
        .rollback_to_savepoint(txn_id, savepoint_id2)
        .expect("回滚到保存点失败");

    // Verify that the third save point has been removed.
    let savepoints = txn_manager.get_active_savepoints(txn_id);
    assert_eq!(savepoints.len(), 2);

    // Verify the remaining save point IDs.
    let savepoint_ids: Vec<_> = savepoints.iter().map(|sp| sp.id).collect();
    assert!(savepoint_ids.contains(&savepoint_id1));
    assert!(savepoint_ids.contains(&savepoint_id2));
    assert!(!savepoint_ids.contains(&savepoint_id3));

    // Commit a transaction
    txn_manager
        .commit_transaction(txn_id)
        .await
        .expect("提交事务失败");
}

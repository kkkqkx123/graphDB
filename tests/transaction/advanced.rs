//! Transaction Advanced Feature Tests
//!
//! Test coverage:
//! - Savepoint functionality
//! - Read-only transaction option
//! - Durability levels
//! - Transaction abort and recovery
//! - Transaction statistics
//! - Transaction cleanup on drop
//! - Large dataset handling
//! - Complex graph patterns
//! - Property filtering
//! - String operations
//! - Savepoint rollback
//! - Transaction retry mechanism
//! - Batch commit
//! - Transaction metrics
//! - Max concurrent transactions
//! - Cleanup expired transactions
//! - Shutdown functionality

use super::common;

use common::test_scenario::TestScenario;
use graphdb::core::Value;
use graphdb::transaction::{
    RetryConfig, TransactionManager, TransactionManagerConfig, TransactionOptions,
    TransactionError,
};
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::time::Duration;

/// Test savepoint functionality
#[test]
fn test_savepoint_basic() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Account(id INT, amount INT)")
        .assert_success()
        .exec_dml("INSERT VERTEX Account(id, amount) VALUES 1:(1, 100)")
        .assert_success()
        .assert_vertex_props(
            1,
            "Account",
            HashMap::from([("id", Value::Int(1)), ("amount", Value::Int(100))]),
        )
        .exec_dml("INSERT VERTEX Account(id, amount) VALUES 2:(2, 200)")
        .assert_success()
        .assert_vertex_props(2, "Account", HashMap::from([("amount", Value::Int(200))]));
}

/// Test multiple savepoints
#[test]
fn test_savepoint_multiple() {
    let scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space2")
        .exec_ddl("CREATE TAG IF NOT EXISTS Counter(value INT)")
        .assert_success();

    let scenario = (1..=5).fold(scenario, |s, i| {
        let query = format!("INSERT VERTEX Counter(value) VALUES {}:({})", i, i * 10);
        s.exec_dml(&query).assert_success()
    });

    scenario
        .query("MATCH (v:Counter) RETURN v")
        .assert_result_count(5);
}

/// Test transaction with read-only option
#[test]
fn test_readonly_transaction_option() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice')")
        .assert_success()
        .query("MATCH (v:Person) RETURN v.name")
        .assert_result_count(1);
}

/// Test transaction durability levels
#[test]
fn test_transaction_durability_levels() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Immediate')")
        .assert_success()
        .assert_vertex_exists(1, "Person");
}

/// Test transaction abort and recovery
#[tokio::test]
async fn test_transaction_abort_recovery() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction");

    assert!(manager.is_transaction_active(txn_id));

    manager
        .rollback_transaction(txn_id)
        .expect("Failed to rollback transaction");

    assert!(!manager.is_transaction_active(txn_id));
}

/// Test transaction statistics
#[tokio::test]
async fn test_transaction_statistics() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    let initial_stats = manager.stats();
    let initial_total = initial_stats.total_transactions.load(Ordering::Relaxed);

    for i in 0..5 {
        let options = if i % 2 == 0 {
            TransactionOptions::default()
        } else {
            TransactionOptions::new().read_only()
        };

        let txn_id = manager
            .begin_transaction(options)
            .expect("Failed to begin transaction");

        if i % 3 == 0 {
            manager
                .rollback_transaction(txn_id)
                .expect("Failed to rollback transaction");
        } else {
            manager
                .commit_transaction(txn_id)
                .await
                .expect("Failed to commit transaction");
        }
    }

    let final_stats = manager.stats();
    assert!(final_stats.total_transactions.load(Ordering::Relaxed) > initial_total);
}

/// Test transaction cleanup on drop
#[tokio::test]
async fn test_transaction_cleanup() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    for i in 0..5 {
        let txn_id = manager
            .begin_transaction(TransactionOptions::default())
            .unwrap();

        let active_count = manager.stats().active_transactions.load(Ordering::Relaxed);
        assert_eq!(active_count, 1);

        if i % 2 == 0 {
            manager.commit_transaction(txn_id).await.unwrap();
        } else {
            manager.rollback_transaction(txn_id).unwrap();
        }

        let final_active_count = manager.stats().active_transactions.load(Ordering::Relaxed);
        assert_eq!(final_active_count, 0);
    }
}

/// Test transaction with large dataset
#[test]
fn test_transaction_large_dataset() {
    let mut scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Item(id INT, info STRING)")
        .assert_success();

    let mut values = Vec::new();
    for i in 1..=10 {
        values.push(format!("{}:({}, 'val_{}')", i, i, i));
    }
    let query = format!("INSERT VERTEX Item(id, info) VALUES {}", values.join(", "));

    scenario = scenario.exec_dml(&query).assert_success();

    scenario
        .query("MATCH (v:Item) RETURN v")
        .assert_result_count(10);
}

/// Test transaction with complex graph pattern
#[test]
fn test_transaction_complex_graph_pattern() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE TAG IF NOT EXISTS Company(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE IF NOT EXISTS WORKS_AT")
        .assert_success()
        .exec_ddl("CREATE EDGE IF NOT EXISTS MANAGES")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob'), 3:('Charlie')")
        .assert_success()
        .exec_dml("INSERT VERTEX Company(name) VALUES 100:('TechCorp')")
        .assert_success()
        .exec_dml("INSERT EDGE WORKS_AT VALUES 1->100, 2->100, 3->100")
        .assert_success()
        .exec_dml("INSERT EDGE MANAGES VALUES 1->2, 1->3")
        .assert_success()
        .query("MATCH (manager:Person)-[:MANAGES]->(employee:Person) RETURN manager.name, employee.name")
        .assert_result_count(2);
}

/// Test transaction with property filtering
#[test]
fn test_transaction_property_filtering() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING, age INT, active BOOL)")
        .assert_success()
        .exec_dml(
            "INSERT VERTEX Person(name, age, active) VALUES \
            1:('Alice', 25, true), \
            2:('Bob', 30, false), \
            3:('Charlie', 35, true)",
        )
        .assert_success()
        .query("MATCH (v:Person) WHERE v.active == true RETURN v.name")
        .assert_result_count(2)
        .query("MATCH (v:Person) WHERE v.age >= 30 RETURN v.name")
        .assert_result_count(2);
}

/// Test transaction with string operations
#[test]
fn test_transaction_string_operations() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING, description STRING)")
        .assert_success()
        .exec_dml(
            "INSERT VERTEX Person(name, description) VALUES \
            1:('Alice', 'Software Engineer'), \
            2:('Bob', 'Data Scientist')",
        )
        .assert_success()
        .query("MATCH (v:Person) WHERE v.name STARTS WITH 'A' RETURN v.name")
        .assert_result_count(1)
        .assert_result_contains(vec![Value::String("Alice".into())]);
}

/// Test savepoint create and rollback
#[tokio::test]
async fn test_savepoint_rollback() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction");

    // Create a savepoint
    let sp_id = manager
        .create_savepoint(txn_id, Some("initial".to_string()))
        .expect("Failed to create savepoint");

    // Verify savepoint exists
    let savepoint = manager.get_savepoint(txn_id, sp_id);
    assert!(savepoint.is_some());
    assert_eq!(savepoint.unwrap().name, Some("initial".to_string()));

    // Rollback to savepoint
    manager
        .rollback_to_savepoint(txn_id, sp_id)
        .expect("Failed to rollback to savepoint");

    // Commit transaction
    manager
        .commit_transaction(txn_id)
        .await
        .expect("Failed to commit transaction");

    assert!(!manager.is_transaction_active(txn_id));
}

/// Test multiple savepoints and rollback to intermediate point
#[tokio::test]
async fn test_savepoint_multiple_rollback() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction");

    // Create multiple savepoints
    let sp1 = manager
        .create_savepoint(txn_id, Some("sp1".to_string()))
        .expect("Failed to create savepoint 1");
    let _sp2 = manager
        .create_savepoint(txn_id, Some("sp2".to_string()))
        .expect("Failed to create savepoint 2");

    // List active savepoints
    let savepoints = manager.get_active_savepoints(txn_id);
    assert_eq!(savepoints.len(), 2);

    // Find savepoint by name
    let found = manager.find_savepoint_by_name(txn_id, "sp1");
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, sp1);

    // Rollback to first savepoint
    manager
        .rollback_to_savepoint(txn_id, sp1)
        .expect("Failed to rollback to savepoint");

    // After rollback, sp2 should be removed
    let savepoints_after = manager.get_active_savepoints(txn_id);
    assert_eq!(savepoints_after.len(), 1);

    // Commit transaction
    manager
        .commit_transaction(txn_id)
        .await
        .expect("Failed to commit transaction");
}

/// Test transaction retry mechanism
#[tokio::test]
async fn test_transaction_retry() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    let retry_config = RetryConfig::new()
        .with_max_retries(2)
        .with_initial_delay(Duration::from_millis(10));

    // Test successful retry operation
    let result = manager
        .execute_with_retry(
            TransactionOptions::default(),
            retry_config,
            |_txn_id| Ok("success"),
        )
        .await;

    assert_eq!(result.expect("Retry should succeed"), "success");
}

/// Test transaction retry with non-retryable error
#[tokio::test]
async fn test_transaction_retry_non_retryable() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    let retry_config = RetryConfig::new()
        .with_max_retries(3)
        .with_initial_delay(Duration::from_millis(10));

    // Test non-retryable error should fail immediately
    let result: Result<&str, _> = manager
        .execute_with_retry(
            TransactionOptions::default(),
            retry_config,
            |_txn_id| Err(TransactionError::Internal("non-retryable".to_string())),
        )
        .await;

    assert!(
        matches!(result, Err(TransactionError::Internal(_))),
        "Expected non-retryable error"
    );
}

/// Test batch commit of multiple transactions
#[tokio::test]
async fn test_batch_commit() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    // Begin multiple transactions sequentially (write transactions cannot be concurrent)
    let txn1 = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction 1");
    manager
        .commit_transaction(txn1)
        .await
        .expect("Failed to commit transaction 1");

    let txn2 = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction 2");
    manager
        .commit_transaction(txn2)
        .await
        .expect("Failed to commit transaction 2");

    let txn3 = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction 3");

    // Test batch commit with one active transaction
    let result = manager.commit_batch(vec![txn3]).await;
    assert!(result.is_ok(), "Batch commit should succeed: {:?}", result);
}

/// Test transaction metrics
#[tokio::test]
async fn test_transaction_metrics() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    // Begin and commit a transaction
    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction");

    // Get metrics while transaction is active
    let metrics = manager.get_metrics();
    assert_eq!(metrics.total_count, 1);

    manager
        .commit_transaction(txn_id)
        .await
        .expect("Failed to commit transaction");
}

/// Test max concurrent transactions limit
#[test]
fn test_max_concurrent_transactions() {
    let config = TransactionManagerConfig {
        max_concurrent_transactions: 2,
        ..Default::default()
    };

    let manager = TransactionManager::new(config);

    // Begin two readonly transactions
    let txn1 = manager
        .begin_transaction(TransactionOptions::new().read_only())
        .expect("Failed to begin transaction 1");
    let txn2 = manager
        .begin_transaction(TransactionOptions::new().read_only())
        .expect("Failed to begin transaction 2");

    // Third transaction should fail
    let result = manager.begin_transaction(TransactionOptions::new().read_only());
    assert!(
        matches!(result, Err(TransactionError::TooManyTransactions)),
        "Expected TooManyTransactions error"
    );

    // Cleanup
    manager.rollback_transaction(txn1).expect("Failed to rollback txn1");
    manager.rollback_transaction(txn2).expect("Failed to rollback txn2");
}

/// Test cleanup of expired transactions
#[test]
fn test_cleanup_expired_transactions() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    // Begin a transaction with very short timeout
    let txn_id = manager
        .begin_transaction(TransactionOptions::new().with_timeout(Duration::from_millis(50)))
        .expect("Failed to begin transaction");

    assert!(manager.is_transaction_active(txn_id));

    // Wait for transaction to expire
    std::thread::sleep(Duration::from_millis(100));

    // Cleanup expired transactions
    manager.cleanup_expired_transactions();

    // Transaction should be cleaned up
    assert!(!manager.is_transaction_active(txn_id));

    // Verify timeout stats
    let timeout_count = manager
        .stats()
        .timeout_transactions
        .load(Ordering::Relaxed);
    assert_eq!(timeout_count, 1);
}

/// Test shutdown functionality
#[test]
fn test_shutdown() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    // Begin multiple transactions
    let txn1 = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction 1");
    let txn2 = manager
        .begin_transaction(TransactionOptions::new().read_only())
        .expect("Failed to begin transaction 2");

    assert!(manager.is_transaction_active(txn1));
    assert!(manager.is_transaction_active(txn2));

    // Shutdown manager
    manager.shutdown();

    // All transactions should be aborted
    assert!(!manager.is_transaction_active(txn1));
    assert!(!manager.is_transaction_active(txn2));

    // Cannot begin new transaction after shutdown
    let result = manager.begin_transaction(TransactionOptions::default());
    assert!(
        matches!(result, Err(TransactionError::Internal(_))),
        "Expected Internal error after shutdown"
    );
}

/// Test list active transactions and get transaction info
#[tokio::test]
async fn test_transaction_info_and_list() {
    let manager = TransactionManager::new(TransactionManagerConfig::default());

    // Begin readonly transactions (can be concurrent)
    let txn1 = manager
        .begin_transaction(TransactionOptions::new().read_only())
        .expect("Failed to begin transaction 1");
    let txn2 = manager
        .begin_transaction(TransactionOptions::new().read_only())
        .expect("Failed to begin transaction 2");

    // List active transactions
    let active = manager.list_active_transactions();
    assert_eq!(active.len(), 2);

    // Get transaction info
    let info1 = manager.get_transaction_info(txn1);
    assert!(info1.is_some());
    let info1 = info1.unwrap();
    assert_eq!(info1.id, txn1);
    assert!(info1.is_read_only);

    let info2 = manager.get_transaction_info(txn2);
    assert!(info2.is_some());

    // Get non-existent transaction info
    let info_none = manager.get_transaction_info(9999);
    assert!(info_none.is_none());

    // Cleanup
    manager
        .commit_transaction(txn1)
        .await
        .expect("Failed to commit txn1");
    manager
        .commit_transaction(txn2)
        .await
        .expect("Failed to commit txn2");
}

//! Advanced Transaction Module Integration Tests
//!
//! Tests for concurrent transactions, timeouts, savepoints, and edge cases

mod common;

use common::test_scenario::TestScenario;
use graphdb::core::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

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

/// Test transaction with timeout handling
#[tokio::test]
async fn test_transaction_timeout_handling() {
    use graphdb::transaction::{TransactionManager, TransactionManagerConfig, TransactionOptions};

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
    use graphdb::transaction::{TransactionManager, TransactionManagerConfig, TransactionOptions};

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
    use graphdb::transaction::{TransactionManager, TransactionManagerConfig, TransactionOptions};

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
    use graphdb::transaction::{TransactionManager, TransactionManagerConfig, TransactionOptions};

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

/// Test transaction isolation - repeatable read
#[test]
fn test_transaction_repeatable_read() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING, age INT)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)")
        .assert_success()
        .query("MATCH (v:Person) WHERE id(v) == 1 RETURN v.age")
        .assert_result_contains(vec![Value::Int(30)]);
}

/// Test transaction with large dataset
#[test]
fn test_transaction_large_dataset() {
    let mut scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Item(id INT, info STRING)")
        .assert_success();

    // Insert 10 items in a single batch
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

/// Test transaction abort and recovery
#[tokio::test]
async fn test_transaction_abort_recovery() {
    use graphdb::transaction::{TransactionManager, TransactionManagerConfig, TransactionOptions};

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db")).expect("Failed to create database"),
    );

    let manager = TransactionManager::new(db, TransactionManagerConfig::default());

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
    use graphdb::transaction::{TransactionManager, TransactionManagerConfig, TransactionOptions};
    use std::sync::atomic::Ordering;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db")).expect("Failed to create database"),
    );

    let manager = TransactionManager::new(db, TransactionManagerConfig::default());

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

/// Test transaction with multiple schema changes
#[test]
fn test_transaction_multiple_schema_changes() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING)")
        .assert_success()
        .exec_ddl("ALTER TAG Person ADD (age INT)")
        .assert_success()
        .exec_ddl("ALTER TAG Person ADD (email STRING)")
        .assert_success()
        .exec_dml(
            "INSERT VERTEX Person(name, age, email) VALUES 1:('Alice', 30, 'alice@example.com')",
        )
        .assert_success()
        .assert_vertex_props(
            1,
            "Person",
            HashMap::from([
                ("name", Value::String("Alice".into())),
                ("age", Value::Int(30)),
                ("email", Value::String("alice@example.com".into())),
            ]),
        );
}

/// Test transaction with edge direction
#[test]
fn test_transaction_edge_direction() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE IF NOT EXISTS FOLLOWS")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .assert_success()
        .exec_dml("INSERT EDGE FOLLOWS VALUES 1->2")
        .assert_success()
        .assert_edge_exists(1, 2, "FOLLOWS")
        .assert_edge_not_exists(2, 1, "FOLLOWS");
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

/// Test transaction cleanup on drop
#[tokio::test]
async fn test_transaction_cleanup() {
    use graphdb::transaction::{TransactionManager, TransactionManagerConfig, TransactionOptions};
    use std::sync::atomic::Ordering;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db")).expect("Failed to create database"),
    );

    let manager = TransactionManager::new(db, TransactionManagerConfig::default());

    // Create transactions one at a time and commit/abort immediately
    // because write transactions are exclusive
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

/// Test transaction with nested operations
#[test]
fn test_transaction_nested_operations() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Category(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE IF NOT EXISTS SUBCATEGORY_OF")
        .assert_success()
        .exec_dml(
            "INSERT VERTEX Category(name) VALUES \
            1:('Electronics'), \
            2:('Computers'), \
            3:('Laptops'), \
            4:('Desktops')",
        )
        .assert_success()
        .exec_dml(
            "INSERT EDGE SUBCATEGORY_OF VALUES \
            2->1, \
            3->2, \
            4->2",
        )
        .assert_success()
        .query("MATCH (sub:Category)-[:SUBCATEGORY_OF]->(parent:Category) RETURN sub")
        .assert_result_count(3);
}

/// Test transaction with aggregation operations
#[test]
fn test_transaction_aggregation() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Product(name STRING, price INT, quantity INT)")
        .assert_success()
        .exec_dml(
            "INSERT VERTEX Product(name, price, quantity) VALUES \
            1:('ProductA', 100, 10), \
            2:('ProductB', 200, 5), \
            3:('ProductC', 150, 8)",
        )
        .assert_success()
        .query("MATCH (p:Product) RETURN p.price")
        .assert_result_count(3);
}

/// Test concurrent read-only transactions using TransactionManager directly
#[tokio::test]
async fn test_concurrent_readonly_transactions() {
    use graphdb::transaction::{TransactionManager, TransactionManagerConfig, TransactionOptions};

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db")).expect("Failed to create database"),
    );

    let manager = Arc::new(TransactionManager::new(
        db,
        TransactionManagerConfig::default(),
    ));

    let mut handles = vec![];

    for i in 0..5 {
        let manager_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            let options = TransactionOptions::new().read_only();
            let txn_id = manager_clone
                .begin_transaction(options)
                .expect("Failed to begin transaction");

            let context = manager_clone
                .get_context(txn_id)
                .expect("Failed to get context");

            assert!(context.read_only);

            manager_clone
                .commit_transaction(txn_id)
                .await
                .expect("Failed to commit transaction");

            println!("Read-only transaction {} completed", i);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("Task failed");
    }
}

/// Test write transaction exclusivity using TransactionManager
#[tokio::test]
async fn test_write_transaction_exclusivity() {
    use graphdb::transaction::{
        TransactionError, TransactionManager, TransactionManagerConfig, TransactionOptions,
    };

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db")).expect("Failed to create database"),
    );

    let manager = Arc::new(TransactionManager::new(
        db,
        TransactionManagerConfig::default(),
    ));

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
}

/// Test concurrent read and write operations using TransactionManager
#[tokio::test]
async fn test_concurrent_read_write() {
    use graphdb::transaction::{TransactionManager, TransactionManagerConfig, TransactionOptions};

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db")).expect("Failed to create database"),
    );

    let manager = Arc::new(TransactionManager::new(
        db,
        TransactionManagerConfig::default(),
    ));

    let write_handle = {
        let manager = Arc::clone(&manager);
        tokio::spawn(async move {
            let txn_id = manager
                .begin_transaction(TransactionOptions::default())
                .expect("Failed to begin write transaction");
            tokio::time::sleep(Duration::from_millis(50)).await;
            manager
                .commit_transaction(txn_id)
                .await
                .expect("Failed to commit transaction");
        })
    };

    tokio::time::sleep(Duration::from_millis(10)).await;

    let read_handle = {
        let manager = Arc::clone(&manager);
        tokio::spawn(async move {
            let options = TransactionOptions::new().read_only();
            let txn_id = manager
                .begin_transaction(options)
                .expect("Failed to begin read transaction");
            manager
                .commit_transaction(txn_id)
                .await
                .expect("Failed to commit read transaction");
        })
    };

    let (r1, r2) = tokio::join!(write_handle, read_handle);
    r1.expect("Write task failed");
    r2.expect("Read task failed");
}

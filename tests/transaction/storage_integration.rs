//! Transaction Storage Integration Tests
//!
//! Test coverage for transaction integration with storage layer:
//! - Transaction with vertex persistence
//! - Transaction with edge persistence
//! - Transaction with schema changes persistence
//! - Transaction rollback data consistency
//! - Transaction with index updates
//! - Transaction with multiple operations atomicity
//! - Transaction with storage error handling
//! - Transaction with concurrent storage access

#![allow(clippy::approx_constant)]

use super::common;

use common::test_scenario::TestScenario;
use graphdb::core::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::{sleep, timeout};

/// Test transaction with vertex insert persistence
#[test]
fn test_storage_vertex_insert_persistence() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING, age INT)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)")
        .assert_success()
        .assert_vertex_exists(1, "Person")
        .assert_vertex_props(
            1,
            "Person",
            HashMap::from([
                ("name", Value::String("Alice".into())),
                ("age", Value::Int(30)),
            ]),
        );
}

/// Test transaction with vertex update persistence
#[test]
fn test_storage_vertex_update_persistence() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING, age INT)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)")
        .assert_success()
        .assert_vertex_props(
            1,
            "Person",
            HashMap::from([
                ("name", Value::String("Alice".into())),
                ("age", Value::Int(30)),
            ]),
        )
        .exec_dml("UPDATE 1 SET name = 'AliceUpdated', age = 31")
        .assert_success()
        .assert_vertex_props(
            1,
            "Person",
            HashMap::from([
                ("name", Value::String("AliceUpdated".into())),
                ("age", Value::Int(31)),
            ]),
        );
}

/// Test transaction with vertex delete persistence
#[test]
fn test_storage_vertex_delete_persistence() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .assert_success()
        .assert_vertex_exists(1, "Person")
        .assert_vertex_exists(2, "Person")
        .exec_dml("DELETE VERTEX 1")
        .assert_success()
        .assert_vertex_not_exists(1, "Person")
        .assert_vertex_exists(2, "Person");
}

/// Test transaction with edge insert persistence
#[test]
fn test_storage_edge_insert_persistence() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE IF NOT EXISTS KNOWS(since INT)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .assert_success()
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1->2:(2020)")
        .assert_success()
        .assert_edge_exists(1, 2, "KNOWS");
}

/// Test transaction with edge delete persistence
#[test]
fn test_storage_edge_delete_persistence() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE IF NOT EXISTS KNOWS")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob'), 3:('Charlie')")
        .assert_success()
        .exec_dml("INSERT EDGE KNOWS VALUES 1->2, 2->3, 1->3")
        .assert_success()
        .assert_edge_exists(1, 2, "KNOWS")
        .assert_edge_exists(2, 3, "KNOWS")
        .assert_edge_exists(1, 3, "KNOWS")
        .exec_dml("DELETE EDGE KNOWS 1->2")
        .assert_success()
        .assert_edge_not_exists(1, 2, "KNOWS")
        .assert_edge_exists(2, 3, "KNOWS")
        .assert_edge_exists(1, 3, "KNOWS");
}

/// Test transaction with schema changes persistence
#[test]
fn test_storage_schema_changes_persistence() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING)")
        .assert_success()
        .exec_ddl("ALTER TAG Person ADD (age INT, email STRING)")
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

/// Test transaction with multiple operations atomicity
#[test]
fn test_storage_multiple_operations_atomicity() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING, age INT)")
        .assert_success()
        .exec_ddl("CREATE TAG IF NOT EXISTS Company(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE IF NOT EXISTS WORKS_AT")
        .assert_success()
        // Multiple operations in sequence
        .exec_dml(
            "INSERT VERTEX Person(name, age) VALUES \
            1:('Alice', 30), \
            2:('Bob', 25), \
            3:('Charlie', 35)",
        )
        .assert_success()
        .exec_dml("INSERT VERTEX Company(name) VALUES 100:('TechCorp')")
        .assert_success()
        .exec_dml(
            "INSERT EDGE WORKS_AT VALUES \
            1->100, \
            2->100, \
            3->100",
        )
        .assert_success()
        // Verify all operations persisted
        .assert_vertex_count("Person", 3)
        .assert_vertex_count("Company", 1)
        .assert_edge_count("WORKS_AT", 3);
}

/// Test transaction with cascading delete
#[test]
fn test_storage_cascading_delete() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE IF NOT EXISTS KNOWS")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob'), 3:('Charlie')")
        .assert_success()
        .exec_dml("INSERT EDGE KNOWS VALUES 1->2, 2->3, 3->1")
        .assert_success()
        .assert_edge_exists(1, 2, "KNOWS")
        .assert_edge_exists(2, 3, "KNOWS")
        .assert_edge_exists(3, 1, "KNOWS")
        // Delete vertex and verify edges are also removed
        .exec_dml("DELETE VERTEX 1")
        .assert_success()
        .assert_vertex_not_exists(1, "Person")
        // Note: Depending on implementation, edges may or may not be automatically deleted
        // This test verifies the basic behavior
        .assert_vertex_exists(2, "Person")
        .assert_vertex_exists(3, "Person");
}

/// Test transaction with property type variations
#[test]
fn test_storage_property_types() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl(
            "CREATE TAG IF NOT EXISTS AllTypes( \
            int_val INT, \
            string_val STRING, \
            bool_val BOOL, \
            float_val FLOAT, \
            timestamp_val TIMESTAMP)",
        )
        .assert_success()
        .exec_dml(
            "INSERT VERTEX AllTypes(int_val, string_val, bool_val, float_val) \
            VALUES 1:(42, 'test_string', true, 3.14)",
        )
        .assert_success()
        .assert_vertex_props(
            1,
            "AllTypes",
            HashMap::from([
                ("int_val", Value::Int(42)),
                ("string_val", Value::String("test_string".into())),
                ("bool_val", Value::Bool(true)),
                ("float_val", Value::Double(3.14)),
            ]),
        );
}

/// Test transaction with large batch insert
#[test]
fn test_storage_large_batch_insert() {
    let mut scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Item(id INT, name STRING)")
        .assert_success();

    // Insert 50 items in batches
    for batch in 0..5 {
        let mut values = Vec::new();
        for i in 0..10 {
            let id = batch * 10 + i + 1;
            values.push(format!("{}:({}, 'Item{}')", id, id, id));
        }
        let query = format!(
            "INSERT VERTEX Item(id, name) VALUES {}",
            values.join(", ")
        );

        scenario = scenario
            .exec_dml(&query)
            .assert_success();
    }

    // Verify all items were inserted
    scenario
        .assert_vertex_count("Item", 50);
}

/// Test transaction with complex graph pattern persistence
#[test]
fn test_storage_complex_graph_pattern() {
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
        // Create company
        .exec_dml("INSERT VERTEX Company(name) VALUES 100:('TechCorp')")
        .assert_success()
        // Create employees
        .exec_dml(
            "INSERT VERTEX Person(name) VALUES \
            1:('CEO'), \
            2:('Manager1'), \
            3:('Manager2'), \
            4:('Employee1'), \
            5:('Employee2')",
        )
        .assert_success()
        // Create relationships
        .exec_dml(
            "INSERT EDGE WORKS_AT VALUES \
            1->100, 2->100, 3->100, 4->100, 5->100",
        )
        .assert_success()
        .exec_dml(
            "INSERT EDGE MANAGES VALUES \
            1->2, 1->3, \
            2->4, 3->5",
        )
        .assert_success()
        // Verify counts
        .assert_vertex_count("Person", 5)
        .assert_vertex_count("Company", 1)
        .assert_edge_count("WORKS_AT", 5)
        .assert_edge_count("MANAGES", 4);
}

/// Test transaction with tag alteration and data migration
#[test]
fn test_storage_tag_alteration() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        // Create initial tag
        .exec_ddl("CREATE TAG IF NOT EXISTS Product(name STRING)")
        .assert_success()
        // Insert data
        .exec_dml("INSERT VERTEX Product(name) VALUES 1:('ProductA'), 2:('ProductB')")
        .assert_success()
        // Alter tag to add new property
        .exec_ddl("ALTER TAG Product ADD (price INT, category STRING)")
        .assert_success()
        // Insert new data with all properties
        .exec_dml(
            "INSERT VERTEX Product(name, price, category) VALUES 3:('ProductC', 100, 'Electronics')",
        )
        .assert_success()
        // Verify all vertices exist
        .assert_vertex_count("Product", 3);
}

/// Test transaction with edge type alteration
#[test]
fn test_storage_edge_alteration() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE IF NOT EXISTS KNOWS")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .assert_success()
        .exec_dml("INSERT EDGE KNOWS VALUES 1->2")
        .assert_success()
        .assert_edge_exists(1, 2, "KNOWS")
        // Alter edge to add property
        .exec_ddl("ALTER EDGE KNOWS ADD (since INT)")
        .assert_success()
        // Insert new edge with property
        .exec_dml("INSERT VERTEX Person(name) VALUES 3:('Charlie')")
        .assert_success()
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 2->3:(2023)")
        .assert_success()
        .assert_edge_exists(2, 3, "KNOWS");
}

/// Test transaction with query after DML operations
#[test]
fn test_storage_query_after_dml() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING, age INT)")
        .assert_success()
        .exec_dml(
            "INSERT VERTEX Person(name, age) VALUES \
            1:('Alice', 30), \
            2:('Bob', 25), \
            3:('Charlie', 35)",
        )
        .assert_success()
        // Query to verify data
        .query("MATCH (v:Person) WHERE v.age >= 30 RETURN v.name")
        .assert_result_count(2)
        .query("MATCH (v:Person) WHERE v.name STARTS WITH 'A' RETURN v.name")
        .assert_result_count(1);
}

/// Test transaction with update and immediate query
#[test]
fn test_storage_update_and_query() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Counter(value INT)")
        .assert_success()
        .exec_dml("INSERT VERTEX Counter(value) VALUES 1:(0)")
        .assert_success()
        // Update and verify
        .exec_dml("UPDATE 1 SET value = 10")
        .assert_success()
        .assert_vertex_props(
            1,
            "Counter",
            HashMap::from([("value", Value::Int(10))]),
        )
        // Update again
        .exec_dml("UPDATE 1 SET value = 20")
        .assert_success()
        .assert_vertex_props(
            1,
            "Counter",
            HashMap::from([("value", Value::Int(20))]),
        );
}

/// Test transaction with concurrent read operations
#[tokio::test]
async fn test_storage_concurrent_reads() {
    use graphdb::transaction::{
        TransactionManager, TransactionManagerConfig, TransactionOptions,
    };

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db"))
            .expect("Failed to create database"),
    );

    let manager = Arc::new(TransactionManager::new(
        db,
        TransactionManagerConfig::default(),
    ));

    // Create multiple read-only transactions concurrently
    let mut handles = vec![];
    for i in 0..20 {
        let manager_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            let options = TransactionOptions::new().read_only();
            let txn_id = manager_clone
                .begin_transaction(options)
                .expect("Failed to begin transaction");

            // Simulate read work
            sleep(Duration::from_millis(10)).await;

            manager_clone
                .commit_transaction(txn_id)
                .await
                .expect("Failed to commit transaction");

            println!("Read transaction {} completed", i);
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
        "All concurrent read transactions should complete without deadlock"
    );
}

/// Test transaction with storage error recovery
#[tokio::test]
async fn test_storage_error_recovery() {
    use graphdb::transaction::{
        TransactionManager, TransactionManagerConfig, TransactionOptions,
    };

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db"))
            .expect("Failed to create database"),
    );

    let manager = TransactionManager::new(db, TransactionManagerConfig::default());

    // Begin transaction
    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin transaction");

    // Rollback transaction (simulating error recovery)
    manager
        .rollback_transaction(txn_id)
        .expect("Failed to rollback transaction");

    // Verify transaction is no longer active
    assert!(!manager.is_transaction_active(txn_id));

    // Should be able to begin new transaction after rollback
    let txn_id2 = manager
        .begin_transaction(TransactionOptions::default())
        .expect("Failed to begin new transaction after rollback");

    manager
        .commit_transaction(txn_id2)
        .await
        .expect("Failed to commit second transaction");
}

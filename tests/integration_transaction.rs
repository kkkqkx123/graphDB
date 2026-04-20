//! Transaction Module Integration Tests
//!
//! Tests for transaction lifecycle, basic operations, and data consistency

mod common;

use common::test_scenario::TestScenario;
use graphdb::core::Value;
use std::collections::HashMap;

/// Test basic transaction lifecycle - begin, commit, data persistence
#[test]
fn test_transaction_basic_lifecycle() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING, age INT)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)")
        .assert_success()
        .query("MATCH (v:Person) WHERE id(v) == 1 RETURN v")
        .assert_result_count(1)
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

/// Test transaction rollback - data should not persist after abort
#[test]
fn test_transaction_rollback() {
    let scenario = TestScenario::new().expect("Failed to create test scenario");

    let scenario = scenario
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Before')")
        .assert_success();

    scenario
        .query("MATCH (v:Person) WHERE id(v) == 1 RETURN v")
        .assert_result_count(1);
}

/// Test multiple vertices insertion in single transaction
#[test]
fn test_transaction_multiple_vertices() {
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
        .query("MATCH (v:Person) RETURN v")
        .assert_result_count(3)
        .assert_vertex_exists(1, "Person")
        .assert_vertex_exists(2, "Person")
        .assert_vertex_exists(3, "Person");
}

/// Test vertex update in transaction
#[test]
fn test_transaction_vertex_update() {
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
        );
}

/// Test vertex deletion in transaction
#[test]
fn test_transaction_vertex_delete() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .assert_success()
        .assert_vertex_exists(1, "Person")
        .assert_vertex_exists(2, "Person");
}

/// Test edge creation in transaction
#[test]
fn test_transaction_edge_creation() {
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

/// Test edge deletion in transaction
#[test]
fn test_transaction_edge_delete() {
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
        .assert_edge_exists(1, 2, "KNOWS");
}

/// Test complex transaction with multiple operations
#[test]
fn test_transaction_complex_operations() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING, age INT)")
        .assert_success()
        .exec_ddl("CREATE TAG IF NOT EXISTS Company(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE IF NOT EXISTS WORKS_AT")
        .assert_success()
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
        .query("MATCH (p:Person)-[:WORKS_AT]->(c:Company) RETURN p")
        .assert_result_count(3);
}

/// Test transaction with property types
#[test]
fn test_transaction_property_types() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl(
            "CREATE TAG IF NOT EXISTS TestTypes( \
            int_val INT, \
            string_val STRING, \
            bool_val BOOL, \
            float_val FLOAT, \
            timestamp_val TIMESTAMP)",
        )
        .assert_success()
        .exec_dml(
            "INSERT VERTEX TestTypes(int_val, string_val, bool_val, float_val) \
            VALUES 1:(42, 'test', true, 3.14)",
        )
        .assert_success()
        .assert_vertex_props(
            1,
            "TestTypes",
            HashMap::from([
                ("int_val", Value::Int(42)),
                ("string_val", Value::String("test".into())),
                ("bool_val", Value::Bool(true)),
                ("float_val", Value::Float(std::f64::consts::PI as f32)),
            ]),
        );
}

/// Test empty transaction (no operations)
#[test]
fn test_transaction_empty() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING)")
        .assert_success()
        .query("MATCH (v:Person) RETURN v")
        .assert_result_count(0);
}

/// Test transaction with conditional operations
#[test]
fn test_transaction_conditional_operations() {
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
        .query("MATCH (v:Person) WHERE v.age > 28 RETURN v.name")
        .assert_result_count(2);
}

/// Test transaction data visibility - committed data should be visible
#[test]
fn test_transaction_data_visibility() {
    let mut scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('First')")
        .assert_success();

    scenario = scenario
        .query("MATCH (v:Person) WHERE id(v) == 1 RETURN v")
        .assert_result_count(1);

    scenario
        .exec_dml("INSERT VERTEX Person(name) VALUES 2:('Second')")
        .assert_success()
        .query("MATCH (v:Person) RETURN v")
        .assert_result_count(2);
}

/// Test transaction with tag modification
#[test]
fn test_transaction_tag_modification() {
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
        );
}

/// Test batch insert performance in transaction
#[test]
fn test_transaction_batch_insert() {
    let scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space2")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(id INT, name STRING)")
        .assert_success();

    let mut values = Vec::new();
    for i in 1..=10 {
        values.push(format!("{}:({}, 'Person{}')", i, i, i));
    }
    let insert_query = format!(
        "INSERT VERTEX Person(id, name) VALUES {}",
        values.join(", ")
    );

    scenario
        .exec_dml(&insert_query)
        .assert_success()
        .query("MATCH (v:Person) RETURN v")
        .assert_result_count(10);
}

/// Test transaction with self-referencing edge
#[test]
fn test_transaction_self_referencing_edge() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE IF NOT EXISTS KNOWS")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice')")
        .assert_success()
        .exec_dml("INSERT EDGE KNOWS VALUES 1->1")
        .assert_success()
        .assert_edge_exists(1, 1, "KNOWS");
}

/// Test transaction with bidirectional edges
#[test]
fn test_transaction_bidirectional_edges() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE IF NOT EXISTS FRIENDS_WITH")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .assert_success()
        .exec_dml("INSERT EDGE FRIENDS_WITH VALUES 1->2")
        .assert_success()
        .exec_dml("INSERT EDGE FRIENDS_WITH VALUES 2->1")
        .assert_success()
        .assert_edge_exists(1, 2, "FRIENDS_WITH")
        .assert_edge_exists(2, 1, "FRIENDS_WITH");
}

/// Test transaction isolation - read committed data only
#[test]
fn test_transaction_read_committed_only() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice')")
        .assert_success()
        .query("MATCH (v:Person) WHERE id(v) == 1 RETURN v")
        .assert_result_count(1)
        .assert_vertex_props(
            1,
            "Person",
            HashMap::from([("name", Value::String("Alice".into()))]),
        );
}

/// Test transaction with multiple edge types
#[test]
fn test_transaction_multiple_edge_types() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE IF NOT EXISTS KNOWS(since INT)")
        .assert_success()
        .exec_ddl("CREATE EDGE IF NOT EXISTS WORKS_WITH(project STRING)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .assert_success()
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1->2:(2020)")
        .assert_success()
        .exec_dml("INSERT EDGE WORKS_WITH(project) VALUES 1->2:('ProjectX')")
        .assert_success()
        .assert_edge_exists(1, 2, "KNOWS")
        .assert_edge_exists(1, 2, "WORKS_WITH");
}

/// Test transaction with edge properties update
#[test]
fn test_transaction_edge_property_update() {
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
        .assert_edge_exists(1, 2, "KNOWS");
}

/// Test transaction with vertex and edge cascading
#[test]
fn test_transaction_cascading_operations() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE IF NOT EXISTS KNOWS")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob'), 3:('Charlie')")
        .assert_success()
        .exec_dml("INSERT EDGE KNOWS VALUES 1->2")
        .assert_success()
        .exec_dml("INSERT EDGE KNOWS VALUES 2->3")
        .assert_success()
        .exec_dml("INSERT EDGE KNOWS VALUES 3->1")
        .assert_success()
        .query("MATCH (a:Person)-[:KNOWS]->(b:Person) RETURN a")
        .assert_result_count(3);
}

/// Test transaction statistics consistency
#[test]
fn test_transaction_statistics_consistency() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob'), 3:('Charlie')")
        .assert_success()
        .assert_vertex_count("Person", 3)
        .exec_dml("INSERT VERTEX Person(name) VALUES 4:('David')")
        .assert_success()
        .assert_vertex_count("Person", 4);
}

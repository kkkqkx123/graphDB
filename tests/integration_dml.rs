//! Data Manipulation Language (DML) Integration Tests
//!
//! Test coverage:
//! - INSERT - Insert data
//! - CREATE - Create data (Cypher style)
//! - UPDATE - Update data
//! - DELETE - Delete data
//! - MERGE - Merge data
//! - SET - Set properties
//! - REMOVE - Remove properties
//! - UPSERT - Upsert operation

mod common;

use common::test_scenario::TestScenario;
use common::TestStorage;
use graphdb::core::stats::StatsManager;
use graphdb::core::Value;
use graphdb::query::optimizer::OptimizerEngine;
use graphdb::query::parser::Parser;
use graphdb::query::query_pipeline_manager::QueryPipelineManager;
use std::collections::HashMap;
use std::sync::Arc;

// ==================== INSERT Statement Tests ====================

#[test]
fn test_insert_parser_vertex() {
    let query = "INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "INSERT VERTEX parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("INSERT statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "INSERT");
}

#[test]
fn test_insert_parser_multiple_vertices() {
    let query = "INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30), 2:('Bob', 25)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "INSERT multiple vertices parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("INSERT statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "INSERT");
}

#[test]
fn test_insert_parser_edge() {
    let query = "INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01')";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "INSERT EDGE parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("INSERT statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "INSERT");
}

#[test]
fn test_insert_parser_edge_with_rank() {
    let query = "INSERT EDGE KNOWS(since) VALUES 1 -> 2 @0:('2020-01-01')";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "INSERT EDGE with rank parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("INSERT statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "INSERT");
}

#[test]
fn test_insert_parser_multiple_edges() {
    let query = "INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01'), 2 -> 3:('2021-01-01')";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "INSERT multiple edges parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("INSERT statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "INSERT");
}

#[test]
fn test_insert_parser_invalid_syntax() {
    let query = "INSERT VERTEX Person(name, age) VALUES 1:'Alice', 30";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(result.is_err(), "Invalid syntax should trigger an error.");
}

#[test]
fn test_insert_execution_vertex() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
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

#[test]
fn test_insert_execution_multiple_vertices() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30), 2:('Bob', 25)")
        .assert_success()
        .assert_vertex_exists(1, "Person")
        .assert_vertex_exists(2, "Person")
        .assert_vertex_count("Person", 2);
}

#[test]
fn test_insert_execution_edge() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01')")
        .assert_success()
        .assert_edge_exists(1, 2, "KNOWS");
}

#[test]
fn test_insert_execution_multiple_edges() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob'), 3:('Charlie')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01'), 2 -> 3:('2021-01-01')")
        .assert_success()
        .assert_edge_exists(1, 2, "KNOWS")
        .assert_edge_exists(2, 3, "KNOWS")
        .assert_edge_count("KNOWS", 2);
}

// ==================== CREATE Statement Tests (Cypher style) ====================

#[test]
fn test_create_parser_vertex() {
    let query = "CREATE (p:Person {name: 'Alice', age: 30})";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    println!("CREATE vertex parsing result: {:?}", result);
    let _ = result;
}

#[test]
fn test_create_parser_edge() {
    let query = "CREATE (a:Person)-[:KNOWS {since: '2020-01-01'}]->(b:Person)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    println!("CREATE edge parsing result: {:?}", result);
    let _ = result;
}

#[test]
fn test_create_parser_multiple() {
    let query = "CREATE (a:Person {name: 'Alice'}), (b:Person {name: 'Bob'})";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    println!("CREATE multiple vertices parsing result: {:?}", result);
    let _ = result;
}

#[test]
fn test_create_execution_vertex() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        .exec_dml("CREATE (p:Person {name: 'Alice', age: 30})")
        .assert_success();
}

#[test]
fn test_create_execution_edge() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        .exec_dml("CREATE (a:Person {name: 'Alice'}), (b:Person {name: 'Bob'})")
        .exec_dml("CREATE (a)-[:KNOWS {since: '2020-01-01'}]->(b)")
        .assert_success();
}

// ==================== UPDATE Statement Tests ====================

#[test]
fn test_update_parser_vertex() {
    let query = "UPDATE 1 SET age = 26, name = 'Alice Smith'";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "UPDATE vertex parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("UPDATE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "UPDATE");
}

#[test]
fn test_update_parser_vertex_with_when() {
    let query = "UPDATE 1 SET age = 26 WHEN age > 20";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "UPDATE vertex with WHEN parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("UPDATE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "UPDATE");
}

#[test]
fn test_update_parser_edge() {
    let query = "UPDATE 1 -> 2 @0 OF KNOWS SET since = '2021-01-01'";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "UPDATE edge parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("UPDATE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "UPDATE");
}

#[test]
fn test_update_parser_edge_with_when() {
    let query = "UPDATE 1 -> 2 @0 OF KNOWS SET since = '2021-01-01' WHEN since < '2021-01-01'";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "UPDATE edge with WHEN parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("UPDATE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "UPDATE");
}

#[test]
fn test_update_parser_multiple_props() {
    let query = "UPDATE 1 SET age = 26, name = 'Alice', updated = true";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "UPDATE multiple properties parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("UPDATE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "UPDATE");
}

#[test]
fn test_update_execution_vertex() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)")
        .exec_dml("UPDATE 1 SET age = 31")
        .assert_success()
        .assert_vertex_props(
            1,
            "Person",
            HashMap::from([
                ("name", Value::String("Alice".into())),
                ("age", Value::Int(31)),
            ]),
        );
}

#[test]
fn test_update_execution_vertex_multiple_props() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT, city: STRING)")
        .exec_dml("INSERT VERTEX Person(name, age, city) VALUES 1:('Alice', 30, 'NYC')")
        .exec_dml("UPDATE 1 SET age = 31, name = 'Alice Smith', city = 'LA'")
        .assert_success()
        .assert_vertex_props(
            1,
            "Person",
            HashMap::from([
                ("name", Value::String("Alice Smith".into())),
                ("age", Value::Int(31)),
                ("city", Value::String("LA".into())),
            ]),
        );
}

#[test]
fn test_update_execution_edge() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01')")
        .exec_dml("UPDATE 1 -> 2 OF KNOWS SET since = '2021-01-01'")
        .assert_success();
}

// ==================== DELETE Statement Tests ====================

#[test]
fn test_delete_parser_vertex() {
    let query = "DELETE VERTEX 1";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DELETE VERTEX parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DELETE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DELETE");
}

#[test]
fn test_delete_parser_multiple_vertices() {
    let query = "DELETE VERTEX 1, 2, 3";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DELETE multiple vertices parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DELETE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DELETE");
}

#[test]
fn test_delete_parser_edge() {
    let query = "DELETE EDGE KNOWS 1 -> 2";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DELETE EDGE parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DELETE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DELETE");
}

#[test]
fn test_delete_parser_edge_with_rank() {
    let query = "DELETE EDGE KNOWS 1 -> 2 @0";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DELETE EDGE with rank parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DELETE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DELETE");
}

#[test]
fn test_delete_parser_multiple_edges() {
    let query = "DELETE EDGE KNOWS 1 -> 2, 2 -> 3";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DELETE multiple edges parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DELETE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DELETE");
}

#[test]
fn test_delete_execution_vertex() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice')")
        .assert_vertex_exists(1, "Person")
        .exec_dml("DELETE VERTEX 1")
        .assert_success()
        .assert_vertex_not_exists(1, "Person");
}

#[test]
fn test_delete_execution_multiple_vertices() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob'), 3:('Charlie')")
        .assert_vertex_count("Person", 3)
        .exec_dml("DELETE VERTEX 1, 2")
        .assert_success()
        .assert_vertex_not_exists(1, "Person")
        .assert_vertex_not_exists(2, "Person")
        .assert_vertex_exists(3, "Person")
        .assert_vertex_count("Person", 1);
}

#[test]
fn test_delete_execution_edge() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01')")
        .assert_edge_exists(1, 2, "KNOWS")
        .exec_dml("DELETE EDGE KNOWS 1 -> 2")
        .assert_success()
        .assert_edge_not_exists(1, 2, "KNOWS");
}

#[test]
fn test_delete_execution_multiple_edges() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob'), 3:('Charlie')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01'), 2 -> 3:('2021-01-01')")
        .assert_edge_count("KNOWS", 2)
        .exec_dml("DELETE EDGE KNOWS 1 -> 2, 2 -> 3")
        .assert_success()
        .assert_edge_not_exists(1, 2, "KNOWS")
        .assert_edge_not_exists(2, 3, "KNOWS")
        .assert_edge_count("KNOWS", 0);
}

// ==================== Additional DML Functionality Tests ====================

#[test]
fn test_insert_if_not_exists_parser() {
    let query = "INSERT VERTEX IF NOT EXISTS Person(name, age) VALUES 1:('Alice', 30)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "INSERT IF NOT EXISTS parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("INSERT statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "INSERT");
}

#[test]
fn test_insert_if_not_exists_execution() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        .exec_dml("INSERT VERTEX IF NOT EXISTS Person(name, age) VALUES 1:('Alice', 30)")
        .assert_success()
        .assert_vertex_exists(1, "Person")
        .assert_vertex_props(
            1,
            "Person",
            HashMap::from([
                ("name", Value::String("Alice".into())),
                ("age", Value::Int(30)),
            ]),
        )
        // Second insert should not fail
        .exec_dml("INSERT VERTEX IF NOT EXISTS Person(name, age) VALUES 1:('Bob', 25)")
        .assert_success()
        // Properties should remain unchanged
        .assert_vertex_props(
            1,
            "Person",
            HashMap::from([
                ("name", Value::String("Alice".into())),
                ("age", Value::Int(30)),
            ]),
        );
}

#[test]
fn test_insert_multiple_tags_parser() {
    let query = "INSERT VERTEX Person(name, age), Employee(department, salary) VALUES 1:('Alice', 30):('Engineering', 100000)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "INSERT multiple tags parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("INSERT statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "INSERT");
}

#[test]
fn test_upsert_vertex_parser() {
    let query = "UPSERT VERTEX 1 ON Person SET age = 26, name = 'Alice Smith'";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "UPSERT VERTEX parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("UPSERT statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "UPDATE");
}

#[test]
fn test_upsert_edge_parser() {
    let query = "UPSERT EDGE 1 -> 2 @0 OF KNOWS SET since = '2021-01-01'";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "UPSERT EDGE parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("UPSERT statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "UPDATE");
}

#[test]
fn test_update_with_yield_parser() {
    let query = "UPDATE 1 SET age = 26 YIELD age AS new_age";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "UPDATE with YIELD parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("UPDATE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "UPDATE");
}

#[test]
fn test_update_vertex_on_tag_parser() {
    let query = "UPDATE VERTEX 1 ON Person SET age = 26";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "UPDATE VERTEX ON Tag parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("UPDATE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "UPDATE");
}

#[test]
fn test_delete_tag_wildcard_parser() {
    let query = "DELETE TAG * FROM 1";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DELETE TAG * parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DELETE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DELETE");
}

#[test]
fn test_delete_tag_specific_parser() {
    let query = "DELETE TAG Person, Employee FROM 1";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DELETE TAG specific tags parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DELETE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DELETE");
}

#[test]
fn test_delete_tag_multiple_vertices_parser() {
    let query = "DELETE TAG Person FROM 1, 2, 3";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DELETE TAG multiple vertices parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DELETE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DELETE");
}

// ==================== MERGE Statement Tests ====================

#[test]
fn test_merge_parser_basic() {
    let query = "MERGE (p:Person {name: 'Alice'})";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    println!("MERGE basic parsing result: {:?}", result);
    let _ = result;
}

#[test]
fn test_merge_parser_on_match() {
    let query = "MERGE (p:Person {name: 'Alice'}) ON MATCH SET p.last_seen = timestamp()";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    println!("MERGE with ON MATCH parsing result: {:?}", result);
    let _ = result;
}

#[test]
fn test_merge_parser_on_create() {
    let query = "MERGE (p:Person {name: 'Alice'}) ON CREATE SET p.created_at = timestamp()";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    println!("MERGE with ON CREATE parsing result: {:?}", result);
    let _ = result;
}

#[test]
fn test_merge_parser_both() {
    let query = "MERGE (p:Person {name: 'Alice'}) ON MATCH SET p.last_seen = timestamp() ON CREATE SET p.created_at = timestamp()";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    println!("MERGE with ON MATCH and ON CREATE parsing result: {:?}", result);
    let _ = result;
}

#[test]
fn test_merge_execution_basic() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .exec_dml("MERGE (p:Person {name: 'Alice'})")
        .assert_success();
}

// ==================== SET Statement Tests ====================

#[test]
fn test_set_parser_basic() {
    let query = "SET p.age = 26";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "SET basic parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("SET statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "SET");
}

#[test]
fn test_set_parser_multiple() {
    let query = "SET p.age = 26, p.name = 'Alice', p.updated = true";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "SET multiple properties parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("SET statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "SET");
}

#[test]
fn test_set_parser_with_expression() {
    let query = "SET p.age = p.age + 1";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "SET with expression parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("SET statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "SET");
}

#[test]
fn test_set_execution_basic() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)")
        .exec_dml("SET 1.age = 31")
        .assert_success()
        .assert_vertex_props(
            1,
            "Person",
            HashMap::from([
                ("name", Value::String("Alice".into())),
                ("age", Value::Int(31)),
            ]),
        );
}

// ==================== REMOVE Statement Tests ====================

#[test]
fn test_remove_parser_property() {
    let query = "REMOVE p.temp_field";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "REMOVE property parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("REMOVE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "REMOVE");
}

#[test]
fn test_remove_parser_multiple_properties() {
    let query = "REMOVE p.temp_field, p.old_field";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "REMOVE multiple properties parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("REMOVE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "REMOVE");
}

#[test]
fn test_remove_parser_label() {
    let query = "REMOVE p:OldLabel";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "REMOVE label parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("REMOVE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "REMOVE");
}

#[test]
fn test_remove_parser_multiple_labels() {
    let query = "REMOVE p:OldLabel, p:AnotherLabel";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "REMOVE multiple labels parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("REMOVE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "REMOVE");
}

#[test]
fn test_remove_parser_mixed() {
    let query = "REMOVE p.temp_field, p:OldLabel";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "REMOVE mixed parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("REMOVE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "REMOVE");
}

#[test]
fn test_remove_execution_property() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, temp_field: STRING)")
        .exec_dml("INSERT VERTEX Person(name, temp_field) VALUES 1:('Alice', 'temp_value')")
        .exec_dml("REMOVE 1.temp_field")
        .assert_success();
}

// ==================== Comprehensive DML Tests ====================

#[test]
fn test_dml_crud_operations() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        // Create schema
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        // Create
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)")
        .assert_success()
        .assert_vertex_exists(1, "Person")
        // Read
        .query("FETCH PROP ON Person 1")
        .assert_success()
        .assert_result_count(1)
        // Update
        .exec_dml("UPDATE 1 SET age = 31")
        .assert_success()
        .assert_vertex_props(
            1,
            "Person",
            HashMap::from([("age", Value::Int(31))]),
        )
        // Delete
        .exec_dml("DELETE VERTEX 1")
        .assert_success()
        .assert_vertex_not_exists(1, "Person");
}

#[test]
fn test_dml_batch_operations() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        // Batch insert vertices
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30), 2:('Bob', 25), 3:('Charlie', 35)")
        .assert_success()
        .assert_vertex_count("Person", 3)
        // Batch insert edges
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01'), 2 -> 3:('2021-01-01')")
        .assert_success()
        .assert_edge_count("KNOWS", 2)
        // Batch update
        .exec_dml("UPDATE 1 SET age = 31, name = 'Alice Smith'")
        .assert_success()
        // Batch delete
        .exec_dml("DELETE VERTEX 1, 2, 3")
        .assert_success()
        .assert_vertex_count("Person", 0);
}

#[test]
fn test_dml_error_handling() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    let invalid_queries = vec![
        "INSERT VERTEX Person(name, age) VALUES 1:'Alice', 30", // Invalid grammar
        "UPDATE SET age = 26",                                  // Missing vertex ID
        "DELETE VERTEX",                                        // Missing vertex ID
        "SET = 26",                                             // Missing variable
    ];

    for query in invalid_queries {
        let result = pipeline_manager.execute_query(query);
        assert!(result.is_err(), "Invalid query should return error: {}", query);
    }
}

#[test]
fn test_dml_transaction_like_operations() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        // Insert vertex
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)")
        .assert_success()
        // Insert edge (requires both vertices)
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 2:('Bob', 25)")
        .assert_success()
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01')")
        .assert_success()
        .assert_edge_exists(1, 2, "KNOWS")
        // Update
        .exec_dml("UPDATE 1 SET age = 31")
        .assert_success()
        // Fetch
        .query("FETCH PROP ON Person 1")
        .assert_success()
        .assert_result_count(1);
}

// ==================== Index Optimization Tests ====================

#[test]
fn test_index_scan_with_limit() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        .exec_ddl("CREATE TAG INDEX person_age_index ON Person(age)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30), 2:('Bob', 25), 3:('Charlie', 35), 4:('David', 28), 5:('Eve', 32)")
        .query("LOOKUP ON Person WHERE Person.age > 25 YIELD Person.name, Person.age LIMIT 2")
        .assert_success()
        .assert_result_count(2);
}

#[test]
fn test_index_scan_with_order_by_limit() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        .exec_ddl("CREATE TAG INDEX person_age_index ON Person(age)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30), 2:('Bob', 25), 3:('Charlie', 35), 4:('David', 28), 5:('Eve', 32)")
        .query("LOOKUP ON Person WHERE Person.age > 20 YIELD Person.name, Person.age ORDER BY Person.age DESC LIMIT 3")
        .assert_success()
        .assert_result_count(3);
}

#[test]
fn test_index_covering_scan() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        .exec_ddl("CREATE TAG INDEX person_name_age_index ON Person(name, age)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30), 2:('Bob', 25), 3:('Charlie', 35)")
        .query("LOOKUP ON Person WHERE Person.name == 'Alice' YIELD Person.name, Person.age")
        .assert_success()
        .assert_result_count(1);
}

#[test]
fn test_index_scan_with_filter_optimization() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT, city: STRING)")
        .exec_ddl("CREATE TAG INDEX person_age_city_index ON Person(age, city)")
        .exec_dml("INSERT VERTEX Person(name, age, city) VALUES 1:('Alice', 30, 'Beijing'), 2:('Bob', 25, 'Shanghai'), 3:('Charlie', 35, 'Beijing')")
        .query("LOOKUP ON Person WHERE Person.age > 25 AND Person.city == 'Beijing' YIELD Person.name, Person.age, Person.city")
        .assert_success()
        .assert_result_count(2);
}

#[test]
fn test_dml_with_index_optimization() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        .exec_ddl("CREATE TAG INDEX person_age_index ON Person(age)")
        // Insert data
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30), 2:('Bob', 25), 3:('Charlie', 35)")
        .assert_success()
        // Update
        .exec_dml("UPDATE 1 SET age = 31")
        .assert_success()
        // Query with index
        .query("LOOKUP ON Person WHERE Person.age > 25 YIELD Person.name, Person.age LIMIT 2")
        .assert_success()
        // Delete
        .exec_dml("DELETE VERTEX 3")
        .assert_success()
        .assert_vertex_not_exists(3, "Person");
}

#[test]
fn test_edge_index_scan_with_limit() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since: STRING)")
        .exec_ddl("CREATE EDGE INDEX knows_since_index ON KNOWS(since)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob'), 3:('Charlie')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01'), 1 -> 3:('2021-01-01'), 2 -> 3:('2019-01-01')")
        .query("LOOKUP ON KNOWS WHERE KNOWS.since > '2019-06-01' YIELD KNOWS.since LIMIT 2")
        .assert_success()
        .assert_result_count(2);
}

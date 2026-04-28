//! DML Upsert Tests
//!
//! Test coverage:
//! - UPSERT VERTEX - Insert or update vertex
//! - UPSERT EDGE - Insert or update edge
//! - MERGE - Merge operation

use super::common;

use common::test_scenario::TestScenario;
use graphdb::query::parser::Parser;

// ==================== UPSERT VERTEX Parser Tests ====================

#[test]
fn test_upsert_parser_vertex() {
    let query = "UPSERT VERTEX ON Person SET name = 'Alice', age = 30 WHERE id(vid) == 1";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "UPSERT VERTEX parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("UPSERT statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "UPSERT");
}

#[test]
fn test_upsert_parser_vertex_with_when() {
    let query = "UPSERT VERTEX ON Person SET age = age + 1 WHERE id(vid) == 1 WHEN age < 100";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "UPSERT with WHEN parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("UPSERT statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "UPSERT");
}

#[test]
fn test_upsert_parser_vertex_with_yield() {
    let query = "UPSERT VERTEX ON Person SET name = 'Bob' WHERE id(vid) == 1 YIELD name, age";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "UPSERT with YIELD parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("UPSERT statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "UPSERT");
}

// ==================== UPSERT VERTEX Execution Tests ====================

#[test]
fn test_upsert_execution_vertex_insert() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .exec_dml("UPSERT VERTEX ON Person SET name = 'Alice', age = 30 WHERE id(vid) == 1")
        .assert_success()
        .assert_vertex_exists(1, "Person");
}

#[test]
fn test_upsert_execution_vertex_update() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)")
        .assert_success()
        .exec_dml("UPSERT VERTEX ON Person SET name = 'Bob', age = 35 WHERE id(vid) == 1")
        .assert_success();
}

// ==================== UPSERT EDGE Tests ====================

#[test]
fn test_upsert_parser_edge() {
    let query = "UPSERT EDGE ON KNOWS SET since = '2024-01-01' WHERE id(src) == 1 AND id(dst) == 2";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "UPSERT EDGE parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("UPSERT statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "UPSERT");
}

#[test]
fn test_upsert_execution_edge_insert() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .exec_dml("UPSERT EDGE ON KNOWS SET since = '2024-01-01' WHERE id(src) == 1 AND id(dst) == 2")
        .assert_success()
        .assert_edge_exists(1, 2, "KNOWS");
}

#[test]
fn test_upsert_execution_edge_update() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2024-01-01')")
        .assert_success()
        .exec_dml("UPSERT EDGE ON KNOWS SET since = '2024-02-01' WHERE id(src) == 1 AND id(dst) == 2")
        .assert_success();
}

// ==================== MERGE Tests ====================

#[test]
fn test_merge_parser_vertex() {
    let query = "MERGE (v:Person {name: 'Alice'}) SET v.age = 30";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "MERGE VERTEX parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("MERGE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "MERGE");
}

#[test]
fn test_merge_parser_edge() {
    let query = "MERGE (a)-[r:KNOWS {since: '2024-01-01'}]->(b) SET r.weight = 1.0";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "MERGE EDGE parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("MERGE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "MERGE");
}

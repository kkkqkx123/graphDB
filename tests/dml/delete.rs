//! DML Delete Tests
//!
//! Test coverage:
//! - DELETE VERTEX - Delete vertices
//! - DELETE EDGE - Delete edges
//! - DELETE with CASCADE

use super::common;

use common::test_scenario::TestScenario;
use graphdb::query::parser::Parser;

// ==================== DELETE VERTEX Parser Tests ====================

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
fn test_delete_parser_vertex_with_edge() {
    let query = "DELETE VERTEX 1 WITH EDGE";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DELETE VERTEX WITH EDGE parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DELETE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DELETE");
}

// ==================== DELETE VERTEX Execution Tests ====================

#[test]
fn test_delete_execution_vertex() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice')")
        .assert_success()
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
        .exec_ddl("CREATE TAG Person(name STRING)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob'), 3:('Charlie')")
        .assert_success()
        .exec_dml("DELETE VERTEX 1, 2")
        .assert_success()
        .assert_vertex_not_exists(1, "Person")
        .assert_vertex_not_exists(2, "Person")
        .assert_vertex_exists(3, "Person");
}

#[test]
fn test_delete_vertex_with_edge() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2024-01-01')")
        .assert_success()
        .assert_edge_exists(1, 2, "KNOWS")
        .exec_dml("DELETE VERTEX 1 WITH EDGE")
        .assert_success()
        .assert_vertex_not_exists(1, "Person")
        .assert_edge_not_exists(1, 2, "KNOWS");
}

// ==================== DELETE EDGE Parser Tests ====================

#[test]
fn test_delete_parser_edge() {
    let query = "DELETE EDGE 1 -> 2 OF KNOWS";
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
fn test_delete_parser_multiple_edges() {
    let query = "DELETE EDGE 1 -> 2, 1 -> 3 OF KNOWS";
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

// ==================== DELETE EDGE Execution Tests ====================

#[test]
fn test_delete_execution_edge() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2024-01-01')")
        .assert_success()
        .assert_edge_exists(1, 2, "KNOWS")
        .exec_dml("DELETE EDGE 1 -> 2 OF KNOWS")
        .assert_success()
        .assert_edge_not_exists(1, 2, "KNOWS");
}

// ==================== Error Handling Tests ====================

#[test]
fn test_delete_nonexistent_vertex() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .exec_dml("DELETE VERTEX 999")
        .assert_error();
}

#[test]
fn test_delete_nonexistent_edge() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
        .exec_dml("DELETE EDGE 1 -> 2 OF KNOWS")
        .assert_error();
}

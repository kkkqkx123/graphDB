//! DML Update Tests
//!
//! Test coverage:
//! - UPDATE VERTEX - Update vertex properties
//! - UPDATE EDGE - Update edge properties
//! - UPDATE with WHEN condition

use super::common;

use common::test_scenario::TestScenario;
use graphdb::query::parser::Parser;

// ==================== UPDATE VERTEX Parser Tests ====================

#[test]
fn test_update_parser_vertex() {
    let query = "UPDATE 1 SET name = 'Bob', age = 35";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "UPDATE VERTEX parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("UPDATE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "UPDATE");
}

#[test]
fn test_update_parser_vertex_with_when() {
    let query = "UPDATE 1 SET age = 35 WHEN age < 30";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "UPDATE with WHEN parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("UPDATE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "UPDATE");
}

#[test]
fn test_update_parser_vertex_yield() {
    let query = "UPDATE 1 SET name = 'Bob' YIELD name, age";
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

// ==================== UPDATE VERTEX Execution Tests ====================

#[test]
fn test_update_execution_vertex() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)")
        .assert_success()
        .exec_dml("UPDATE 1 SET name = 'Bob', age = 35")
        .assert_success();
}

#[test]
fn test_update_execution_vertex_with_when() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)")
        .assert_success()
        .exec_dml("UPDATE 1 SET age = 35 WHEN age < 40")
        .assert_success();
}

// ==================== UPDATE EDGE Tests ====================

#[test]
fn test_update_parser_edge() {
    let query = "UPDATE EDGE 1 -> 2 OF KNOWS SET since = '2024-02-01'";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "UPDATE EDGE parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("UPDATE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "UPDATE");
}

#[test]
fn test_update_execution_edge() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2024-01-01')")
        .assert_success()
        .exec_dml("UPDATE EDGE 1 -> 2 OF KNOWS SET since = '2024-02-01'")
        .assert_success();
}

// ==================== Error Handling Tests ====================

#[test]
fn test_update_nonexistent_vertex() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .exec_dml("UPDATE 999 SET name = 'Nobody'")
        .assert_error();
}

#[test]
fn test_update_nonexistent_property() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice')")
        .assert_success()
        .exec_dml("UPDATE 1 SET nonexistent = 'value'")
        .assert_error();
}

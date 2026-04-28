//! DQL LOOKUP Query Tests
//!
//! Test coverage:
//! - LOOKUP ON - Index-based lookup
//! - LOOKUP with WHERE clause
//! - LOOKUP with YIELD

use super::common;

use common::test_scenario::TestScenario;
use graphdb::query::parser::Parser;

// ==================== LOOKUP Parser Tests ====================

#[test]
fn test_lookup_parser_basic() {
    let query = "LOOKUP ON Person";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "LOOKUP basic parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("LOOKUP statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "LOOKUP");
}

#[test]
fn test_lookup_parser_with_where() {
    let query = "LOOKUP ON Person WHERE Person.name == 'Alice'";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "LOOKUP with WHERE parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("LOOKUP statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "LOOKUP");
}

#[test]
fn test_lookup_parser_with_yield() {
    let query = "LOOKUP ON Person YIELD Person.name, Person.age";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "LOOKUP with YIELD parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("LOOKUP statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "LOOKUP");
}

#[test]
fn test_lookup_parser_with_where_and_yield() {
    let query = "LOOKUP ON Person WHERE Person.age > 25 YIELD Person.name AS name";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "LOOKUP with WHERE and YIELD parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("LOOKUP statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "LOOKUP");
}

#[test]
fn test_lookup_parser_edge() {
    let query = "LOOKUP ON KNOWS";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "LOOKUP ON edge parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("LOOKUP statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "LOOKUP");
}

// ==================== LOOKUP Execution Tests ====================

#[test]
fn test_lookup_execution_basic() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30), 2:('Bob', 25)")
        .assert_success()
        .query("LOOKUP ON Person")
        .assert_success();
}

#[test]
fn test_lookup_execution_with_where() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30), 2:('Bob', 20), 3:('Charlie', 35)")
        .assert_success()
        .query("LOOKUP ON Person WHERE Person.age > 25")
        .assert_success();
}

#[test]
fn test_lookup_execution_edge() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2024-01-01')")
        .assert_success()
        .query("LOOKUP ON KNOWS")
        .assert_success();
}

// ==================== LOOKUP Error Handling Tests ====================

#[test]
fn test_lookup_nonexistent_tag() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .query("LOOKUP ON NonExistentTag")
        .assert_error();
}

#[test]
fn test_lookup_empty_result() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .query("LOOKUP ON Person WHERE Person.name == 'NonExistent'")
        .assert_success()
        .assert_result_count(0);
}

//! DQL MATCH Query Tests
//!
//! Test coverage:
//! - MATCH - Pattern matching
//! - MATCH with WHERE clause
//! - MATCH with RETURN
//! - MATCH with multiple patterns

use super::common;

use common::test_scenario::TestScenario;
use graphdb::query::parser::Parser;

// ==================== MATCH Parser Tests ====================

#[test]
fn test_match_parser_basic() {
    let query = "MATCH (v:Person) RETURN v";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "MATCH basic parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("MATCH statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "MATCH");
}

#[test]
fn test_match_parser_with_edge() {
    let query = "MATCH (a:Person)-[r:KNOWS]->(b:Person) RETURN a, b, r";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "MATCH with edge parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("MATCH statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "MATCH");
}

#[test]
fn test_match_parser_with_where() {
    let query = "MATCH (v:Person) WHERE v.age > 25 RETURN v.name, v.age";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "MATCH with WHERE parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("MATCH statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "MATCH");
}

#[test]
fn test_match_parser_multi_hop() {
    let query = "MATCH (a)-[r1:KNOWS]->(b)-[r2:KNOWS]->(c) RETURN a, c";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "MATCH multi-hop parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("MATCH statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "MATCH");
}

#[test]
fn test_match_parser_bidirectional() {
    let query = "MATCH (a)-[r:KNOWS]-(b) RETURN a, b";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "MATCH bidirectional parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("MATCH statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "MATCH");
}

#[test]
fn test_match_parser_with_properties() {
    let query = "MATCH (v:Person {name: 'Alice'}) RETURN v";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "MATCH with properties parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("MATCH statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "MATCH");
}

// ==================== MATCH Execution Tests ====================

#[test]
fn test_match_execution_basic() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob'), 3:('Charlie')")
        .assert_success()
        .query("MATCH (v:Person) RETURN v.name AS name")
        .assert_success()
        .assert_result_count(3);
}

#[test]
fn test_match_execution_with_edge() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2024-01-01')")
        .assert_success()
        .query("MATCH (a:Person)-[r:KNOWS]->(b:Person) RETURN a.name, b.name")
        .assert_success()
        .assert_result_count(1);
}

#[test]
fn test_match_execution_with_where() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30), 2:('Bob', 20), 3:('Charlie', 35)")
        .assert_success()
        .query("MATCH (v:Person) WHERE v.age > 25 RETURN v.name, v.age")
        .assert_success()
        .assert_result_count(2);
}

#[test]
fn test_match_execution_with_properties() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30), 2:('Bob', 25)")
        .assert_success()
        .query("MATCH (v:Person {name: 'Alice'}) RETURN v.name, v.age")
        .assert_success()
        .assert_result_count(1);
}

// ==================== MATCH Error Handling Tests ====================

#[test]
fn test_match_nonexistent_tag() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .query("MATCH (v:NonExistentTag) RETURN v")
        .assert_error();
}

#[test]
fn test_match_invalid_pattern() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .query("MATCH (a)-[r]-> RETURN a")
        .assert_error();
}

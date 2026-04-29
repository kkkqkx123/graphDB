//! DQL SUBGRAPH Tests
//!
//! Test coverage:
//! - GET SUBGRAPH

use super::common;

use common::test_scenario::TestScenario;
use graphdb::query::parser::Parser;

// ==================== GET SUBGRAPH Parser Tests ====================

#[test]
fn test_get_subgraph_parser() {
    let query = "GET SUBGRAPH FROM 1";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "GET SUBGRAPH parsing should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_get_subgraph_with_steps_parser() {
    let query = "GET SUBGRAPH 2 STEPS FROM 1";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "GET SUBGRAPH with steps parsing should succeed: {:?}",
        result.err()
    );
}

// ==================== GET SUBGRAPH Execution Tests ====================

#[test]
fn test_get_subgraph_execution() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob'), 3:('Charlie'), 4:('David')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01'), 1 -> 3:('2021-01-01'), 2 -> 4:('2022-01-01')")
        .assert_success()
        .query("GET SUBGRAPH FROM 1")
        .assert_success();
}

//! DQL FIND PATH Tests
//!
//! Test coverage:
//! - FIND SHORTEST PATH
//! - FIND ALL PATH
//! - FIND PATH with UPTO steps limit

use super::common;

use common::test_scenario::TestScenario;
use graphdb::query::parser::Parser;

// ==================== FIND PATH Parser Tests ====================

#[test]
fn test_find_shortest_path_parser() {
    let query = "FIND SHORTEST PATH FROM 1 TO 4 OVER KNOWS";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "FIND SHORTEST PATH parsing should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_find_all_path_parser() {
    let query = "FIND ALL PATH FROM 1 TO 4 OVER KNOWS";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "FIND ALL PATH parsing should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_find_path_with_steps_parser() {
    let query = "FIND SHORTEST PATH FROM 1 TO 4 OVER KNOWS UPTO 2 STEPS";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "FIND PATH with steps parsing should succeed: {:?}",
        result.err()
    );
}

// ==================== FIND SHORTEST PATH Execution Tests ====================

#[test]
fn test_find_shortest_path_execution() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob'), 3:('Charlie'), 4:('David')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01'), 2 -> 3:('2021-01-01'), 3 -> 4:('2022-01-01')")
        .assert_success()
        .query("FIND SHORTEST PATH FROM 1 TO 4 OVER KNOWS")
        .assert_success()
        .assert_result_count(1);
}

// ==================== FIND ALL PATH Execution Tests ====================

#[test]
fn test_find_all_path_execution() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob'), 3:('Charlie'), 4:('David')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01'), 2 -> 4:('2021-01-01'), 1 -> 3:('2020-01-01'), 3 -> 4:('2021-01-01')")
        .assert_success()
        .query("FIND ALL PATH FROM 1 TO 4 OVER KNOWS")
        .assert_success()
        .assert_result_count(2);
}

// ==================== FIND PATH with Steps Limit Tests ====================

#[test]
fn test_find_path_with_steps_limit() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob'), 3:('Charlie'), 4:('David')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01'), 2 -> 3:('2021-01-01'), 3 -> 4:('2022-01-01')")
        .assert_success()
        .query("FIND SHORTEST PATH FROM 1 TO 4 OVER KNOWS UPTO 2 STEPS")
        .assert_success()
        .assert_result_empty();
}

//! DQL Aggregation Tests
//!
//! Test coverage:
//! - GROUP BY - Grouping results
//! - ORDER BY - Sorting results
//! - LIMIT - Limiting results
//! - SKIP - Skipping results
//! - Aggregate functions: COUNT, SUM, AVG, MIN, MAX

use super::common;

use common::test_scenario::TestScenario;
use graphdb::query::parser::Parser;

// ==================== GROUP BY Parser Tests ====================

#[test]
fn test_group_by_parser_basic() {
    let query = "MATCH (v:Person) RETURN v.age AS age, COUNT(*) AS count GROUP BY v.age";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "GROUP BY parsing should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_group_by_parser_multiple() {
    let query = "MATCH (v:Person) RETURN v.age, v.name, COUNT(*) GROUP BY v.age, v.name";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "GROUP BY multiple fields parsing should succeed: {:?}",
        result.err()
    );
}

// ==================== ORDER BY Parser Tests ====================

#[test]
fn test_order_by_parser_asc() {
    let query = "MATCH (v:Person) RETURN v.name ORDER BY v.age ASC";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "ORDER BY ASC parsing should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_order_by_parser_desc() {
    let query = "MATCH (v:Person) RETURN v.name ORDER BY v.age DESC";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "ORDER BY DESC parsing should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_order_by_parser_multiple() {
    let query = "MATCH (v:Person) RETURN v.name ORDER BY v.age ASC, v.name DESC";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "ORDER BY multiple fields parsing should succeed: {:?}",
        result.err()
    );
}

// ==================== LIMIT/SKIP Parser Tests ====================

#[test]
fn test_limit_parser_basic() {
    let query = "MATCH (v:Person) RETURN v.name LIMIT 10";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "LIMIT parsing should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_skip_parser_basic() {
    let query = "MATCH (v:Person) RETURN v.name SKIP 5 LIMIT 10";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "SKIP parsing should succeed: {:?}",
        result.err()
    );
}

// ==================== Aggregate Function Parser Tests ====================

#[test]
fn test_count_parser() {
    let query = "MATCH (v:Person) RETURN COUNT(v) AS total";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "COUNT parsing should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_sum_parser() {
    let query = "MATCH (v:Person) RETURN SUM(v.age) AS total_age";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "SUM parsing should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_avg_parser() {
    let query = "MATCH (v:Person) RETURN AVG(v.age) AS avg_age";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "AVG parsing should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_min_max_parser() {
    let query = "MATCH (v:Person) RETURN MIN(v.age) AS min_age, MAX(v.age) AS max_age";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "MIN/MAX parsing should succeed: {:?}",
        result.err()
    );
}

// ==================== Aggregation Execution Tests ====================

#[test]
fn test_count_execution() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('A'), 2:('B'), 3:('C')")
        .assert_success()
        .query("MATCH (v:Person) RETURN COUNT(v) AS total")
        .assert_success();
}

#[test]
fn test_order_by_execution() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30), 2:('Bob', 20), 3:('Charlie', 25)")
        .assert_success()
        .query("MATCH (v:Person) RETURN v.name, v.age ORDER BY v.age ASC")
        .assert_success();
}

#[test]
fn test_limit_execution() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('A'), 2:('B'), 3:('C'), 4:('D'), 5:('E')")
        .assert_success()
        .query("MATCH (v:Person) RETURN v.name LIMIT 3")
        .assert_success()
        .assert_result_count(3);
}

#[test]
fn test_skip_limit_execution() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('A'), 2:('B'), 3:('C'), 4:('D'), 5:('E')")
        .assert_success()
        .query("MATCH (v:Person) RETURN v.name SKIP 2 LIMIT 2")
        .assert_success()
        .assert_result_count(2);
}

//! DML Update Tests
//!
//! Test coverage:
//! - UPDATE VERTEX - Update vertex properties
//! - UPDATE EDGE - Update edge properties
//! - UPDATE with WHEN condition

use super::common;

use common::test_scenario::TestScenario;
use graphdb::core::Value;
use graphdb::query::parser::Parser;
use std::collections::HashMap;

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

// ==================== UPDATE Vertex with Verification Tests ====================

#[test]
fn test_update_vertex_and_verify() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT, city STRING)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name, age, city) VALUES 1:('Alice', 30, 'NYC')")
        .assert_success()
        .assert_vertex_props(1, "Person", {
            let mut map = std::collections::HashMap::new();
            map.insert("age", graphdb::core::Value::Int(30));
            map.insert("city", graphdb::core::Value::String("NYC".into()));
            map
        })
        .exec_dml("UPDATE 1 SET age = 31")
        .assert_success()
        .assert_vertex_props(1, "Person", {
            let mut map = std::collections::HashMap::new();
            map.insert("age", graphdb::core::Value::Int(31));
            map
        })
        .exec_dml("UPDATE 1 SET age = 32, city = 'LA'")
        .assert_success()
        .assert_vertex_props(1, "Person", {
            let mut map = std::collections::HashMap::new();
            map.insert("age", graphdb::core::Value::Int(32));
            map.insert("city", graphdb::core::Value::String("LA".into()));
            map
        });
}

#[test]
fn test_update_vertex_with_condition() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT, state STRING)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name, age, state) VALUES 1:('Alice', 30, 'active'), 2:('Bob', 25, 'inactive'), 3:('Charlie', 35, 'active')")
        .assert_success()
        .exec_dml("UPDATE 1 SET state = 'premium' WHEN state == 'active'")
        .assert_success()
        .assert_vertex_props(1, "Person", {
            let mut map = std::collections::HashMap::new();
            map.insert("state", graphdb::core::Value::String("premium".into()));
            map
        })
        .query("FETCH PROP ON Person 2")
        .assert_vertex_or_edge_has_property("state", graphdb::core::Value::String("inactive".into()));
}

#[test]
fn test_update_edge_and_verify() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since DATE, strength DOUBLE)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .assert_success()
        .exec_dml("INSERT EDGE KNOWS(since, strength) VALUES 1 -> 2:('2020-01-01', 0.5)")
        .assert_success()
        .exec_dml("UPDATE 1 -> 2 OF KNOWS SET strength = 0.9")
        .assert_success()
        .query("FETCH PROP ON KNOWS 1 -> 2")
        .assert_vertex_or_edge_has_property("strength", graphdb::core::Value::Double(0.9));
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

// ==================== UPDATE Arithmetic Expression Tests ====================

#[test]
fn test_update_arithmetic_add() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Counter(val INT)")
        .exec_dml("INSERT VERTEX Counter(val) VALUES 1:(10)")
        .assert_success()
        .exec_dml("UPDATE 1 SET val = val + 5")
        .assert_success()
        .assert_vertex_props(
            1,
            "Counter",
            HashMap::from([("val", Value::Int(15))]),
        );
}

#[test]
fn test_update_arithmetic_subtract() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Counter(val INT)")
        .exec_dml("INSERT VERTEX Counter(val) VALUES 1:(10)")
        .assert_success()
        .exec_dml("UPDATE 1 SET val = val - 3")
        .assert_success()
        .assert_vertex_props(
            1,
            "Counter",
            HashMap::from([("val", Value::Int(7))]),
        );
}

#[test]
fn test_update_arithmetic_multiply() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Counter(val INT)")
        .exec_dml("INSERT VERTEX Counter(val) VALUES 1:(10)")
        .assert_success()
        .exec_dml("UPDATE 1 SET val = val * 2")
        .assert_success()
        .assert_vertex_props(
            1,
            "Counter",
            HashMap::from([("val", Value::Int(20))]),
        );
}

#[test]
fn test_update_arithmetic_divide() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Counter(val DOUBLE)")
        .exec_dml("INSERT VERTEX Counter(val) VALUES 1:(10.0)")
        .assert_success()
        .exec_dml("UPDATE 1 SET val = val / 3")
        .assert_success();
}

#[test]
fn test_update_double_decrement() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Counter(val INT)")
        .exec_dml("INSERT VERTEX Counter(val) VALUES 1:(5)")
        .assert_success()
        .exec_dml("UPDATE 1 SET val = val - 1")
        .assert_success()
        .assert_vertex_props(
            1,
            "Counter",
            HashMap::from([("val", Value::Int(4))]),
        )
        .exec_dml("UPDATE 1 SET val = val - 1")
        .assert_success()
        .assert_vertex_props(
            1,
            "Counter",
            HashMap::from([("val", Value::Int(3))]),
        );
}

#[test]
fn test_update_arithmetic_multiple_fields() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Counter(a INT, b INT, total INT)")
        .exec_dml("INSERT VERTEX Counter(a, b, total) VALUES 1:(3, 7, 0)")
        .assert_success()
        .exec_dml("UPDATE 1 SET total = a + b")
        .assert_success()
        .assert_vertex_props(
            1,
            "Counter",
            HashMap::from([
                ("a", Value::Int(3)),
                ("b", Value::Int(7)),
                ("total", Value::Int(10)),
            ]),
        );
}

#[test]
fn test_update_arithmetic_chain() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Calc(val INT)")
        .exec_dml("INSERT VERTEX Calc(val) VALUES 1:(2)")
        .assert_success()
        .exec_dml("UPDATE 1 SET val = val * 2 + 1")
        .assert_success()
        .assert_vertex_props(
            1,
            "Calc",
            HashMap::from([("val", Value::Int(5))]),
        );
}

// ==================== UPDATE to NULL Tests ====================

#[test]
fn test_update_property_to_null() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)")
        .assert_success()
        .exec_dml("UPDATE 1 SET age = NULL")
        .assert_success();
}

// ==================== UPDATE WHEN False Tests ====================

#[test]
fn test_update_when_false_condition() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)")
        .assert_success()
        .exec_dml("UPDATE 1 SET age = 99 WHEN age > 100")
        .assert_success()
        .assert_vertex_props(
            1,
            "Person",
            HashMap::from([("age", Value::Int(30))]),
        );
}

// ==================== UPDATE EDGE with Rank Tests ====================

#[test]
fn test_update_edge_ranked() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since DATE, strength DOUBLE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .exec_dml("INSERT EDGE KNOWS(since, strength) VALUES 1 -> 2 @0:('2020-01-01', 0.5)")
        .assert_success()
        .exec_dml("UPDATE 1 -> 2 OF KNOWS SET strength = 0.9")
        .assert_success()
        .query("FETCH PROP ON KNOWS 1 -> 2")
        .assert_vertex_or_edge_has_property("strength", Value::Double(0.9));
}

// ==================== UPDATE YIELD Verification Tests ====================

#[test]
fn test_update_yield_parser() {
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

#[test]
fn test_update_yield_execution() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)")
        .assert_success()
        .exec_dml("UPDATE 1 SET name = 'Bob', age = 35 YIELD name, age")
        .assert_success();
}

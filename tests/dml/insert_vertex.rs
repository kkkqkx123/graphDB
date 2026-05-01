//! DML Insert Vertex Tests
//!
//! Test coverage:
//! - INSERT VERTEX - Insert vertex data
//! - INSERT VERTEX IF NOT EXISTS

use super::common;

use common::test_scenario::TestScenario;
use graphdb::query::parser::Parser;
use std::collections::HashMap;
use graphdb::core::Value;

// ==================== INSERT VERTEX Parser Tests ====================

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
fn test_insert_parser_invalid_syntax() {
    let query = "INSERT VERTEX Person(name, age) VALUES 1:'Alice', 30";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(result.is_err(), "Invalid syntax should trigger an error.");
}

// ==================== INSERT VERTEX Execution Tests ====================

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

// ==================== INSERT IF NOT EXISTS Tests ====================

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
        .exec_dml("INSERT VERTEX IF NOT EXISTS Person(name, age) VALUES 1:('Bob', 25)")
        .assert_success()
        .assert_vertex_props(
            1,
            "Person",
            HashMap::from([
                ("name", Value::String("Alice".into())),
                ("age", Value::Int(30)),
            ]),
        );
}

// ==================== Multiple Tags Tests ====================

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

// ==================== Error Handling Tests ====================

#[test]
fn test_insert_duplicate_vertex() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice')")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Bob')")
        .assert_error();
}

#[test]
fn test_insert_vertex_with_all_types() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl(r#"
            CREATE TAG TestTypes(
                str_field STRING,
                int_field INT,
                double_field DOUBLE,
                bool_field BOOL
            )
        "#)
        .assert_success()
        .exec_dml(r#"
            INSERT VERTEX TestTypes(str_field, int_field, double_field, bool_field) 
            VALUES 1:('test', 42, 2.71828, true)
        "#)
        .assert_success()
        .assert_vertex_props(1, "TestTypes", {
            let mut map = HashMap::new();
            map.insert("str_field", Value::String("test".into()));
            map.insert("int_field", Value::Int(42));
            map.insert("double_field", Value::Float(std::f32::consts::E));
            map.insert("bool_field", Value::Bool(true));
            map
        });
}

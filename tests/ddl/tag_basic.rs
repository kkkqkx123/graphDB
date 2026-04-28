//! DDL Tag Basic Tests
//!
//! Test coverage:
//! - CREATE TAG - Create vertex tag
//! - DROP TAG - Delete vertex tag
//! - DESC TAG - Describe tag schema

use super::common;

use common::test_scenario::TestScenario;
use graphdb::query::parser::Parser;
use std::collections::HashMap;
use graphdb::core::Value;

// ==================== CREATE TAG Parser Tests ====================

#[test]
fn test_create_tag_parser_basic() {
    let query = "CREATE TAG Person(name: STRING, age: INT)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "CREATE TAG basic parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("CREATE TAG statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "CREATE");
}

#[test]
fn test_create_tag_parser_with_if_not_exists() {
    let query = "CREATE TAG IF NOT EXISTS Person(name: STRING, age: INT)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "CREATE TAG with IF NOT EXISTS parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("CREATE TAG statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "CREATE");
}

#[test]
fn test_create_tag_parser_single_property() {
    let query = "CREATE TAG Person(name: STRING)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "CREATE TAG single property parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("CREATE TAG statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "CREATE");
}

#[test]
fn test_create_tag_parser_multiple_properties() {
    let query = "CREATE TAG Person(name: STRING, age: INT, created_at: TIMESTAMP)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "CREATE TAG multiple properties parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("CREATE TAG statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "CREATE");
}

#[test]
fn test_create_tag_parser_various_types() {
    let query = "CREATE TAG Test(name: STRING, age: INT, score: DOUBLE, active: BOOL, birth: DATE)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "CREATE TAG various types parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("CREATE TAG statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "CREATE");
}

// ==================== CREATE TAG Execution Tests ====================

#[test]
fn test_create_tag_execution_basic() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        .assert_success()
        .assert_tag_exists("Person");
}

#[test]
fn test_create_tag_execution_with_if_not_exists() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name: STRING, age: INT)")
        .assert_success()
        .assert_tag_exists("Person")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name: STRING, age: INT)")
        .assert_success()
        .assert_tag_exists("Person");
}

#[test]
fn test_create_tag_execution_with_data() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        .assert_success()
        .assert_tag_exists("Person")
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

// ==================== DROP TAG Parser Tests ====================

#[test]
fn test_drop_tag_parser_basic() {
    let query = "DROP TAG Person";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DROP TAG basic parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DROP TAG statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DROP");
}

#[test]
fn test_drop_tag_parser_with_if_exists() {
    let query = "DROP TAG IF EXISTS Person";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DROP TAG with IF EXISTS parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DROP TAG statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DROP");
}

#[test]
fn test_drop_tag_parser_multiple() {
    let query = "DROP TAG Person, Company, Location";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DROP TAG multiple tags parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DROP TAG statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DROP");
}

#[test]
fn test_drop_tag_parser_multiple_with_if_exists() {
    let query = "DROP TAG IF EXISTS Person, Company";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DROP TAG multiple tags with IF EXISTS parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DROP TAG statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DROP");
}

// ==================== DROP TAG Execution Tests ====================

#[test]
fn test_drop_tag_execution_basic() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .assert_success()
        .assert_tag_exists("Person")
        .exec_ddl("DROP TAG Person")
        .assert_success()
        .assert_tag_not_exists("Person");
}

#[test]
fn test_drop_tag_execution_with_if_exists() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("DROP TAG IF EXISTS NonExistentTag")
        .assert_success()
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .assert_success()
        .exec_ddl("DROP TAG IF EXISTS Person")
        .assert_success()
        .assert_tag_not_exists("Person");
}

// ==================== DESC TAG Tests ====================

#[test]
fn test_desc_parser_tag() {
    let query = "DESCRIBE TAG Person";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DESCRIBE TAG parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DESCRIBE TAG statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DESC");
}

#[test]
fn test_desc_parser_short_tag() {
    let query = "DESC TAG Person";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DESC TAG parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DESC TAG statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DESC");
}

#[test]
fn test_desc_execution_tag() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        .assert_success()
        .query("DESCRIBE TAG Person")
        .assert_success()
        .assert_result_count(2);
}

// ==================== Tag Lifecycle Tests ====================

#[test]
fn test_ddl_tag_lifecycle() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG TestTag(name: STRING, age: INT)")
        .assert_success()
        .assert_tag_exists("TestTag")
        .query("DESCRIBE TAG TestTag")
        .assert_success()
        .exec_ddl("ALTER TAG TestTag ADD (email: STRING)")
        .assert_success()
        .exec_dml(
            "INSERT VERTEX TestTag(name, age, email) VALUES 1:('Alice', 30, 'alice@test.com')",
        )
        .assert_success()
        .assert_vertex_exists(1, "TestTag")
        .exec_ddl("ALTER TAG TestTag DROP (email)")
        .assert_success()
        .exec_dml("DELETE VERTEX 1")
        .assert_success()
        .exec_ddl("DROP TAG TestTag")
        .assert_success()
        .assert_tag_not_exists("TestTag");
}

#[test]
fn test_ddl_if_not_exists_if_exists() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name: STRING)")
        .assert_success()
        .assert_tag_exists("Person")
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name: STRING)")
        .assert_success()
        .exec_ddl("DROP TAG IF EXISTS Person")
        .assert_success()
        .assert_tag_not_exists("Person")
        .exec_ddl("DROP TAG IF EXISTS Person")
        .assert_success();
}

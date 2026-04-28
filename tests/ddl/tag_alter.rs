//! DDL Tag Alter Tests
//!
//! Test coverage:
//! - ALTER TAG ADD - Add properties to tag
//! - ALTER TAG DROP - Drop properties from tag
//! - ALTER TAG CHANGE - Rename properties

use super::common;

use common::test_scenario::TestScenario;
use graphdb::query::parser::Parser;

// ==================== ALTER TAG Parser Tests ====================

#[test]
fn test_alter_tag_parser_add() {
    let query = "ALTER TAG Person ADD (email: STRING, phone: STRING)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "ALTER TAG ADD parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("ALTER TAG statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "ALTER");
}

#[test]
fn test_alter_tag_parser_drop() {
    let query = "ALTER TAG Person DROP (temp_field, old_field)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "ALTER TAG DROP parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("ALTER TAG statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "ALTER");
}

#[test]
fn test_alter_tag_parser_change() {
    let query = "ALTER TAG Person CHANGE (old_name new_name: STRING)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "ALTER TAG CHANGE parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("ALTER TAG statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "ALTER");
}

#[test]
fn test_alter_tag_parser_add_single() {
    let query = "ALTER TAG Person ADD (email: STRING)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "ALTER TAG ADD single property parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("ALTER TAG statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "ALTER");
}

#[test]
fn test_alter_tag_parser_drop_single() {
    let query = "ALTER TAG Person DROP (temp_field)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "ALTER TAG DROP single property parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("ALTER TAG statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "ALTER");
}

// ==================== ALTER TAG Execution Tests ====================

#[test]
fn test_alter_tag_execution_add() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .assert_success()
        .exec_ddl("ALTER TAG Person ADD (email: STRING)")
        .assert_success();
}

#[test]
fn test_alter_tag_execution_drop() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, temp_field: STRING)")
        .assert_success()
        .exec_ddl("ALTER TAG Person DROP (temp_field)")
        .assert_success();
}

#[test]
fn test_alter_tag_execution_add_multiple() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .assert_success()
        .exec_ddl("ALTER TAG Person ADD (email: STRING, phone: STRING, address: STRING)")
        .assert_success()
        .query("DESCRIBE TAG Person")
        .assert_success()
        .assert_result_count(4);
}

#[test]
fn test_alter_tag_execution_drop_multiple() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, temp1: STRING, temp2: STRING, temp3: STRING)")
        .assert_success()
        .exec_ddl("ALTER TAG Person DROP (temp1, temp2)")
        .assert_success()
        .query("DESCRIBE TAG Person")
        .assert_success()
        .assert_result_count(2);
}

#[test]
fn test_alter_tag_nonexistent() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("ALTER TAG NonExistentTag ADD (field: STRING)")
        .assert_error();
}

#[test]
fn test_alter_tag_drop_nonexistent_field() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .assert_success()
        .exec_ddl("ALTER TAG Person DROP (nonexistent_field)")
        .assert_error();
}

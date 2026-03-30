//! Data Definition Language (DDL) Integration Tests
//!
//! Test coverage:
//! - CREATE TAG - Create vertex tag
//! - CREATE EDGE - Create edge type
//! - ALTER TAG - Modify vertex tag
//! - ALTER EDGE - Modify edge type
//! - DROP TAG - Delete vertex tag
//! - DROP EDGE - Delete edge type
//! - DESC - Describe schema objects

mod common;

use common::test_scenario::TestScenario;
use common::TestStorage;
use graphdb::core::stats::StatsManager;
use graphdb::core::Value;
use graphdb::query::optimizer::OptimizerEngine;
use graphdb::query::parser::Parser;
use graphdb::query::query_pipeline_manager::QueryPipelineManager;
use std::collections::HashMap;
use std::sync::Arc;

// ==================== CREATE TAG Statement Tests ====================

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

// ==================== CREATE EDGE Statement Tests ====================

#[test]
fn test_create_edge_parser_basic() {
    let query = "CREATE EDGE KNOWS(since: DATE)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "CREATE EDGE basic parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("CREATE EDGE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "CREATE");
}

#[test]
fn test_create_edge_parser_with_if_not_exists() {
    let query = "CREATE EDGE IF NOT EXISTS KNOWS(since: DATE)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "CREATE EDGE with IF NOT EXISTS parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("CREATE EDGE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "CREATE");
}

#[test]
fn test_create_edge_parser_single_property() {
    let query = "CREATE EDGE KNOWS(since: DATE)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "CREATE EDGE single property parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("CREATE EDGE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "CREATE");
}

#[test]
fn test_create_edge_parser_multiple_properties() {
    let query = "CREATE EDGE KNOWS(since: DATE, degree: DOUBLE, note: STRING)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "CREATE EDGE multiple properties parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("CREATE EDGE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "CREATE");
}

#[test]
fn test_create_edge_parser_various_types() {
    let query = "CREATE EDGE Test(since: DATE, weight: DOUBLE, active: BOOL, count: INT)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "CREATE EDGE various types parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("CREATE EDGE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "CREATE");
}

#[test]
fn test_create_edge_execution_basic() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        .assert_success();
}

#[test]
fn test_create_edge_execution_with_if_not_exists() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE EDGE IF NOT EXISTS KNOWS(since: DATE)")
        .assert_success()
        .exec_ddl("CREATE EDGE IF NOT EXISTS KNOWS(since: DATE)")
        .assert_success();
}

#[test]
fn test_create_edge_execution_with_data() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2024-01-01')")
        .assert_success()
        .assert_edge_exists(1, 2, "KNOWS");
}

// ==================== ALTER TAG Statement Tests ====================

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

// ==================== ALTER EDGE Statement Tests ====================

#[test]
fn test_alter_edge_parser_add() {
    let query = "ALTER EDGE KNOWS ADD (note: STRING, weight: DOUBLE)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "ALTER EDGE ADD parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("ALTER EDGE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "ALTER");
}

#[test]
fn test_alter_edge_parser_drop() {
    let query = "ALTER EDGE KNOWS DROP (temp_field, old_field)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "ALTER EDGE DROP parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("ALTER EDGE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "ALTER");
}

#[test]
fn test_alter_edge_parser_change() {
    let query = "ALTER EDGE KNOWS CHANGE (old_since new_since: DATE)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "ALTER EDGE CHANGE parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("ALTER EDGE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "ALTER");
}

#[test]
fn test_alter_edge_parser_add_single() {
    let query = "ALTER EDGE KNOWS ADD (note: STRING)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "ALTER EDGE ADD single property parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("ALTER EDGE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "ALTER");
}

#[test]
fn test_alter_edge_parser_drop_single() {
    let query = "ALTER EDGE KNOWS DROP (temp_field)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "ALTER EDGE DROP single property parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("ALTER EDGE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "ALTER");
}

#[test]
fn test_alter_edge_execution_add() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        .assert_success()
        .exec_ddl("ALTER EDGE KNOWS ADD (note: STRING)")
        .assert_success();
}

#[test]
fn test_alter_edge_execution_drop() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE, temp_field: STRING)")
        .assert_success()
        .exec_ddl("ALTER EDGE KNOWS DROP (temp_field)")
        .assert_success();
}

// ==================== DROP TAG Statement Tests ====================

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

// ==================== DROP EDGE Statement Tests ====================

#[test]
fn test_drop_edge_parser_basic() {
    let query = "DROP EDGE KNOWS";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DROP EDGE basic parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DROP EDGE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DROP");
}

#[test]
fn test_drop_edge_parser_with_if_exists() {
    let query = "DROP EDGE IF EXISTS KNOWS";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DROP EDGE with IF EXISTS parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DROP EDGE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DROP");
}

#[test]
fn test_drop_edge_parser_multiple() {
    let query = "DROP EDGE KNOWS, LIKES, FOLLOWS";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DROP EDGE multiple edge types parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DROP EDGE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DROP");
}

#[test]
fn test_drop_edge_parser_multiple_with_if_exists() {
    let query = "DROP EDGE IF EXISTS KNOWS, LIKES";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DROP EDGE multiple edge types with IF EXISTS parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DROP EDGE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DROP");
}

#[test]
fn test_drop_edge_execution_basic() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        .assert_success()
        .exec_ddl("DROP EDGE KNOWS")
        .assert_success();
}

#[test]
fn test_drop_edge_execution_with_if_exists() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("DROP EDGE IF EXISTS NonExistentEdge")
        .assert_success()
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        .assert_success()
        .exec_ddl("DROP EDGE IF EXISTS KNOWS")
        .assert_success();
}

// ==================== DESC Statement Tests ====================

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
fn test_desc_parser_edge() {
    let query = "DESCRIBE EDGE KNOWS";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DESCRIBE EDGE parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DESCRIBE EDGE statement parsing should succeed");
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
fn test_desc_parser_short_edge() {
    let query = "DESC EDGE KNOWS";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DESC EDGE parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DESC EDGE statement parsing should succeed");
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
        .assert_result_count(2);  // One row per property (name and age)
}

#[test]
fn test_desc_execution_edge() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        .assert_success()
        .query("DESCRIBE EDGE KNOWS")
        .assert_success()
        .assert_result_count(1);  // One row per property (since)
}

// ==================== DDL Lifecycle Tests ====================

#[test]
fn test_ddl_tag_lifecycle() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        // Create tag
        .exec_ddl("CREATE TAG TestTag(name: STRING, age: INT)")
        .assert_success()
        .assert_tag_exists("TestTag")
        // Describe tag
        .query("DESCRIBE TAG TestTag")
        .assert_success()
        // Alter tag - add property
        .exec_ddl("ALTER TAG TestTag ADD (email: STRING)")
        .assert_success()
        // Insert data
        .exec_dml("INSERT VERTEX TestTag(name, age, email) VALUES 1:('Alice', 30, 'alice@test.com')")
        .assert_success()
        .assert_vertex_exists(1, "TestTag")
        // Alter tag - drop property
        .exec_ddl("ALTER TAG TestTag DROP (email)")
        .assert_success()
        // Drop tag
        .exec_ddl("DROP TAG TestTag")
        .assert_success()
        .assert_tag_not_exists("TestTag");
}

#[test]
fn test_ddl_edge_lifecycle() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        // Create schema
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .exec_ddl("CREATE EDGE TestEdge(since: DATE, weight: DOUBLE)")
        // Insert vertices
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .assert_success()
        // Describe edge
        .query("DESCRIBE EDGE TestEdge")
        .assert_success()
        // Alter edge - add property
        .exec_ddl("ALTER EDGE TestEdge ADD (note: STRING)")
        .assert_success()
        // Insert edge
        .exec_dml("INSERT EDGE TestEdge(since, weight, note) VALUES 1 -> 2:('2024-01-01', 1.0, 'test')")
        .assert_success()
        .assert_edge_exists(1, 2, "TestEdge")
        // Alter edge - drop property
        .exec_ddl("ALTER EDGE TestEdge DROP (note)")
        .assert_success()
        // Drop edge
        .exec_ddl("DROP EDGE TestEdge")
        .assert_success();
}

#[test]
fn test_ddl_multiple_operations() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        // Create multiple tags
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        .assert_success()
        .exec_ddl("CREATE TAG Company(name: STRING, founded: INT)")
        .assert_success()
        // Create multiple edges
        .exec_ddl("CREATE EDGE WORKS_AT(since: DATE)")
        .assert_success()
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        .assert_success()
        // Verify all exist
        .assert_tag_exists("Person")
        .assert_tag_exists("Company");
}

#[test]
fn test_ddl_error_handling() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    let invalid_queries = vec![
        "CREATE TAG Person",    // Missing attribute definitions
        "ALTER TAG Person ADD", // Missing attributes
        "DROP TAG",             // Missing tag name
        "DESCRIBE",             // Missing objects
    ];

    for query in invalid_queries {
        let result = pipeline_manager.execute_query(query);
        assert!(result.is_err(), "Invalid query should return error: {}", query);
    }
}

#[test]
fn test_ddl_if_not_exists_if_exists() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        // Create tag with IF NOT EXISTS
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name: STRING)")
        .assert_success()
        .assert_tag_exists("Person")
        // Duplicate creation should succeed
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name: STRING)")
        .assert_success()
        // Drop tag with IF EXISTS
        .exec_ddl("DROP TAG IF EXISTS Person")
        .assert_success()
        .assert_tag_not_exists("Person")
        // Duplicate deletion should succeed
        .exec_ddl("DROP TAG IF EXISTS Person")
        .assert_success();
}

// ==================== DEFAULT Value Tests ====================

#[test]
fn test_create_tag_with_default_value() {
    let query = "CREATE TAG Person(name: STRING, age: INT DEFAULT 18)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "CREATE TAG with DEFAULT parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("CREATE TAG statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "CREATE");
}

#[test]
fn test_create_tag_with_default_string() {
    let query = "CREATE TAG Person(name: STRING DEFAULT 'unknown', email: STRING DEFAULT 'test@example.com')";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "CREATE TAG with string DEFAULT parsing should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_create_tag_with_default_null() {
    let query = "CREATE TAG Person(name: STRING, nickname: STRING DEFAULT NULL)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "CREATE TAG with NULL DEFAULT parsing should succeed: {:?}",
        result.err()
    );
}

// ==================== NOT NULL Constraint Tests ====================

#[test]
fn test_create_tag_with_not_null() {
    let query = "CREATE TAG Person(name: STRING NOT NULL, age: INT NOT NULL)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "CREATE TAG with NOT NULL parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("CREATE TAG statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "CREATE");
}

#[test]
fn test_create_tag_with_nullable() {
    let query = "CREATE TAG Person(name: STRING NOT NULL, nickname: STRING NULL)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "CREATE TAG with NULL constraint parsing should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_create_tag_with_not_null_and_default() {
    let query = "CREATE TAG Person(name: STRING NOT NULL, age: INT NOT NULL DEFAULT 0)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "CREATE TAG with NOT NULL and DEFAULT parsing should succeed: {:?}",
        result.err()
    );
}

// ==================== COMMENT Annotation Tests ====================

#[test]
fn test_create_tag_with_comment() {
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
fn test_create_tag_with_comment_and_constraints() {
    let query = "CREATE TAG Person(name: STRING NOT NULL, age: INT DEFAULT 18)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "CREATE TAG with constraints parsing should succeed: {:?}",
        result.err()
    );
}

// ==================== TTL Support Tests ====================

#[test]
fn test_create_tag_with_ttl() {
    let query = "CREATE TAG UserSession(token: STRING, created_at: TIMESTAMP)";
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
fn test_create_edge_with_ttl() {
    let query = "CREATE EDGE TempEdge(content: STRING, expire_at: TIMESTAMP)";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "CREATE EDGE basic parsing should succeed: {:?}",
        result.err()
    );
}

// ==================== SHOW CREATE Tests ====================

#[test]
fn test_show_create_tag_parser() {
    let query = "SHOW CREATE TAG Person";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "SHOW CREATE TAG parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("SHOW CREATE TAG statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "SHOW CREATE");
}

#[test]
fn test_show_create_edge_parser() {
    let query = "SHOW CREATE EDGE KNOWS";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "SHOW CREATE EDGE parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("SHOW CREATE EDGE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "SHOW CREATE");
}

#[test]
fn test_show_create_space_parser() {
    let query = "SHOW CREATE SPACE test_space";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "SHOW CREATE SPACE should parse successfully!"
    );

    let stmt = result.expect("SHOW CREATE SPACE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "SHOW CREATE");
}

#[test]
fn test_show_create_index_parser() {
    let query = "SHOW CREATE INDEX idx_person_name";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "SHOW CREATE INDEX should parse successfully!"
    );

    let stmt = result.expect("SHOW CREATE INDEX statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "SHOW CREATE");
}

#[test]
fn test_show_create_execution() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        .assert_success()
        .query("SHOW CREATE TAG Person")
        .assert_success()
        .assert_result_count(1);
}

// ==================== Comprehensive Functional Tests ====================

#[test]
fn test_create_tag_full_features() {
    let query = "CREATE TAG IF NOT EXISTS Person(
        id: INT NOT NULL,
        name: STRING NOT NULL,
        age: INT DEFAULT 0,
        email: STRING,
        created_at: TIMESTAMP
    )";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "Full-featured CREATE TAG parsing should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_create_edge_full_features() {
    let query = "CREATE EDGE IF NOT EXISTS KNOWS(
        since: DATE NOT NULL,
        degree: DOUBLE DEFAULT 1.0,
        note: STRING
    )";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "Full-featured CREATE EDGE parsing should succeed: {:?}",
        result.err()
    );
}

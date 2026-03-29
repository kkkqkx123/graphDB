//! Extended DDL Integration Tests
//!
//! This file demonstrates how to use the new test framework to validate
//! actual execution effects of DDL statements.

mod common;

use common::test_scenario::TestScenario;
use graphdb::core::Value;
use std::collections::HashMap;

// ==================== CREATE TAG Extended Tests ====================

#[test]
fn test_create_tag_with_validation() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .assert_success()
        .assert_tag_exists("Person")
        .exec_ddl("DESC TAG Person")
        .assert_result_count(2)
        .assert_result_columns(&["Field", "Type"]);
}

#[test]
fn test_create_tag_if_not_exists() {
    let mut scenario = TestScenario::new();
    scenario
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .assert_tag_exists("Person")
        .exec_ddl("CREATE TAG Person(name STRING)") // Should fail without IF NOT EXISTS
        .assert_error()
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING)") // Should succeed
        .assert_success();
}

#[test]
fn test_create_tag_with_various_types() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl(r#"
            CREATE TAG TestTypes(
                name STRING,
                age INT,
                score DOUBLE,
                active BOOL,
                created_at TIMESTAMP,
                data BINARY
            )
        "#)
        .assert_success()
        .assert_tag_exists("TestTypes")
        .exec_ddl("DESC TAG TestTypes")
        .assert_result_count(6);
}

// ==================== CREATE EDGE Extended Tests ====================

#[test]
fn test_create_edge_with_validation() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE EDGE KNOWS(since DATE, strength DOUBLE)")
        .assert_success()
        .exec_ddl("DESC EDGE KNOWS")
        .assert_result_count(2);
}

#[test]
fn test_create_edge_if_not_exists() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
        .assert_success()
        .exec_ddl("CREATE EDGE IF NOT EXISTS KNOWS(since DATE)")
        .assert_success();
}

// ==================== ALTER TAG Extended Tests ====================

#[test]
fn test_alter_tag_add_field() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_ddl("ALTER TAG Person ADD (age INT, email STRING)")
        .assert_success()
        .exec_ddl("DESC TAG Person")
        .assert_result_count(3); // name, age, email
}

#[test]
fn test_alter_tag_drop_field() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, temp_field INT)")
        .assert_success()
        .exec_ddl("ALTER TAG Person DROP (temp_field)")
        .assert_success()
        .exec_ddl("DESC TAG Person")
        .assert_result_count(1); // only name
}

#[test]
fn test_alter_tag_change_field() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(old_name STRING)")
        .assert_success()
        .exec_ddl("ALTER TAG Person CHANGE (old_name name: STRING)")
        .assert_success()
        .exec_ddl("DESC TAG Person")
        .assert_result_count(1)
        .assert_result_contains(vec![
            Value::String("name".into()),
            Value::String("STRING".into()),
        ]);
}

// ==================== DROP TAG Extended Tests ====================

#[test]
fn test_drop_tag_with_validation() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .assert_tag_exists("Person")
        .exec_ddl("DROP TAG Person")
        .assert_success()
        .assert_tag_not_exists("Person");
}

#[test]
fn test_drop_tag_if_exists() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("DROP TAG NonExistent") // Should fail
        .assert_error()
        .exec_ddl("DROP TAG IF EXISTS NonExistent") // Should succeed
        .assert_success();
}

// ==================== DROP EDGE Extended Tests ====================

#[test]
fn test_drop_edge_with_validation() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
        .assert_success()
        .exec_ddl("DROP EDGE KNOWS")
        .assert_success()
        .exec_ddl("DESC EDGE KNOWS")
        .assert_result_empty();
}

// ==================== Schema Evolution Tests ====================

#[test]
fn test_schema_evolution_complete_flow() {
    TestScenario::new()
        .setup_space("test_space")
        // Create initial schema
        .exec_ddl("CREATE TAG User(username STRING, created_at TIMESTAMP)")
        .assert_success()
        .exec_ddl("CREATE EDGE FOLLOWS(since DATE)")
        .assert_success()
        // Insert some data
        .exec_dml("INSERT VERTEX User(username, created_at) VALUES 1:('alice', now())")
        .assert_success()
        .exec_dml("INSERT VERTEX User(username, created_at) VALUES 2:('bob', now())")
        .assert_success()
        .exec_dml("INSERT EDGE FOLLOWS(since) VALUES 1 -> 2:('2024-01-01')")
        .assert_success()
        // Verify data exists
        .assert_vertex_count("User", 2)
        .assert_edge_count("FOLLOWS", 1)
        // Evolve schema - add fields
        .exec_ddl("ALTER TAG User ADD (email STRING, bio STRING)")
        .assert_success()
        // Update existing data with new fields
        .exec_dml("UPDATE 1 SET email = 'alice@example.com', bio = 'Hello world'")
        .assert_success()
        // Verify updated data
        .query("FETCH PROP ON User 1")
        .assert_result_count(1)
        .assert_vertex_props(1, "User", {
            let mut map = HashMap::new();
            map.insert("username", Value::String("alice".into()));
            map.insert("email", Value::String("alice@example.com".into()));
            map.insert("bio", Value::String("Hello world".into()));
            map
        });
}

// ==================== Error Handling Tests ====================

#[test]
fn test_create_tag_duplicate_field() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, name INT)") // Duplicate field
        .assert_error();
}

#[test]
fn test_alter_tag_nonexistent_field() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_ddl("ALTER TAG Person DROP (nonexistent_field)")
        .assert_error();
}

#[test]
fn test_drop_tag_with_data() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice')")
        .assert_success()
        .assert_vertex_exists(1, "Person")
        // Try to drop tag with existing data
        .exec_ddl("DROP TAG Person")
        .assert_error(); // Should fail because data exists
}

// ==================== Complex Schema Tests ====================

#[test]
fn test_complex_schema_with_multiple_tags_and_edges() {
    TestScenario::new()
        .setup_space("social_network")
        // Create tags
        .exec_ddl(r#"
            CREATE TAG Person(
                name STRING,
                age INT,
                email STRING,
                created_at TIMESTAMP
            )
        "#)
        .assert_success()
        .exec_ddl(r#"
            CREATE TAG Company(
                name STRING,
                founded_year INT,
                industry STRING
            )
        "#)
        .assert_success()
        // Create edges
        .exec_ddl(r#"
            CREATE EDGE KNOWS(
                since DATE,
                strength DOUBLE
            )
        "#)
        .assert_success()
        .exec_ddl(r#"
            CREATE EDGE WORKS_AT(
                since DATE,
                position STRING,
                salary DOUBLE
            )
        "#)
        .assert_success()
        // Verify all schema objects exist
        .assert_tag_exists("Person")
        .assert_tag_exists("Company")
        // Insert data
        .exec_dml("INSERT VERTEX Person(name, age, email) VALUES 1:('Alice', 30, 'alice@example.com')")
        .assert_success()
        .exec_dml("INSERT VERTEX Company(name, founded_year, industry) VALUES 101:('TechCorp', 2010, 'Technology')")
        .assert_success()
        .exec_dml("INSERT EDGE WORKS_AT(since, position, salary) VALUES 1 -> 101:('2020-01-01', 'Engineer', 100000.0)")
        .assert_success()
        // Verify relationships
        .assert_vertex_exists(1, "Person")
        .assert_vertex_exists(101, "Company")
        .assert_edge_exists(1, 101, "WORKS_AT")
        // Query with join-like operation
        .query(r#"
            GO FROM 1 OVER WORKS_AT YIELD 
                $^.Person.name AS person_name,
                $$.Company.name AS company_name,
                WORKS_AT.position AS position
        "#)
        .assert_result_count(1)
        .assert_result_columns(&["person_name", "company_name", "position"]);
}

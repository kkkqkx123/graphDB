//! Extended DML Integration Tests
//!
//! This file demonstrates how to use the new test framework to validate
//! actual execution effects of DML statements.

mod common;

use common::test_scenario::TestScenario;
use graphdb::core::Value;
use std::collections::HashMap;

// ==================== INSERT VERTEX Extended Tests ====================

#[test]
fn test_insert_vertex_and_verify() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)")
        .assert_success()
        .assert_vertex_exists(1, "Person")
        .assert_vertex_props(1, "Person", {
            let mut map = HashMap::new();
            map.insert("name", Value::String("Alice".into()));
            map.insert("age", Value::Int(30));
            map
        });
}

#[test]
fn test_insert_multiple_vertices() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .assert_success()
        .exec_dml(r#"
            INSERT VERTEX Person(name, age) VALUES 
                1:('Alice', 30),
                2:('Bob', 25),
                3:('Charlie', 35)
        "#)
        .assert_success()
        .assert_vertex_count("Person", 3)
        .assert_vertex_props(1, "Person", {
            let mut map = HashMap::new();
            map.insert("name", Value::String("Alice".into()));
            map
        })
        .assert_vertex_props(2, "Person", {
            let mut map = HashMap::new();
            map.insert("name", Value::String("Bob".into()));
            map
        })
        .assert_vertex_props(3, "Person", {
            let mut map = HashMap::new();
            map.insert("name", Value::String("Charlie".into()));
            map
        });
}

#[test]
fn test_insert_vertex_with_all_types() {
    TestScenario::new()
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
            VALUES 1:('test', 42, 3.14, true)
        "#)
        .assert_success()
        .assert_vertex_props(1, "TestTypes", {
            let mut map = HashMap::new();
            map.insert("str_field", Value::String("test".into()));
            map.insert("int_field", Value::Int(42));
            map.insert("double_field", Value::Double(3.14));
            map.insert("bool_field", Value::Bool(true));
            map
        });
}

// ==================== INSERT EDGE Extended Tests ====================

#[test]
fn test_insert_edge_and_verify() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .assert_success()
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2024-01-01')")
        .assert_success()
        .assert_edge_exists(1, 2, "KNOWS");
}

#[test]
fn test_insert_edge_with_rank() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE KNOWS(since DATE, strength DOUBLE)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .assert_success()
        // Insert multiple edges between same vertices with different ranks
        .exec_dml("INSERT EDGE KNOWS(since, strength) VALUES 1 -> 2 @0:('2020-01-01', 0.8)")
        .assert_success()
        .exec_dml("INSERT EDGE KNOWS(since, strength) VALUES 1 -> 2 @1:('2021-01-01', 0.9)")
        .assert_success()
        .assert_edge_count("KNOWS", 2);
}

// ==================== UPDATE Extended Tests ====================

#[test]
fn test_update_vertex_and_verify() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT, city STRING)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name, age, city) VALUES 1:('Alice', 30, 'NYC')")
        .assert_success()
        .assert_vertex_props(1, "Person", {
            let mut map = HashMap::new();
            map.insert("age", Value::Int(30));
            map.insert("city", Value::String("NYC".into()));
            map
        })
        // Update single field
        .exec_dml("UPDATE 1 SET age = 31")
        .assert_success()
        .assert_vertex_props(1, "Person", {
            let mut map = HashMap::new();
            map.insert("age", Value::Int(31));
            map
        })
        // Update multiple fields
        .exec_dml("UPDATE 1 SET age = 32, city = 'LA'")
        .assert_success()
        .assert_vertex_props(1, "Person", {
            let mut map = HashMap::new();
            map.insert("age", Value::Int(32));
            map.insert("city", Value::String("LA".into()));
            map
        });
}

#[test]
fn test_update_vertex_with_condition() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT, status STRING)")
        .assert_success()
        .exec_dml(r#"
            INSERT VERTEX Person(name, age, status) VALUES 
                1:('Alice', 30, 'active'),
                2:('Bob', 25, 'inactive'),
                3:('Charlie', 35, 'active')
        "#)
        .assert_success()
        // Update only active users
        .exec_dml("UPDATE 1 SET status = 'premium' WHEN status == 'active'")
        .assert_success()
        .assert_vertex_props(1, "Person", {
            let mut map = HashMap::new();
            map.insert("status", Value::String("premium".into()));
            map
        })
        // Verify Bob's status unchanged
        .query("FETCH PROP ON Person 2")
        .assert_result_contains(vec![Value::String("status".into()), Value::String("inactive".into())]);
}

#[test]
fn test_update_edge_and_verify() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE KNOWS(since DATE, strength DOUBLE)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .assert_success()
        .exec_dml("INSERT EDGE KNOWS(since, strength) VALUES 1 -> 2:('2020-01-01', 0.5)")
        .assert_success()
        // Update edge
        .exec_dml("UPDATE 1 -> 2 OF KNOWS SET strength = 0.9")
        .assert_success()
        .query("FETCH PROP ON KNOWS 1 -> 2")
        .assert_result_contains(vec![Value::String("strength".into()), Value::Double(0.9)]);
}

// ==================== DELETE Extended Tests ====================

#[test]
fn test_delete_vertex_and_verify() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .assert_success()
        .assert_vertex_count("Person", 2)
        // Delete one vertex
        .exec_dml("DELETE VERTEX 1")
        .assert_success()
        .assert_vertex_not_exists(1, "Person")
        .assert_vertex_exists(2, "Person")
        .assert_vertex_count("Person", 1);
}

#[test]
fn test_delete_multiple_vertices() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_dml(r#"
            INSERT VERTEX Person(name) VALUES 
                1:('Alice'),
                2:('Bob'),
                3:('Charlie'),
                4:('David')
        "#)
        .assert_success()
        .assert_vertex_count("Person", 4)
        // Delete multiple vertices
        .exec_dml("DELETE VERTEX 1, 2, 3")
        .assert_success()
        .assert_vertex_count("Person", 1)
        .assert_vertex_exists(4, "Person");
}

#[test]
fn test_delete_edge_and_verify() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob'), 3:('Charlie')")
        .assert_success()
        .exec_dml(r#"
            INSERT EDGE KNOWS(since) VALUES 
                1 -> 2:('2020-01-01'),
                1 -> 3:('2021-01-01')
        "#)
        .assert_success()
        .assert_edge_count("KNOWS", 2)
        // Delete one edge
        .exec_dml("DELETE EDGE KNOWS 1 -> 2")
        .assert_success()
        .assert_edge_not_exists(1, 2, "KNOWS")
        .assert_edge_exists(1, 3, "KNOWS")
        .assert_edge_count("KNOWS", 1);
}

// ==================== Data Flow Tests ====================

#[test]
fn test_complete_crud_flow() {
    TestScenario::new()
        .setup_space("test_space")
        // Create schema
        .exec_ddl("CREATE TAG Product(name STRING, price DOUBLE, stock INT)")
        .assert_success()
        // Create
        .exec_dml("INSERT VERTEX Product(name, price, stock) VALUES 101:('Laptop', 999.99, 10)")
        .assert_success()
        .assert_vertex_exists(101, "Product")
        .assert_vertex_props(101, "Product", {
            let mut map = HashMap::new();
            map.insert("stock", Value::Int(10));
            map
        })
        // Read
        .query("FETCH PROP ON Product 101")
        .assert_result_count(1)
        .assert_result_contains(vec![Value::String("name".into()), Value::String("Laptop".into())])
        // Update
        .exec_dml("UPDATE 101 SET stock = stock - 1")
        .assert_success()
        .assert_vertex_props(101, "Product", {
            let mut map = HashMap::new();
            map.insert("stock", Value::Int(9));
            map
        })
        // Delete
        .exec_dml("DELETE VERTEX 101")
        .assert_success()
        .assert_vertex_not_exists(101, "Product");
}

#[test]
fn test_insert_update_delete_sequence() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG User(username STRING, email STRING, active BOOL)")
        .assert_success()
        // Insert batch
        .exec_dml(r#"
            INSERT VERTEX User(username, email, active) VALUES 
                1:('user1', 'user1@example.com', true),
                2:('user2', 'user2@example.com', true),
                3:('user3', 'user3@example.com', true)
        "#)
        .assert_success()
        .assert_vertex_count("User", 3)
        // Update batch
        .exec_dml("UPDATE 1 SET email = 'new1@example.com'")
        .assert_success()
        .exec_dml("UPDATE 2 SET active = false")
        .assert_success()
        .assert_vertex_props(1, "User", {
            let mut map = HashMap::new();
            map.insert("email", Value::String("new1@example.com".into()));
            map
        })
        .assert_vertex_props(2, "User", {
            let mut map = HashMap::new();
            map.insert("active", Value::Bool(false));
            map
        })
        // Delete batch
        .exec_dml("DELETE VERTEX 1, 2")
        .assert_success()
        .assert_vertex_count("User", 1)
        .assert_vertex_not_exists(1, "User")
        .assert_vertex_not_exists(2, "User")
        .assert_vertex_exists(3, "User");
}

// ==================== Error Handling Tests ====================

#[test]
fn test_insert_duplicate_vertex() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice')")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Bob')") // Duplicate VID
        .assert_error();
}

#[test]
fn test_update_nonexistent_vertex() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_dml("UPDATE 999 SET name = 'Ghost'")
        .assert_success() // May succeed with 0 rows affected
        .assert_vertex_not_exists(999, "Person");
}

#[test]
fn test_delete_nonexistent_vertex() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_dml("DELETE VERTEX 999")
        .assert_success(); // Should succeed even if vertex doesn't exist
}

// ==================== Complex Relationship Tests ====================

#[test]
fn test_social_network_data_flow() {
    TestScenario::new()
        .setup_space("social_network")
        // Create schema
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .assert_success()
        .exec_ddl("CREATE EDGE KNOWS(since DATE, strength DOUBLE)")
        .assert_success()
        // Insert people
        .exec_dml(r#"
            INSERT VERTEX Person(name, age) VALUES 
                1:('Alice', 30),
                2:('Bob', 25),
                3:('Charlie', 35),
                4:('David', 28)
        "#)
        .assert_success()
        .assert_vertex_count("Person", 4)
        // Create relationships
        .exec_dml(r#"
            INSERT EDGE KNOWS(since, strength) VALUES 
                1 -> 2:('2020-01-01', 0.9),
                1 -> 3:('2021-01-01', 0.8),
                2 -> 3:('2020-06-01', 0.7),
                3 -> 4:('2022-01-01', 0.9)
        "#)
        .assert_success()
        .assert_edge_count("KNOWS", 4)
        // Query Alice's friends
        .query("GO FROM 1 OVER KNOWS YIELD $$.Person.name AS friend_name")
        .assert_result_count(2)
        // Update relationship strength
        .exec_dml("UPDATE 1 -> 2 OF KNOWS SET strength = 1.0")
        .assert_success()
        // Remove a relationship
        .exec_dml("DELETE EDGE KNOWS 2 -> 3")
        .assert_success()
        .assert_edge_not_exists(2, 3, "KNOWS")
        .assert_edge_count("KNOWS", 3);
}

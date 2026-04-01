//! Extended DQL Integration Tests
//!
//! This file demonstrates how to use the new test framework to validate
//! actual execution effects of DQL statements.

mod common;

use common::test_scenario::TestScenario;
use graphdb::core::Value;

// ==================== MATCH Query Extended Tests ====================

#[test]
fn test_match_basic_with_data() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .assert_success()
        .exec_dml(
            r#"
            INSERT VERTEX Person(name, age) VALUES 
                1:('Alice', 30),
                2:('Bob', 25),
                3:('Charlie', 35)
        "#,
        )
        .assert_success()
        // Query all persons
        .query("MATCH (n:Person) RETURN n.name, n.age")
        .assert_success()
        .assert_result_count(3)
        .assert_result_columns(&["n.name", "n.age"]);
}

#[test]
fn test_match_with_where_clause() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .assert_success()
        .exec_dml(
            r#"
            INSERT VERTEX Person(name, age) VALUES 
                1:('Alice', 30),
                2:('Bob', 25),
                3:('Charlie', 35),
                4:('David', 20)
        "#,
        )
        .assert_success()
        // Query persons older than 25
        .query("MATCH (n:Person) WHERE n.age > 25 RETURN n.name, n.age")
        .assert_success()
        .assert_result_count(2)
        // Query persons with specific name
        .query("MATCH (n:Person) WHERE n.name == 'Alice' RETURN n.name, n.age")
        .assert_success()
        .assert_result_count(1)
        .assert_result_contains(vec![Value::String("Alice".into()), Value::Int(30)]);
}

#[test]
fn test_match_with_edge_traversal() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
        .assert_success()
        .exec_dml(
            r#"
            INSERT VERTEX Person(name) VALUES 
                1:('Alice'),
                2:('Bob'),
                3:('Charlie')
        "#,
        )
        .assert_success()
        .exec_dml(
            r#"
            INSERT EDGE KNOWS(since) VALUES 
                1 -> 2:('2020-01-01'),
                1 -> 3:('2021-01-01')
        "#,
        )
        .assert_success()
        // Query Alice's friends
        .query("MATCH (a:Person)-[:KNOWS]->(b:Person) WHERE a.name == 'Alice' RETURN b.name")
        .assert_success()
        .assert_result_count(2)
        // Query all relationships
        .query("MATCH (a:Person)-[:KNOWS]->(b:Person) RETURN a.name, b.name")
        .assert_success()
        .assert_result_count(2);
}

#[test]
fn test_match_with_order_and_limit() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .assert_success()
        .exec_dml(
            r#"
            INSERT VERTEX Person(name, age) VALUES 
                1:('Alice', 30),
                2:('Bob', 25),
                3:('Charlie', 35),
                4:('David', 28)
        "#,
        )
        .assert_success()
        // Query ordered by age ascending
        .query("MATCH (n:Person) RETURN n.name, n.age ORDER BY n.age ASC")
        .assert_success()
        .assert_result_count(4)
        // Query ordered by age descending with limit
        .query("MATCH (n:Person) RETURN n.name, n.age ORDER BY n.age DESC LIMIT 2")
        .assert_success()
        .assert_result_count(2);
}

// ==================== GO Query Extended Tests ====================

#[test]
fn test_go_basic_traversal() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
        .assert_success()
        .exec_dml(
            r#"
            INSERT VERTEX Person(name) VALUES 
                1:('Alice'),
                2:('Bob'),
                3:('Charlie'),
                4:('David')
        "#,
        )
        .assert_success()
        .exec_dml(
            r#"
            INSERT EDGE KNOWS(since) VALUES 
                1 -> 2:('2020-01-01'),
                1 -> 3:('2021-01-01'),
                2 -> 4:('2022-01-01')
        "#,
        )
        .assert_success()
        // GO from Alice
        .query("GO FROM 1 OVER KNOWS YIELD $$.Person.name AS friend_name")
        .assert_success()
        .assert_result_count(2)
        // GO 2 steps from Alice
        .query("GO 2 FROM 1 OVER KNOWS YIELD $$.Person.name AS friend_of_friend")
        .assert_success()
        .assert_result_count(1); // Only David
}

#[test]
fn test_go_with_reversely() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE FOLLOWS(since DATE)")
        .assert_success()
        .exec_dml(
            r#"
            INSERT VERTEX Person(name) VALUES 
                1:('Alice'),
                2:('Bob'),
                3:('Charlie')
        "#,
        )
        .assert_success()
        .exec_dml(
            r#"
            INSERT EDGE FOLLOWS(since) VALUES 
                2 -> 1:('2020-01-01'),
                3 -> 1:('2021-01-01')
        "#,
        )
        .assert_success()
        // Who follows Alice?
        .query("GO FROM 1 OVER FOLLOWS REVERSELY YIELD $^.Person.name AS follower")
        .assert_success()
        .assert_result_count(2);
}

#[test]
fn test_go_with_bidirect() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE FRIEND(since DATE)")
        .assert_success()
        .exec_dml(
            r#"
            INSERT VERTEX Person(name) VALUES 
                1:('Alice'),
                2:('Bob'),
                3:('Charlie')
        "#,
        )
        .assert_success()
        .exec_dml(
            r#"
            INSERT EDGE FRIEND(since) VALUES 
                1 -> 2:('2020-01-01'),
                3 -> 1:('2021-01-01')
        "#,
        )
        .assert_success()
        // Bidirectional query
        .query("GO FROM 1 OVER FRIEND BIDIRECT YIELD $$.Person.name AS friend")
        .assert_success()
        .assert_result_count(2);
}

// ==================== LOOKUP Query Extended Tests ====================

#[test]
fn test_lookup_basic() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .assert_success()
        .exec_ddl("CREATE TAG INDEX idx_person_age ON Person(age)")
        .assert_success()
        .exec_dml(
            r#"
            INSERT VERTEX Person(name, age) VALUES 
                1:('Alice', 30),
                2:('Bob', 25),
                3:('Charlie', 30)
        "#,
        )
        .assert_success()
        // Lookup by age
        .query("LOOKUP ON Person WHERE Person.age == 30")
        .assert_success()
        .assert_result_count(2);
}

#[test]
fn test_lookup_with_yield() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .assert_success()
        .exec_ddl("CREATE TAG INDEX idx_person_age ON Person(age)")
        .assert_success()
        .exec_dml(
            r#"
            INSERT VERTEX Person(name, age) VALUES 
                1:('Alice', 30),
                2:('Bob', 25)
        "#,
        )
        .assert_success()
        // Lookup with YIELD
        .query("LOOKUP ON Person WHERE Person.age == 30 YIELD Person.name")
        .assert_success()
        .assert_result_count(1)
        .assert_result_contains(vec![Value::String("Alice".into())]);
}

// ==================== FETCH Query Extended Tests ====================

#[test]
fn test_fetch_vertex() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT, city STRING)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name, age, city) VALUES 1:('Alice', 30, 'NYC')")
        .assert_success()
        // Fetch vertex properties
        .query("FETCH PROP ON Person 1")
        .assert_success()
        .assert_result_count(1)
        .assert_result_contains(vec![
            Value::String("name".into()),
            Value::String("Alice".into()),
        ])
        .assert_result_contains(vec![Value::String("age".into()), Value::Int(30)]);
}

#[test]
fn test_fetch_multiple_vertices() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_dml(
            r#"
            INSERT VERTEX Person(name) VALUES 
                1:('Alice'),
                2:('Bob'),
                3:('Charlie')
        "#,
        )
        .assert_success()
        // Fetch multiple vertices
        .query("FETCH PROP ON Person 1, 2, 3")
        .assert_success()
        .assert_result_count(3);
}

#[test]
fn test_fetch_edge() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE KNOWS(since DATE, strength DOUBLE)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .assert_success()
        .exec_dml("INSERT EDGE KNOWS(since, strength) VALUES 1 -> 2:('2020-01-01', 0.9)")
        .assert_success()
        // Fetch edge properties
        .query("FETCH PROP ON KNOWS 1 -> 2")
        .assert_success()
        .assert_result_count(1)
        .assert_result_contains(vec![
            Value::String("since".into()),
            Value::String("2020-01-01".into()),
        ]);
}

// ==================== FIND PATH Extended Tests ====================

#[test]
fn test_find_shortest_path() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
        .assert_success()
        .exec_dml(
            r#"
            INSERT VERTEX Person(name) VALUES 
                1:('Alice'),
                2:('Bob'),
                3:('Charlie'),
                4:('David')
        "#,
        )
        .assert_success()
        .exec_dml(
            r#"
            INSERT EDGE KNOWS(since) VALUES 
                1 -> 2:('2020-01-01'),
                2 -> 3:('2021-01-01'),
                3 -> 4:('2022-01-01')
        "#,
        )
        .assert_success()
        // Find shortest path from Alice to David
        .query("FIND SHORTEST PATH FROM 1 TO 4 OVER KNOWS")
        .assert_success()
        .assert_result_count(1);
}

#[test]
fn test_find_all_paths() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
        .assert_success()
        .exec_dml(
            r#"
            INSERT VERTEX Person(name) VALUES 
                1:('Alice'),
                2:('Bob'),
                3:('Charlie'),
                4:('David')
        "#,
        )
        .assert_success()
        // Create two paths from Alice to David
        .exec_dml(
            r#"
            INSERT EDGE KNOWS(since) VALUES 
                1 -> 2:('2020-01-01'),
                2 -> 4:('2021-01-01'),
                1 -> 3:('2020-01-01'),
                3 -> 4:('2021-01-01')
        "#,
        )
        .assert_success()
        // Find all paths
        .query("FIND ALL PATH FROM 1 TO 4 OVER KNOWS")
        .assert_success()
        .assert_result_count(2);
}

#[test]
fn test_find_path_with_steps_limit() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
        .assert_success()
        .exec_dml(
            r#"
            INSERT VERTEX Person(name) VALUES 
                1:('Alice'),
                2:('Bob'),
                3:('Charlie'),
                4:('David')
        "#,
        )
        .assert_success()
        .exec_dml(
            r#"
            INSERT EDGE KNOWS(since) VALUES 
                1 -> 2:('2020-01-01'),
                2 -> 3:('2021-01-01'),
                3 -> 4:('2022-01-01')
        "#,
        )
        .assert_success()
        // Find path with step limit
        .query("FIND SHORTEST PATH FROM 1 TO 4 OVER KNOWS UPTO 2 STEPS")
        .assert_success()
        .assert_result_empty(); // Path requires 3 steps
}

// ==================== SUBGRAPH Extended Tests ====================

#[test]
fn test_get_subgraph() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
        .assert_success()
        .exec_dml(
            r#"
            INSERT VERTEX Person(name) VALUES 
                1:('Alice'),
                2:('Bob'),
                3:('Charlie'),
                4:('David')
        "#,
        )
        .assert_success()
        .exec_dml(
            r#"
            INSERT EDGE KNOWS(since) VALUES 
                1 -> 2:('2020-01-01'),
                1 -> 3:('2021-01-01'),
                2 -> 4:('2022-01-01')
        "#,
        )
        .assert_success()
        // Get subgraph from Alice
        .query("GET SUBGRAPH FROM 1")
        .assert_success();
}

// ==================== Complex Query Tests ====================

#[test]
fn test_complex_social_network_query() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("social_network")
        // Setup schema
        .exec_ddl("CREATE TAG Person(name STRING, age INT, city STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE KNOWS(since DATE, strength DOUBLE)")
        .assert_success()
        // Insert data
        .exec_dml(
            r#"
            INSERT VERTEX Person(name, age, city) VALUES 
                1:('Alice', 30, 'NYC'),
                2:('Bob', 25, 'LA'),
                3:('Charlie', 35, 'NYC'),
                4:('David', 28, 'LA')
        "#,
        )
        .assert_success()
        .exec_dml(
            r#"
            INSERT EDGE KNOWS(since, strength) VALUES 
                1 -> 2:('2020-01-01', 0.9),
                1 -> 3:('2021-01-01', 0.8),
                2 -> 4:('2022-01-01', 0.7),
                3 -> 4:('2022-01-01', 0.9)
        "#,
        )
        .assert_success()
        // Complex query: Find friends of friends who live in LA
        // Note: There are 2 paths to David (via Bob and via Charlie), so we get 2 rows
        // Using DISTINCT to get unique results
        .query(
            r#"
            MATCH (a:Person)-[:KNOWS]->(b:Person)-[:KNOWS]->(c:Person)
            WHERE a.name == 'Alice' AND c.city == 'LA'
            RETURN DISTINCT c.name, c.age
        "#,
        )
        .assert_success()
        .assert_result_count(1)
        .assert_result_contains(vec![Value::String("David".into()), Value::Int(28)]);
}

#[test]
fn test_aggregation_query() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Product(name STRING, category STRING, price DOUBLE)")
        .assert_success()
        .exec_dml(
            r#"
            INSERT VERTEX Product(name, category, price) VALUES 
                1:('Laptop', 'Electronics', 999.99),
                2:('Mouse', 'Electronics', 29.99),
                3:('Keyboard', 'Electronics', 79.99),
                4:('Desk', 'Furniture', 299.99)
        "#,
        )
        .assert_success()
        // Count by category
        .query(
            r#"
            MATCH (p:Product)
            RETURN p.category, count(*) AS count
            ORDER BY count DESC
        "#,
        )
        .assert_success()
        .debug_print_result()
        .assert_result_count(2);
}

// ==================== Error Handling Tests ====================

#[test]
fn test_query_nonexistent_tag() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .query("MATCH (n:NonExistent) RETURN n")
        .assert_success()
        .assert_result_empty();
}

#[test]
fn test_query_invalid_syntax() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .query("MATCH (n:Person RETURN n")
        .assert_error();
}

#[test]
fn test_query_nonexistent_property() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice')")
        .assert_success()
        // Query non-existent property
        .query("MATCH (n:Person) RETURN n.nonexistent")
        .assert_success()
        .assert_result_count(1);
}

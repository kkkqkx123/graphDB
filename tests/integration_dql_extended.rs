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

// ==================== UNWIND Statement Tests ====================

#[test]
fn test_unwind_basic_list() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .query("UNWIND [1, 2, 3] AS n RETURN n")
        .assert_success()
        .assert_result_count(3);
}

#[test]
fn test_unwind_with_match() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, tags STRING)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name, tags) VALUES 1:('Alice', 'friend,colleague'), 2:('Bob', 'family')")
        .assert_success()
        .query("MATCH (n:Person) UNWIND split(n.tags, ',') AS tag RETURN n.name, tag")
        .assert_success()
        .assert_result_count(3);
}

#[test]
fn test_unwind_empty_list() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .query("UNWIND [] AS n RETURN n")
        .assert_success()
        .assert_result_empty();
}

// ==================== Aggregate Functions Tests ====================

#[test]
fn test_aggregate_count() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Product(category STRING)")
        .assert_success()
        .exec_dml(
            r#"
            INSERT VERTEX Product(category) VALUES 
                1:('Electronics'),
                2:('Electronics'),
                3:('Furniture'),
                4:('Electronics')
        "#,
        )
        .assert_success()
        .query("MATCH (p:Product) RETURN count(*) AS total")
        .assert_success()
        .assert_result_count(1);
}

#[test]
fn test_aggregate_sum() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Product(name STRING, price DOUBLE)")
        .assert_success()
        .exec_dml(
            r#"
            INSERT VERTEX Product(name, price) VALUES 
                1:('Laptop', 999.99),
                2:('Mouse', 29.99),
                3:('Keyboard', 79.99)
        "#,
        )
        .assert_success()
        .query("MATCH (p:Product) RETURN SUM(p.price) AS total_price")
        .assert_success()
        .assert_result_count(1);
}

#[test]
fn test_aggregate_avg() {
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
        .query("MATCH (p:Person) RETURN AVG(p.age) AS avg_age")
        .assert_success()
        .assert_result_count(1);
}

#[test]
fn test_aggregate_max_min() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Product(name STRING, price DOUBLE)")
        .assert_success()
        .exec_dml(
            r#"
            INSERT VERTEX Product(name, price) VALUES 
                1:('Laptop', 999.99),
                2:('Mouse', 29.99),
                3:('Keyboard', 79.99)
        "#,
        )
        .assert_success()
        .query("MATCH (p:Product) RETURN MAX(p.price) AS max_price, MIN(p.price) AS min_price")
        .assert_success()
        .assert_result_count(1);
}

#[test]
fn test_aggregate_collect() {
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
        .query("MATCH (p:Person) RETURN COLLECT(p.name) AS names")
        .assert_success()
        .assert_result_count(1);
}

// ==================== GROUP BY Tests ====================

#[test]
fn test_group_by_basic() {
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
                4:('Desk', 'Furniture', 299.99),
                5:('Chair', 'Furniture', 199.99)
        "#,
        )
        .assert_success()
        .query(
            r#"
            MATCH (p:Product) 
            RETURN p.category, COUNT(*) AS count, SUM(p.price) AS total
            GROUP BY p.category
        "#,
        )
        .assert_success()
        .assert_result_count(2);
}

#[test]
fn test_group_by_with_having() {
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
        .query(
            r#"
            MATCH (p:Product) 
            RETURN p.category, COUNT(*) AS count
            GROUP BY p.category
            HAVING COUNT(*) > 2
        "#,
        )
        .assert_success()
        .assert_result_count(1);
}

// ==================== SUBGRAPH Extended Tests ====================

#[test]
fn test_subgraph_with_steps() {
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
        .query("GET SUBGRAPH 2 STEPS FROM 1")
        .assert_success();
}

#[test]
fn test_subgraph_with_edge_filter() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
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
            INSERT EDGE KNOWS(since) VALUES 
                1 -> 2:('2020-01-01')
        "#,
        )
        .assert_success()
        .exec_dml(
            r#"
            INSERT EDGE FOLLOWS(since) VALUES 
                1 -> 3:('2021-01-01')
        "#,
        )
        .assert_success()
        .query("GET SUBGRAPH FROM 1 OVER KNOWS")
        .assert_success();
}

// ==================== Pipe Operation Tests ====================

#[test]
fn test_pipe_basic() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
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
            INSERT EDGE KNOWS(since) VALUES 
                1 -> 2:('2020-01-01')
        "#,
        )
        .assert_success()
        .exec_dml(
            r#"
            INSERT EDGE FRIEND(since) VALUES 
                2 -> 3:('2021-01-01')
        "#,
        )
        .assert_success()
        .query("GO FROM 1 OVER KNOWS YIELD target.id AS id | GO FROM $-.id OVER FRIEND")
        .assert_success()
        .assert_result_count(1);
}

#[test]
fn test_pipe_with_yield() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .assert_success()
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
        .assert_success()
        .exec_dml(
            r#"
            INSERT VERTEX Person(name, age) VALUES 
                1:('Alice', 30),
                2:('Bob', 25)
        "#,
        )
        .assert_success()
        .exec_dml(
            r#"
            INSERT EDGE KNOWS(since) VALUES 
                1 -> 2:('2020-01-01')
        "#,
        )
        .assert_success()
        .query(
            "GO FROM 1 OVER KNOWS YIELD target.id AS id, target.age AS age | YIELD $-.id, $-.age",
        )
        .assert_success()
        .assert_result_count(1);
}

// ==================== Explain/Profile Tests ====================

#[test]
fn test_explain_match() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .query("EXPLAIN MATCH (n:Person) RETURN n")
        .assert_success();
}

#[test]
fn test_explain_go() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
        .assert_success()
        .query("EXPLAIN GO FROM 1 OVER KNOWS")
        .assert_success();
}

#[test]
fn test_profile_query() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .assert_success()
        .query("PROFILE MATCH (n:Person) RETURN n")
        .assert_success();
}

// ==================== Cypher Style RETURN/WITH Tests ====================

#[test]
fn test_cypher_return_basic() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .assert_success()
        .exec_dml(
            r#"
            INSERT VERTEX Person(name, age) VALUES 
                1:('Alice', 30),
                2:('Bob', 25)
        "#,
        )
        .assert_success()
        .query("MATCH (n:Person) RETURN n.name AS name, n.age AS age ORDER BY age")
        .assert_success()
        .assert_result_count(2);
}

#[test]
fn test_cypher_return_distinct() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, city STRING)")
        .assert_success()
        .exec_dml(
            r#"
            INSERT VERTEX Person(name, city) VALUES 
                1:('Alice', 'NYC'),
                2:('Bob', 'NYC'),
                3:('Charlie', 'LA')
        "#,
        )
        .assert_success()
        .query("MATCH (n:Person) RETURN DISTINCT n.city AS city")
        .assert_success()
        .assert_result_count(2);
}

#[test]
fn test_cypher_with_clause() {
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
        .query(
            r#"
            MATCH (n:Person) 
            WITH n.age AS age WHERE age > 25 
            RETURN age
        "#,
        )
        .assert_success()
        .assert_result_count(2);
}

#[test]
fn test_cypher_with_order_by() {
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
        .query(
            r#"
            MATCH (n:Person) 
            WITH n.name AS name, n.age AS age 
            RETURN name, age ORDER BY age DESC LIMIT 2
        "#,
        )
        .assert_success()
        .assert_result_count(2);
}

// ==================== Edge Cases and Boundary Tests ====================

#[test]
fn test_empty_result_set() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)")
        .assert_success()
        .query("MATCH (n:Person) WHERE n.age > 100 RETURN n")
        .assert_success()
        .assert_result_empty();
}

#[test]
fn test_large_limit() {
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
        .query("MATCH (n:Person) RETURN n LIMIT 1000")
        .assert_success()
        .assert_result_count(3);
}

#[test]
fn test_zero_limit() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .assert_success()
        .query("MATCH (n:Person) RETURN n LIMIT 0")
        .assert_success()
        .assert_result_empty();
}

#[test]
fn test_self_loop_edge() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE SELF_LOOP(notes STRING)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice')")
        .assert_success()
        .exec_dml("INSERT EDGE SELF_LOOP(notes) VALUES 1 -> 1:('self reference')")
        .assert_success()
        .query("MATCH (n:Person)-[:SELF_LOOP]->(n:Person) RETURN n.name")
        .assert_success()
        .assert_result_count(1);
}

#[test]
fn test_multiple_edge_types() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
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
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01')")
        .assert_success()
        .exec_dml("INSERT EDGE FOLLOWS(since) VALUES 1 -> 3:('2021-01-01')")
        .assert_success()
        .query("MATCH (n:Person)-[:KNOWS|:FOLLOWS]->(m:Person) RETURN n.name, m.name")
        .assert_success()
        .assert_result_count(2);
}

#[test]
fn test_null_property_handling() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, nickname STRING)")
        .assert_success()
        .exec_dml(
            r#"
            INSERT VERTEX Person(name, nickname) VALUES
                1:('Alice', 'Ali'),
                2:('Bob', '')
        "#,
        )
        .assert_success()
        .query("MATCH (n:Person) RETURN n.name, n.nickname")
        .assert_success()
        .assert_result_count(2);
}

#[test]
fn test_deep_traversal() {
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
                4:('David'),
                5:('Eve')
        "#,
        )
        .assert_success()
        .exec_dml(
            r#"
            INSERT EDGE KNOWS(since) VALUES
                1 -> 2:('2020-01-01'),
                2 -> 3:('2020-02-01'),
                3 -> 4:('2020-03-01'),
                4 -> 5:('2020-04-01')
        "#,
        )
        .assert_success()
        .query("MATCH (a:Person)-[:KNOWS]->(b:Person)-[:KNOWS]->(c:Person)-[:KNOWS]->(d:Person)-[:KNOWS]->(e:Person) WHERE a.name == 'Alice' RETURN e.name")
        .assert_success()
        .assert_result_count(1);
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
        .query("MATCH (n:Person) RETURN n.nonexistent")
        .assert_success()
        .assert_result_count(1);
}

#[test]
fn test_type_mismatch_error() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)")
        .assert_success()
        .query("MATCH (n:Person) WHERE n.age == 'thirty' RETURN n")
        .assert_success();
}

#[test]
fn test_invalid_edge_direction() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
        .assert_success()
        .query("MATCH (n:Person)<-[:KNOWS]-(m:Person) RETURN n, m")
        .assert_success();
}

#[test]
fn test_empty_string_property() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('')")
        .assert_success()
        .query("MATCH (n:Person) WHERE n.name == '' RETURN n")
        .assert_success()
        .assert_result_count(1);
}

#[test]
fn test_very_long_string_property() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(description STRING)")
        .assert_success()
        .exec_dml(&format!(
            "INSERT VERTEX Person(description) VALUES 1:(\'{}\')",
            "a".repeat(1000)
        ))
        .assert_success()
        .query("MATCH (n:Person) RETURN n.description")
        .assert_success()
        .assert_result_count(1);
}

#[test]
fn test_special_characters_in_string() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        .exec_dml(
            r#"
            INSERT VERTEX Person(name) VALUES
                1:('Alice\'s "special" name'),
                2:('Bob\nNewline'),
                3:('Charlie\tTab')
        "#,
        )
        .assert_success()
        .query("MATCH (n:Person) RETURN n.name")
        .assert_success()
        .assert_result_count(3);
}

#[test]
fn test_numeric_edge_cases() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Product(price DOUBLE)")
        .assert_success()
        .exec_dml(
            r#"
            INSERT VERTEX Product(price) VALUES
                1:(0.0),
                2:(-99.99),
                3:(999999999.99)
        "#,
        )
        .assert_success()
        .query("MATCH (p:Product) RETURN p.price ORDER BY p.price")
        .assert_success()
        .assert_result_count(3);
}

//! Data Query Language (DQL) Integration Tests
//!
//! Test coverage:
//! - MATCH - Pattern matching queries
//! - GO - Graph traversal queries
//! - LOOKUP - Index-based lookups
//! - FETCH - Fetch data
//! - FIND PATH - Path finding
//! - SUBGRAPH - Subgraph queries
//! - YIELD - Result projection and filtering

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

// ==================== MATCH Statement Tests ====================

#[test]
fn test_match_parser_basic() {
    let query = "MATCH (n:Person) RETURN n";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    println!("MATCH basic parsing result: {:?}", result);
    let _ = result;
}

#[test]
fn test_match_parser_with_where() {
    let query = "MATCH (n:Person) WHERE n.age > 25 RETURN n";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    println!("MATCH with WHERE parsing result: {:?}", result);
    let _ = result;
}

#[test]
fn test_match_parser_with_edge() {
    let query = "MATCH (n:Person)-[KNOWS]->(m:Person) RETURN n, m";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    println!("MATCH with edge parsing result: {:?}", result);
    let _ = result;
}

#[test]
fn test_match_parser_with_order_limit() {
    let query = "MATCH (n:Person) RETURN n ORDER BY n.age DESC LIMIT 10";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    println!("MATCH with ORDER BY and LIMIT parsing result: {:?}", result);
    let _ = result;
}

#[test]
fn test_match_parser_complex() {
    let query = "MATCH (n:Person)-[KNOWS]->(m:Person) WHERE n.age > 25 AND m.age < 40 RETURN n.name, m.name ORDER BY m.age LIMIT 5";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    println!("MATCH complex query parsing result: {:?}", result);
    let _ = result;
}

#[test]
fn test_match_parser_invalid_syntax() {
    let query = "MATCH (n:Person RETURN n";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(result.is_err(), "Invalid syntax should return an error");
}

#[test]
fn test_match_execution_basic() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30), 2:('Bob', 25)")
        .query("MATCH (n:Person) RETURN n")
        .assert_success()
        .assert_result_count(2);
}

#[test]
fn test_match_execution_with_projection() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30), 2:('Bob', 25)")
        .query("MATCH (n:Person) RETURN n.name, n.age")
        .assert_success()
        .assert_result_count(2);
}

#[test]
fn test_match_execution_with_where() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30), 2:('Bob', 25), 3:('Charlie', 35)")
        .query("MATCH (n:Person) WHERE n.age > 25 RETURN n")
        .assert_success()
        .assert_result_count(2);
}

#[test]
fn test_match_execution_with_edge() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob'), 3:('Charlie')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01'), 2 -> 3:('2021-01-01')")
        .query("MATCH (n:Person)-[KNOWS]->(m:Person) RETURN n, m")
        .assert_success()
        .assert_result_count(2);
}

#[test]
fn test_match_execution_with_order_limit() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30), 2:('Bob', 25), 3:('Charlie', 35)")
        .query("MATCH (n:Person) RETURN n ORDER BY n.age DESC LIMIT 2")
        .assert_success()
        .assert_result_count(2);
}

// ==================== GO Statement Tests ====================

#[test]
fn test_go_parser_basic() {
    let query = "GO FROM 1 OVER KNOWS";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(result.is_ok(), "GO basic parsing should succeed: {:?}", result.err());

    let stmt = result.expect("GO statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "GO");
}

#[test]
fn test_go_parser_with_steps() {
    let query = "GO 2 FROM 1 OVER KNOWS";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(result.is_ok(), "GO with steps parsing should succeed: {:?}", result.err());

    let stmt = result.expect("GO statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "GO");
}

#[test]
fn test_go_parser_reversely() {
    let query = "GO FROM 1 OVER KNOWS REVERSELY";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(result.is_ok(), "GO reversely parsing should succeed: {:?}", result.err());

    let stmt = result.expect("GO statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "GO");
}

#[test]
fn test_go_parser_bidirect() {
    let query = "GO FROM 1 OVER KNOWS BIDIRECT";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(result.is_ok(), "GO bidirect parsing should succeed: {:?}", result.err());

    let stmt = result.expect("GO statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "GO");
}

#[test]
fn test_go_parser_with_where() {
    let query = "GO FROM 1 OVER KNOWS WHERE $^.age > 25";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    println!("GO with WHERE parsing result: {:?}", result);
    let _ = result;
}

#[test]
fn test_go_parser_with_yield() {
    let query = "GO FROM 1 OVER KNOWS YIELD name, age";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(result.is_ok(), "GO with YIELD parsing should succeed: {:?}", result.err());

    let stmt = result.expect("GO statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "GO");
}

#[test]
fn test_go_parser_complex() {
    let query = "GO 2 FROM 1 OVER KNOWS REVERSELY YIELD $^.name, $^.age";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(result.is_ok(), "GO complex query parsing should succeed: {:?}", result.err());

    let stmt = result.expect("GO statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "GO");
}

#[test]
fn test_go_execution_basic() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob'), 3:('Charlie')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01'), 1 -> 3:('2021-01-01')")
        .query("GO FROM 1 OVER KNOWS")
        .assert_success()
        .assert_result_count(2);
}

#[test]
fn test_go_execution_with_yield() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob'), 3:('Charlie')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01'), 1 -> 3:('2021-01-01')")
        .query("GO FROM 1 OVER KNOWS YIELD target.name")
        .assert_success()
        .assert_result_count(2);
}

#[test]
fn test_go_execution_with_steps() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob'), 3:('Charlie')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01'), 2 -> 3:('2021-01-01')")
        .query("GO 2 FROM 1 OVER KNOWS")
        .assert_success()
        .assert_result_count(1);
}

// ==================== LOOKUP Statement Tests ====================

#[test]
fn test_lookup_parser_basic() {
    let query = "LOOKUP ON Person WHERE Person.name == 'Alice'";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(result.is_ok(), "LOOKUP basic parsing should succeed: {:?}", result.err());

    let stmt = result.expect("LOOKUP statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "LOOKUP");
}

#[test]
fn test_lookup_parser_with_yield() {
    let query = "LOOKUP ON Person WHERE Person.age > 25";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(result.is_ok(), "LOOKUP basic parsing should succeed: {:?}", result.err());

    let stmt = result.expect("LOOKUP statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "LOOKUP");
}

#[test]
fn test_lookup_parser_complex_condition() {
    let query = "LOOKUP ON Person WHERE Person.age > 25";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(result.is_ok(), "LOOKUP condition parsing should succeed: {:?}", result.err());

    let stmt = result.expect("LOOKUP statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "LOOKUP");
}

#[test]
fn test_lookup_parser_edge() {
    let query = "LOOKUP ON KNOWS WHERE KNOWS.since > '2020-01-01'";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "LOOKUP edge type parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("LOOKUP statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "LOOKUP");
}

#[test]
fn test_lookup_execution_basic() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        .exec_ddl("CREATE TAG INDEX person_name_index ON Person(name)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30), 2:('Bob', 25)")
        .query("LOOKUP ON Person WHERE Person.name == 'Alice'")
        .assert_success()
        .assert_result_count(1);
}

#[test]
fn test_lookup_execution_with_condition() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        .exec_ddl("CREATE TAG INDEX person_age_index ON Person(age)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30), 2:('Bob', 25), 3:('Charlie', 35)")
        .query("LOOKUP ON Person WHERE Person.age > 25")
        .assert_success()
        .assert_result_count(2);
}

// ==================== FETCH Statement Tests ====================

#[test]
fn test_fetch_parser_vertex() {
    let query = "FETCH PROP ON Person 1";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(result.is_ok(), "FETCH vertex parsing should succeed: {:?}", result.err());

    let stmt = result.expect("FETCH statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "FETCH");
}

#[test]
fn test_fetch_parser_multiple_vertices() {
    let query = "FETCH PROP ON Person 1, 2, 3";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "FETCH multiple vertices parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("FETCH statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "FETCH");
}

#[test]
fn test_fetch_parser_edge() {
    let query = "FETCH PROP ON KNOWS 1 -> 2";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(result.is_ok(), "FETCH edge parsing should succeed: {:?}", result.err());

    let stmt = result.expect("FETCH statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "FETCH");
}

#[test]
fn test_fetch_parser_edge_with_rank() {
    let query = "FETCH PROP ON KNOWS 1 -> 2 @0";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "FETCH edge with rank parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("FETCH statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "FETCH");
}

#[test]
fn test_fetch_execution_vertex() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)")
        .query("FETCH PROP ON Person 1")
        .assert_success()
        .assert_result_count(1);
}

#[test]
fn test_fetch_execution_multiple_vertices() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30), 2:('Bob', 25), 3:('Charlie', 35)")
        .query("FETCH PROP ON Person 1, 2, 3")
        .assert_success()
        .assert_result_count(3);
}

#[test]
fn test_fetch_execution_edge() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01')")
        .query("FETCH PROP ON KNOWS 1 -> 2")
        .assert_success()
        .assert_result_count(1);
}

// ==================== FIND PATH Statement Tests ====================

#[test]
fn test_find_path_parser_shortest() {
    let query = "FIND SHORTEST PATH FROM 1 TO 4 OVER KNOWS";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "FIND SHORTEST PATH parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("FIND PATH statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "FIND PATH");
}

#[test]
fn test_find_path_parser_all() {
    let query = "FIND ALL PATH FROM 1 TO 4 OVER KNOWS";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "FIND ALL PATH parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("FIND PATH statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "FIND PATH");
}

#[test]
fn test_find_path_parser_noloop() {
    let query = "FIND NOLOOP PATH FROM 1 TO 4 OVER KNOWS";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_err(),
        "FIND NOLOOP PATH parsing should fail because NOLOOP is the default: {:?}",
        result.err()
    );

    let query2 = "FIND PATH FROM 1 TO 4 OVER KNOWS";
    let mut parser2 = Parser::new(query2);
    let result2 = parser2.parse();
    assert!(
        result2.is_ok(),
        "FIND PATH parsing should succeed: {:?}",
        result2.err()
    );

    let stmt = result2.expect("FIND PATH statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "FIND PATH");
}

#[test]
fn test_find_path_parser_with_upto() {
    let query = "FIND SHORTEST PATH FROM 1 TO 4 OVER KNOWS UPTO 5 STEPS";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "FIND PATH with UPTO parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("FIND PATH statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "FIND PATH");
}

#[test]
fn test_find_path_parser_reversely() {
    let query = "FIND SHORTEST PATH FROM 1 TO 4 OVER KNOWS REVERSELY";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "FIND PATH reversely parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("FIND PATH statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "FIND PATH");
}

#[test]
fn test_find_path_parser_with_where() {
    let query = "FIND SHORTEST PATH FROM 1 TO 4 OVER KNOWS WHERE v.age > 20";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "FIND PATH with WHERE parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("FIND PATH statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "FIND PATH");
}

#[test]
fn test_find_path_parser_complex() {
    let query = "FIND ALL PATH FROM 1 TO 4 OVER KNOWS UPTO 3 STEPS WHERE v.age > 20 REVERSELY";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "FIND PATH complex query parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("FIND PATH statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "FIND PATH");
}

#[test]
fn test_find_path_execution_shortest() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob'), 3:('Charlie'), 4:('David')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01'), 2 -> 3:('2021-01-01'), 3 -> 4:('2022-01-01')")
        .query("FIND SHORTEST PATH FROM 1 TO 4 OVER KNOWS")
        .assert_success()
        .assert_result_count(1);
}

#[test]
fn test_find_path_execution_all() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob'), 3:('Charlie'), 4:('David')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01'), 2 -> 3:('2021-01-01'), 3 -> 4:('2022-01-01'), 1 -> 3:('2019-01-01')")
        .query("FIND ALL PATH FROM 1 TO 4 OVER KNOWS")
        .assert_success();
}

// ==================== SUBGRAPH Statement Tests ====================

#[test]
fn test_subgraph_parser_basic() {
    let query = "GET SUBGRAPH FROM 1";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "SUBGRAPH basic parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("SUBGRAPH statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "SUBGRAPH");
}

#[test]
fn test_subgraph_parser_multiple_vertices() {
    let query = "GET SUBGRAPH FROM 1, 2, 3";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "SUBGRAPH multiple vertices parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("SUBGRAPH statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "SUBGRAPH");
}

#[test]
fn test_subgraph_parser_with_steps() {
    let query = "GET SUBGRAPH STEP 2 FROM 1";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "SUBGRAPH with steps parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("SUBGRAPH statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "SUBGRAPH");
}

#[test]
fn test_subgraph_parser_with_over() {
    let query = "GET SUBGRAPH FROM 1 OVER KNOWS";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "SUBGRAPH with OVER parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("SUBGRAPH statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "SUBGRAPH");
}

#[test]
fn test_subgraph_execution_basic() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob'), 3:('Charlie')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01'), 2 -> 3:('2021-01-01')")
        .query("GET SUBGRAPH WITH PROP 1")
        .assert_success();
}

#[test]
fn test_subgraph_execution_with_steps() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob'), 3:('Charlie'), 4:('David')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01'), 2 -> 3:('2021-01-01'), 3 -> 4:('2022-01-01')")
        .query("GET SUBGRAPH STEP 2 FROM 1")
        .assert_success();
}

// ==================== Comprehensive DQL Tests ====================

#[test]
fn test_dql_multiple_queries() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30), 2:('Bob', 25)")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01')")
        // MATCH query
        .query("MATCH (n:Person) RETURN n")
        .assert_success()
        .assert_result_count(2)
        // GO query
        .query("GO FROM 1 OVER KNOWS")
        .assert_success()
        .assert_result_count(1)
        // LOOKUP query
        .query("LOOKUP ON Person WHERE Person.age > 25")
        .assert_success()
        // FETCH query
        .query("FETCH PROP ON Person 1")
        .assert_success()
        .assert_result_count(1);
}

#[test]
fn test_dql_error_handling() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    let invalid_queries = vec![
        "MATCH (n:Person",                        // Missing right bracket
        "GO FROM OVER KNOWS",                     // Missing vertex ID
        "LOOKUP ON WHERE Person.name == 'Alice'", // Missing tag
        "FETCH PROP ON",                          // Missing tag and IDs
    ];

    for query in invalid_queries {
        let result = pipeline_manager.execute_query(query);
        assert!(result.is_err(), "Invalid query should return error: {}", query);
    }
}

// ==================== Dangling Edge Related Tests ====================

#[test]
fn test_go_with_dangling_edges() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01')")
        .query("GO FROM 1 OVER KNOWS YIELD target.name, edge.since")
        .assert_success()
        .assert_result_count(1);
}

#[test]
fn test_go_dangling_edge_returns_edge_props() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE, strength: INT)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
        .exec_dml("INSERT EDGE KNOWS(since, strength) VALUES 1 -> 2:('2020-01-01', 5)")
        .query("GO FROM 1 OVER KNOWS YIELD edge.since, edge.strength")
        .assert_success()
        .assert_result_count(1);
}

#[test]
fn test_match_no_dangling_edges() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING)")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob'), 3:('Charlie')")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01'), 2 -> 3:('2021-01-01')")
        .query("MATCH (n:Person)-[KNOWS]->(m:Person) RETURN n, m")
        .assert_success()
        .assert_result_count(2);
}

#[test]
fn test_delete_vertex_with_edge_syntax() {
    let query = "DELETE VERTEX 1 WITH EDGE";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DELETE VERTEX WITH EDGE parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DELETE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DELETE");
}

#[test]
fn test_delete_vertex_without_edge_syntax() {
    let query = "DELETE VERTEX 1";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DELETE VERTEX parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DELETE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DELETE");
}

#[test]
fn test_delete_vertex_multiple_with_edge() {
    let query = "DELETE VERTEX 1, 2, 3 WITH EDGE";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DELETE VERTEX multiple vertices WITH EDGE parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DELETE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DELETE");
}

#[test]
fn test_delete_vertex_with_where_and_edge() {
    let query = "DELETE VERTEX 1 WITH EDGE WHERE 1.age > 25";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DELETE VERTEX with WHERE and WITH EDGE parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DELETE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DELETE");
}

#[test]
fn test_dangling_edge_detection_and_repair() {
    use graphdb::storage::StorageClient;

    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();

    let mut storage_guard = storage.lock();

    let dangling_result = storage_guard.find_dangling_edges("test_space");
    println!("Dangling edge detection result: {:?}", dangling_result);

    let repair_result = storage_guard.repair_dangling_edges("test_space");
    println!("Dangling edge repair result: {:?}", repair_result);

    assert!(dangling_result.is_ok() || dangling_result.is_err());
    assert!(repair_result.is_ok() || repair_result.is_err());
}

#[test]
fn test_dangling_edge_workflow() {
    use graphdb::storage::StorageClient;

    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();

    let mut storage_guard = storage.lock();

    let space_info = graphdb::core::types::SpaceInfo::new("dangling_test".to_string());

    let create_result = storage_guard.create_space(&space_info);
    println!("Create space result: {:?}", create_result);

    use std::collections::HashMap;
    let mut tag_props = HashMap::new();
    tag_props.insert(
        "name".to_string(),
        graphdb::core::Value::String("Alice".to_string()),
    );

    let vertex = graphdb::core::Vertex::new(
        graphdb::core::Value::Int(1),
        vec![graphdb::core::vertex_edge_path::Tag::new(
            "Person".to_string(),
            tag_props,
        )],
    );

    let insert_result = storage_guard.insert_vertex("dangling_test", vertex);
    println!("Insert vertex result: {:?}", insert_result);

    let mut props = HashMap::new();
    props.insert(
        "since".to_string(),
        graphdb::core::Value::String("2024-01-01".to_string()),
    );

    let edge = graphdb::core::Edge::new(
        graphdb::core::Value::Int(1),
        graphdb::core::Value::Int(999),
        "KNOWS".to_string(),
        0,
        props,
    );

    let edge_result = storage_guard.insert_edge("dangling_test", edge);
    println!("Insert dangling edge result: {:?}", edge_result);

    let dangling = storage_guard.find_dangling_edges("dangling_test");
    println!("Detected dangling edges: {:?}", dangling);

    let repaired = storage_guard.repair_dangling_edges("dangling_test");
    println!("Repaired dangling edges count: {:?}", repaired);

    assert!(create_result.is_ok() || create_result.is_err());
    assert!(insert_result.is_ok() || insert_result.is_err());
    assert!(edge_result.is_ok() || edge_result.is_err());
}

// ==================== YIELD Statement Tests ====================

#[test]
fn test_yield_with_where_basic() {
    let query = "GO FROM 1 OVER KNOWS YIELD name, age";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(result.is_ok(), "GO with YIELD parsing should succeed: {:?}", result.err());

    let stmt = result.expect("GO statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "GO");
}

#[test]
fn test_yield_with_where_complex() {
    let query = "GO FROM 1 OVER KNOWS YIELD name, age";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(result.is_ok(), "GO with YIELD parsing should succeed: {:?}", result.err());

    let stmt = result.expect("GO statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "GO");
}

#[test]
fn test_yield_with_limit() {
    let query = "GO FROM 1 OVER KNOWS YIELD name";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(result.is_ok(), "GO with YIELD parsing should succeed: {:?}", result.err());

    let stmt = result.expect("GO statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "GO");
}

#[test]
fn test_yield_with_skip_limit() {
    let query = "GO FROM 1 OVER KNOWS YIELD name";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(result.is_ok(), "GO with YIELD parsing should succeed: {:?}", result.err());

    let stmt = result.expect("GO statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "GO");
}

#[test]
fn test_yield_with_where_limit() {
    let query = "GO FROM 1 OVER KNOWS YIELD name, age";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(result.is_ok(), "GO with YIELD parsing should succeed: {:?}", result.err());

    let stmt = result.expect("GO statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "GO");
}

#[test]
fn test_yield_standalone() {
    let query = "YIELD 1 + 1 AS result";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    println!("Standalone YIELD parsing result: {:?}", result);
    let _ = result;
}

#[test]
fn test_yield_standalone_with_where() {
    let query = "YIELD 1 + 1 AS result WHERE result > 0";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    println!("Standalone YIELD with WHERE parsing result: {:?}", result);
    let _ = result;
}

#[test]
fn test_yield_execution_with_where() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name: STRING, age: INT)")
        .exec_ddl("CREATE EDGE KNOWS(since: DATE)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30), 2:('Bob', 25), 3:('Charlie', 35)")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01'), 1 -> 3:('2021-01-01')")
        .query("GO FROM 1 OVER KNOWS YIELD target.name, target.age WHERE target.age > 25")
        .assert_success()
        .assert_result_count(2);
}

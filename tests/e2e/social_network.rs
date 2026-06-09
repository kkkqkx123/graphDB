//! E2E Test Suite for Social Network Scenario
//!
//! Tests basic graph operations including:
//! - Schema management
//! - Data insertion (vertices and edges)
//! - MATCH queries
//! - GO traversals
//! - LOOKUP queries
//! - Transaction management

use graphdb::core::stats::StatsManager;
use graphdb::query::optimizer::OptimizerEngine;
use graphdb::query::query_pipeline_manager::QueryPipelineManager;
use std::sync::Arc;

use crate::common::TestStorage;

/// Test basic connection and schema management
#[tokio::test]
async fn test_connect_and_show_spaces() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let schema_manager = test_storage.schema_manager();
    let stats_manager = Arc::new(StatsManager::new());
    let optimizer = Arc::new(OptimizerEngine::default());

    let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
        .with_schema_manager(schema_manager);

    // Execute SHOW SPACES
    let result = pipeline.execute_query("SHOW SPACES");
    assert!(result.is_ok(), "SHOW SPACES should succeed");
}

/// Test creating and using a space
#[tokio::test]
async fn test_create_and_use_space() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let schema_manager = test_storage.schema_manager();
    let stats_manager = Arc::new(StatsManager::new());
    let optimizer = Arc::new(OptimizerEngine::default());

    let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
        .with_schema_manager(schema_manager);

    // Create space
    let result = pipeline.execute_query("CREATE SPACE e2e_social_network (vid_type=STRING)");
    assert!(result.is_ok(), "CREATE SPACE should succeed");

    // Use space
    let result = pipeline.execute_query("USE e2e_social_network");
    assert!(result.is_ok(), "USE should succeed");
}

/// Test creating tags and edges
#[tokio::test]
async fn test_create_tags_and_edges() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let schema_manager = test_storage.schema_manager();
    let stats_manager = Arc::new(StatsManager::new());
    let optimizer = Arc::new(OptimizerEngine::default());

    let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
        .with_schema_manager(schema_manager);

    // Create space
    pipeline.execute_query("CREATE SPACE e2e_social_network_tags (vid_type=STRING)")
        .expect("CREATE SPACE should succeed");
    pipeline.execute_query("USE e2e_social_network_tags")
        .expect("USE should succeed");

    // Create person tag
    let result = pipeline.execute_query(
        "CREATE TAG IF NOT EXISTS person(name: STRING NOT NULL, age: INT, email: STRING, city: STRING)"
    );
    assert!(result.is_ok(), "CREATE TAG person should succeed");

    // Create company tag
    let result = pipeline.execute_query(
        "CREATE TAG IF NOT EXISTS company(name: STRING NOT NULL, industry: STRING)"
    );
    assert!(result.is_ok(), "CREATE TAG company should succeed");

    // Create friend edge
    let result = pipeline.execute_query("CREATE EDGE IF NOT EXISTS friend(degree: FLOAT)");
    assert!(result.is_ok(), "CREATE EDGE friend should succeed");

    // Create works_at edge
    let result = pipeline.execute_query("CREATE EDGE IF NOT EXISTS works_at(position: STRING)");
    assert!(result.is_ok(), "CREATE EDGE works_at should succeed");
}

/// Test showing tags
#[tokio::test]
async fn test_show_tags() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let schema_manager = test_storage.schema_manager();
    let stats_manager = Arc::new(StatsManager::new());
    let optimizer = Arc::new(OptimizerEngine::default());

    let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
        .with_schema_manager(schema_manager);

    // Create space and tags
    pipeline.execute_query("CREATE SPACE e2e_show_tags (vid_type=STRING)")
        .expect("CREATE SPACE should succeed");
    pipeline.execute_query("USE e2e_show_tags")
        .expect("USE should succeed");
    pipeline.execute_query("CREATE TAG person(name: STRING, age: INT)")
        .expect("CREATE TAG should succeed");

    // Show tags
    let result = pipeline.execute_query("SHOW TAGS");
    assert!(result.is_ok(), "SHOW TAGS should succeed");
}

/// Test showing edges
#[tokio::test]
async fn test_show_edges() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let schema_manager = test_storage.schema_manager();
    let stats_manager = Arc::new(StatsManager::new());
    let optimizer = Arc::new(OptimizerEngine::default());

    let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
        .with_schema_manager(schema_manager);

    // Create space and edges
    pipeline.execute_query("CREATE SPACE e2e_show_edges (vid_type=STRING)")
        .expect("CREATE SPACE should succeed");
    pipeline.execute_query("USE e2e_show_edges")
        .expect("USE should succeed");
    pipeline.execute_query("CREATE TAG person(name: STRING)")
        .expect("CREATE TAG should succeed");
    pipeline.execute_query("CREATE EDGE friend(degree: FLOAT)")
        .expect("CREATE EDGE should succeed");

    // Show edges
    let result = pipeline.execute_query("SHOW EDGES");
    assert!(result.is_ok(), "SHOW EDGES should succeed");
}

/// Test inserting vertex data
#[tokio::test]
async fn test_insert_vertex() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let schema_manager = test_storage.schema_manager();
    let stats_manager = Arc::new(StatsManager::new());
    let optimizer = Arc::new(OptimizerEngine::default());

    let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
        .with_schema_manager(schema_manager);

    // Setup
    pipeline.execute_query("CREATE SPACE e2e_insert_vertex (vid_type=STRING)")
        .expect("CREATE SPACE should succeed");
    pipeline.execute_query("USE e2e_insert_vertex")
        .expect("USE should succeed");
    pipeline.execute_query("CREATE TAG person(name: STRING NOT NULL, age: INT, email: STRING)")
        .expect("CREATE TAG should succeed");

    // Insert vertex
    let result = pipeline.execute_query(
        "INSERT VERTEX person(name, age, email) VALUES \"p1\": (\"Alice\", 30, \"alice@example.com\")"
    );
    assert!(result.is_ok(), "INSERT VERTEX should succeed");
}

/// Test inserting multiple vertices
#[tokio::test]
async fn test_insert_multiple_vertices() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let schema_manager = test_storage.schema_manager();
    let stats_manager = Arc::new(StatsManager::new());
    let optimizer = Arc::new(OptimizerEngine::default());

    let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
        .with_schema_manager(schema_manager);

    // Setup
    pipeline.execute_query("CREATE SPACE e2e_insert_multiple (vid_type=STRING)")
        .expect("CREATE SPACE should succeed");
    pipeline.execute_query("USE e2e_insert_multiple")
        .expect("USE should succeed");
    pipeline.execute_query("CREATE TAG person(name: STRING NOT NULL, age: INT)")
        .expect("CREATE TAG should succeed");

    // Insert multiple vertices
    let result = pipeline.execute_query(
        "INSERT VERTEX person(name, age) VALUES \"p1\": (\"Alice\", 30), \"p2\": (\"Bob\", 25)"
    );
    assert!(result.is_ok(), "INSERT VERTEX with multiple values should succeed");
}

/// Test inserting edge data
#[tokio::test]
async fn test_insert_edge() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let schema_manager = test_storage.schema_manager();
    let stats_manager = Arc::new(StatsManager::new());
    let optimizer = Arc::new(OptimizerEngine::default());

    let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
        .with_schema_manager(schema_manager);

    // Setup
    pipeline.execute_query("CREATE SPACE e2e_insert_edge (vid_type=STRING)")
        .expect("CREATE SPACE should succeed");
    pipeline.execute_query("USE e2e_insert_edge")
        .expect("USE should succeed");
    pipeline.execute_query("CREATE TAG person(name: STRING NOT NULL, age: INT)")
        .expect("CREATE TAG should succeed");
    pipeline.execute_query("CREATE EDGE friend(degree: FLOAT)")
        .expect("CREATE EDGE should succeed");

    // Insert vertices first
    pipeline.execute_query("INSERT VERTEX person(name, age) VALUES \"p1\": (\"Alice\", 30)")
        .expect("INSERT VERTEX should succeed");
    pipeline.execute_query("INSERT VERTEX person(name, age) VALUES \"p2\": (\"Bob\", 25)")
        .expect("INSERT VERTEX should succeed");

    // Insert edge
    let result = pipeline.execute_query(
        "INSERT EDGE friend(degree) VALUES \"p1\" -> \"p2\": (0.8)"
    );
    assert!(result.is_ok(), "INSERT EDGE should succeed");
}

/// Test fetching vertex properties
#[tokio::test]
async fn test_fetch_vertex() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let schema_manager = test_storage.schema_manager();
    let stats_manager = Arc::new(StatsManager::new());
    let optimizer = Arc::new(OptimizerEngine::default());

    let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
        .with_schema_manager(schema_manager);

    // Setup
    pipeline.execute_query("CREATE SPACE e2e_fetch_vertex (vid_type=STRING)")
        .expect("CREATE SPACE should succeed");
    pipeline.execute_query("USE e2e_fetch_vertex")
        .expect("USE should succeed");
    pipeline.execute_query("CREATE TAG person(name: STRING NOT NULL, age: INT, email: STRING)")
        .expect("CREATE TAG should succeed");

    // Insert vertex
    pipeline.execute_query(
        "INSERT VERTEX person(name, age, email) VALUES \"p_fetch\": (\"Alice\", 30, \"alice@test.com\")"
    ).expect("INSERT VERTEX should succeed");

    // Fetch vertex
    let result = pipeline.execute_query("FETCH PROP ON person \"p_fetch\"");
    assert!(result.is_ok(), "FETCH PROP should succeed");
}

/// Test fetching edge properties
#[tokio::test]
async fn test_fetch_edge() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let schema_manager = test_storage.schema_manager();
    let stats_manager = Arc::new(StatsManager::new());
    let optimizer = Arc::new(OptimizerEngine::default());

    let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
        .with_schema_manager(schema_manager);

    // Setup
    pipeline.execute_query("CREATE SPACE e2e_fetch_edge (vid_type=STRING)")
        .expect("CREATE SPACE should succeed");
    pipeline.execute_query("USE e2e_fetch_edge")
        .expect("USE should succeed");
    pipeline.execute_query("CREATE TAG person(name: STRING NOT NULL, age: INT)")
        .expect("CREATE TAG should succeed");
    pipeline.execute_query("CREATE EDGE friend(degree: FLOAT)")
        .expect("CREATE EDGE should succeed");

    // Insert vertices and edge
    pipeline.execute_query("INSERT VERTEX person(name, age) VALUES \"p1\": (\"Alice\", 30)")
        .expect("INSERT VERTEX should succeed");
    pipeline.execute_query("INSERT VERTEX person(name, age) VALUES \"p2\": (\"Bob\", 25)")
        .expect("INSERT VERTEX should succeed");
    pipeline.execute_query("INSERT EDGE friend(degree) VALUES \"p1\" -> \"p2\" @0: (0.8)")
        .expect("INSERT EDGE should succeed");

    // Fetch edge
    let result = pipeline.execute_query("FETCH PROP ON friend \"p1\" -> \"p2\"");
    assert!(result.is_ok(), "FETCH PROP ON EDGE should succeed");
}

/// Test basic MATCH query
#[tokio::test]
async fn test_match_basic() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let schema_manager = test_storage.schema_manager();
    let stats_manager = Arc::new(StatsManager::new());
    let optimizer = Arc::new(OptimizerEngine::default());

    let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
        .with_schema_manager(schema_manager);

    // Setup
    pipeline.execute_query("CREATE SPACE e2e_match_basic (vid_type=STRING)")
        .expect("CREATE SPACE should succeed");
    pipeline.execute_query("USE e2e_match_basic")
        .expect("USE should succeed");
    pipeline.execute_query("CREATE TAG person(name: STRING NOT NULL, age: INT, city: STRING)")
        .expect("CREATE TAG should succeed");
    pipeline.execute_query("CREATE EDGE friend(degree: FLOAT)")
        .expect("CREATE EDGE should succeed");

    // Insert data
    pipeline.execute_query(
        "INSERT VERTEX person(name, age, city) VALUES \"p1\": (\"Alice\", 30, \"Beijing\")"
    ).expect("INSERT VERTEX should succeed");
    pipeline.execute_query(
        "INSERT VERTEX person(name, age, city) VALUES \"p2\": (\"Bob\", 25, \"Shanghai\")"
    ).expect("INSERT VERTEX should succeed");

    // Match query
    let result = pipeline.execute_query("MATCH (p:person) RETURN p.name, p.age");
    assert!(result.is_ok(), "MATCH should succeed");
}

/// Test MATCH with filter
#[tokio::test]
async fn test_match_with_filter() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let schema_manager = test_storage.schema_manager();
    let stats_manager = Arc::new(StatsManager::new());
    let optimizer = Arc::new(OptimizerEngine::default());

    let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
        .with_schema_manager(schema_manager);

    // Setup
    pipeline.execute_query("CREATE SPACE e2e_match_filter (vid_type=STRING)")
        .expect("CREATE SPACE should succeed");
    pipeline.execute_query("USE e2e_match_filter")
        .expect("USE should succeed");
    pipeline.execute_query("CREATE TAG person(name: STRING NOT NULL, age: INT)")
        .expect("CREATE TAG should succeed");

    // Insert data
    pipeline.execute_query(
        "INSERT VERTEX person(name, age) VALUES \"p1\": (\"Alice\", 30), \"p2\": (\"Bob\", 25)"
    ).expect("INSERT VERTEX should succeed");

    // Match with filter
    let result = pipeline.execute_query("MATCH (p:person) WHERE p.age > 28 RETURN p.name");
    assert!(result.is_ok(), "MATCH with filter should succeed");
}

/// Test MATCH path query
#[tokio::test]
async fn test_match_path() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let schema_manager = test_storage.schema_manager();
    let stats_manager = Arc::new(StatsManager::new());
    let optimizer = Arc::new(OptimizerEngine::default());

    let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
        .with_schema_manager(schema_manager);

    // Setup
    pipeline.execute_query("CREATE SPACE e2e_match_path (vid_type=STRING)")
        .expect("CREATE SPACE should succeed");
    pipeline.execute_query("USE e2e_match_path")
        .expect("USE should succeed");
    pipeline.execute_query("CREATE TAG person(name: STRING NOT NULL, age: INT)")
        .expect("CREATE TAG should succeed");
    pipeline.execute_query("CREATE EDGE friend(degree: FLOAT)")
        .expect("CREATE EDGE should succeed");

    // Insert data
    pipeline.execute_query("INSERT VERTEX person(name, age) VALUES \"p1\": (\"Alice\", 30)")
        .expect("INSERT VERTEX should succeed");
    pipeline.execute_query("INSERT VERTEX person(name, age) VALUES \"p2\": (\"Bob\", 25)")
        .expect("INSERT VERTEX should succeed");
    pipeline.execute_query("INSERT EDGE friend(degree) VALUES \"p1\" -> \"p2\": (0.8)")
        .expect("INSERT EDGE should succeed");

    // Match path
    let result = pipeline.execute_query(
        "MATCH (p:person)-[:friend]->(f:person) RETURN p.name, f.name"
    );
    assert!(result.is_ok(), "MATCH path should succeed");
}

/// Test GO traversal
#[tokio::test]
async fn test_go_traversal() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let schema_manager = test_storage.schema_manager();
    let stats_manager = Arc::new(StatsManager::new());
    let optimizer = Arc::new(OptimizerEngine::default());

    let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
        .with_schema_manager(schema_manager);

    // Setup
    pipeline.execute_query("CREATE SPACE e2e_go_traversal (vid_type=STRING)")
        .expect("CREATE SPACE should succeed");
    pipeline.execute_query("USE e2e_go_traversal")
        .expect("USE should succeed");
    pipeline.execute_query("CREATE TAG person(name: STRING NOT NULL, age: INT)")
        .expect("CREATE TAG should succeed");
    pipeline.execute_query("CREATE EDGE friend(degree: FLOAT)")
        .expect("CREATE EDGE should succeed");

    // Insert data
    pipeline.execute_query("INSERT VERTEX person(name, age) VALUES \"p1\": (\"Alice\", 30)")
        .expect("INSERT VERTEX should succeed");
    pipeline.execute_query("INSERT VERTEX person(name, age) VALUES \"p2\": (\"Bob\", 25)")
        .expect("INSERT VERTEX should succeed");
    pipeline.execute_query("INSERT EDGE friend(degree) VALUES \"p1\" -> \"p2\": (0.8)")
        .expect("INSERT EDGE should succeed");

    // GO traversal
    let result = pipeline.execute_query("GO 1 STEP FROM \"p1\" OVER friend YIELD friend.name");
    assert!(result.is_ok(), "GO traversal should succeed");
}

/// Test GO multi-step traversal
#[tokio::test]
async fn test_go_multiple_steps() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let schema_manager = test_storage.schema_manager();
    let stats_manager = Arc::new(StatsManager::new());
    let optimizer = Arc::new(OptimizerEngine::default());

    let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
        .with_schema_manager(schema_manager);

    // Setup
    pipeline.execute_query("CREATE SPACE e2e_go_multi (vid_type=STRING)")
        .expect("CREATE SPACE should succeed");
    pipeline.execute_query("USE e2e_go_multi")
        .expect("USE should succeed");
    pipeline.execute_query("CREATE TAG person(name: STRING NOT NULL, age: INT)")
        .expect("CREATE TAG should succeed");
    pipeline.execute_query("CREATE EDGE friend(degree: FLOAT)")
        .expect("CREATE EDGE should succeed");

    // Insert data
    pipeline.execute_query("INSERT VERTEX person(name, age) VALUES \"p1\": (\"Alice\", 30)")
        .expect("INSERT VERTEX should succeed");
    pipeline.execute_query("INSERT VERTEX person(name, age) VALUES \"p2\": (\"Bob\", 25)")
        .expect("INSERT VERTEX should succeed");
    pipeline.execute_query("INSERT VERTEX person(name, age) VALUES \"p3\": (\"Charlie\", 35)")
        .expect("INSERT VERTEX should succeed");
    pipeline.execute_query("INSERT EDGE friend(degree) VALUES \"p1\" -> \"p2\": (0.8)")
        .expect("INSERT EDGE should succeed");
    pipeline.execute_query("INSERT EDGE friend(degree) VALUES \"p2\" -> \"p3\": (0.7)")
        .expect("INSERT EDGE should succeed");

    // GO multi-step
    let result = pipeline.execute_query("GO 2 STEPS FROM \"p1\" OVER friend YIELD friend.name");
    assert!(result.is_ok(), "GO multi-step should succeed");
}

/// Test LOOKUP index query
#[tokio::test]
async fn test_lookup_index() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let schema_manager = test_storage.schema_manager();
    let stats_manager = Arc::new(StatsManager::new());
    let optimizer = Arc::new(OptimizerEngine::default());

    let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
        .with_schema_manager(schema_manager);

    // Setup
    pipeline.execute_query("CREATE SPACE e2e_lookup (vid_type=STRING)")
        .expect("CREATE SPACE should succeed");
    pipeline.execute_query("USE e2e_lookup")
        .expect("USE should succeed");
    pipeline.execute_query("CREATE TAG person(name: STRING NOT NULL, age: INT)")
        .expect("CREATE TAG should succeed");
    pipeline.execute_query("CREATE TAG INDEX idx_person_name ON person(name)")
        .expect("CREATE INDEX should succeed");

    // Insert data
    pipeline.execute_query("INSERT VERTEX person(name, age) VALUES \"p1\": (\"Alice\", 30)")
        .expect("INSERT VERTEX should succeed");

    // LOOKUP
    let result = pipeline.execute_query(
        "LOOKUP ON person WHERE person.name == \"Alice\" YIELD person.age"
    );
    assert!(result.is_ok(), "LOOKUP should succeed");
}

/// Test EXPLAIN command
#[tokio::test]
async fn test_explain_basic() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let schema_manager = test_storage.schema_manager();
    let stats_manager = Arc::new(StatsManager::new());
    let optimizer = Arc::new(OptimizerEngine::default());

    let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
        .with_schema_manager(schema_manager);

    // Setup
    pipeline.execute_query("CREATE SPACE e2e_explain (vid_type=STRING)")
        .expect("CREATE SPACE should succeed");
    pipeline.execute_query("USE e2e_explain")
        .expect("USE should succeed");
    pipeline.execute_query("CREATE TAG person(name: STRING, age: INT)")
        .expect("CREATE TAG should succeed");

    // EXPLAIN
    let result = pipeline.execute_query("EXPLAIN MATCH (p:person) RETURN p.name");
    assert!(result.is_ok(), "EXPLAIN should succeed");
}

/// Test PROFILE command
#[tokio::test]
async fn test_profile_query() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let schema_manager = test_storage.schema_manager();
    let stats_manager = Arc::new(StatsManager::new());
    let optimizer = Arc::new(OptimizerEngine::default());

    let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
        .with_schema_manager(schema_manager);

    // Setup
    pipeline.execute_query("CREATE SPACE e2e_profile (vid_type=STRING)")
        .expect("CREATE SPACE should succeed");
    pipeline.execute_query("USE e2e_profile")
        .expect("USE should succeed");
    pipeline.execute_query("CREATE TAG person(name: STRING, age: INT)")
        .expect("CREATE TAG should succeed");

    // Insert data
    pipeline.execute_query("INSERT VERTEX person(name, age) VALUES \"p1\": (\"Alice\", 30)")
        .expect("INSERT VERTEX should succeed");

    // PROFILE
    let result = pipeline.execute_query("PROFILE MATCH (p:person) RETURN count(p)");
    assert!(result.is_ok(), "PROFILE should succeed");
}

/// Test transaction commit
#[tokio::test]
async fn test_transaction_commit() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let schema_manager = test_storage.schema_manager();
    let stats_manager = Arc::new(StatsManager::new());
    let optimizer = Arc::new(OptimizerEngine::default());

    let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
        .with_schema_manager(schema_manager);

    // Setup
    pipeline.execute_query("CREATE SPACE e2e_tx_commit (vid_type=STRING)")
        .expect("CREATE SPACE should succeed");
    pipeline.execute_query("USE e2e_tx_commit")
        .expect("USE should succeed");
    pipeline.execute_query("CREATE TAG person(name: STRING, age: INT)")
        .expect("CREATE TAG should succeed");

    // Begin transaction
    let result = pipeline.execute_query("BEGIN");
    assert!(result.is_ok(), "BEGIN should succeed");

    // Insert data
    let result = pipeline.execute_query(
        "INSERT VERTEX person(name, age) VALUES \"tx1\": (\"TX_Test\", 20)"
    );
    assert!(result.is_ok(), "INSERT should succeed");

    // Commit
    let result = pipeline.execute_query("COMMIT");
    assert!(result.is_ok(), "COMMIT should succeed");
}

/// Test transaction rollback
#[tokio::test]
async fn test_transaction_rollback() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let schema_manager = test_storage.schema_manager();
    let stats_manager = Arc::new(StatsManager::new());
    let optimizer = Arc::new(OptimizerEngine::default());

    let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
        .with_schema_manager(schema_manager);

    // Setup
    pipeline.execute_query("CREATE SPACE e2e_tx_rollback (vid_type=STRING)")
        .expect("CREATE SPACE should succeed");
    pipeline.execute_query("USE e2e_tx_rollback")
        .expect("USE should succeed");
    pipeline.execute_query("CREATE TAG person(name: STRING, age: INT)")
        .expect("CREATE TAG should succeed");

    // Begin transaction
    let result = pipeline.execute_query("BEGIN");
    assert!(result.is_ok(), "BEGIN should succeed");

    // Insert data
    let result = pipeline.execute_query(
        "INSERT VERTEX person(name, age) VALUES \"tx2\": (\"Rollback\", 25)"
    );
    assert!(result.is_ok(), "INSERT should succeed");

    // Rollback
    let result = pipeline.execute_query("ROLLBACK");
    assert!(result.is_ok(), "ROLLBACK should succeed");
}

/// Cleanup test spaces
#[tokio::test]
async fn test_cleanup_spaces() {
    let spaces = [
        "e2e_social_network",
        "e2e_social_network_tags",
        "e2e_show_tags",
        "e2e_show_edges",
        "e2e_insert_vertex",
        "e2e_insert_multiple",
        "e2e_insert_edge",
        "e2e_fetch_vertex",
        "e2e_fetch_edge",
        "e2e_match_basic",
        "e2e_match_filter",
        "e2e_match_path",
        "e2e_go_traversal",
        "e2e_go_multi",
        "e2e_lookup",
        "e2e_explain",
        "e2e_profile",
        "e2e_tx_commit",
        "e2e_tx_rollback",
    ];

    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let schema_manager = test_storage.schema_manager();
    let stats_manager = Arc::new(StatsManager::new());
    let optimizer = Arc::new(OptimizerEngine::default());

    let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
        .with_schema_manager(schema_manager);

    for space in &spaces {
        let _ = pipeline.execute_query(&format!("DROP SPACE IF EXISTS {}", space));
    }
}

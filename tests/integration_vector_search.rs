//! Vector Search Integration Tests
//!
//! Test scope:
//! - VectorManager lifecycle operations (create, drop, search)
//! - VectorSyncCoordinator integration with graph data
//! - Qdrant engine functionality
//! - Vector synchronization with vertex operations
//! - Batch operations and filtering
//!
//! Requirements:
//! - Qdrant service must be running on localhost:6333 (HTTP) and localhost:6334 (gRPC)

mod common;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use graphdb::core::vertex_edge_path::Tag;
use graphdb::core::{Value, Vertex};
use graphdb::sync::vector_sync::VectorSyncCoordinator;
use vector_client::types::{
    CollectionConfig, FilterCondition, SearchQuery, VectorFilter, VectorPoint,
};
use vector_client::{DistanceMetric, VectorClientConfig, VectorManager};

// ==================== Test Fixtures ====================

struct VectorTestContext {
    coordinator: Arc<VectorSyncCoordinator>,
    manager: Arc<VectorManager>,
    collection_prefix: String,
}

impl VectorTestContext {
    async fn with_qdrant_engine(test_name: &str) -> Self {
        // Create manager with Qdrant engine
        let vector_config = VectorClientConfig::qdrant_local("localhost", 6334, 6333);

        let manager = Arc::new(
            VectorManager::new(vector_config)
                .await
                .expect("Failed to create manager"),
        );
        let coordinator = Arc::new(VectorSyncCoordinator::new(manager.clone(), None));

        // Use test name as collection prefix for isolation
        let collection_prefix = format!("test_{}", test_name);

        Self {
            coordinator,
            manager,
            collection_prefix,
        }
    }

    async fn with_mock_engine(test_name: &str) -> Self {
        // Create manager with disabled config for mock tests
        let vector_config = VectorClientConfig::disabled();

        let manager = Arc::new(
            VectorManager::new(vector_config)
                .await
                .expect("Failed to create manager"),
        );
        let coordinator = Arc::new(VectorSyncCoordinator::new(manager.clone(), None));

        // Use test name as collection prefix for isolation
        let collection_prefix = format!("test_mock_{}", test_name);

        Self {
            coordinator,
            manager,
            collection_prefix,
        }
    }

    fn collection_name(&self, name: &str) -> String {
        format!("{}_{}", self.collection_prefix, name)
    }
}

fn create_test_vector(size: usize, offset: f32) -> Vec<f32> {
    (0..size)
        .map(|i| (i as f32 + offset) / size as f32)
        .collect::<Vec<f32>>()
}

fn create_test_vertex_with_vector(
    vid: i64,
    tag_name: &str,
    field_name: &str,
    vector: Vec<f32>,
) -> Vertex {
    let mut props = HashMap::new();
    let list_values: Vec<Value> = vector.iter().map(|&v| Value::Float(v as f64)).collect();
    props.insert(
        field_name.to_string(),
        Value::List(Box::new(graphdb::core::List {
            values: list_values,
        })),
    );
    let tag = Tag::new(tag_name.to_string(), props);
    Vertex::new(Value::Int(vid), vec![tag])
}

// ==================== VectorManager Basic Tests ====================

#[tokio::test]
async fn test_vector_manager_create_index() {
    let ctx = VectorTestContext::with_qdrant_engine("create_index").await;

    let config = CollectionConfig::new(3, DistanceMetric::Cosine);
    let result = ctx
        .manager
        .create_index(&ctx.collection_name("test"), config)
        .await;

    assert!(result.is_ok(), "Creating index should succeed");
}

#[tokio::test]
async fn test_vector_manager_create_duplicate_index() {
    let ctx = VectorTestContext::with_mock_engine("create_duplicate_index").await;

    let config = CollectionConfig::new(3, DistanceMetric::Cosine);
    ctx.manager
        .create_index("test_collection", config.clone())
        .await
        .expect("First creation should succeed");

    let result = ctx.manager.create_index("test_collection", config).await;

    assert!(result.is_err(), "Creating duplicate index should fail");
}

#[tokio::test]
async fn test_vector_manager_drop_index() {
    let ctx = VectorTestContext::with_mock_engine("drop_index").await;

    let config = CollectionConfig::new(3, DistanceMetric::Cosine);
    ctx.manager
        .create_index("test_collection", config)
        .await
        .expect("Creating index should succeed");

    let result = ctx.manager.drop_index("test_collection").await;
    assert!(result.is_ok(), "Dropping index should succeed");
}

#[tokio::test]
async fn test_vector_manager_metadata() {
    let ctx = VectorTestContext::with_mock_engine("metadata").await;

    let config = CollectionConfig::new(3, DistanceMetric::Cosine);
    ctx.manager
        .create_index("test_collection", config)
        .await
        .expect("Creating index should succeed");

    let metadata = ctx.manager.get_index_metadata("test_collection");
    assert!(metadata.is_some(), "Metadata should exist");

    let exists = ctx.manager.index_exists("test_collection");
    assert!(exists, "Index should exist");

    let not_exists = ctx.manager.index_exists("non_existent");
    assert!(!not_exists, "Non-existent index should not exist");
}

// ==================== VectorSyncCoordinator Tests ====================

#[tokio::test]
async fn test_vector_coordinator_create_index() {
    let ctx = VectorTestContext::with_mock_engine("create_index").await;

    let result = ctx
        .coordinator
        .create_vector_index(1, "Document", "embedding", 3, DistanceMetric::Cosine)
        .await;

    assert!(
        result.is_ok(),
        "Creating index via coordinator should succeed"
    );
    let collection_name = result.unwrap();
    assert!(collection_name.contains("1_Document_embedding"));
}

#[tokio::test]
async fn test_vector_coordinator_drop_index() {
    let ctx = VectorTestContext::with_mock_engine("drop_index").await;

    ctx.coordinator
        .create_vector_index(1, "Document", "embedding", 3, DistanceMetric::Cosine)
        .await
        .expect("Creating index should succeed");

    let result = ctx
        .coordinator
        .drop_vector_index(1, "Document", "embedding")
        .await;

    assert!(
        result.is_ok(),
        "Dropping index via coordinator should succeed"
    );
}

// ==================== Vector Search Tests ====================

#[tokio::test]
async fn test_vector_search_basic() {
    let ctx = VectorTestContext::with_mock_engine("search_basic").await;

    // Create index
    ctx.coordinator
        .create_vector_index(1, "Document", "embedding", 3, DistanceMetric::Cosine)
        .await
        .expect("Failed to create index");

    // Give mock engine time to create collection
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Insert vectors
    let vector1 = create_test_vector(3, 0.1);
    let point1 = VectorPoint::new("1", vector1.clone());
    ctx.coordinator
        .vector_manager()
        .upsert("space_1_Document_embedding", point1)
        .await
        .expect("Failed to upsert vector");

    let query_vector: Vec<f32> = create_test_vector(3, 0.1);
    let search_query = SearchQuery::new(query_vector, 10);

    let results = ctx
        .coordinator
        .search("space_1_Document_embedding", search_query)
        .await
        .expect("Failed to search");

    assert!(!results.is_empty(), "Search should return results");
}

#[tokio::test]
#[ignore = "MockEngine does not fully support filtering"]
async fn test_vector_search_with_filter() {
    let ctx = VectorTestContext::with_mock_engine("search_with_filter").await;

    // Create index
    ctx.coordinator
        .create_vector_index(1, "Document", "embedding", 3, DistanceMetric::Cosine)
        .await
        .expect("Failed to create index");

    // Give mock engine time to create collection
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Insert vectors with payload
    let vector1 = create_test_vector(3, 0.1);
    let mut payload1 = HashMap::new();
    payload1.insert("category".to_string(), serde_json::json!("A"));
    let point1 = VectorPoint::new("1", vector1).with_payload(payload1);

    let vector2 = create_test_vector(3, 0.2);
    let mut payload2 = HashMap::new();
    payload2.insert("category".to_string(), serde_json::json!("B"));
    let point2 = VectorPoint::new("2", vector2).with_payload(payload2);

    ctx.coordinator
        .vector_manager()
        .upsert("space_1_Document_embedding", point1)
        .await
        .expect("Failed to upsert vector 1");
    ctx.coordinator
        .vector_manager()
        .upsert("space_1_Document_embedding", point2)
        .await
        .expect("Failed to upsert vector 2");

    // Search with filter
    let query_vector: Vec<f32> = create_test_vector(3, 0.1);
    let filter = VectorFilter::new().must(FilterCondition::match_value("category", "A"));
    let search_query = SearchQuery::new(query_vector, 10).with_filter(filter);

    let results = ctx
        .coordinator
        .search("space_1_Document_embedding", search_query)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 1, "Should return only matching result");
    assert_eq!(results[0].id, "1");
}

// ==================== Vertex Synchronization Tests ====================

#[tokio::test]
async fn test_vertex_insert_with_vector() {
    let ctx = VectorTestContext::with_mock_engine("vertex_insert").await;

    // Create index
    ctx.coordinator
        .create_vector_index(1, "Document", "embedding", 3, DistanceMetric::Cosine)
        .await
        .expect("Failed to create index");

    // Give mock engine time to create collection
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Insert vertex with vector
    let vector = create_test_vector(3, 0.5);
    let vertex = create_test_vertex_with_vector(1, "Document", "embedding", vector.clone());

    ctx.coordinator
        .on_vertex_inserted(1, &vertex)
        .await
        .expect("Failed to insert vertex");

    // Verify vector is searchable
    let query_vector: Vec<f32> = create_test_vector(3, 0.5);
    let search_query = SearchQuery::new(query_vector, 10);

    let results = ctx
        .coordinator
        .search("space_1_Document_embedding", search_query)
        .await
        .expect("Failed to search");

    assert!(!results.is_empty(), "Should find the inserted vector");
}

#[tokio::test]
async fn test_vertex_delete_with_vector() {
    let ctx = VectorTestContext::with_mock_engine("vertex_delete").await;

    // Create index
    ctx.coordinator
        .create_vector_index(1, "Document", "embedding", 3, DistanceMetric::Cosine)
        .await
        .expect("Failed to create index");

    // Insert vertex with vector
    let vector = create_test_vector(3, 0.5);
    let vertex = create_test_vertex_with_vector(1, "Document", "embedding", vector);

    ctx.coordinator
        .on_vertex_inserted(1, &vertex)
        .await
        .expect("Failed to insert vertex");

    // Delete vertex
    ctx.coordinator
        .on_vertex_deleted(1, "Document", &vertex.vid)
        .await
        .expect("Failed to delete vertex");

    // Verify vector is not searchable
    let query_vector: Vec<f32> = create_test_vector(3, 0.5);
    let search_query = SearchQuery::new(query_vector, 10);

    let results = ctx
        .coordinator
        .search("space_1_Document_embedding", search_query)
        .await
        .expect("Failed to search");

    assert!(results.is_empty(), "Should not find the deleted vector");
}

// ==================== Batch Operations Tests ====================

#[tokio::test]
async fn test_batch_upsert() {
    let ctx = VectorTestContext::with_mock_engine("batch_upsert").await;

    // Create index
    ctx.coordinator
        .create_vector_index(1, "Document", "embedding", 3, DistanceMetric::Cosine)
        .await
        .expect("Failed to create index");

    // Give mock engine time to create collection
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Batch insert vectors
    let points: Vec<VectorPoint> = (0..5)
        .map(|i| {
            let vector = create_test_vector(3, i as f32 * 0.1);
            VectorPoint::new(i.to_string(), vector)
        })
        .collect();

    ctx.coordinator
        .vector_manager()
        .upsert_batch("space_1_Document_embedding", points)
        .await
        .expect("Failed to batch upsert");

    // Verify all vectors are searchable
    let query_vector: Vec<f32> = create_test_vector(3, 0.0);
    let search_query = SearchQuery::new(query_vector, 10);

    let results = ctx
        .coordinator
        .search("space_1_Document_embedding", search_query)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 5, "Should find all inserted vectors");
}

#[tokio::test]
async fn test_batch_delete() {
    let ctx = VectorTestContext::with_mock_engine("batch_delete").await;

    // Create index
    ctx.coordinator
        .create_vector_index(1, "Document", "embedding", 3, DistanceMetric::Cosine)
        .await
        .expect("Failed to create index");

    // Batch insert vectors
    let points: Vec<VectorPoint> = (0..5)
        .map(|i| {
            let vector = create_test_vector(3, i as f32 * 0.1);
            VectorPoint::new(i.to_string(), vector)
        })
        .collect();

    ctx.coordinator
        .vector_manager()
        .upsert_batch("space_1_Document_embedding", points)
        .await
        .expect("Failed to batch upsert");

    // Batch delete vectors
    let ids_to_delete: Vec<&str> = vec!["0", "1"];
    ctx.coordinator
        .vector_manager()
        .delete_batch("space_1_Document_embedding", ids_to_delete)
        .await
        .expect("Failed to batch delete");

    // Verify remaining vectors
    let query_vector: Vec<f32> = create_test_vector(3, 0.0);
    let search_query = SearchQuery::new(query_vector, 10);

    let results = ctx
        .coordinator
        .search("space_1_Document_embedding", search_query)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 3, "Should find remaining vectors");
}

// ==================== Health Check Tests ====================

#[tokio::test]
async fn test_health_check() {
    let ctx = VectorTestContext::with_mock_engine("health_check").await;

    let health = ctx
        .coordinator
        .vector_manager()
        .engine()
        .health_check()
        .await
        .expect("Failed to perform health check");

    assert!(health.is_healthy, "Mock engine should be healthy");
}

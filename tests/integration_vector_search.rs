//! Vector Search Integration Tests
//!
//! Test scope:
//! - VectorIndexManager lifecycle operations (create, drop, search)
//! - VectorCoordinator integration with graph data
//! - Mock engine functionality
//! - Vector synchronization with vertex operations
//! - Batch operations and filtering

mod common;

use std::collections::HashMap;
use std::sync::Arc;

use graphdb::core::vertex_edge_path::Tag;
use graphdb::core::{Value, Vertex};
use graphdb::vector::config::{VectorConfig, VectorDistance, VectorIndexConfig};
use graphdb::vector::coordinator::VectorCoordinator;
use graphdb::vector::manager::VectorIndexManager;

use vector_client::types::{FilterCondition, VectorFilter, VectorPoint};

// ==================== Test Fixtures ====================

struct VectorTestContext {
    coordinator: Arc<VectorCoordinator>,
    manager: Arc<VectorIndexManager>,
}

impl VectorTestContext {
    async fn with_mock_engine() -> Self {
        // Create manager with disabled vector search (uses MockEngine)
        let mut vector_config = VectorConfig::default();
        vector_config.enabled = false;  // Disabled = uses MockEngine

        let manager = Arc::new(
            VectorIndexManager::new(vector_config.clone())
                .await
                .expect("Failed to create manager"),
        );
        let coordinator = Arc::new(VectorCoordinator::new(manager.clone()));

        Self {
            coordinator,
            manager,
        }
    }
}

fn create_test_vector(size: usize, offset: f32) -> Vec<f32> {
    (0..size).map(|i| (i as f32 + offset) / size as f32).collect()
}

fn create_test_vertex_with_vector(
    vid: i64,
    tag_name: &str,
    field_name: &str,
    vector: Vec<f32>,
) -> Vertex {
    let mut props = HashMap::new();
    let list_values: Vec<Value> = vector.iter().map(|&v| Value::Float(v as f64)).collect();
    props.insert(field_name.to_string(), Value::List(graphdb::core::List { values: list_values }));
    let tag = Tag::new(tag_name.to_string(), props);
    Vertex::new(Value::Int(vid), vec![tag])
}

// ==================== VectorIndexManager Basic Tests ====================

#[tokio::test]
async fn test_vector_index_manager_create_index() {
    let ctx = VectorTestContext::with_mock_engine().await;

    let result = ctx
        .manager
        .create_index(
            1,
            "Document",
            "embedding",
            Some(VectorIndexConfig {
                vector_size: 3,
                distance: VectorDistance::Cosine,
                hnsw: None,
                quantization: None,
            }),
        )
        .await;

    assert!(result.is_ok(), "Creating index should succeed");
    let collection_name = result.unwrap();
    assert!(collection_name.contains("1_Document_embedding"));
}

#[tokio::test]
async fn test_vector_index_manager_create_duplicate_index() {
    let ctx = VectorTestContext::with_mock_engine().await;

    ctx.manager
        .create_index(
            1,
            "Document",
            "embedding",
            Some(VectorIndexConfig {
                vector_size: 3,
                distance: VectorDistance::Cosine,
                hnsw: None,
                quantization: None,
            }),
        )
        .await
        .expect("First creation should succeed");

    let result = ctx
        .manager
        .create_index(
            1,
            "Document",
            "embedding",
            Some(VectorIndexConfig {
                vector_size: 3,
                distance: VectorDistance::Cosine,
                hnsw: None,
                quantization: None,
            }),
        )
        .await;

    assert!(
        result.is_err(),
        "Creating duplicate index should fail"
    );
}

#[tokio::test]
async fn test_vector_index_manager_drop_index() {
    let ctx = VectorTestContext::with_mock_engine().await;

    ctx.manager
        .create_index(
            1,
            "Document",
            "embedding",
            Some(VectorIndexConfig {
                vector_size: 3,
                distance: VectorDistance::Cosine,
                hnsw: None,
                quantization: None,
            }),
        )
        .await
        .expect("Creating index should succeed");

    let result = ctx.manager.drop_index(1, "Document", "embedding").await;
    assert!(result.is_ok(), "Dropping index should succeed");
}

#[tokio::test]
async fn test_vector_index_manager_metadata() {
    let ctx = VectorTestContext::with_mock_engine().await;

    ctx.manager
        .create_index(
            1,
            "Document",
            "embedding",
            Some(VectorIndexConfig {
                vector_size: 3,
                distance: VectorDistance::Cosine,
                hnsw: None,
                quantization: None,
            }),
        )
        .await
        .expect("Creating index should succeed");

    let metadata = ctx.manager.get_metadata(1, "Document", "embedding");
    assert!(metadata.is_some(), "Metadata should exist");

    let exists = ctx.manager.index_exists(1, "Document", "embedding");
    assert!(exists, "Index should exist");

    let not_exists = ctx.manager.index_exists(1, "NonExistent", "field");
    assert!(!not_exists, "Non-existent index should not exist");
}

// ==================== VectorCoordinator Tests ====================

#[tokio::test]
async fn test_vector_coordinator_create_index() {
    let ctx = VectorTestContext::with_mock_engine().await;

    let result = ctx
        .coordinator
        .create_vector_index(1, "Document", "embedding", 3, VectorDistance::Cosine)
        .await;

    assert!(result.is_ok(), "Creating index via coordinator should succeed");
    let index_name = result.unwrap();
    assert!(index_name.contains("1_Document_embedding"));
}

#[tokio::test]
async fn test_vector_coordinator_drop_index() {
    let ctx = VectorTestContext::with_mock_engine().await;

    ctx.coordinator
        .create_vector_index(1, "Document", "embedding", 3, VectorDistance::Cosine)
        .await
        .expect("Creating index should succeed");

    let result = ctx
        .coordinator
        .drop_vector_index(1, "Document", "embedding")
        .await;

    assert!(result.is_ok(), "Dropping index via coordinator should succeed");
}

#[tokio::test]
async fn test_vector_coordinator_list_indexes() {
    let ctx = VectorTestContext::with_mock_engine().await;

    ctx.coordinator
        .create_vector_index(1, "Document", "embedding", 3, VectorDistance::Cosine)
        .await
        .expect("Creating first index should succeed");

    ctx.coordinator
        .create_vector_index(1, "Article", "content_vector", 3, VectorDistance::Cosine)
        .await
        .expect("Creating second index should succeed");

    let indexes = ctx.coordinator.list_indexes();
    assert_eq!(indexes.len(), 2, "Should have 2 indexes");
}

// ==================== Vector Search Tests ====================

#[tokio::test]
async fn test_vector_search_basic() {
    let ctx = VectorTestContext::with_mock_engine().await;

    ctx.coordinator
        .create_vector_index(1, "Document", "embedding", 3, VectorDistance::Cosine)
        .await
        .expect("Creating index should succeed");

    let vector1 = create_test_vector(3, 0.0);
    let vector2 = create_test_vector(3, 1.0);
    let vector3 = create_test_vector(3, 2.0);

    let point1 = VectorPoint::new("1".to_string(), vector1.clone());
    let point2 = VectorPoint::new("2".to_string(), vector2.clone());
    let point3 = VectorPoint::new("3".to_string(), vector3.clone());

    ctx.coordinator
        .upsert_batch(1, "Document", "embedding", vec![point1, point2, point3])
        .await
        .expect("Upserting points should succeed");

    let query_vector = vector1;
    let results = ctx
        .coordinator
        .search(1, "Document", "embedding", query_vector, 10)
        .await
        .expect("Searching should succeed");

    assert!(!results.is_empty(), "Should return search results");
    assert!(results.len() <= 10, "Should respect limit");

    for result in &results {
        assert!(!result.id.is_empty(), "Result should have ID");
        assert!(result.score >= 0.0 && result.score <= 1.0, "Score should be in [0, 1]");
    }
}

#[tokio::test]
async fn test_vector_search_with_threshold() {
    let ctx = VectorTestContext::with_mock_engine().await;

    ctx.coordinator
        .create_vector_index(1, "Document", "embedding", 3, VectorDistance::Cosine)
        .await
        .expect("Creating index should succeed");

    let vector1 = create_test_vector(3, 0.0);
    let vector2 = create_test_vector(3, 10.0);

    let point1 = VectorPoint::new("1".to_string(), vector1.clone());
    let point2 = VectorPoint::new("2".to_string(), vector2.clone());

    ctx.coordinator
        .upsert_batch(1, "Document", "embedding", vec![point1, point2])
        .await
        .expect("Upserting points should succeed");

    let results = ctx
        .coordinator
        .search_with_threshold(1, "Document", "embedding", vector1, 10, 0.9)
        .await
        .expect("Searching with threshold should succeed");

    assert!(!results.is_empty(), "Should return at least one similar result");

    for result in &results {
        assert!(result.score >= 0.9, "Score should meet threshold");
    }
}

#[tokio::test]
async fn test_vector_search_with_filter() {
    let ctx = VectorTestContext::with_mock_engine().await;

    ctx.coordinator
        .create_vector_index(1, "Document", "embedding", 3, VectorDistance::Cosine)
        .await
        .expect("Creating index should succeed");

    let vector1 = create_test_vector(3, 0.0);
    let vector2 = create_test_vector(3, 1.0);

    let mut payload1 = HashMap::new();
    payload1.insert(
        "category".to_string(),
        serde_json::Value::String("tech".to_string()),
    );
    let mut payload2 = HashMap::new();
    payload2.insert(
        "category".to_string(),
        serde_json::Value::String("science".to_string()),
    );

    let point1 = VectorPoint::new("1".to_string(), vector1.clone()).with_payload(payload1);
    let point2 = VectorPoint::new("2".to_string(), vector2.clone()).with_payload(payload2);

    ctx.coordinator
        .upsert_batch(1, "Document", "embedding", vec![point1, point2])
        .await
        .expect("Upserting points should succeed");

    let filter = VectorFilter::new().must(FilterCondition::match_value("category", "tech"));
    let results = ctx
        .coordinator
        .search_with_filter(1, "Document", "embedding", vector1, 10, filter)
        .await
        .expect("Searching with filter should succeed");

    assert!(!results.is_empty(), "Should return filtered results");
}

// ==================== Vertex Operations Integration Tests ====================

#[tokio::test]
async fn test_coordinator_on_vertex_inserted() {
    let ctx = VectorTestContext::with_mock_engine().await;

    ctx.coordinator
        .create_vector_index(1, "Document", "embedding", 3, VectorDistance::Cosine)
        .await
        .expect("Creating index should succeed");

    let vector = create_test_vector(3, 0.5);
    let vertex = create_test_vertex_with_vector(1, "Document", "embedding", vector.clone());

    let result = ctx.coordinator.on_vertex_inserted(1, &vertex).await;
    assert!(result.is_ok(), "Processing vertex insertion should succeed");

    let search_results = ctx
        .coordinator
        .search(1, "Document", "embedding", vector, 10)
        .await
        .expect("Searching should succeed");

    assert!(
        search_results.iter().any(|r| r.id == "1"),
        "Should find inserted vertex"
    );
}

#[tokio::test]
async fn test_coordinator_on_vertex_updated() {
    let ctx = VectorTestContext::with_mock_engine().await;

    ctx.coordinator
        .create_vector_index(1, "Document", "embedding", 3, VectorDistance::Cosine)
        .await
        .expect("Creating index should succeed");

    let old_vector = create_test_vector(3, 0.0);
    let vertex_old = create_test_vertex_with_vector(1, "Document", "embedding", old_vector.clone());

    ctx.coordinator
        .on_vertex_inserted(1, &vertex_old)
        .await
        .expect("Inserting vertex should succeed");

    let new_vector = create_test_vector(3, 1.0);
    let vertex_new = create_test_vertex_with_vector(1, "Document", "embedding", new_vector.clone());

    let changed_fields = vec!["embedding".to_string()];
    let result = ctx
        .coordinator
        .on_vertex_updated(1, &vertex_new, &changed_fields)
        .await;

    assert!(result.is_ok(), "Processing vertex update should succeed");

    let search_results = ctx
        .coordinator
        .search(1, "Document", "embedding", new_vector, 10)
        .await
        .expect("Searching should succeed");

    assert!(
        search_results.iter().any(|r| r.id == "1"),
        "Should find updated vertex"
    );
}

#[tokio::test]
async fn test_coordinator_on_vertex_deleted() {
    let ctx = VectorTestContext::with_mock_engine().await;

    ctx.coordinator
        .create_vector_index(1, "Document", "embedding", 3, VectorDistance::Cosine)
        .await
        .expect("Creating index should succeed");

    let vector = create_test_vector(3, 0.5);
    let vertex = create_test_vertex_with_vector(1, "Document", "embedding", vector.clone());

    ctx.coordinator
        .on_vertex_inserted(1, &vertex)
        .await
        .expect("Inserting vertex should succeed");

    let result = ctx
        .coordinator
        .on_vertex_deleted(1, "Document", &Value::Int(1))
        .await;

    assert!(result.is_ok(), "Processing vertex deletion should succeed");

    let search_results = ctx
        .coordinator
        .search(1, "Document", "embedding", vector, 10)
        .await
        .expect("Searching should succeed");

    assert!(
        !search_results.iter().any(|r| r.id == "1"),
        "Should not find deleted vertex"
    );
}

// ==================== Batch Operations Tests ====================

#[tokio::test]
async fn test_vector_batch_upsert() {
    let ctx = VectorTestContext::with_mock_engine().await;

    ctx.coordinator
        .create_vector_index(1, "Document", "embedding", 3, VectorDistance::Cosine)
        .await
        .expect("Creating index should succeed");

    let points: Vec<VectorPoint> = (0..10)
        .map(|i| {
            let vector = create_test_vector(3, i as f32);
            VectorPoint::new(i.to_string(), vector)
        })
        .collect();

    let result = ctx
        .coordinator
        .upsert_batch(1, "Document", "embedding", points)
        .await;

    assert!(result.is_ok(), "Batch upsert should succeed");
}

#[tokio::test]
async fn test_vector_batch_delete() {
    let ctx = VectorTestContext::with_mock_engine().await;

    ctx.coordinator
        .create_vector_index(1, "Document", "embedding", 3, VectorDistance::Cosine)
        .await
        .expect("Creating index should succeed");

    let points: Vec<VectorPoint> = (0..5)
        .map(|i| {
            let vector = create_test_vector(3, i as f32);
            VectorPoint::new(i.to_string(), vector)
        })
        .collect();

    ctx.coordinator
        .upsert_batch(1, "Document", "embedding", points)
        .await
        .expect("Batch upsert should succeed");

    let point_ids: Vec<&str> = vec!["0", "1", "2"];
    let result = ctx
        .coordinator
        .delete_batch(1, "Document", "embedding", point_ids)
        .await;

    assert!(result.is_ok(), "Batch delete should succeed");

    let remaining = ctx
        .coordinator
        .search(1, "Document", "embedding", create_test_vector(3, 0.0), 10)
        .await
        .expect("Searching should succeed");

    assert!(remaining.len() <= 2, "Should have at most 2 remaining points");
}

// ==================== Health Check Tests ====================

#[tokio::test]
async fn test_vector_health_check() {
    let ctx = VectorTestContext::with_mock_engine().await;

    let health = ctx.coordinator.health_check().await;
    assert!(health.is_ok(), "Health check should succeed");
    assert!(health.unwrap(), "Mock engine should be healthy");
}

// ==================== Multiple Indexes Tests ====================

#[tokio::test]
async fn test_multiple_indexes_independent() {
    let ctx = VectorTestContext::with_mock_engine().await;

    ctx.coordinator
        .create_vector_index(1, "Document", "embedding", 3, VectorDistance::Cosine)
        .await
        .expect("Creating first index should succeed");

    ctx.coordinator
        .create_vector_index(1, "Article", "content_vector", 3, VectorDistance::Euclid)
        .await
        .expect("Creating second index should succeed");

    let doc_vector = create_test_vector(3, 0.0);
    let article_vector = create_test_vector(3, 1.0);

    let doc_point = VectorPoint::new("doc1".to_string(), doc_vector.clone());
    let article_point = VectorPoint::new("article1".to_string(), article_vector.clone());

    ctx.coordinator
        .upsert_batch(1, "Document", "embedding", vec![doc_point])
        .await
        .expect("Upserting to first index should succeed");

    ctx.coordinator
        .upsert_batch(1, "Article", "content_vector", vec![article_point])
        .await
        .expect("Upserting to second index should succeed");

    let doc_results = ctx
        .coordinator
        .search(1, "Document", "embedding", doc_vector, 10)
        .await
        .expect("Searching first index should succeed");

    let article_results = ctx
        .coordinator
        .search(1, "Article", "content_vector", article_vector, 10)
        .await
        .expect("Searching second index should succeed");

    assert_eq!(doc_results.len(), 1, "First index should have 1 result");
    assert_eq!(article_results.len(), 1, "Second index should have 1 result");
    assert_eq!(doc_results[0].id, "doc1");
    assert_eq!(article_results[0].id, "article1");
}

// ==================== Distance Metrics Tests ====================

#[tokio::test]
async fn test_distance_metrics_cosine() {
    let ctx = VectorTestContext::with_mock_engine().await;

    ctx.coordinator
        .create_vector_index(1, "Document", "embedding", 3, VectorDistance::Cosine)
        .await
        .expect("Creating index should succeed");

    let identical_vector = vec![1.0, 0.0, 0.0];
    let orthogonal_vector = vec![0.0, 1.0, 0.0];

    let point1 = VectorPoint::new("1".to_string(), identical_vector.clone());
    let point2 = VectorPoint::new("2".to_string(), orthogonal_vector.clone());

    ctx.coordinator
        .upsert_batch(1, "Document", "embedding", vec![point1, point2])
        .await
        .expect("Upserting points should succeed");

    let results = ctx
        .coordinator
        .search(1, "Document", "embedding", identical_vector, 10)
        .await
        .expect("Searching should succeed");

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].id, "1");

    if results.len() > 1 {
        assert!(results[0].score > results[1].score, "Identical vector should have higher score");
    }
}

#[tokio::test]
async fn test_distance_metrics_euclidean() {
    let ctx = VectorTestContext::with_mock_engine().await;

    ctx.coordinator
        .create_vector_index(1, "Document", "embedding", 3, VectorDistance::Euclid)
        .await
        .expect("Creating index should succeed");

    let close_vector = vec![1.0, 1.0, 1.0];
    let far_vector = vec![10.0, 10.0, 10.0];

    let point1 = VectorPoint::new("1".to_string(), close_vector.clone());
    let point2 = VectorPoint::new("2".to_string(), far_vector.clone());

    ctx.coordinator
        .upsert_batch(1, "Document", "embedding", vec![point1, point2])
        .await
        .expect("Upserting points should succeed");

    let results = ctx
        .coordinator
        .search(1, "Document", "embedding", close_vector, 10)
        .await
        .expect("Searching should succeed");

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].id, "1", "Close vector should be first");
}

// ==================== Edge Cases Tests ====================

#[tokio::test]
async fn test_search_empty_index() {
    let ctx = VectorTestContext::with_mock_engine().await;

    ctx.coordinator
        .create_vector_index(1, "Document", "embedding", 3, VectorDistance::Cosine)
        .await
        .expect("Creating index should succeed");

    let query_vector = create_test_vector(3, 0.0);
    let results = ctx
        .coordinator
        .search(1, "Document", "embedding", query_vector, 10)
        .await
        .expect("Searching empty index should succeed");

    assert_eq!(results.len(), 0, "Empty index should return no results");
}

#[tokio::test]
async fn test_search_nonexistent_index() {
    let ctx = VectorTestContext::with_mock_engine().await;

    let query_vector = create_test_vector(3, 0.0);
    let results = ctx
        .coordinator
        .search(1, "NonExistent", "field", query_vector, 10)
        .await;

    assert!(results.is_err(), "Searching non-existent index should fail");
}

#[tokio::test]
async fn test_upsert_nonexistent_index() {
    let ctx = VectorTestContext::with_mock_engine().await;

    let point = VectorPoint::new("1".to_string(), vec![1.0, 2.0, 3.0]);
    let result = ctx
        .coordinator
        .upsert_batch(1, "NonExistent", "field", vec![point])
        .await;

    assert!(result.is_err(), "Upserting to non-existent index should fail");
}

#[tokio::test]
async fn test_vector_dimension_mismatch() {
    let ctx = VectorTestContext::with_mock_engine().await;

    ctx.coordinator
        .create_vector_index(1, "Document", "embedding", 3, VectorDistance::Cosine)
        .await
        .expect("Creating index should succeed");

    // Note: Mock engine doesn't validate vector dimensions
    // This test documents the behavior for future reference
    let wrong_dimension_vector = vec![1.0, 2.0];

    let point = VectorPoint::new("1".to_string(), wrong_dimension_vector);
    let result = ctx
        .coordinator
        .upsert_batch(1, "Document", "embedding", vec![point])
        .await;

    // Mock engine accepts any vector dimension (for testing flexibility)
    // Real Qdrant engine would validate dimensions
    assert!(result.is_ok(), "Mock engine accepts any dimension");
}

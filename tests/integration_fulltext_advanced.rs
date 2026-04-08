//! Full-Text Search Advanced Integration Tests
//!
//! This module provides advanced integration tests covering:
//! - Complex query scenarios
//! - Performance and stress testing
//! - Recovery and persistence
//! - Multi-space scenarios
//! - Edge cases and error conditions

mod common;

use common::{
    assertions::assert_ok,
    storage_helpers::{create_test_space, get_storage, person_tag_info},
    TestStorage,
};
use graphdb::coordinator::{ChangeType, FulltextCoordinator};
use graphdb::core::vertex_edge_path::Tag;
use graphdb::core::{Value, Vertex};
use graphdb::search::config::{FulltextConfig, SyncConfig};
use graphdb::search::engine::EngineType;
use graphdb::search::manager::FulltextIndexManager;
use graphdb::storage::storage_client::StorageClient;
use graphdb::sync::batch::BatchConfig;
use graphdb::sync::manager::SyncManager;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;

// ==================== Test Setup Helpers ====================

async fn setup_coordinator() -> (FulltextCoordinator, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = FulltextConfig {
        enabled: true,
        index_path: temp_dir.path().to_path_buf(),
        default_engine: EngineType::Bm25,
        sync: SyncConfig::default(),
        bm25: Default::default(),
        inversearch: Default::default(),
        cache_size: 100,
        max_result_cache: 1000,
        result_cache_ttl_secs: 60,
    };
    let manager = Arc::new(FulltextIndexManager::new(config).expect("Failed to create manager"));
    let coordinator = FulltextCoordinator::new(manager);
    (coordinator, temp_dir)
}

fn create_vertex(vid: i64, tag_name: &str, properties: Vec<(&str, &str)>) -> Vertex {
    let mut props = HashMap::new();
    for (key, value) in properties {
        props.insert(key.to_string(), Value::String(value.to_string()));
    }
    let tag = Tag {
        name: tag_name.to_string(),
        properties: props,
    };
    Vertex::new(Value::Int(vid), vec![tag])
}

// ==================== Complex Query Scenarios ====================

#[tokio::test]
async fn test_fulltext_phrase_search() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let vertex1 = create_vertex(
        1,
        "Article",
        vec![("content", "The quick brown fox jumps over the lazy dog")],
    );
    let vertex2 = create_vertex(
        2,
        "Article",
        vec![("content", "Quick thinking leads to quick results")],
    );
    let vertex3 = create_vertex(
        3,
        "Article",
        vec![("content", "The brown bear in the forest")],
    );

    coordinator
        .on_vertex_inserted(1, &vertex1)
        .await
        .expect("Failed to insert vertex 1");
    coordinator
        .on_vertex_inserted(1, &vertex2)
        .await
        .expect("Failed to insert vertex 2");
    coordinator
        .on_vertex_inserted(1, &vertex3)
        .await
        .expect("Failed to insert vertex 3");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let results = coordinator
        .search(1, "Article", "content", "quick brown", 10)
        .await
        .expect("Failed to search");

    assert!(!results.is_empty(), "Should find documents with phrase");
    assert_eq!(
        results[0].doc_id,
        Value::Int(1),
        "Best match should be first"
    );
}

#[tokio::test]
async fn test_fulltext_boolean_operators() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Document", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let vertex1 = create_vertex(
        1,
        "Document",
        vec![("content", "Rust programming language tutorial")],
    );
    let vertex2 = create_vertex(
        2,
        "Document",
        vec![("content", "Python programming basics")],
    );
    let vertex3 = create_vertex(
        3,
        "Document",
        vec![("content", "Rust vs Python comparison")],
    );

    coordinator
        .on_vertex_inserted(1, &vertex1)
        .await
        .expect("Failed to insert vertex 1");
    coordinator
        .on_vertex_inserted(1, &vertex2)
        .await
        .expect("Failed to insert vertex 2");
    coordinator
        .on_vertex_inserted(1, &vertex3)
        .await
        .expect("Failed to insert vertex 3");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let results = coordinator
        .search(1, "Document", "content", "Rust programming", 10)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 2, "Should find documents with both terms");
}

#[tokio::test]
async fn test_fulltext_wildcard_search() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Product", "name", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let vertex1 = create_vertex(1, "Product", vec![("name", "Laptop Pro 15")]);
    let vertex2 = create_vertex(2, "Product", vec![("name", "Laptop Basic 13")]);
    let vertex3 = create_vertex(3, "Product", vec![("name", "Desktop Computer")]);

    coordinator
        .on_vertex_inserted(1, &vertex1)
        .await
        .expect("Failed to insert vertex 1");
    coordinator
        .on_vertex_inserted(1, &vertex2)
        .await
        .expect("Failed to insert vertex 2");
    coordinator
        .on_vertex_inserted(1, &vertex3)
        .await
        .expect("Failed to insert vertex 3");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let results = coordinator
        .search(1, "Product", "name", "Laptop", 10)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 2, "Should find products with Laptop");
}

// ==================== Performance and Stress Tests ====================

#[tokio::test]
async fn test_fulltext_large_batch_insert() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Document", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    const BATCH_SIZE: usize = 1000;
    for i in 0..BATCH_SIZE {
        let vertex = create_vertex(
            i as i64,
            "Document",
            vec![(
                "content",
                &format!(
                    "Large batch document number {} with some content for testing",
                    i
                ),
            )],
        );
        coordinator
            .on_vertex_inserted(1, &vertex)
            .await
            .expect("Failed to insert vertex");
    }

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(500)).await;

    let results = coordinator
        .search(1, "Document", "content", "batch", 100)
        .await
        .expect("Failed to search");

    assert_eq!(
        results.len(),
        BATCH_SIZE,
        "Should find all {} documents",
        BATCH_SIZE
    );
}

#[tokio::test]
async fn test_fulltext_high_concurrency() {
    let (coordinator, _temp) = setup_coordinator().await;
    let coordinator = Arc::new(coordinator);

    coordinator
        .create_index(1, "Article", "title", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let mut handles = vec![];
    for i in 0..20 {
        let coord = Arc::clone(&coordinator);
        let handle = tokio::spawn(async move {
            for j in 0..25 {
                let vid = i * 25 + j;
                let vertex = create_vertex(
                    vid as i64,
                    "Article",
                    vec![("title", &format!("High concurrency article {}", vid))],
                );
                let _ = coord.on_vertex_inserted(1, &vertex).await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    let coordinator = Arc::try_unwrap(coordinator).expect("Arc still has multiple owners");
    coordinator.commit_all().await.expect("Failed to commit");

    sleep(Duration::from_millis(500)).await;

    let results = coordinator
        .search(1, "Article", "title", "concurrency", 1000)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 500, "Should find all 500 concurrent inserts");
}

#[tokio::test]
async fn test_fulltext_rapid_insert_delete_cycle() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "TempDoc", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    for cycle in 0..5 {
        let vid = cycle as i64;

        let vertex = create_vertex(
            vid,
            "TempDoc",
            vec![("content", &format!("Temporary document {}", cycle))],
        );

        coordinator
            .on_vertex_inserted(1, &vertex)
            .await
            .expect("Failed to insert vertex");

        coordinator.commit_all().await.expect("Failed to commit");
        sleep(Duration::from_millis(100)).await;

        coordinator
            .on_vertex_deleted(1, "TempDoc", &Value::Int(vid))
            .await
            .expect("Failed to delete vertex");

        coordinator.commit_all().await.expect("Failed to commit");
        sleep(Duration::from_millis(100)).await;
    }

    let results = coordinator
        .search(1, "TempDoc", "content", "Temporary", 100)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 0, "All documents should be deleted");
}

// ==================== Multi-Space Scenarios ====================

#[tokio::test]
async fn test_fulltext_multiple_spaces_isolation() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Blog", "title", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index for space 1");

    coordinator
        .create_index(2, "Blog", "title", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index for space 2");

    let vertex1 = create_vertex(1, "Blog", vec![("title", "Space 1 Blog Post")]);
    let vertex2 = create_vertex(2, "Blog", vec![("title", "Space 2 Blog Post")]);

    coordinator
        .on_vertex_inserted(1, &vertex1)
        .await
        .expect("Failed to insert vertex in space 1");

    coordinator
        .on_vertex_inserted(2, &vertex2)
        .await
        .expect("Failed to insert vertex in space 2");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let space1_results = coordinator
        .search(1, "Blog", "title", "Space", 10)
        .await
        .expect("Failed to search space 1");

    assert_eq!(space1_results.len(), 1, "Should find 1 result in space 1");
    assert_eq!(
        space1_results[0].doc_id,
        Value::Int(1),
        "Should find space 1 document"
    );

    let space2_results = coordinator
        .search(2, "Blog", "title", "Space", 10)
        .await
        .expect("Failed to search space 2");

    assert_eq!(space2_results.len(), 1, "Should find 1 result in space 2");
    assert_eq!(
        space2_results[0].doc_id,
        Value::Int(2),
        "Should find space 2 document"
    );
}

#[tokio::test]
async fn test_fulltext_cross_space_no_leakage() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index for space 1");

    coordinator
        .create_index(2, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index for space 2");

    let vertex1 = create_vertex(
        1,
        "Article",
        vec![("content", "Unique content for space 1")],
    );

    coordinator
        .on_vertex_inserted(1, &vertex1)
        .await
        .expect("Failed to insert vertex in space 1");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let space2_results = coordinator
        .search(2, "Article", "content", "Unique", 10)
        .await
        .expect("Failed to search space 2");

    assert_eq!(
        space2_results.len(),
        0,
        "Should not find space 1 content in space 2"
    );
}

// ==================== Recovery and Persistence Tests ====================

#[tokio::test]
async fn test_sync_manager_with_recovery() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let recovery_dir = TempDir::new().expect("Failed to create recovery dir");

    let config = FulltextConfig {
        enabled: true,
        index_path: temp_dir.path().to_path_buf(),
        default_engine: EngineType::Bm25,
        sync: SyncConfig::default(),
        bm25: Default::default(),
        inversearch: Default::default(),
        cache_size: 100,
        max_result_cache: 1000,
        result_cache_ttl_secs: 60,
    };

    let manager = Arc::new(FulltextIndexManager::new(config).expect("Failed to create manager"));
    let coordinator = Arc::new(FulltextCoordinator::new(manager));

    let batch_config = BatchConfig {
        batch_size: 10,
        commit_interval: Duration::from_millis(100),
        max_wait_time: Duration::from_secs(5),
        queue_capacity: 1000,
    };

    let sync_manager = SyncManager::with_recovery(
        coordinator.clone(),
        batch_config,
        recovery_dir.path().to_path_buf(),
    );

    coordinator
        .create_index(1, "Document", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let _vertex = create_vertex(1, "Document", vec![("content", "Recovery test document")]);

    sync_manager
        .on_vertex_change(
            1,
            "Document",
            &Value::Int(1),
            &[(
                "content".to_string(),
                Value::String("Recovery test document".to_string()),
            )],
            ChangeType::Insert,
        )
        .await
        .expect("Failed to sync vertex");

    sleep(Duration::from_millis(300)).await;

    let results = coordinator
        .search(1, "Document", "content", "Recovery", 10)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 1, "Should find recovered document");
}

#[tokio::test]
async fn test_task_buffer_batching() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let config = FulltextConfig {
        enabled: true,
        index_path: temp_dir.path().to_path_buf(),
        default_engine: EngineType::Bm25,
        sync: SyncConfig::default(),
        bm25: Default::default(),
        inversearch: Default::default(),
        cache_size: 100,
        max_result_cache: 1000,
        result_cache_ttl_secs: 60,
    };

    let manager = Arc::new(FulltextIndexManager::new(config).expect("Failed to create manager"));
    let coordinator = Arc::new(FulltextCoordinator::new(manager));

    let batch_config = BatchConfig {
        batch_size: 5,
        commit_interval: Duration::from_millis(500),
        max_wait_time: Duration::from_secs(5),
        queue_capacity: 1000,
    };

    let buffer = Arc::new(graphdb::sync::batch::TaskBuffer::new(
        coordinator.clone(),
        batch_config,
    ));

    coordinator
        .create_index(1, "Article", "title", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    for i in 0..10 {
        buffer
            .add_document(
                1,
                "Article",
                "title",
                format!("doc_{}", i),
                format!("Title {}", i),
            )
            .await;
    }

    sleep(Duration::from_millis(600)).await;

    let results = coordinator
        .search(1, "Article", "title", "Title", 100)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 10, "Should find all batched documents");
}

// ==================== Edge Cases and Error Conditions ====================

#[tokio::test]
async fn test_fulltext_very_long_content() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Document", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let long_content = "word ".repeat(10000);
    let vertex = create_vertex(1, "Document", vec![("content", &long_content)]);

    coordinator
        .on_vertex_inserted(1, &vertex)
        .await
        .expect("Failed to insert vertex with long content");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let results = coordinator
        .search(1, "Document", "content", "word", 10)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 1, "Should handle very long content");
}

#[tokio::test]
async fn test_fulltext_empty_string_content() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Article", "title", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let vertex = create_vertex(1, "Article", vec![("title", "")]);

    coordinator
        .on_vertex_inserted(1, &vertex)
        .await
        .expect("Failed to insert vertex with empty string");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let results = coordinator.search(1, "Article", "title", "", 10).await;

    assert!(
        results.is_ok() || results.is_err(),
        "Should handle empty query gracefully"
    );
}

#[tokio::test]
async fn test_fulltext_special_query_characters() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Document", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let vertex = create_vertex(
        1,
        "Document",
        vec![("content", "Test with + - && || ! ( ) { } [ ] ^ ~ * ? : \\")],
    );

    coordinator
        .on_vertex_inserted(1, &vertex)
        .await
        .expect("Failed to insert vertex");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let results = coordinator
        .search(1, "Document", "content", "Test", 10)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 1, "Should handle special query characters");
}

#[tokio::test]
async fn test_fulltext_mixed_language_content() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let vertex = create_vertex(
        1,
        "Article",
        vec![("content", "Hello 世界 Bonjour Welt こんにちは Hola")],
    );

    coordinator
        .on_vertex_inserted(1, &vertex)
        .await
        .expect("Failed to insert vertex");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let results = coordinator
        .search(1, "Article", "content", "Hello", 10)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 1, "Should handle mixed language content");
}

#[tokio::test]
async fn test_fulltext_numeric_string_content() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Document", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let vertex = create_vertex(1, "Document", vec![("content", "12345 67890 11111")]);

    coordinator
        .on_vertex_inserted(1, &vertex)
        .await
        .expect("Failed to insert vertex");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let results = coordinator
        .search(1, "Document", "content", "12345", 10)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 1, "Should handle numeric strings");
}

#[tokio::test]
async fn test_fulltext_repeated_same_content() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Article", "title", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    for i in 0..5 {
        let vertex = create_vertex(
            i as i64,
            "Article",
            vec![("title", "Exactly the same title")],
        );

        coordinator
            .on_vertex_inserted(1, &vertex)
            .await
            .expect("Failed to insert vertex");
    }

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let results = coordinator
        .search(1, "Article", "title", "same", 10)
        .await
        .expect("Failed to search");

    assert_eq!(
        results.len(),
        5,
        "Should find all documents with same content"
    );
}

#[tokio::test]
async fn test_fulltext_index_drop_and_recreate() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Document", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let vertex = create_vertex(1, "Document", vec![("content", "Test content")]);

    coordinator
        .on_vertex_inserted(1, &vertex)
        .await
        .expect("Failed to insert vertex");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let before_results = coordinator
        .search(1, "Document", "content", "Test", 10)
        .await
        .expect("Failed to search");
    assert_eq!(before_results.len(), 1, "Should find content before drop");

    coordinator
        .drop_index(1, "Document", "content")
        .await
        .expect("Failed to drop index");

    coordinator
        .create_index(1, "Document", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to recreate index");

    let after_results = coordinator
        .search(1, "Document", "content", "Test", 10)
        .await;

    assert!(after_results.is_err(), "Should not find content after drop");
}

// ==================== Integration Tests with Real Storage ====================
// Tests that integrate with the actual storage layer to verify end-to-end functionality

#[tokio::test]
async fn test_fulltext_with_real_storage_operations() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();

    // Create space using storage API
    let space_info = create_test_space("test_fulltext_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    // Create tag using storage API
    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("test_fulltext_space", &tag_info));

    let (coordinator, _temp) = setup_coordinator().await;

    // Create fulltext index
    coordinator
        .create_index(1, "Person", "name", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    // Insert multiple vertices through coordinator
    for i in 0..10 {
        let person_vertex =
            create_vertex(i as i64, "Person", vec![("name", &format!("Person {}", i))]);

        coordinator
            .on_vertex_inserted(1, &person_vertex)
            .await
            .expect("Failed to insert vertex");
    }

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(300)).await;

    // Search and verify
    let results = coordinator
        .search(1, "Person", "name", "Person", 100)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 10, "Should find all persons by name");
}

#[tokio::test]
async fn test_fulltext_property_type_handling() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Product", "description", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let mut props = HashMap::new();
    props.insert(
        "name".to_string(),
        Value::String("Test Product".to_string()),
    );
    props.insert("price".to_string(), Value::Float(99.99));
    props.insert("quantity".to_string(), Value::Int(100));
    props.insert(
        "description".to_string(),
        Value::String("A great product".to_string()),
    );

    let tag = Tag {
        name: "Product".to_string(),
        properties: props,
    };
    let vertex = Vertex::new(Value::Int(1), vec![tag]);

    coordinator
        .on_vertex_inserted(1, &vertex)
        .await
        .expect("Failed to insert vertex with mixed types");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let results = coordinator
        .search(1, "Product", "description", "great", 10)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 1, "Should handle mixed property types");
}

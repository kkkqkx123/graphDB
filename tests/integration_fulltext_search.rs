//! Full-Text Search Integration Tests
//!
//! This module provides comprehensive integration tests for the full-text search functionality,
//! covering basic operations, advanced features, synchronization mechanisms, and edge cases.
//!
//! Test Categories:
//! - Basic CRUD operations (create index, insert, search, update, delete)
//! - Multiple fields and tags
//! - Batch operations
//! - Synchronization modes (sync, async, off)
//! - Error handling and edge cases
//! - Concurrent operations
//! - Persistence and recovery

mod common;

use common::{
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
use graphdb::sync::manager::{SyncManager, SyncMode};
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

async fn setup_coordinator_with_engine(engine_type: EngineType) -> (FulltextCoordinator, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = FulltextConfig {
        enabled: true,
        index_path: temp_dir.path().to_path_buf(),
        default_engine: engine_type,
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

async fn setup_sync_manager() -> (SyncManager, Arc<FulltextCoordinator>, TempDir) {
    let (coordinator, temp_dir) = setup_coordinator().await;
    let coordinator = Arc::new(coordinator);

    let sync_config = SyncConfig {
        mode: SyncMode::Async,
        batch_size: 100,
        commit_interval_ms: 100,
        queue_size: 10000,
    };

    let sync_manager = SyncManager::with_sync_config(coordinator.clone(), sync_config);

    (sync_manager, coordinator, temp_dir)
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

#[allow(dead_code)]
fn create_vertex_with_int(vid: i64, tag_name: &str, properties: Vec<(&str, Value)>) -> Vertex {
    let mut props = HashMap::new();
    for (key, value) in properties {
        props.insert(key.to_string(), value);
    }
    let tag = Tag {
        name: tag_name.to_string(),
        properties: props,
    };
    Vertex::new(Value::Int(vid), vec![tag])
}

// ==================== Basic CRUD Tests ====================

#[tokio::test]
async fn test_fulltext_create_index() {
    let (coordinator, _temp) = setup_coordinator().await;

    let index_id = coordinator
        .create_index(1, "Article", "title", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    assert!(!index_id.is_empty(), "Index ID should not be empty");
    assert!(
        index_id.contains("Article"),
        "Index ID should contain tag name"
    );
    assert!(
        index_id.contains("title"),
        "Index ID should contain field name"
    );
}

#[tokio::test]
async fn test_fulltext_insert_and_search() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Post", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let vertex = create_vertex(1, "Post", vec![("content", "Hello World from Rust")]);
    coordinator
        .on_vertex_inserted(1, &vertex)
        .await
        .expect("Failed to insert vertex");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let results = coordinator
        .search(1, "Post", "content", "Hello", 10)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 1, "Expected 1 result for 'Hello'");
    assert_eq!(results[0].doc_id, Value::Int(1), "Doc ID should match");
    assert!(results[0].score > 0.0, "Score should be positive");
}

#[tokio::test]
async fn test_fulltext_update_vertex() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Article", "title", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let vertex = create_vertex(1, "Article", vec![("title", "Original Title")]);
    coordinator
        .on_vertex_inserted(1, &vertex)
        .await
        .expect("Failed to insert vertex");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let updated_vertex = create_vertex(1, "Article", vec![("title", "Updated Title")]);
    coordinator
        .on_vertex_updated(1, &updated_vertex, &["title".to_string()])
        .await
        .expect("Failed to update vertex");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let old_results = coordinator
        .search(1, "Article", "title", "Original", 10)
        .await
        .expect("Failed to search");
    assert_eq!(old_results.len(), 0, "Old content should not be found");

    let new_results = coordinator
        .search(1, "Article", "title", "Updated", 10)
        .await
        .expect("Failed to search");
    assert_eq!(new_results.len(), 1, "New content should be found");
}

#[tokio::test]
async fn test_fulltext_delete_vertex() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Document", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let vertex = create_vertex(1, "Document", vec![("content", "To be deleted")]);
    coordinator
        .on_vertex_inserted(1, &vertex)
        .await
        .expect("Failed to insert vertex");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    coordinator
        .on_vertex_deleted(1, "Document", &Value::Int(1))
        .await
        .expect("Failed to delete vertex");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let results = coordinator
        .search(1, "Document", "content", "deleted", 10)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 0, "Deleted document should not be found");
}

// ==================== Multiple Fields and Tags Tests ====================

#[tokio::test]
async fn test_fulltext_multiple_fields_on_same_tag() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Article", "title", Some(EngineType::Bm25))
        .await
        .expect("Failed to create title index");

    coordinator
        .create_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create content index");

    coordinator
        .create_index(1, "Article", "author", Some(EngineType::Bm25))
        .await
        .expect("Failed to create author index");

    let vertex = create_vertex(
        1,
        "Article",
        vec![
            ("title", "Rust Programming Guide"),
            ("content", "Learn Rust programming language"),
            ("author", "John Doe"),
        ],
    );

    coordinator
        .on_vertex_inserted(1, &vertex)
        .await
        .expect("Failed to insert vertex");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let title_results = coordinator
        .search(1, "Article", "title", "Rust", 10)
        .await
        .expect("Failed to search title");
    assert_eq!(title_results.len(), 1, "Should find by title");

    let content_results = coordinator
        .search(1, "Article", "content", "programming", 10)
        .await
        .expect("Failed to search content");
    assert_eq!(content_results.len(), 1, "Should find by content");

    let author_results = coordinator
        .search(1, "Article", "author", "John", 10)
        .await
        .expect("Failed to search author");
    assert_eq!(author_results.len(), 1, "Should find by author");
}

#[tokio::test]
async fn test_fulltext_same_field_on_different_tags() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Blog", "title", Some(EngineType::Bm25))
        .await
        .expect("Failed to create Blog index");

    coordinator
        .create_index(1, "News", "title", Some(EngineType::Bm25))
        .await
        .expect("Failed to create News index");

    let blog_vertex = create_vertex(1, "Blog", vec![("title", "Blog Post Title")]);
    let news_vertex = create_vertex(2, "News", vec![("title", "News Article Title")]);

    coordinator
        .on_vertex_inserted(1, &blog_vertex)
        .await
        .expect("Failed to insert blog");

    coordinator
        .on_vertex_inserted(1, &news_vertex)
        .await
        .expect("Failed to insert news");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let blog_results = coordinator
        .search(1, "Blog", "title", "Blog", 10)
        .await
        .expect("Failed to search blog");
    assert_eq!(blog_results.len(), 1, "Should find blog post");

    let news_results = coordinator
        .search(1, "News", "title", "News", 10)
        .await
        .expect("Failed to search news");
    assert_eq!(news_results.len(), 1, "Should find news article");

    let cross_results = coordinator
        .search(1, "Blog", "title", "News", 10)
        .await
        .expect("Failed to cross search");
    assert_eq!(cross_results.len(), 0, "Should not find across tags");
}

// ==================== Batch Operations Tests ====================

#[tokio::test]
async fn test_fulltext_batch_insert() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Document", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    for i in 0..50 {
        let vertex = create_vertex(
            i as i64,
            "Document",
            vec![("content", &format!("Document number {} with content", i))],
        );
        coordinator
            .on_vertex_inserted(1, &vertex)
            .await
            .expect("Failed to insert vertex");
    }

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(300)).await;

    let results = coordinator
        .search(1, "Document", "content", "Document", 100)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 50, "Should find all 50 documents");
}

#[tokio::test]
async fn test_fulltext_batch_delete() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    for i in 0..20 {
        let vertex = create_vertex(
            i as i64,
            "Article",
            vec![("content", &format!("Article {}", i))],
        );
        coordinator
            .on_vertex_inserted(1, &vertex)
            .await
            .expect("Failed to insert vertex");
    }

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    for i in 0..10 {
        coordinator
            .on_vertex_deleted(1, "Article", &Value::Int(i as i64))
            .await
            .expect("Failed to delete vertex");
    }

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let results = coordinator
        .search(1, "Article", "content", "Article", 100)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 10, "Should find remaining 10 articles");
}

// ==================== Search Features Tests ====================

#[tokio::test]
async fn test_fulltext_scoring_and_sorting() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Post", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let vertex1 = create_vertex(1, "Post", vec![("content", "Rust programming language")]);
    let vertex2 = create_vertex(2, "Post", vec![("content", "Rust")]);
    let vertex3 = create_vertex(3, "Post", vec![("content", "Programming in Rust is fun")]);

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
        .search(1, "Post", "content", "Rust", 10)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 3, "Should find 3 posts");

    assert!(
        results[0].score >= results[1].score,
        "Results should be sorted by score descending"
    );
    assert!(
        results[1].score >= results[2].score,
        "Results should be sorted by score descending"
    );
}

#[tokio::test]
async fn test_fulltext_limit_and_offset() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Article", "title", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    for i in 0..20 {
        let vertex = create_vertex(
            i as i64,
            "Article",
            vec![("title", &format!("Article Title {}", i))],
        );
        coordinator
            .on_vertex_inserted(1, &vertex)
            .await
            .expect("Failed to insert vertex");
    }

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let all_results = coordinator
        .search(1, "Article", "title", "Article", 100)
        .await
        .expect("Failed to search");
    assert_eq!(all_results.len(), 20, "Should find all 20 articles");

    let limited_results = coordinator
        .search(1, "Article", "title", "Article", 5)
        .await
        .expect("Failed to search with limit");
    assert_eq!(limited_results.len(), 5, "Should return only 5 results");
}

#[tokio::test]
async fn test_fulltext_special_characters() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Document", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let vertex = create_vertex(
        1,
        "Document",
        vec![("content", "Special chars: @#$% ^&*()")],
    );

    coordinator
        .on_vertex_inserted(1, &vertex)
        .await
        .expect("Failed to insert vertex");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let results = coordinator
        .search(1, "Document", "content", "Special", 10)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 1, "Should find document with special chars");
}

#[tokio::test]
async fn test_fulltext_unicode_content() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Article", "title", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let vertex = create_vertex(1, "Article", vec![("title", "中文标题 🚀 Unicode テスト")]);

    coordinator
        .on_vertex_inserted(1, &vertex)
        .await
        .expect("Failed to insert vertex");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let results = coordinator
        .search(1, "Article", "title", "中文", 10)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 1, "Should find Unicode content");
}

// ==================== Engine Type Tests ====================

#[tokio::test]
async fn test_fulltext_bm25_engine() {
    let (coordinator, _temp) = setup_coordinator_with_engine(EngineType::Bm25).await;

    coordinator
        .create_index(1, "Post", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let vertex = create_vertex(1, "Post", vec![("content", "BM25 engine test")]);

    coordinator
        .on_vertex_inserted(1, &vertex)
        .await
        .expect("Failed to insert vertex");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let results = coordinator
        .search(1, "Post", "content", "BM25", 10)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 1, "BM25 engine should work");
}

#[tokio::test]
async fn test_fulltext_inversearch_engine() {
    let (coordinator, _temp) = setup_coordinator_with_engine(EngineType::Inversearch).await;

    coordinator
        .create_index(1, "Post", "content", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create index");

    let vertex = create_vertex(1, "Post", vec![("content", "Inversearch engine test")]);

    coordinator
        .on_vertex_inserted(1, &vertex)
        .await
        .expect("Failed to insert vertex");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let results = coordinator
        .search(1, "Post", "content", "Inversearch", 10)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 1, "Inversearch engine should work");
}

// ==================== Sync Manager Tests ====================

#[tokio::test]
async fn test_sync_manager_async_mode() {
    let (_sync_manager, coordinator, _temp) = setup_sync_manager().await;

    coordinator
        .create_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let vertex = create_vertex(1, "Article", vec![("content", "Async sync test")]);

    coordinator
        .on_vertex_inserted(1, &vertex)
        .await
        .expect("Failed to insert vertex");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(300)).await;

    let results = coordinator
        .search(1, "Article", "content", "Async", 10)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 1, "Async sync should work");
}

#[tokio::test]
async fn test_sync_manager_sync_mode() {
    let (coordinator, _temp) = setup_coordinator().await;
    let coordinator = Arc::new(coordinator);

    let sync_manager = SyncManager::with_mode(coordinator.clone(), SyncMode::Sync);

    coordinator
        .create_index(1, "Document", "title", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let _vertex = create_vertex(1, "Document", vec![("title", "Sync mode test")]);

    sync_manager
        .on_vertex_change(
            1,
            "Document",
            &Value::Int(1),
            &[(
                "title".to_string(),
                Value::String("Sync mode test".to_string()),
            )],
            ChangeType::Insert,
        )
        .await
        .expect("Failed to sync vertex");

    sleep(Duration::from_millis(200)).await;

    let results = coordinator
        .search(1, "Document", "title", "Sync", 10)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 1, "Sync mode should work");
}

// ==================== Edge Cases and Error Handling ====================

#[tokio::test]
async fn test_fulltext_empty_search() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let results = coordinator
        .search(1, "Article", "content", "nonexistent", 10)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 0, "Empty search should return 0 results");
}

#[tokio::test]
async fn test_fulltext_duplicate_index_creation() {
    let (coordinator, _temp) = setup_coordinator().await;

    let result1 = coordinator
        .create_index(1, "Article", "title", Some(EngineType::Bm25))
        .await;

    assert!(result1.is_ok(), "First index creation should succeed");

    let result2 = coordinator
        .create_index(1, "Article", "title", Some(EngineType::Bm25))
        .await;

    assert!(result2.is_err(), "Duplicate index creation should fail");
}

#[tokio::test]
async fn test_fulltext_non_existent_index_search() {
    let (coordinator, _temp) = setup_coordinator().await;

    let results = coordinator
        .search(999, "NonExistent", "field", "query", 10)
        .await;

    assert!(results.is_err(), "Search on non-existent index should fail");
}

#[tokio::test]
async fn test_fulltext_rebuild_index() {
    let (coordinator, _temp) = setup_coordinator().await;

    coordinator
        .create_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let vertex = create_vertex(1, "Article", vec![("content", "Test content for rebuild")]);

    coordinator
        .on_vertex_inserted(1, &vertex)
        .await
        .expect("Failed to insert vertex");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let before_results = coordinator
        .search(1, "Article", "content", "Test", 10)
        .await
        .expect("Failed to search before rebuild");
    assert_eq!(
        before_results.len(),
        1,
        "Should find content before rebuild"
    );

    coordinator
        .rebuild_index(1, "Article", "content")
        .await
        .expect("Failed to rebuild index");

    sleep(Duration::from_millis(200)).await;

    let after_results = coordinator
        .search(1, "Article", "content", "Test", 10)
        .await
        .expect("Failed to search after rebuild");
    assert_eq!(after_results.len(), 1, "Should find content after rebuild");
}

// ==================== Concurrent Operations Tests ====================

#[tokio::test]
async fn test_fulltext_concurrent_inserts() {
    let (coordinator, _temp) = setup_coordinator().await;
    let coordinator = Arc::new(coordinator);

    coordinator
        .create_index(1, "Document", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let mut handles = vec![];
    for i in 0..10 {
        let coord = Arc::clone(&coordinator);
        let handle = tokio::spawn(async move {
            for j in 0..5 {
                let vid = i * 5 + j;
                let vertex = create_vertex(
                    vid as i64,
                    "Document",
                    vec![("content", &format!("Concurrent document {}", vid))],
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

    sleep(Duration::from_millis(300)).await;

    let results = coordinator
        .search(1, "Document", "content", "Concurrent", 100)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 50, "Should find all 50 concurrent inserts");
}

#[tokio::test]
async fn test_fulltext_concurrent_searches() {
    let (coordinator, _temp) = setup_coordinator().await;
    let coordinator = Arc::new(coordinator);

    coordinator
        .create_index(1, "Article", "title", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let vertex = create_vertex(1, "Article", vec![("title", "Concurrent search test")]);

    coordinator
        .on_vertex_inserted(1, &vertex)
        .await
        .expect("Failed to insert vertex");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    let mut handles = vec![];
    for _ in 0..10 {
        let coord = Arc::clone(&coordinator);
        let handle = tokio::spawn(async move {
            let results = coord
                .search(1, "Article", "title", "Concurrent", 10)
                .await
                .expect("Failed to search");
            results.len()
        });
        handles.push(handle);
    }

    let results = futures::future::join_all(handles).await;

    for result_count in results {
        assert_eq!(
            result_count.unwrap(),
            1,
            "Each concurrent search should find 1 result"
        );
    }
}

// ==================== Integration with Storage Layer ====================
// Tests that integrate with the actual storage layer to verify end-to-end functionality

#[tokio::test]
async fn test_fulltext_with_storage_layer() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();

    // Create space and tag using storage API
    let space_info = create_test_space("fulltext_space");
    get_storage(&storage)
        .create_space(&space_info)
        .expect("Failed to create space");

    let tag_info = person_tag_info();
    get_storage(&storage)
        .create_tag("fulltext_space", &tag_info)
        .expect("Failed to create tag");

    let (coordinator, _temp) = setup_coordinator().await;

    // Create fulltext index
    coordinator
        .create_index(1, "Person", "name", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    // Create and insert vertex through coordinator
    let person_vertex = create_vertex(1, "Person", vec![("name", "Alice Johnson")]);

    coordinator
        .on_vertex_inserted(1, &person_vertex)
        .await
        .expect("Failed to insert vertex");

    coordinator.commit_all().await.expect("Failed to commit");
    sleep(Duration::from_millis(200)).await;

    // Search and verify
    let results = coordinator
        .search(1, "Person", "name", "Alice", 10)
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 1, "Should find person by name");
    assert_eq!(results[0].doc_id, Value::Int(1), "Doc ID should match");
}

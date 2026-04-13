use graphdb::core::vertex_edge_path::Tag;
use graphdb::core::{Value, Vertex};
use graphdb::search::config::FulltextConfig;
use graphdb::search::engine::EngineType;
use graphdb::search::manager::FulltextIndexManager;
use graphdb::sync::batch::BatchConfig;
use graphdb::sync::coordinator::SyncCoordinator;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

async fn setup_test_coordinator() -> (Arc<SyncCoordinator>, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = FulltextConfig {
        enabled: true,
        index_path: temp_dir.path().to_path_buf(),
        default_engine: EngineType::Bm25,
        sync: Default::default(),
        bm25: Default::default(),
        inversearch: Default::default(),
        cache_size: 100,
        max_result_cache: 1000,
        result_cache_ttl_secs: 60,
    };
    let manager = Arc::new(FulltextIndexManager::new(config).expect("Failed to create manager"));
    let coordinator = Arc::new(SyncCoordinator::new(manager, BatchConfig::default()));
    (coordinator, temp_dir)
}

fn create_test_vertex(vid: i64, tag_name: &str, properties: Vec<(&str, &str)>) -> Vertex {
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

#[tokio::test]
async fn test_fulltext_end_to_end() {
    let (coordinator, _temp) = setup_test_coordinator().await;

    // 1. Create fulltext index
    let index_id = coordinator
        .create_index(1, "Article", "title", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");
    assert!(!index_id.is_empty(), "Index ID should not be empty");

    // 2. Insert data
    let vertex1 = create_test_vertex(
        1,
        "Article",
        vec![("title", "Hello World"), ("content", "First article")],
    );
    let vertex2 = create_test_vertex(
        2,
        "Article",
        vec![("title", "Rust Programming"), ("content", "Second article")],
    );
    let vertex3 = create_test_vertex(
        3,
        "Article",
        vec![("title", "Hello Rust"), ("content", "Third article")],
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

    // 3. Wait for indexing
    coordinator.commit_all().await.expect("Failed to commit");
    tokio::time::sleep(Duration::from_millis(500)).await;

    // 4. Execute fulltext search
    let results = coordinator
        .search(1, "Article", "title", "Hello", 10)
        .await
        .expect("Failed to search");
    assert_eq!(results.len(), 2, "Expected 2 results for 'Hello'");

    // 5. Search with scoring
    let results = coordinator
        .search(1, "Article", "title", "Rust", 10)
        .await
        .expect("Failed to search");
    assert_eq!(results.len(), 2, "Expected 2 results for 'Rust'");
    // Verify sorting: "Rust Programming" should have higher score than "Hello Rust"
    assert!(
        results[0].score >= results[1].score,
        "Results should be sorted by score descending"
    );
}

#[tokio::test]
async fn test_fulltext_with_updates() {
    let (coordinator, _temp) = setup_test_coordinator().await;

    // Create index
    coordinator
        .create_index(1, "Post", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    // Insert
    let vertex = create_test_vertex(1, "Post", vec![("content", "Original content")]);
    coordinator
        .on_vertex_inserted(1, &vertex)
        .await
        .expect("Failed to insert vertex");
    coordinator.commit_all().await.expect("Failed to commit");

    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify
    let results = coordinator
        .search(1, "Post", "content", "Original", 10)
        .await
        .expect("Failed to search");
    assert_eq!(results.len(), 1, "Expected 1 result for 'Original'");

    // Update
    let updated_vertex = create_test_vertex(1, "Post", vec![("content", "Updated content")]);
    coordinator
        .on_vertex_updated(1, &updated_vertex, &["content".to_string()])
        .await
        .expect("Failed to update vertex");
    coordinator.commit_all().await.expect("Failed to commit");

    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify old content not found
    let results = coordinator
        .search(1, "Post", "content", "Original", 10)
        .await
        .expect("Failed to search");
    assert_eq!(results.len(), 0, "Expected 0 results for old content");

    // Verify new content found
    let results = coordinator
        .search(1, "Post", "content", "Updated", 10)
        .await
        .expect("Failed to search");
    assert_eq!(results.len(), 1, "Expected 1 result for new content");
}

#[tokio::test]
async fn test_fulltext_rebuild() {
    let (coordinator, _temp) = setup_test_coordinator().await;

    // Create index and insert data
    coordinator
        .create_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let vertex = create_test_vertex(1, "Article", vec![("content", "Test content")]);
    coordinator
        .on_vertex_inserted(1, &vertex)
        .await
        .expect("Failed to insert vertex");
    coordinator.commit_all().await.expect("Failed to commit");

    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify data exists
    let results = coordinator
        .search(1, "Article", "content", "Test", 10)
        .await
        .expect("Failed to search");
    assert_eq!(results.len(), 1, "Expected 1 result before rebuild");

    // Rebuild index
    coordinator
        .rebuild_index(1, "Article", "content")
        .await
        .expect("Failed to rebuild index");

    // After rebuild, data should still be searchable (depending on implementation)
    let results = coordinator
        .search(1, "Article", "content", "Test", 10)
        .await
        .expect("Failed to search");
    assert_eq!(results.len(), 1, "Expected 1 result after rebuild");
}

#[tokio::test]
async fn test_fulltext_multiple_fields() {
    let (coordinator, _temp) = setup_test_coordinator().await;

    // Create indexes on multiple fields
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

    // Insert data
    let vertex = create_test_vertex(
        1,
        "Article",
        vec![
            ("title", "Rust Tutorial"),
            ("content", "Learn Rust programming"),
            ("author", "John Doe"),
        ],
    );
    coordinator
        .on_vertex_inserted(1, &vertex)
        .await
        .expect("Failed to insert vertex");
    coordinator.commit_all().await.expect("Failed to commit");

    // Search each field
    let title_results = coordinator
        .search(1, "Article", "title", "Rust", 10)
        .await
        .expect("Failed to search title");
    assert_eq!(title_results.len(), 1, "Expected 1 title result");

    let content_results = coordinator
        .search(1, "Article", "content", "programming", 10)
        .await
        .expect("Failed to search content");
    assert_eq!(content_results.len(), 1, "Expected 1 content result");

    let author_results = coordinator
        .search(1, "Article", "author", "John", 10)
        .await
        .expect("Failed to search author");
    assert_eq!(author_results.len(), 1, "Expected 1 author result");
}

#[tokio::test]
async fn test_fulltext_multiple_tags() {
    let (coordinator, _temp) = setup_test_coordinator().await;

    // Create indexes for different tags
    coordinator
        .create_index(1, "Blog", "title", Some(EngineType::Bm25))
        .await
        .expect("Failed to create Blog index");
    coordinator
        .create_index(1, "News", "title", Some(EngineType::Bm25))
        .await
        .expect("Failed to create News index");

    // Insert data
    let blog_vertex = create_test_vertex(1, "Blog", vec![("title", "Blog Post")]);
    let news_vertex = create_test_vertex(2, "News", vec![("title", "News Article")]);

    coordinator
        .on_vertex_inserted(1, &blog_vertex)
        .await
        .expect("Failed to insert blog");
    coordinator
        .on_vertex_inserted(1, &news_vertex)
        .await
        .expect("Failed to insert news");
    coordinator.commit_all().await.expect("Failed to commit");

    // Search each tag
    let blog_results = coordinator
        .search(1, "Blog", "title", "Blog", 10)
        .await
        .expect("Failed to search blog");
    assert_eq!(blog_results.len(), 1, "Expected 1 blog result");

    let news_results = coordinator
        .search(1, "News", "title", "News", 10)
        .await
        .expect("Failed to search news");
    assert_eq!(news_results.len(), 1, "Expected 1 news result");

    // Cross-tag search should not find results
    let cross_results = coordinator
        .search(1, "Blog", "title", "News", 10)
        .await
        .expect("Failed to cross search");
    assert_eq!(cross_results.len(), 0, "Expected 0 cross-tag results");
}

#[tokio::test]
async fn test_fulltext_batch_operations() {
    let (coordinator, _temp) = setup_test_coordinator().await;

    // Create index
    coordinator
        .create_index(1, "Document", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    // Batch insert
    for i in 0..100 {
        let vertex = create_test_vertex(
            i as i64,
            "Document",
            vec![("content", &format!("Document number {}", i))],
        );
        coordinator
            .on_vertex_inserted(1, &vertex)
            .await
            .expect("Failed to insert vertex");
    }
    coordinator.commit_all().await.expect("Failed to commit");

    tokio::time::sleep(Duration::from_millis(300)).await;

    // Search
    let results = coordinator
        .search(1, "Document", "content", "Document", 100)
        .await
        .expect("Failed to search");
    assert_eq!(results.len(), 100, "Expected 100 results");
}

#[tokio::test]
async fn test_fulltext_delete_and_reinsert() {
    let (coordinator, _temp) = setup_test_coordinator().await;

    // Create index
    coordinator
        .create_index(1, "Article", "title", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    // Insert
    let vertex = create_test_vertex(1, "Article", vec![("title", "Test Article")]);
    coordinator
        .on_vertex_inserted(1, &vertex)
        .await
        .expect("Failed to insert vertex");
    coordinator.commit_all().await.expect("Failed to commit");

    // Verify
    let results = coordinator
        .search(1, "Article", "title", "Test", 10)
        .await
        .expect("Failed to search");
    assert_eq!(results.len(), 1, "Expected 1 result after insert");

    // Delete
    coordinator
        .on_vertex_deleted(1, "Article", &Value::Int(1))
        .await
        .expect("Failed to delete vertex");
    coordinator.commit_all().await.expect("Failed to commit");

    // Verify deleted
    let results = coordinator
        .search(1, "Article", "title", "Test", 10)
        .await
        .expect("Failed to search");
    assert_eq!(results.len(), 0, "Expected 0 results after delete");

    // Re-insert with same ID
    let vertex2 = create_test_vertex(1, "Article", vec![("title", "Reinserted Article")]);
    coordinator
        .on_vertex_inserted(1, &vertex2)
        .await
        .expect("Failed to reinsert vertex");
    coordinator.commit_all().await.expect("Failed to commit");

    // Verify re-inserted
    let results = coordinator
        .search(1, "Article", "title", "Reinserted", 10)
        .await
        .expect("Failed to search");
    assert_eq!(results.len(), 1, "Expected 1 result after reinsert");
}

#[tokio::test]
async fn test_fulltext_empty_and_special_queries() {
    let (coordinator, _temp) = setup_test_coordinator().await;

    // Create index
    coordinator
        .create_index(1, "Article", "title", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    // Insert data
    let vertex = create_test_vertex(1, "Article", vec![("title", "Special Characters: @#$%")]);
    coordinator
        .on_vertex_inserted(1, &vertex)
        .await
        .expect("Failed to insert vertex");
    coordinator.commit_all().await.expect("Failed to commit");

    // Search for non-existent term
    let results = coordinator
        .search(1, "Article", "title", "nonexistent", 10)
        .await
        .expect("Failed to search");
    assert_eq!(results.len(), 0, "Expected 0 results for non-existent term");

    // Search with partial match
    let results = coordinator
        .search(1, "Article", "title", "Special", 10)
        .await
        .expect("Failed to search");
    assert_eq!(results.len(), 1, "Expected 1 result for partial match");
}

#[tokio::test]
async fn test_fulltext_concurrent_operations() {
    use std::sync::Arc;

    let (coordinator, _temp) = setup_test_coordinator().await;
    let coordinator = Arc::new(coordinator);

    // Create index
    coordinator
        .create_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    // Concurrent inserts
    let mut handles = vec![];
    for i in 0..10 {
        let coord = Arc::clone(&coordinator);
        let handle = tokio::spawn(async move {
            for j in 0..10 {
                let vid = i * 10 + j;
                let vertex = create_test_vertex(
                    vid as i64,
                    "Article",
                    vec![("content", &format!("Content {}", vid))],
                );
                let _ = coord.on_vertex_inserted(1, &vertex).await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    Arc::try_unwrap(coordinator)
        .expect("Arc still has multiple owners")
        .commit_all()
        .await
        .expect("Failed to commit");
    tokio::time::sleep(Duration::from_millis(300)).await;
}

//! SyncManager Integration Tests
//!
//! Test scope:
//! - Sync mode behavior (Sync/Async/Off)
//! - SyncConfig integration
//! - TransactionManager integration
//! - Data synchronization correctness
//! - Concurrent safety

mod common;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use graphdb::coordinator::{ChangeType, FulltextCoordinator};
use graphdb::core::vertex_edge_path::Tag;
use graphdb::core::{Value, Vertex};
use graphdb::search::{EngineType, FulltextConfig, FulltextIndexManager, SyncConfig};
use graphdb::sync::{SyncManager, SyncMode};
use tempfile::TempDir;

// ==================== Test Fixtures ====================

struct SyncTestContext {
    coordinator: Arc<FulltextCoordinator>,
    sync_manager: Arc<SyncManager>,
    _temp_dir: TempDir,
}

impl SyncTestContext {
    fn new() -> Self {
        Self::with_sync_mode(SyncMode::Async)
    }

    fn with_sync_mode(mode: SyncMode) -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = FulltextConfig {
            enabled: true,
            index_path: temp_dir.path().to_path_buf(),
            default_engine: EngineType::Bm25,
            sync: SyncConfig {
                mode,
                queue_size: 100,
                commit_interval_ms: 100,
                batch_size: 10,
            },
            bm25: Default::default(),
            inversearch: Default::default(),
            cache_size: 100,
            max_result_cache: 1000,
            result_cache_ttl_secs: 60,
        };

        let manager = Arc::new(
            FulltextIndexManager::new(config.clone()).expect("Failed to create manager"),
        );
        let coordinator = Arc::new(FulltextCoordinator::new(manager));
        let sync_manager = Arc::new(SyncManager::with_sync_config(
            coordinator.clone(),
            config.sync,
        ));

        Self {
            coordinator,
            sync_manager,
            _temp_dir: temp_dir,
        }
    }

    fn with_sync_config(sync_config: SyncConfig) -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = FulltextConfig {
            enabled: true,
            index_path: temp_dir.path().to_path_buf(),
            default_engine: EngineType::Bm25,
            sync: sync_config.clone(),
            bm25: Default::default(),
            inversearch: Default::default(),
            cache_size: 100,
            max_result_cache: 1000,
            result_cache_ttl_secs: 60,
        };

        let manager = Arc::new(
            FulltextIndexManager::new(config.clone()).expect("Failed to create manager"),
        );
        let coordinator = Arc::new(FulltextCoordinator::new(manager));
        let sync_manager = Arc::new(SyncManager::with_sync_config(
            coordinator.clone(),
            sync_config,
        ));

        Self {
            coordinator,
            sync_manager,
            _temp_dir: temp_dir,
        }
    }
}

fn create_test_vertex(vid: i64, tag_name: &str, content: &str) -> Vertex {
    let mut props = HashMap::new();
    props.insert("content".to_string(), Value::String(content.to_string()));
    let tag = Tag::new(tag_name.to_string(), props);
    Vertex::new(Value::Int(vid), vec![tag])
}

// ==================== Sync Mode Tests ====================

#[tokio::test]
async fn test_sync_mode_sync_processes_immediately() {
    let ctx = SyncTestContext::with_sync_mode(SyncMode::Sync);

    ctx.coordinator
        .create_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let vertex = create_test_vertex(1, "Article", "Hello World");
    let properties: Vec<(String, Value)> = vertex.tags[0]
        .properties
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    ctx.sync_manager
        .on_vertex_change(1, "Article", &Value::Int(1), &properties, ChangeType::Insert)
        .await
        .expect("Failed to process vertex change");

    ctx.coordinator.commit_all().await.expect("Failed to commit");
    tokio::time::sleep(Duration::from_millis(100)).await;

    let results = ctx
        .coordinator
        .search(1, "Article", "content", "Hello", 10)
        .await
        .expect("Failed to search");
    assert_eq!(results.len(), 1, "Sync mode should process immediately");
}

#[tokio::test]
async fn test_sync_mode_async_queues_task() {
    let ctx = SyncTestContext::with_sync_mode(SyncMode::Async);

    ctx.coordinator
        .create_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let properties = vec![(
        "content".to_string(),
        Value::String("Hello World".to_string()),
    )];

    ctx.sync_manager
        .on_vertex_change(1, "Article", &Value::Int(1), &properties, ChangeType::Insert)
        .await
        .expect("Failed to submit task");

    ctx.sync_manager.force_commit().await.expect("Failed to commit");
    tokio::time::sleep(Duration::from_millis(100)).await;

    let results = ctx
        .coordinator
        .search(1, "Article", "content", "Hello", 10)
        .await
        .expect("Failed to search");
    assert_eq!(results.len(), 1, "Async mode should process after commit");
}

#[tokio::test]
async fn test_sync_mode_off_skips_processing() {
    let ctx = SyncTestContext::with_sync_mode(SyncMode::Off);

    ctx.coordinator
        .create_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let properties = vec![(
        "content".to_string(),
        Value::String("Hello World".to_string()),
    )];

    ctx.sync_manager
        .on_vertex_change(1, "Article", &Value::Int(1), &properties, ChangeType::Insert)
        .await
        .expect("Should not fail in Off mode");

    ctx.sync_manager.force_commit().await.expect("Failed to commit");
    tokio::time::sleep(Duration::from_millis(100)).await;

    let results = ctx
        .coordinator
        .search(1, "Article", "content", "Hello", 10)
        .await
        .expect("Failed to search");
    assert_eq!(results.len(), 0, "Off mode should skip indexing");
}

// ==================== Mode Switching Tests ====================

#[tokio::test]
async fn test_sync_mode_runtime_switch() {
    let ctx = SyncTestContext::new();

    ctx.coordinator
        .create_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    assert_eq!(ctx.sync_manager.get_mode().await, SyncMode::Async);

    ctx.sync_manager.set_mode(SyncMode::Sync).await;
    assert_eq!(ctx.sync_manager.get_mode().await, SyncMode::Sync);

    let properties = vec![(
        "content".to_string(),
        Value::String("Test Content".to_string()),
    )];
    ctx.sync_manager
        .on_vertex_change(1, "Article", &Value::Int(1), &properties, ChangeType::Insert)
        .await
        .expect("Failed to process");

    ctx.coordinator.commit_all().await.expect("Failed to commit");
    tokio::time::sleep(Duration::from_millis(100)).await;

    ctx.sync_manager.set_mode(SyncMode::Off).await;
    assert_eq!(ctx.sync_manager.get_mode().await, SyncMode::Off);

    let properties2 = vec![(
        "content".to_string(),
        Value::String("Skipped Content".to_string()),
    )];
    ctx.sync_manager
        .on_vertex_change(1, "Article", &Value::Int(2), &properties2, ChangeType::Insert)
        .await
        .expect("Should not fail");

    ctx.coordinator.commit_all().await.expect("Failed to commit");
    tokio::time::sleep(Duration::from_millis(100)).await;

    let results = ctx
        .coordinator
        .search(1, "Article", "content", "Content", 10)
        .await
        .expect("Failed to search");
    assert_eq!(results.len(), 1, "Only Sync mode insert should be indexed");
}

#[tokio::test]
async fn test_sync_mode_switch_to_async_after_off() {
    let ctx = SyncTestContext::with_sync_mode(SyncMode::Off);

    ctx.coordinator
        .create_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let properties = vec![(
        "content".to_string(),
        Value::String("Skipped".to_string()),
    )];
    ctx.sync_manager
        .on_vertex_change(1, "Article", &Value::Int(1), &properties, ChangeType::Insert)
        .await
        .expect("Should not fail");

    ctx.sync_manager.set_mode(SyncMode::Async).await;

    let properties2 = vec![(
        "content".to_string(),
        Value::String("Processed".to_string()),
    )];
    ctx.sync_manager
        .on_vertex_change(1, "Article", &Value::Int(2), &properties2, ChangeType::Insert)
        .await
        .expect("Should not fail");

    ctx.sync_manager.force_commit().await.expect("Failed to commit");
    tokio::time::sleep(Duration::from_millis(100)).await;

    let results = ctx
        .coordinator
        .search(1, "Article", "content", "Processed", 10)
        .await
        .expect("Failed to search");
    assert_eq!(results.len(), 1, "Async mode insert should be indexed after mode switch");
}

// ==================== Config Integration Tests ====================

#[test]
fn test_sync_config_parameters_applied() {
    let sync_config = SyncConfig {
        mode: SyncMode::Sync,
        queue_size: 5000,
        commit_interval_ms: 500,
        batch_size: 50,
    };

    assert_eq!(sync_config.mode, SyncMode::Sync);
    assert_eq!(sync_config.queue_size, 5000);
    assert_eq!(sync_config.commit_interval_ms, 500);
    assert_eq!(sync_config.batch_size, 50);
}

#[test]
fn test_sync_config_default_values() {
    let config = SyncConfig::default();
    assert_eq!(config.mode, SyncMode::Async);
    assert_eq!(config.queue_size, 10000);
    assert_eq!(config.commit_interval_ms, 1000);
    assert_eq!(config.batch_size, 100);
}

#[tokio::test]
async fn test_sync_config_custom_queue_size() {
    let sync_config = SyncConfig {
        mode: SyncMode::Async,
        queue_size: 50,
        commit_interval_ms: 100,
        batch_size: 5,
    };

    let ctx = SyncTestContext::with_sync_config(sync_config);

    ctx.coordinator
        .create_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    for i in 0..10 {
        let properties = vec![(
            "content".to_string(),
            Value::String(format!("Content {}", i)),
        )];
        ctx.sync_manager
            .on_vertex_change(1, "Article", &Value::Int(i), &properties, ChangeType::Insert)
            .await
            .expect("Failed to submit task");
    }

    ctx.sync_manager.force_commit().await.expect("Failed to commit");
    tokio::time::sleep(Duration::from_millis(200)).await;

    let results = ctx
        .coordinator
        .search(1, "Article", "content", "Content", 100)
        .await
        .expect("Failed to search");
    assert_eq!(results.len(), 10, "All inserts should be indexed");
}

// ==================== Data Change Type Tests ====================

#[tokio::test]
async fn test_vertex_insert_change() {
    let ctx = SyncTestContext::with_sync_mode(SyncMode::Sync);

    ctx.coordinator
        .create_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let properties = vec![(
        "content".to_string(),
        Value::String("New Article".to_string()),
    )];
    ctx.sync_manager
        .on_vertex_change(1, "Article", &Value::Int(1), &properties, ChangeType::Insert)
        .await
        .expect("Failed to process insert");

    ctx.coordinator.commit_all().await.expect("Failed to commit");
    tokio::time::sleep(Duration::from_millis(100)).await;

    let results = ctx
        .coordinator
        .search(1, "Article", "content", "Article", 10)
        .await
        .expect("Failed to search");
    assert_eq!(results.len(), 1);
}

#[tokio::test]
async fn test_vertex_update_change() {
    let ctx = SyncTestContext::with_sync_mode(SyncMode::Sync);

    ctx.coordinator
        .create_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let properties = vec![(
        "content".to_string(),
        Value::String("Original Content".to_string()),
    )];
    ctx.sync_manager
        .on_vertex_change(1, "Article", &Value::Int(1), &properties, ChangeType::Insert)
        .await
        .expect("Failed to process insert");

    ctx.coordinator.commit_all().await.expect("Failed to commit");
    tokio::time::sleep(Duration::from_millis(100)).await;

    let properties_update = vec![(
        "content".to_string(),
        Value::String("Updated Content".to_string()),
    )];
    ctx.sync_manager
        .on_vertex_change(1, "Article", &Value::Int(1), &properties_update, ChangeType::Update)
        .await
        .expect("Failed to process update");

    ctx.coordinator.commit_all().await.expect("Failed to commit");
    tokio::time::sleep(Duration::from_millis(100)).await;

    let results = ctx
        .coordinator
        .search(1, "Article", "content", "Updated", 10)
        .await
        .expect("Failed to search");
    assert_eq!(results.len(), 1, "Updated content should be searchable");

    let old_results = ctx
        .coordinator
        .search(1, "Article", "content", "Original", 10)
        .await
        .expect("Failed to search");
    assert_eq!(old_results.len(), 0, "Old content should not be found");
}

#[tokio::test]
async fn test_vertex_delete_change() {
    let ctx = SyncTestContext::with_sync_mode(SyncMode::Sync);

    ctx.coordinator
        .create_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let properties = vec![(
        "content".to_string(),
        Value::String("To Be Deleted".to_string()),
    )];
    ctx.sync_manager
        .on_vertex_change(1, "Article", &Value::Int(1), &properties, ChangeType::Insert)
        .await
        .expect("Failed to process insert");

    ctx.coordinator.commit_all().await.expect("Failed to commit");
    tokio::time::sleep(Duration::from_millis(100)).await;

    let results = ctx
        .coordinator
        .search(1, "Article", "content", "Deleted", 10)
        .await
        .expect("Failed to search");
    assert_eq!(results.len(), 1, "Content should be found before delete");

    let delete_props = vec![(
        "content".to_string(),
        Value::String("To Be Deleted".to_string()),
    )];
    ctx.sync_manager
        .on_vertex_change(1, "Article", &Value::Int(1), &delete_props, ChangeType::Delete)
        .await
        .expect("Failed to process delete");

    ctx.coordinator.commit_all().await.expect("Failed to commit");
    tokio::time::sleep(Duration::from_millis(100)).await;

    let results = ctx
        .coordinator
        .search(1, "Article", "content", "Deleted", 10)
        .await
        .expect("Failed to search");
    assert_eq!(results.len(), 0, "Deleted content should not be found");
}

// ==================== Concurrent Tests ====================

#[tokio::test]
async fn test_concurrent_vertex_changes() {
    let ctx = SyncTestContext::with_sync_mode(SyncMode::Async);

    ctx.coordinator
        .create_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let mut handles = vec![];
    for i in 0..20 {
        let sync = Arc::clone(&ctx.sync_manager);
        let handle = tokio::spawn(async move {
            let properties = vec![(
                "content".to_string(),
                Value::String(format!("Content {}", i)),
            )];
            sync.on_vertex_change(1, "Article", &Value::Int(i), &properties, ChangeType::Insert)
                .await
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("Task panicked").expect("Insert failed");
    }

    ctx.sync_manager.force_commit().await.expect("Failed to commit");
    tokio::time::sleep(Duration::from_millis(200)).await;

    let results = ctx
        .coordinator
        .search(1, "Article", "content", "Content", 100)
        .await
        .expect("Failed to search");
    assert_eq!(results.len(), 20, "All concurrent inserts should be indexed");
}

#[tokio::test]
async fn test_concurrent_mode_switches() {
    let ctx = SyncTestContext::new();

    let sync = Arc::clone(&ctx.sync_manager);
    let handle1 = tokio::spawn(async move {
        for _ in 0..10 {
            sync.set_mode(SyncMode::Sync).await;
            tokio::time::sleep(Duration::from_millis(10)).await;
            sync.set_mode(SyncMode::Async).await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    });

    let sync2 = Arc::clone(&ctx.sync_manager);
    let handle2 = tokio::spawn(async move {
        for _ in 0..10 {
            sync2.set_mode(SyncMode::Off).await;
            tokio::time::sleep(Duration::from_millis(10)).await;
            sync2.set_mode(SyncMode::Async).await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    });

    let (r1, r2) = tokio::join!(handle1, handle2);
    r1.expect("Task 1 panicked");
    r2.expect("Task 2 panicked");

    let mode = ctx.sync_manager.get_mode().await;
    assert!(
        mode == SyncMode::Sync || mode == SyncMode::Async || mode == SyncMode::Off,
        "Mode should be valid after concurrent switches"
    );
}

// ==================== Multiple Index Tests ====================

#[tokio::test]
async fn test_multiple_indexes_sync() {
    let ctx = SyncTestContext::with_sync_mode(SyncMode::Sync);

    ctx.coordinator
        .create_index(1, "Article", "title", Some(EngineType::Bm25))
        .await
        .expect("Failed to create title index");
    ctx.coordinator
        .create_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create content index");

    let properties = vec![
        ("title".to_string(), Value::String("Test Title".to_string())),
        ("content".to_string(), Value::String("Test Content".to_string())),
    ];
    ctx.sync_manager
        .on_vertex_change(1, "Article", &Value::Int(1), &properties, ChangeType::Insert)
        .await
        .expect("Failed to process");

    ctx.coordinator.commit_all().await.expect("Failed to commit");
    tokio::time::sleep(Duration::from_millis(100)).await;

    let title_results = ctx
        .coordinator
        .search(1, "Article", "title", "Title", 10)
        .await
        .expect("Failed to search title");
    assert_eq!(title_results.len(), 1);

    let content_results = ctx
        .coordinator
        .search(1, "Article", "content", "Content", 10)
        .await
        .expect("Failed to search content");
    assert_eq!(content_results.len(), 1);
}

#[tokio::test]
async fn test_multiple_tags_sync() {
    let ctx = SyncTestContext::with_sync_mode(SyncMode::Sync);

    ctx.coordinator
        .create_index(1, "Blog", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create Blog index");
    ctx.coordinator
        .create_index(1, "News", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create News index");

    let blog_props = vec![("content".to_string(), Value::String("Blog Post".to_string()))];
    ctx.sync_manager
        .on_vertex_change(1, "Blog", &Value::Int(1), &blog_props, ChangeType::Insert)
        .await
        .expect("Failed to process blog");

    let news_props = vec![("content".to_string(), Value::String("News Article".to_string()))];
    ctx.sync_manager
        .on_vertex_change(1, "News", &Value::Int(2), &news_props, ChangeType::Insert)
        .await
        .expect("Failed to process news");

    ctx.coordinator.commit_all().await.expect("Failed to commit");
    tokio::time::sleep(Duration::from_millis(100)).await;

    let blog_results = ctx
        .coordinator
        .search(1, "Blog", "content", "Blog", 10)
        .await
        .expect("Failed to search blog");
    assert_eq!(blog_results.len(), 1);

    let news_results = ctx
        .coordinator
        .search(1, "News", "content", "News", 10)
        .await
        .expect("Failed to search news");
    assert_eq!(news_results.len(), 1);
}

// ==================== Edge Cases ====================

#[tokio::test]
async fn test_empty_properties() {
    let ctx = SyncTestContext::with_sync_mode(SyncMode::Sync);

    ctx.coordinator
        .create_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let result = ctx
        .sync_manager
        .on_vertex_change(1, "Article", &Value::Int(1), &[], ChangeType::Insert)
        .await;
    assert!(result.is_ok(), "Empty properties should not fail");
}

#[tokio::test]
async fn test_nonexistent_index() {
    let ctx = SyncTestContext::with_sync_mode(SyncMode::Sync);

    let properties = vec![(
        "content".to_string(),
        Value::String("Test".to_string()),
    )];
    let result = ctx
        .sync_manager
        .on_vertex_change(1, "NonExistent", &Value::Int(1), &properties, ChangeType::Insert)
        .await;
    assert!(result.is_ok(), "Non-existent index should not fail");
}

#[tokio::test]
async fn test_large_content() {
    let ctx = SyncTestContext::with_sync_mode(SyncMode::Sync);

    ctx.coordinator
        .create_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let large_content = "word ".repeat(1000);
    let properties = vec![("content".to_string(), Value::String(large_content.clone()))];
    ctx.sync_manager
        .on_vertex_change(1, "Article", &Value::Int(1), &properties, ChangeType::Insert)
        .await
        .expect("Failed to process large content");

    ctx.coordinator.commit_all().await.expect("Failed to commit");
    tokio::time::sleep(Duration::from_millis(100)).await;

    let results = ctx
        .coordinator
        .search(1, "Article", "content", "word", 10)
        .await
        .expect("Failed to search");
    assert_eq!(results.len(), 1);
}

// ==================== Serde Tests ====================

#[test]
fn test_sync_mode_serde_roundtrip() {
    let modes = vec![SyncMode::Sync, SyncMode::Async, SyncMode::Off];

    for mode in modes {
        let json = serde_json::to_string(&mode).expect("Failed to serialize");
        let decoded: SyncMode = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(mode, decoded, "Serde roundtrip failed for {:?}", mode);
    }
}

#[test]
fn test_sync_mode_json_values() {
    assert_eq!(
        serde_json::to_string(&SyncMode::Sync).expect("Failed"),
        "\"sync\""
    );
    assert_eq!(
        serde_json::to_string(&SyncMode::Async).expect("Failed"),
        "\"async\""
    );
    assert_eq!(
        serde_json::to_string(&SyncMode::Off).expect("Failed"),
        "\"off\""
    );
}

#[test]
fn test_sync_config_serde_roundtrip() {
    let config = SyncConfig {
        mode: SyncMode::Sync,
        queue_size: 5000,
        commit_interval_ms: 500,
        batch_size: 50,
    };

    let json = serde_json::to_string(&config).expect("Failed to serialize");
    let decoded: SyncConfig = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(config.mode, decoded.mode);
    assert_eq!(config.queue_size, decoded.queue_size);
    assert_eq!(config.commit_interval_ms, decoded.commit_interval_ms);
    assert_eq!(config.batch_size, decoded.batch_size);
}

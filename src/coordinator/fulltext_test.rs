#[cfg(test)]
mod tests {
    use super::super::fulltext::{FulltextCoordinator, ChangeType};
    use crate::search::manager::FulltextIndexManager;
    use crate::search::config::FulltextConfig;
    use crate::search::engine::EngineType;
    use crate::core::{Value, Vertex, Tag};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tempfile::TempDir;

    async fn create_test_coordinator() -> (FulltextCoordinator, TempDir) {
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
        let coordinator = FulltextCoordinator::new(manager);
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
    async fn test_coordinator_create_and_search() {
        let (coordinator, _temp) = create_test_coordinator().await;

        // Create index
        let index_id = coordinator
            .create_index(1, "Article", "title", Some(EngineType::Bm25))
            .await
            .expect("Failed to create index");

        assert!(!index_id.is_empty(), "Index ID should not be empty");

        // Simulate vertex insertion
        let vertex = create_test_vertex(1, "Article", vec![("title", "Hello World")]);
        coordinator.on_vertex_inserted(1, &vertex).await.expect("Failed to insert vertex");
        coordinator.commit_all().await.expect("Failed to commit");

        // Search
        let results = coordinator.search(1, "Article", "title", "Hello", 10).await.expect("Failed to search");
        assert_eq!(results.len(), 1, "Expected 1 result");
    }

    #[tokio::test]
    async fn test_coordinator_multiple_indexes() {
        let (coordinator, _temp) = create_test_coordinator().await;

        // Create multiple indexes
        coordinator.create_index(1, "Post", "title", None).await.expect("Failed to create title index");
        coordinator.create_index(1, "Post", "content", None).await.expect("Failed to create content index");

        // Insert data
        let vertex = create_test_vertex(1, "Post", vec![
            ("title", "Rust Tutorial"),
            ("content", "Learn Rust programming"),
        ]);
        coordinator.on_vertex_inserted(1, &vertex).await.expect("Failed to insert vertex");
        coordinator.commit_all().await.expect("Failed to commit");

        // Search separately
        let title_results = coordinator.search(1, "Post", "title", "Rust", 10).await.expect("Failed to search title");
        let content_results = coordinator.search(1, "Post", "content", "programming", 10).await.expect("Failed to search content");

        assert_eq!(title_results.len(), 1, "Expected 1 title result");
        assert_eq!(content_results.len(), 1, "Expected 1 content result");
    }

    #[tokio::test]
    async fn test_coordinator_vertex_update() {
        let (coordinator, _temp) = create_test_coordinator().await;

        // Create index
        coordinator.create_index(1, "Article", "title", None).await.expect("Failed to create index");

        // Insert vertex
        let vertex = create_test_vertex(1, "Article", vec![("title", "Original Title")]);
        coordinator.on_vertex_inserted(1, &vertex).await.expect("Failed to insert vertex");
        coordinator.commit_all().await.expect("Failed to commit");

        // Verify original
        let results = coordinator.search(1, "Article", "title", "Original", 10).await.expect("Failed to search");
        assert_eq!(results.len(), 1, "Expected 1 result for original");

        // Update vertex
        let updated_vertex = create_test_vertex(1, "Article", vec![("title", "Updated Title")]);
        coordinator.on_vertex_updated(1, &updated_vertex, &["title".to_string()]).await.expect("Failed to update vertex");
        coordinator.commit_all().await.expect("Failed to commit");

        // Verify old content not found
        let results = coordinator.search(1, "Article", "title", "Original", 10).await.expect("Failed to search");
        assert_eq!(results.len(), 0, "Expected 0 results for old content");

        // Verify new content found
        let results = coordinator.search(1, "Article", "title", "Updated", 10).await.expect("Failed to search");
        assert_eq!(results.len(), 1, "Expected 1 result for new content");
    }

    #[tokio::test]
    async fn test_coordinator_vertex_delete() {
        let (coordinator, _temp) = create_test_coordinator().await;

        // Create index
        coordinator.create_index(1, "Article", "title", None).await.expect("Failed to create index");

        // Insert vertex
        let vertex = create_test_vertex(1, "Article", vec![("title", "To Be Deleted")]);
        coordinator.on_vertex_inserted(1, &vertex).await.expect("Failed to insert vertex");
        coordinator.commit_all().await.expect("Failed to commit");

        // Verify exists
        let results = coordinator.search(1, "Article", "title", "Deleted", 10).await.expect("Failed to search");
        assert_eq!(results.len(), 1, "Expected 1 result before delete");

        // Delete vertex
        coordinator.on_vertex_deleted(1, "Article", &Value::Int(1)).await.expect("Failed to delete vertex");
        coordinator.commit_all().await.expect("Failed to commit");

        // Verify deleted
        let results = coordinator.search(1, "Article", "title", "Deleted", 10).await.expect("Failed to search");
        assert_eq!(results.len(), 0, "Expected 0 results after delete");
    }

    #[tokio::test]
    async fn test_coordinator_list_indexes() {
        let (coordinator, _temp) = create_test_coordinator().await;

        // Initially empty
        let indexes = coordinator.list_indexes();
        assert!(indexes.is_empty(), "Expected no indexes initially");

        // Create indexes
        coordinator.create_index(1, "Article", "title", None).await.expect("Failed to create index");
        coordinator.create_index(1, "Article", "content", None).await.expect("Failed to create index");
        coordinator.create_index(2, "Post", "title", None).await.expect("Failed to create index");

        // List indexes
        let indexes = coordinator.list_indexes();
        assert_eq!(indexes.len(), 3, "Expected 3 indexes");
    }

    #[tokio::test]
    async fn test_coordinator_drop_index() {
        let (coordinator, _temp) = create_test_coordinator().await;

        // Create index
        coordinator.create_index(1, "Article", "title", None).await.expect("Failed to create index");

        // Insert data
        let vertex = create_test_vertex(1, "Article", vec![("title", "Test Title")]);
        coordinator.on_vertex_inserted(1, &vertex).await.expect("Failed to insert vertex");
        coordinator.commit_all().await.expect("Failed to commit");

        // Verify exists
        let results = coordinator.search(1, "Article", "title", "Test", 10).await.expect("Failed to search");
        assert_eq!(results.len(), 1, "Expected 1 result");

        // Drop index
        coordinator.drop_index(1, "Article", "title").await.expect("Failed to drop index");

        // Verify dropped - search should fail
        let result = coordinator.search(1, "Article", "title", "Test", 10).await;
        assert!(result.is_err(), "Expected error after dropping index");
    }

    #[tokio::test]
    async fn test_coordinator_on_vertex_change() {
        let (coordinator, _temp) = create_test_coordinator().await;

        // Create index
        coordinator.create_index(1, "Article", "title", None).await.expect("Failed to create index");

        let mut properties = HashMap::new();
        properties.insert("title".to_string(), Value::String("Test Title".to_string()));

        // Test insert
        coordinator.on_vertex_change(1, "Article", &Value::Int(1), &properties, ChangeType::Insert)
            .await
            .expect("Failed to handle insert");
        coordinator.commit_all().await.expect("Failed to commit");

        let results = coordinator.search(1, "Article", "title", "Test", 10).await.expect("Failed to search");
        assert_eq!(results.len(), 1, "Expected 1 result after insert");

        // Test update
        properties.insert("title".to_string(), Value::String("Updated Title".to_string()));
        coordinator.on_vertex_change(1, "Article", &Value::Int(1), &properties, ChangeType::Update)
            .await
            .expect("Failed to handle update");
        coordinator.commit_all().await.expect("Failed to commit");

        let results = coordinator.search(1, "Article", "title", "Updated", 10).await.expect("Failed to search");
        assert_eq!(results.len(), 1, "Expected 1 result after update");

        // Test delete
        coordinator.on_vertex_change(1, "Article", &Value::Int(1), &properties, ChangeType::Delete)
            .await
            .expect("Failed to handle delete");
        coordinator.commit_all().await.expect("Failed to commit");

        let results = coordinator.search(1, "Article", "title", "Updated", 10).await.expect("Failed to search");
        assert_eq!(results.len(), 0, "Expected 0 results after delete");
    }

    #[tokio::test]
    async fn test_coordinator_rebuild_index() {
        let (coordinator, _temp) = create_test_coordinator().await;

        // Create index and insert data
        coordinator.create_index(1, "Article", "title", None).await.expect("Failed to create index");
        let vertex = create_test_vertex(1, "Article", vec![("title", "Test Title")]);
        coordinator.on_vertex_inserted(1, &vertex).await.expect("Failed to insert vertex");
        coordinator.commit_all().await.expect("Failed to commit");

        // Rebuild index
        coordinator.rebuild_index(1, "Article", "title").await.expect("Failed to rebuild index");

        // Verify data still searchable
        let results = coordinator.search(1, "Article", "title", "Test", 10).await.expect("Failed to search");
        assert_eq!(results.len(), 1, "Expected 1 result after rebuild");
    }

    #[tokio::test]
    async fn test_coordinator_get_engine() {
        let (coordinator, _temp) = create_test_coordinator().await;

        // No engine before creation
        let engine = coordinator.get_engine(1, "Article", "title");
        assert!(engine.is_none(), "Expected no engine before creation");

        // Create index
        coordinator.create_index(1, "Article", "title", None).await.expect("Failed to create index");

        // Engine should exist
        let engine = coordinator.get_engine(1, "Article", "title");
        assert!(engine.is_some(), "Expected engine after creation");
    }

    #[tokio::test]
    async fn test_coordinator_multiple_spaces() {
        let (coordinator, _temp) = create_test_coordinator().await;

        // Create indexes in different spaces
        coordinator.create_index(1, "Article", "title", None).await.expect("Failed to create index in space 1");
        coordinator.create_index(2, "Article", "title", None).await.expect("Failed to create index in space 2");

        // Insert data in space 1
        let vertex1 = create_test_vertex(1, "Article", vec![("title", "Space 1 Title")]);
        coordinator.on_vertex_inserted(1, &vertex1).await.expect("Failed to insert vertex in space 1");

        // Insert data in space 2
        let vertex2 = create_test_vertex(1, "Article", vec![("title", "Space 2 Title")]);
        coordinator.on_vertex_inserted(2, &vertex2).await.expect("Failed to insert vertex in space 2");

        coordinator.commit_all().await.expect("Failed to commit");

        // Search in space 1
        let results = coordinator.search(1, "Article", "title", "Space", 10).await.expect("Failed to search space 1");
        assert_eq!(results.len(), 1, "Expected 1 result in space 1");

        // Search in space 2
        let results = coordinator.search(2, "Article", "title", "Space", 10).await.expect("Failed to search space 2");
        assert_eq!(results.len(), 1, "Expected 1 result in space 2");
    }
}

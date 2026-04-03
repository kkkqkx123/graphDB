use std::sync::Arc;
use crate::coordinator::FulltextCoordinator;

/// Index warmer for preloading frequently accessed indexes
pub struct IndexWarmer {
    coordinator: Arc<FulltextCoordinator>,
}

impl IndexWarmer {
    pub fn new(coordinator: Arc<FulltextCoordinator>) -> Self {
        Self { coordinator }
    }

    /// Warm up common queries
    pub async fn warm_common_queries(&self) {
        let common_queries = vec![
            (1, "Post", "content", "tutorial"),
            (1, "Article", "title", "Rust"),
            (1, "User", "name", "admin"),
        ];

        for (space_id, tag, field, query) in common_queries {
            // Execute search to load index into memory
            let _ = self.coordinator.search(space_id, tag, field, query, 10).await;
        }
    }

    /// Warm up specific index
    pub async fn warm_index(&self, space_id: u64, tag: &str, field: &str) {
        if let Some(engine) = self.coordinator.get_engine(space_id, tag, field) {
            // Execute wildcard search to load index structure
            let _ = engine.search("*", 1).await;
        }
    }

    /// Warm up all indexes in a space
    pub async fn warm_space(&self, space_id: u64) {
        let indexes = self.coordinator.list_indexes();
        for metadata in indexes {
            if metadata.space_id == space_id {
                self.warm_index(space_id, &metadata.tag_name, &metadata.field_name).await;
            }
        }
    }

    /// Warm up with custom query patterns
    pub async fn warm_with_patterns(&self, space_id: u64, tag: &str, field: &str, patterns: Vec<&str>) {
        for pattern in patterns {
            let _ = self.coordinator.search(space_id, tag, field, pattern, 10).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::manager::FulltextIndexManager;
    use crate::search::config::FulltextConfig;
    use crate::search::engine::EngineType;
    use crate::core::{Value, Vertex, Tag};
    use std::collections::HashMap;
    use tempfile::TempDir;

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

    async fn setup_test_coordinator() -> (Arc<FulltextCoordinator>, TempDir) {
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
        let coordinator = Arc::new(FulltextCoordinator::new(manager));
        (coordinator, temp_dir)
    }

    #[tokio::test]
    async fn test_warm_index() {
        let (coordinator, _temp) = setup_test_coordinator().await;

        // Create index and insert data
        coordinator.create_index(1, "Article", "title", Some(EngineType::Bm25))
            .await
            .expect("Failed to create index");

        let vertex = create_test_vertex(1, "Article", vec![("title", "Test Article")]);
        coordinator.on_vertex_inserted(1, &vertex).await.expect("Failed to insert");
        coordinator.commit_all().await.expect("Failed to commit");

        // Warm up index
        let warmer = IndexWarmer::new(coordinator.clone());
        warmer.warm_index(1, "Article", "title").await;

        // After warming, search should work
        let results = coordinator.search(1, "Article", "title", "Test", 10).await.expect("Failed to search");
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_warm_space() {
        let (coordinator, _temp) = setup_test_coordinator().await;

        // Create multiple indexes
        coordinator.create_index(1, "Article", "title", Some(EngineType::Bm25))
            .await
            .expect("Failed to create index");
        coordinator.create_index(1, "Article", "content", Some(EngineType::Bm25))
            .await
            .expect("Failed to create index");

        // Insert data
        let vertex = create_test_vertex(1, "Article", vec![
            ("title", "Test Title"),
            ("content", "Test Content"),
        ]);
        coordinator.on_vertex_inserted(1, &vertex).await.expect("Failed to insert");
        coordinator.commit_all().await.expect("Failed to commit");

        // Warm up entire space
        let warmer = IndexWarmer::new(coordinator.clone());
        warmer.warm_space(1).await;

        // Both indexes should be searchable
        let title_results = coordinator.search(1, "Article", "title", "Test", 10).await.expect("Failed to search title");
        let content_results = coordinator.search(1, "Article", "content", "Test", 10).await.expect("Failed to search content");

        assert_eq!(title_results.len(), 1);
        assert_eq!(content_results.len(), 1);
    }

    #[tokio::test]
    async fn test_warm_with_patterns() {
        let (coordinator, _temp) = setup_test_coordinator().await;

        // Create index and insert data
        coordinator.create_index(1, "Article", "title", Some(EngineType::Bm25))
            .await
            .expect("Failed to create index");

        for i in 0..10 {
            let vertex = create_test_vertex(i as i64, "Article", vec![("title", &format!("Article {}", i))]);
            coordinator.on_vertex_inserted(1, &vertex).await.expect("Failed to insert");
        }
        coordinator.commit_all().await.expect("Failed to commit");

        // Warm up with specific patterns
        let warmer = IndexWarmer::new(coordinator.clone());
        warmer.warm_with_patterns(1, "Article", "title", vec!["Article 1", "Article 5", "Article 9"]).await;

        // All patterns should be searchable
        let results = coordinator.search(1, "Article", "title", "Article", 10).await.expect("Failed to search");
        assert_eq!(results.len(), 10);
    }
}

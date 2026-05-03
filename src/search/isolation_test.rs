//! Space Isolation and Index Naming Tests
//!
//! Test scope:
//! - Index naming consistency (vector and fulltext)
//! - Space isolation levels (Shared, Directory, Device)
//! - Space existence validation
//! - Tag existence validation
//! - Storage path configuration

#[cfg(test)]
mod tests {
    use crate::core::types::{IsolationLevel, SpaceInfo, TagInfo, EdgeTypeInfo, Index};
    use crate::search::metadata::IndexKey;
    use crate::search::{
        EngineType, FulltextConfig, FulltextIndexManager, SearchError,
    };
    use crate::storage::metadata::SchemaManager;
    use crate::storage::Schema;
    use crate::core::StorageError;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Arc;
    use tempfile::TempDir;

    const FULLTEXT_INDEX_PREFIX: &str = "space_ft";
    const VECTOR_INDEX_PREFIX: &str = "space_vec";

    // ==================== Index Naming Tests ====================

    /// Test fulltext index ID format: space_ft_{space_id}_{tag}_{field}
    #[test]
    fn test_fulltext_index_naming_format() {
        let key = IndexKey::new(1, "Article", "content");
        let index_id = key.to_index_id();

        assert_eq!(index_id, "space_ft_1_Article_content");
        assert!(index_id.starts_with(FULLTEXT_INDEX_PREFIX));
    }

    /// Test vector index naming format: space_vec_{space_id}_{tag}_{field}
    #[test]
    fn test_vector_index_naming_format() {
        use crate::sync::vector_sync::VectorIndexLocation;

        let location = VectorIndexLocation::new(1, "Article", "content");
        let collection_name = location.to_collection_name();

        assert_eq!(collection_name, "space_vec_1_Article_content");
        assert!(collection_name.starts_with(VECTOR_INDEX_PREFIX));
    }

    /// Test index naming with special characters in tag/field names
    #[test]
    fn test_index_naming_with_special_chars() {
        let key = IndexKey::new(1, "User_Profile", "email_address");
        let index_id = key.to_index_id();

        assert_eq!(index_id, "space_ft_1_User_Profile_email_address");
    }

    /// Test index naming consistency between vector and fulltext
    #[test]
    fn test_index_naming_consistency() {
        use crate::sync::vector_sync::VectorIndexLocation;

        let space_id = 42;
        let tag = "Product";
        let field = "description";

        let ft_key = IndexKey::new(space_id, tag, field);
        let ft_index_id = ft_key.to_index_id();

        let vec_location = VectorIndexLocation::new(space_id, tag, field);
        let vec_collection = vec_location.to_collection_name();

        // Both should contain the same components
        assert!(ft_index_id.contains(&space_id.to_string()));
        assert!(ft_index_id.contains(tag));
        assert!(ft_index_id.contains(field));

        assert!(vec_collection.contains(&space_id.to_string()));
        assert!(vec_collection.contains(tag));
        assert!(vec_collection.contains(field));

        // Prefixes should be different
        assert!(ft_index_id.starts_with("space_ft_"));
        assert!(vec_collection.starts_with("space_vec_"));
    }

    // ==================== SpaceInfo Isolation Level Tests ====================

    /// Test default isolation level is Shared
    #[test]
    fn test_default_isolation_level() {
        let space = SpaceInfo::new("test_space".to_string());

        assert_eq!(space.isolation_level, IsolationLevel::Shared);
        assert!(space.storage_path.is_none());
    }

    /// Test Directory isolation level
    #[test]
    fn test_directory_isolation_level() {
        let space = SpaceInfo::new("test_space".to_string())
            .with_isolation_level(IsolationLevel::Directory);

        assert_eq!(space.isolation_level, IsolationLevel::Directory);
    }

    /// Test Device isolation level with custom path
    #[test]
    fn test_device_isolation_level_with_path() {
        let custom_path = PathBuf::from("/custom/storage/path");
        let space = SpaceInfo::new("test_space".to_string())
            .with_storage_path(Some(custom_path.clone()));

        assert_eq!(space.isolation_level, IsolationLevel::Device);
        assert_eq!(space.storage_path, Some(custom_path));
    }

    /// Test setting isolation level explicitly
    #[test]
    fn test_explicit_isolation_level() {
        let space = SpaceInfo::new("test_space".to_string())
            .with_isolation_level(IsolationLevel::Device);

        assert_eq!(space.isolation_level, IsolationLevel::Device);
    }

    // ==================== Mock SchemaManager ====================

    #[derive(Debug)]
    struct MockSchemaManager {
        spaces: HashMap<u64, SpaceInfo>,
    }

    impl MockSchemaManager {
        fn new() -> Self {
            Self {
                spaces: HashMap::new(),
            }
        }

        fn add_space(&mut self, space: SpaceInfo) {
            self.spaces.insert(space.space_id, space);
        }
    }

    impl SchemaManager for MockSchemaManager {
        fn create_space(&self, _space: &mut SpaceInfo) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn drop_space(&self, _space_name: &str) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn get_space(&self, space_name: &str) -> Result<Option<SpaceInfo>, StorageError> {
            for space in self.spaces.values() {
                if space.space_name == space_name {
                    return Ok(Some(space.clone()));
                }
            }
            Ok(None)
        }

        fn get_space_by_id(&self, space_id: u64) -> Result<Option<SpaceInfo>, StorageError> {
            Ok(self.spaces.get(&space_id).cloned())
        }

        fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError> {
            Ok(self.spaces.values().cloned().collect())
        }

        fn update_space(&self, _space: &SpaceInfo) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn create_tag(&self, _space: &str, _tag: &TagInfo) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn get_tag(&self, space: &str, tag_name: &str) -> Result<Option<TagInfo>, StorageError> {
            if let Ok(Some(space_info)) = self.get_space(space) {
                return Ok(space_info.tags.iter().find(|t| t.tag_name == tag_name).cloned());
            }
            Ok(None)
        }

        fn list_tags(&self, _space: &str) -> Result<Vec<TagInfo>, StorageError> {
            Ok(vec![])
        }

        fn drop_tag(&self, _space: &str, _tag_name: &str) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn update_tag(&self, _space: &str, _tag: &TagInfo) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn create_edge_type(&self, _space: &str, _edge: &EdgeTypeInfo) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn get_edge_type(
            &self,
            _space: &str,
            _edge_type_name: &str,
        ) -> Result<Option<EdgeTypeInfo>, StorageError> {
            Ok(None)
        }

        fn list_edge_types(&self, _space: &str) -> Result<Vec<EdgeTypeInfo>, StorageError> {
            Ok(vec![])
        }

        fn drop_edge_type(&self, _space: &str, _edge_type_name: &str) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn update_edge_type(&self, _space: &str, _edge: &EdgeTypeInfo) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn get_tag_schema(&self, _space: &str, _tag: &str) -> Result<Schema, StorageError> {
            Ok(Schema::new("test".to_string(), 1))
        }

        fn get_edge_type_schema(&self, _space: &str, _edge: &str) -> Result<Schema, StorageError> {
            Ok(Schema::new("test".to_string(), 1))
        }

        fn list_tag_indexes(&self, _space: &str) -> Result<Vec<Index>, StorageError> {
            Ok(vec![])
        }

        fn list_edge_indexes(&self, _space: &str) -> Result<Vec<Index>, StorageError> {
            Ok(vec![])
        }
    }

    // ==================== Space Existence Validation Tests ====================

    /// Test index creation fails when space does not exist
    #[tokio::test]
    async fn test_create_index_fails_when_space_not_exists() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = FulltextConfig {
            enabled: true,
            index_path: temp_dir.path().to_path_buf(),
            default_engine: EngineType::Bm25,
            ..Default::default()
        };

        let mut mock_schema = MockSchemaManager::new();
        // Add a space with ID 1, but we'll try to create index for space 999
        mock_schema.add_space(SpaceInfo::new("existing_space".to_string()));

        let manager = FulltextIndexManager::new(config)
            .expect("Failed to create manager")
            .with_schema_manager(Arc::new(mock_schema));

        let result = manager.create_index(999, "Article", "content", None).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SearchError::SpaceNotFound(999)));
    }

    /// Test index creation succeeds when space exists
    #[tokio::test]
    async fn test_create_index_succeeds_when_space_exists() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = FulltextConfig {
            enabled: true,
            index_path: temp_dir.path().to_path_buf(),
            default_engine: EngineType::Bm25,
            ..Default::default()
        };

        let mut mock_schema = MockSchemaManager::new();
        let mut space = SpaceInfo::new("test_space".to_string());
        space.space_id = 1;
        // Add a tag to the space
        space.tags.push(TagInfo::new("Article".to_string()));
        mock_schema.add_space(space);

        let manager = FulltextIndexManager::new(config)
            .expect("Failed to create manager")
            .with_schema_manager(Arc::new(mock_schema));

        let result = manager.create_index(1, "Article", "content", None).await;

        assert!(result.is_ok());
    }

    /// Test index creation fails when tag does not exist
    #[tokio::test]
    async fn test_create_index_fails_when_tag_not_exists() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = FulltextConfig {
            enabled: true,
            index_path: temp_dir.path().to_path_buf(),
            default_engine: EngineType::Bm25,
            ..Default::default()
        };

        let mut mock_schema = MockSchemaManager::new();
        let mut space = SpaceInfo::new("test_space".to_string());
        space.space_id = 1;
        // Space exists but has no tags
        mock_schema.add_space(space);

        let manager = FulltextIndexManager::new(config)
            .expect("Failed to create manager")
            .with_schema_manager(Arc::new(mock_schema));

        let result = manager.create_index(1, "NonExistentTag", "content", None).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SearchError::TagNotFound(_)));
    }

    // ==================== Storage Path Tests ====================

    /// Test shared isolation level uses base path
    #[tokio::test]
    async fn test_shared_isolation_uses_base_path() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path().to_path_buf();

        let config = FulltextConfig {
            enabled: true,
            index_path: base_path.clone(),
            default_engine: EngineType::Bm25,
            ..Default::default()
        };

        let mut mock_schema = MockSchemaManager::new();
        let mut space = SpaceInfo::new("test_space".to_string());
        space.space_id = 1;
        space.isolation_level = IsolationLevel::Shared;
        space.tags.push(TagInfo::new("Article".to_string()));
        mock_schema.add_space(space);

        let manager = FulltextIndexManager::new(config)
            .expect("Failed to create manager")
            .with_schema_manager(Arc::new(mock_schema));

        let _index_id = manager
            .create_index(1, "Article", "content", None)
            .await
            .expect("Failed to create index");

        let metadata = manager
            .get_metadata(1, "Article", "content")
            .expect("Metadata should exist");

        // Storage path should be under base_path
        assert!(metadata.storage_path.starts_with(base_path.to_string_lossy().as_ref()));
    }

    /// Test directory isolation level creates subdirectory
    #[tokio::test]
    async fn test_directory_isolation_creates_subdirectory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path().to_path_buf();

        let config = FulltextConfig {
            enabled: true,
            index_path: base_path.clone(),
            default_engine: EngineType::Bm25,
            ..Default::default()
        };

        let mut mock_schema = MockSchemaManager::new();
        let mut space = SpaceInfo::new("test_space".to_string());
        space.space_id = 1;
        space.isolation_level = IsolationLevel::Directory;
        space.tags.push(TagInfo::new("Article".to_string()));
        mock_schema.add_space(space);

        let manager = FulltextIndexManager::new(config)
            .expect("Failed to create manager")
            .with_schema_manager(Arc::new(mock_schema));

        let _index_id = manager
            .create_index(1, "Article", "content", None)
            .await
            .expect("Failed to create index");

        let metadata = manager
            .get_metadata(1, "Article", "content")
            .expect("Metadata should exist");

        // Storage path should contain space_1 subdirectory
        assert!(metadata.storage_path.contains("space_1"));
    }

    /// Test device isolation level uses custom path
    #[tokio::test]
    async fn test_device_isolation_uses_custom_path() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let base_path = temp_dir.path().to_path_buf();
        let custom_path = temp_dir.path().join("custom_device");

        let config = FulltextConfig {
            enabled: true,
            index_path: base_path,
            default_engine: EngineType::Bm25,
            ..Default::default()
        };

        let mut mock_schema = MockSchemaManager::new();
        let mut space = SpaceInfo::new("test_space".to_string());
        space.space_id = 1;
        space.isolation_level = IsolationLevel::Device;
        space.storage_path = Some(custom_path.clone());
        space.tags.push(TagInfo::new("Article".to_string()));
        mock_schema.add_space(space);

        let manager = FulltextIndexManager::new(config)
            .expect("Failed to create manager")
            .with_schema_manager(Arc::new(mock_schema));

        let _index_id = manager
            .create_index(1, "Article", "content", None)
            .await
            .expect("Failed to create index");

        let metadata = manager
            .get_metadata(1, "Article", "content")
            .expect("Metadata should exist");

        // Storage path should be under custom_path/fulltext
        assert!(metadata.storage_path.starts_with(custom_path.to_string_lossy().as_ref()));
    }

    // ==================== Drop Space Indexes Tests ====================

    /// Test drop_space_indexes removes all indexes for a space
    #[tokio::test]
    async fn test_drop_space_indexes_removes_all() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = FulltextConfig {
            enabled: true,
            index_path: temp_dir.path().to_path_buf(),
            default_engine: EngineType::Bm25,
            ..Default::default()
        };

        let mut mock_schema = MockSchemaManager::new();
        let mut space = SpaceInfo::new("test_space".to_string());
        space.space_id = 1;
        space.tags.push(TagInfo::new("Article".to_string()));
        space.tags.push(TagInfo::new("Product".to_string()));
        mock_schema.add_space(space);

        let manager = FulltextIndexManager::new(config)
            .expect("Failed to create manager")
            .with_schema_manager(Arc::new(mock_schema));

        // Create multiple indexes
        manager
            .create_index(1, "Article", "title", None)
            .await
            .expect("Failed to create index 1");
        manager
            .create_index(1, "Article", "content", None)
            .await
            .expect("Failed to create index 2");
        manager
            .create_index(1, "Product", "description", None)
            .await
            .expect("Failed to create index 3");

        // Verify indexes exist
        assert_eq!(manager.get_space_indexes(1).len(), 3);

        // Drop all indexes for space 1
        manager
            .drop_space_indexes(1)
            .await
            .expect("Failed to drop space indexes");

        // Verify all indexes are removed
        assert_eq!(manager.get_space_indexes(1).len(), 0);
        assert!(!manager.has_index(1, "Article", "title"));
        assert!(!manager.has_index(1, "Article", "content"));
        assert!(!manager.has_index(1, "Product", "description"));
    }

    /// Test drop_space_indexes only affects specified space
    #[tokio::test]
    async fn test_drop_space_indexes_isolation() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = FulltextConfig {
            enabled: true,
            index_path: temp_dir.path().to_path_buf(),
            default_engine: EngineType::Bm25,
            ..Default::default()
        };

        let mut mock_schema = MockSchemaManager::new();

        // Space 1
        let mut space1 = SpaceInfo::new("space1".to_string());
        space1.space_id = 1;
        space1.tags.push(TagInfo::new("Article".to_string()));
        mock_schema.add_space(space1);

        // Space 2
        let mut space2 = SpaceInfo::new("space2".to_string());
        space2.space_id = 2;
        space2.tags.push(TagInfo::new("Product".to_string()));
        mock_schema.add_space(space2);

        let manager = FulltextIndexManager::new(config)
            .expect("Failed to create manager")
            .with_schema_manager(Arc::new(mock_schema));

        // Create indexes for both spaces
        manager
            .create_index(1, "Article", "content", None)
            .await
            .expect("Failed to create space 1 index");
        manager
            .create_index(2, "Product", "description", None)
            .await
            .expect("Failed to create space 2 index");

        // Drop indexes for space 1 only
        manager
            .drop_space_indexes(1)
            .await
            .expect("Failed to drop space 1 indexes");

        // Verify space 1 indexes are removed
        assert!(!manager.has_index(1, "Article", "content"));

        // Verify space 2 indexes still exist
        assert!(manager.has_index(2, "Product", "description"));
    }

    // ==================== Edge Case Tests ====================

    /// Test index creation with space_id = 0
    #[test]
    fn test_index_naming_with_zero_space_id() {
        let key = IndexKey::new(0, "Tag", "field");
        let index_id = key.to_index_id();

        assert_eq!(index_id, "space_ft_0_Tag_field");
    }

    /// Test without schema manager (backward compatibility)
    #[tokio::test]
    async fn test_create_index_without_schema_manager() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = FulltextConfig {
            enabled: true,
            index_path: temp_dir.path().to_path_buf(),
            default_engine: EngineType::Bm25,
            ..Default::default()
        };

        // Create manager without schema manager
        let manager = FulltextIndexManager::new(config).expect("Failed to create manager");

        // Should succeed without validation
        let result = manager.create_index(1, "Article", "content", None).await;
        assert!(result.is_ok());
    }

    /// Test concurrent index creation for same space
    #[tokio::test]
    async fn test_concurrent_index_creation() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config = FulltextConfig {
            enabled: true,
            index_path: temp_dir.path().to_path_buf(),
            default_engine: EngineType::Bm25,
            ..Default::default()
        };

        let mut mock_schema = MockSchemaManager::new();
        let mut space = SpaceInfo::new("test_space".to_string());
        space.space_id = 1;
        for i in 0..5 {
            space.tags.push(TagInfo::new(format!("Tag{}", i)));
        }
        mock_schema.add_space(space);

        let manager = Arc::new(
            FulltextIndexManager::new(config)
                .expect("Failed to create manager")
                .with_schema_manager(Arc::new(mock_schema)),
        );

        // Create multiple indexes concurrently
        let mut handles = vec![];
        for i in 0..5 {
            let mgr = manager.clone();
            let handle = tokio::spawn(async move {
                mgr.create_index(1, &format!("Tag{}", i), "field", None)
                    .await
            });
            handles.push(handle);
        }

        // All should succeed
        for handle in handles {
            let result = handle.await.expect("Task failed");
            assert!(result.is_ok());
        }

        // Verify all indexes exist
        assert_eq!(manager.get_space_indexes(1).len(), 5);
    }
}

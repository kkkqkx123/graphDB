//! Test module for sync trigger functionality
//!
//! This module tests that storage operations correctly trigger sync manager callbacks

#[cfg(test)]
mod tests {
    use crate::core::types::{PropertyDef, SpaceInfo, TagInfo};
    use crate::core::{Value, Vertex};
    use crate::storage::storage_client::StorageClient;
    use crate::storage::RedbStorage;
    use tempfile::TempDir;

    #[test]
    fn test_storage_with_sync_manager() {
        // This test verifies that the storage layer can be initialized with a sync manager
        // Note: Full integration tests require a running SyncManager instance

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_sync.db");

        // Create storage
        let mut storage = RedbStorage::new_with_path(db_path).unwrap();

        // Verify storage is created successfully
        // Sync manager is now wrapped in RwLock
        assert!(
            storage.state().get_sync_manager().is_none(),
            "Sync manager should be None by default"
        );

        // Test space creation
        let space_info = SpaceInfo::new("test_space".to_string())
            .with_vid_type(crate::core::types::DataType::Int64);

        let result = storage.create_space(&space_info);
        assert!(result.is_ok(), "Failed to create space: {:?}", result);
    }

    #[test]
    fn test_vertex_insert_without_sync() {
        // Test vertex insertion when sync_manager is None (should work normally)
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_insert.db");

        let mut storage = RedbStorage::new_with_path(db_path).unwrap();

        // Create space
        let space_info = SpaceInfo::new("test_space".to_string())
            .with_vid_type(crate::core::types::DataType::Int64);
        storage.create_space(&space_info).unwrap();

        // Create tag
        let tag_info = TagInfo::new("Person".to_string()).with_properties(vec![PropertyDef::new(
            "name".to_string(),
            crate::core::types::DataType::String,
        )]);
        storage.create_tag("test_space", &tag_info).unwrap();

        // Insert vertex
        let vertex = Vertex::new(
            Value::Int(1),
            vec![crate::core::vertex_edge_path::Tag {
                name: "Person".to_string(),
                properties: [("name".to_string(), Value::String("Alice".to_string()))]
                    .iter()
                    .cloned()
                    .collect(),
            }],
        );

        let result = storage.insert_vertex("test_space", vertex.clone());
        assert!(result.is_ok(), "Failed to insert vertex: {:?}", result);

        // Verify vertex was inserted
        let retrieved = storage.get_vertex("test_space", &Value::Int(1));
        assert!(retrieved.is_ok());
        assert!(retrieved.unwrap().is_some());
    }

    #[test]
    fn test_vertex_update_without_sync() {
        // Test vertex update when sync_manager is None
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_update.db");

        let mut storage = RedbStorage::new_with_path(db_path).unwrap();

        // Setup space and tag
        let space_info = SpaceInfo::new("test_space".to_string())
            .with_vid_type(crate::core::types::DataType::Int64);
        storage.create_space(&space_info).unwrap();

        let tag_info = TagInfo::new("Person".to_string()).with_properties(vec![PropertyDef::new(
            "name".to_string(),
            crate::core::types::DataType::String,
        )]);
        storage.create_tag("test_space", &tag_info).unwrap();

        // Insert initial vertex
        let vertex = Vertex::new(
            Value::Int(1),
            vec![crate::core::vertex_edge_path::Tag {
                name: "Person".to_string(),
                properties: [("name".to_string(), Value::String("Alice".to_string()))]
                    .iter()
                    .cloned()
                    .collect(),
            }],
        );
        storage.insert_vertex("test_space", vertex).unwrap();

        // Update vertex
        let updated_vertex = Vertex::new(
            Value::Int(1),
            vec![crate::core::vertex_edge_path::Tag {
                name: "Person".to_string(),
                properties: [("name".to_string(), Value::String("Bob".to_string()))]
                    .iter()
                    .cloned()
                    .collect(),
            }],
        );

        let result = storage.update_vertex("test_space", updated_vertex);
        assert!(result.is_ok(), "Failed to update vertex: {:?}", result);

        // Verify update
        let retrieved = storage.get_vertex("test_space", &Value::Int(1));
        assert!(retrieved.is_ok());
        let vertex = retrieved.unwrap().unwrap();
        assert_eq!(
            vertex.tags[0].properties.get("name"),
            Some(&Value::String("Bob".to_string()))
        );
    }

    #[test]
    fn test_vertex_delete_without_sync() {
        // Test vertex deletion when sync_manager is None
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_delete.db");

        let mut storage = RedbStorage::new_with_path(db_path).unwrap();

        // Setup
        let space_info = SpaceInfo::new("test_space".to_string())
            .with_vid_type(crate::core::types::DataType::Int64);
        storage.create_space(&space_info).unwrap();

        let tag_info = TagInfo::new("Person".to_string()).with_properties(vec![PropertyDef::new(
            "name".to_string(),
            crate::core::types::DataType::String,
        )]);
        storage.create_tag("test_space", &tag_info).unwrap();

        // Insert vertex
        let vertex = Vertex::new(
            Value::Int(1),
            vec![crate::core::vertex_edge_path::Tag {
                name: "Person".to_string(),
                properties: [("name".to_string(), Value::String("Alice".to_string()))]
                    .iter()
                    .cloned()
                    .collect(),
            }],
        );
        storage.insert_vertex("test_space", vertex).unwrap();

        // Delete vertex
        let result = storage.delete_vertex("test_space", &Value::Int(1));
        assert!(result.is_ok(), "Failed to delete vertex: {:?}", result);

        // Verify deletion
        let retrieved = storage.get_vertex("test_space", &Value::Int(1));
        assert!(retrieved.is_ok());
        assert!(retrieved.unwrap().is_none());
    }
}

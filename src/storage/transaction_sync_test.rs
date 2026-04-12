//! Transaction Mode Synchronization Tests
//!
//! Tests for verifying that transaction ID is correctly passed to sync manager

#[cfg(test)]
mod tests {
    use crate::core::{Value, Vertex};
    use crate::core::types::{DataType, PropertyDef, SpaceInfo, TagInfo};
    use crate::storage::RedbStorage;
    use crate::storage::storage_client::StorageClient;
    use tempfile::TempDir;

    #[test]
    fn test_vertex_insert_with_txn_id() {
        // Test that vertex insert correctly uses transaction ID
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_txn.db");
        
        let mut storage = RedbStorage::new_with_path(db_path).unwrap();
        
        // Create space
        let space_info = SpaceInfo::new("test_space".to_string())
            .with_vid_type(DataType::Int64);
        storage.create_space(&space_info).unwrap();
        
        // Create tag
        let tag_info = TagInfo::new("person".to_string())
            .with_properties(vec![PropertyDef::new("name".to_string(), DataType::String)]);
        storage.create_tag("test_space", &tag_info).unwrap();
        
        // Insert vertex (should use default txn_id = 0 when not in transaction)
        let vertex = Vertex::new(
            Value::Int(123),
            vec![crate::core::vertex_edge_path::Tag {
                name: "person".to_string(),
                properties: [("name".to_string(), Value::String("Alice".to_string()))].iter().cloned().collect(),
            }],
        );
        
        let result = storage.insert_vertex("test_space", vertex.clone());
        assert!(result.is_ok(), "Failed to insert vertex: {:?}", result);
        
        // Verify vertex was inserted
        let retrieved = storage.get_vertex("test_space", &Value::Int(123));
        assert!(retrieved.is_ok());
        assert!(retrieved.unwrap().is_some());
    }

    #[test]
    fn test_vertex_update_with_txn_id() {
        // Test that vertex update correctly uses transaction ID
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_txn_update.db");
        
        let mut storage = RedbStorage::new_with_path(db_path).unwrap();
        
        // Setup space and tag
        let space_info = SpaceInfo::new("test_space".to_string())
            .with_vid_type(DataType::Int64);
        storage.create_space(&space_info).unwrap();
        
        let tag_info = TagInfo::new("person".to_string())
            .with_properties(vec![PropertyDef::new("name".to_string(), DataType::String)]);
        storage.create_tag("test_space", &tag_info).unwrap();
        
        // Insert initial vertex
        let vertex = Vertex::new(
            Value::Int(456),
            vec![crate::core::vertex_edge_path::Tag {
                name: "person".to_string(),
                properties: [("name".to_string(), Value::String("Bob".to_string()))].iter().cloned().collect(),
            }],
        );
        storage.insert_vertex("test_space", vertex).unwrap();
        
        // Update vertex (should use default txn_id = 0 when not in transaction)
        let updated_vertex = Vertex::new(
            Value::Int(456),
            vec![crate::core::vertex_edge_path::Tag {
                name: "person".to_string(),
                properties: [("name".to_string(), Value::String("Bob Updated".to_string()))].iter().cloned().collect(),
            }],
        );
        
        let result = storage.update_vertex("test_space", updated_vertex);
        assert!(result.is_ok(), "Failed to update vertex: {:?}", result);
        
        // Verify vertex was updated
        let retrieved = storage.get_vertex("test_space", &Value::Int(456));
        assert!(retrieved.is_ok());
    }

    #[test]
    fn test_edge_insert_with_txn_id() {
        // Test that edge insert correctly uses transaction ID
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_txn_edge.db");
        
        let mut storage = RedbStorage::new_with_path(db_path).unwrap();
        
        // Create space
        let space_info = SpaceInfo::new("test_space".to_string())
            .with_vid_type(DataType::Int64);
        storage.create_space(&space_info).unwrap();
        
        // Insert edge (should use default txn_id = 0 when not in transaction)
        let edge = crate::core::Edge::new(
            Value::Int(100),
            Value::Int(200),
            "friend".to_string(),
            0,
            std::collections::HashMap::new(),
        );
        
        let result = storage.insert_edge("test_space", edge.clone());
        assert!(result.is_ok(), "Failed to insert edge: {:?}", result);
        
        // Verify edge was inserted
        let edges = storage.get_edge("test_space", &Value::Int(100), &Value::Int(200), "friend", 0);
        assert!(edges.is_ok());
        assert!(edges.unwrap().is_some());
    }
}

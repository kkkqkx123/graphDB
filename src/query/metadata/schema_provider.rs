//! Schema Metadata Provider
//!
//! This module provides metadata for tags and edge types by querying the SchemaManager.

use std::sync::Arc;

use crate::core::types::{EdgeTypeInfo, SpaceInfo, TagInfo};
use crate::query::metadata::provider::MetadataProviderError;
use crate::query::metadata::{
    EdgeTypeMetadata, IndexMetadata, IndexType, MetadataProvider, TagMetadata,
};
use crate::storage::metadata::schema_manager::SchemaManager;

/// Schema metadata provider
///
/// Provides metadata for tags, edge types, and native indexes from the schema manager.
pub struct SchemaMetadataProvider {
    schema_manager: Arc<dyn SchemaManager>,
}

impl SchemaMetadataProvider {
    /// Create a new schema metadata provider
    pub fn new(schema_manager: Arc<dyn SchemaManager>) -> Self {
        Self { schema_manager }
    }

    /// Get space info by space_id
    fn get_space_by_id(&self, space_id: u64) -> Result<SpaceInfo, MetadataProviderError> {
        self.schema_manager
            .get_space_by_id(space_id)
            .map_err(|e| MetadataProviderError::QueryFailed(e.to_string()))?
            .ok_or_else(|| MetadataProviderError::NotFound(format!("Space {} not found", space_id)))
    }

    /// Convert TagInfo to TagMetadata
    fn convert_tag_info(&self, tag_info: &TagInfo, space_id: u64) -> TagMetadata {
        let mut metadata = TagMetadata::new(tag_info.tag_name.clone(), space_id);

        // Convert properties
        metadata.properties = tag_info
            .properties
            .iter()
            .map(|prop| crate::query::metadata::PropertyDefinition {
                name: prop.name.clone(),
                data_type: crate::query::metadata::PropertyType::String, // Simplified conversion
                nullable: prop.nullable,
                default_value: None, // Simplified
            })
            .collect();

        metadata
    }

    /// Convert EdgeTypeInfo to EdgeTypeMetadata
    fn convert_edge_type_info(&self, edge_info: &EdgeTypeInfo, space_id: u64) -> EdgeTypeMetadata {
        let mut metadata = EdgeTypeMetadata::new(edge_info.edge_type_name.clone(), space_id);

        // Convert properties
        metadata.properties = edge_info
            .properties
            .iter()
            .map(|prop| crate::query::metadata::PropertyDefinition {
                name: prop.name.clone(),
                data_type: crate::query::metadata::PropertyType::String, // Simplified conversion
                nullable: prop.nullable,
                default_value: None, // Simplified
            })
            .collect();

        metadata
    }
}

impl MetadataProvider for SchemaMetadataProvider {
    fn get_index_metadata(
        &self,
        space_id: u64,
        index_name: &str,
    ) -> Result<IndexMetadata, MetadataProviderError> {
        // Get space info
        let space = self.get_space_by_id(space_id)?;
        let space_name = &space.space_name;

        // Search in tag indexes
        let tag_indexes = self
            .schema_manager
            .list_tag_indexes(space_name)
            .map_err(|e| MetadataProviderError::QueryFailed(e.to_string()))?;

        for index in tag_indexes {
            if index.name == index_name {
                return Ok(IndexMetadata::new(
                    index.name,
                    space_id,
                    index.schema_name.clone(),
                    index
                        .fields
                        .first()
                        .map(|f| f.name.clone())
                        .unwrap_or_default(),
                    IndexType::Native,
                ));
            }
        }

        // Search in edge indexes
        let edge_indexes = self
            .schema_manager
            .list_edge_indexes(space_name)
            .map_err(|e| MetadataProviderError::QueryFailed(e.to_string()))?;

        for index in edge_indexes {
            if index.name == index_name {
                return Ok(IndexMetadata::new(
                    index.name,
                    space_id,
                    String::new(), // Edge indexes don't have tag_name
                    index
                        .fields
                        .first()
                        .map(|f| f.name.clone())
                        .unwrap_or_default(),
                    IndexType::Native,
                ));
            }
        }

        Err(MetadataProviderError::NotFound(format!(
            "Index '{}' not found in space {}",
            index_name, space_id
        )))
    }

    fn get_tag_metadata(
        &self,
        space_id: u64,
        tag_name: &str,
    ) -> Result<TagMetadata, MetadataProviderError> {
        // Get space info
        let space = self.get_space_by_id(space_id)?;
        let space_name = &space.space_name;

        // Get tag info
        let tag_info = self
            .schema_manager
            .get_tag(space_name, tag_name)
            .map_err(|e| MetadataProviderError::QueryFailed(e.to_string()))?
            .ok_or_else(|| {
                MetadataProviderError::NotFound(format!(
                    "Tag '{}' not found in space {}",
                    tag_name, space_id
                ))
            })?;

        // Convert to TagMetadata
        let mut metadata = self.convert_tag_info(&tag_info, space_id);

        // Get indexes for this tag
        let indexes = self
            .schema_manager
            .list_tag_indexes(space_name)
            .map_err(|e| MetadataProviderError::QueryFailed(e.to_string()))?;

        metadata.indexes = indexes
            .iter()
            .filter(|idx| idx.schema_name == tag_name)
            .map(|idx| idx.name.clone())
            .collect();

        Ok(metadata)
    }

    fn get_edge_type_metadata(
        &self,
        space_id: u64,
        edge_type: &str,
    ) -> Result<EdgeTypeMetadata, MetadataProviderError> {
        // Get space info
        let space = self.get_space_by_id(space_id)?;
        let space_name = &space.space_name;

        // Get edge type info
        let edge_info = self
            .schema_manager
            .get_edge_type(space_name, edge_type)
            .map_err(|e| MetadataProviderError::QueryFailed(e.to_string()))?
            .ok_or_else(|| {
                MetadataProviderError::NotFound(format!(
                    "Edge type '{}' not found in space {}",
                    edge_type, space_id
                ))
            })?;

        // Convert to EdgeTypeMetadata
        let mut metadata = self.convert_edge_type_info(&edge_info, space_id);

        // Get indexes for this edge type
        let indexes = self
            .schema_manager
            .list_edge_indexes(space_name)
            .map_err(|e| MetadataProviderError::QueryFailed(e.to_string()))?;

        metadata.indexes = indexes
            .iter()
            .filter(|idx| idx.schema_name == edge_type)
            .map(|idx| idx.name.clone())
            .collect();

        Ok(metadata)
    }

    fn list_indexes(&self, space_id: u64) -> Result<Vec<IndexMetadata>, MetadataProviderError> {
        // Get space info
        let space = self.get_space_by_id(space_id)?;
        let space_name = &space.space_name;

        let mut indexes = Vec::new();

        // Get tag indexes
        let tag_indexes = self
            .schema_manager
            .list_tag_indexes(space_name)
            .map_err(|e| MetadataProviderError::QueryFailed(e.to_string()))?;

        for index in tag_indexes {
            indexes.push(IndexMetadata::new(
                index.name,
                space_id,
                index.schema_name.clone(),
                index
                    .fields
                    .first()
                    .map(|f| f.name.clone())
                    .unwrap_or_default(),
                IndexType::Native,
            ));
        }

        // Get edge indexes
        let edge_indexes = self
            .schema_manager
            .list_edge_indexes(space_name)
            .map_err(|e| MetadataProviderError::QueryFailed(e.to_string()))?;

        for index in edge_indexes {
            indexes.push(IndexMetadata::new(
                index.name,
                space_id,
                String::new(),
                index
                    .fields
                    .first()
                    .map(|f| f.name.clone())
                    .unwrap_or_default(),
                IndexType::Native,
            ));
        }

        Ok(indexes)
    }

    fn list_tags(&self, space_id: u64) -> Result<Vec<TagMetadata>, MetadataProviderError> {
        // Get space info
        let space = self.get_space_by_id(space_id)?;
        let space_name = &space.space_name;

        // Get all tags
        let tags = self
            .schema_manager
            .list_tags(space_name)
            .map_err(|e| MetadataProviderError::QueryFailed(e.to_string()))?;

        // Convert to TagMetadata
        let mut result = Vec::new();
        for tag_info in tags {
            let metadata = self.convert_tag_info(&tag_info, space_id);
            result.push(metadata);
        }

        Ok(result)
    }

    fn list_edge_types(
        &self,
        space_id: u64,
    ) -> Result<Vec<EdgeTypeMetadata>, MetadataProviderError> {
        // Get space info
        let space = self.get_space_by_id(space_id)?;
        let space_name = &space.space_name;

        // Get all edge types
        let edge_types = self
            .schema_manager
            .list_edge_types(space_name)
            .map_err(|e| MetadataProviderError::QueryFailed(e.to_string()))?;

        // Convert to EdgeTypeMetadata
        let mut result = Vec::new();
        for edge_info in edge_types {
            let metadata = self.convert_edge_type_info(&edge_info, space_id);
            result.push(metadata);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::DataType;
    use crate::core::StorageError;

    // Mock SchemaManager for testing
    #[derive(Debug)]
    struct MockSchemaManager;

    impl SchemaManager for MockSchemaManager {
        fn create_space(&self, _space: &SpaceInfo) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn drop_space(&self, _space_name: &str) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn get_space(&self, space_name: &str) -> Result<Option<SpaceInfo>, StorageError> {
            if space_name == "test_space" {
                Ok(Some(SpaceInfo {
                    space_id: 1,
                    space_name: "test_space".to_string(),
                    vid_type: DataType::String,
                    tags: vec![],
                    edge_types: vec![],
                    version: Default::default(),
                    comment: None,
                    storage_path: None,
                    isolation_level: crate::core::types::IsolationLevel::default(),
                }))
            } else {
                Ok(None)
            }
        }

        fn get_space_by_id(&self, space_id: u64) -> Result<Option<SpaceInfo>, StorageError> {
            if space_id == 1 {
                Ok(Some(SpaceInfo {
                    space_id: 1,
                    space_name: "test_space".to_string(),
                    vid_type: DataType::String,
                    tags: vec![],
                    edge_types: vec![],
                    version: Default::default(),
                    comment: None,
                    storage_path: None,
                    isolation_level: crate::core::types::IsolationLevel::default(),
                }))
            } else {
                Ok(None)
            }
        }

        fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError> {
            Ok(vec![])
        }

        fn update_space(&self, _space: &SpaceInfo) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn create_tag(&self, _space: &str, _tag: &TagInfo) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn get_tag(&self, _space: &str, tag_name: &str) -> Result<Option<TagInfo>, StorageError> {
            if tag_name == "person" {
                Ok(Some(TagInfo {
                    tag_id: 1,
                    tag_name: "person".to_string(),
                    properties: vec![crate::core::types::PropertyDef {
                        name: "name".to_string(),
                        data_type: DataType::String,
                        nullable: false,
                        default: None,
                        comment: None,
                    }],
                    comment: None,
                    ttl_duration: None,
                    ttl_col: None,
                }))
            } else {
                Ok(None)
            }
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

        fn create_edge_type(
            &self,
            _space: &str,
            _edge: &EdgeTypeInfo,
        ) -> Result<bool, StorageError> {
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

        fn drop_edge_type(
            &self,
            _space: &str,
            _edge_type_name: &str,
        ) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn update_edge_type(
            &self,
            _space: &str,
            _edge: &EdgeTypeInfo,
        ) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn get_tag_schema(
            &self,
            _space: &str,
            _tag: &str,
        ) -> Result<crate::storage::Schema, StorageError> {
            Ok(crate::storage::Schema::default())
        }

        fn get_edge_type_schema(
            &self,
            _space: &str,
            _edge: &str,
        ) -> Result<crate::storage::Schema, StorageError> {
            Ok(crate::storage::Schema::default())
        }

        fn list_tag_indexes(
            &self,
            _space: &str,
        ) -> Result<Vec<crate::core::types::Index>, StorageError> {
            Ok(vec![])
        }

        fn list_edge_indexes(
            &self,
            _space: &str,
        ) -> Result<Vec<crate::core::types::Index>, StorageError> {
            Ok(vec![])
        }
    }

    #[test]
    fn test_get_tag_metadata() {
        let schema_manager = Arc::new(MockSchemaManager);
        let provider = SchemaMetadataProvider::new(schema_manager);

        let result = provider.get_tag_metadata(1, "person");
        assert!(result.is_ok());

        let metadata = result.unwrap();
        assert_eq!(metadata.tag_name, "person");
        assert_eq!(metadata.space_id, 1);
        assert_eq!(metadata.properties.len(), 1);
        assert_eq!(metadata.properties[0].name, "name");
    }

    #[test]
    fn test_get_tag_not_found() {
        let schema_manager = Arc::new(MockSchemaManager);
        let provider = SchemaMetadataProvider::new(schema_manager);

        let result = provider.get_tag_metadata(1, "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_space_not_found() {
        let schema_manager = Arc::new(MockSchemaManager);
        let provider = SchemaMetadataProvider::new(schema_manager);

        let result = provider.get_tag_metadata(999, "person");
        assert!(result.is_err());
    }
}

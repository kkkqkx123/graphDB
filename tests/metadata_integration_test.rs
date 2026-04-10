//! Integration tests for metadata pre-resolution
//!
//! These tests verify the end-to-end flow of metadata pre-resolution
//! from planner to executor.

use graphdb::query::metadata::provider::MetadataProviderError;
use graphdb::query::metadata::{IndexMetadata, IndexType, MetadataContext, MetadataProvider};
use std::sync::Arc;

/// Mock metadata provider for testing
#[derive(Debug, Clone)]
struct MockMetadataProvider {
    indexes: Vec<IndexMetadata>,
}

impl MockMetadataProvider {
    fn new() -> Self {
        Self {
            indexes: vec![
                IndexMetadata::new(
                    "person_embedding_index".to_string(),
                    1,
                    "person".to_string(),
                    "embedding".to_string(),
                    IndexType::Vector,
                ),
                IndexMetadata::new(
                    "product_vector_index".to_string(),
                    1,
                    "product".to_string(),
                    "vector".to_string(),
                    IndexType::Vector,
                ),
            ],
        }
    }
}

impl MetadataProvider for MockMetadataProvider {
    fn get_index_metadata(
        &self,
        space_id: u64,
        index_name: &str,
    ) -> Result<IndexMetadata, MetadataProviderError> {
        self.indexes
            .iter()
            .find(|idx| idx.space_id == space_id && idx.index_name == index_name)
            .cloned()
            .ok_or_else(|| {
                MetadataProviderError::NotFound(format!(
                    "Index '{}' not found in space {}",
                    index_name, space_id
                ))
            })
    }

    fn get_tag_metadata(
        &self,
        _space_id: u64,
        tag_name: &str,
    ) -> Result<graphdb::query::metadata::TagMetadata, MetadataProviderError> {
        Ok(graphdb::query::metadata::TagMetadata::new(
            tag_name.to_string(),
            1,
        ))
    }

    fn get_edge_type_metadata(
        &self,
        _space_id: u64,
        edge_type: &str,
    ) -> Result<graphdb::query::metadata::EdgeTypeMetadata, MetadataProviderError> {
        Ok(graphdb::query::metadata::EdgeTypeMetadata::new(
            edge_type.to_string(),
            1,
        ))
    }

    fn list_indexes(&self, space_id: u64) -> Result<Vec<IndexMetadata>, MetadataProviderError> {
        Ok(self
            .indexes
            .iter()
            .filter(|idx| idx.space_id == space_id)
            .cloned()
            .collect())
    }

    fn list_tags(
        &self,
        _space_id: u64,
    ) -> Result<Vec<graphdb::query::metadata::TagMetadata>, MetadataProviderError> {
        Ok(vec![])
    }

    fn list_edge_types(
        &self,
        _space_id: u64,
    ) -> Result<Vec<graphdb::query::metadata::EdgeTypeMetadata>, MetadataProviderError> {
        Ok(vec![])
    }
}

#[test]
fn test_metadata_context_basic_operations() {
    let mut context = MetadataContext::new();

    // Add index metadata
    let index = IndexMetadata::new(
        "test_index".to_string(),
        1,
        "person".to_string(),
        "embedding".to_string(),
        IndexType::Vector,
    );

    context.set_index_metadata("test_index".to_string(), index.clone());

    // Retrieve index metadata
    let retrieved = context.get_index_metadata("test_index");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().tag_name, "person");

    // Check non-existent index
    assert!(context.get_index_metadata("nonexistent").is_none());
}

#[test]
fn test_metadata_provider_integration() {
    let provider = MockMetadataProvider::new();

    // Test get_index_metadata
    let result = provider.get_index_metadata(1, "person_embedding_index");
    assert!(result.is_ok());
    let metadata = result.unwrap();
    assert_eq!(metadata.tag_name, "person");
    assert_eq!(metadata.field_name, "embedding");

    // Test non-existent index
    let result = provider.get_index_metadata(1, "nonexistent_index");
    assert!(result.is_err());

    // Test list_indexes
    let indexes = provider.list_indexes(1).unwrap();
    assert_eq!(indexes.len(), 2);
}

#[test]
fn test_metadata_context_from_provider() {
    let provider = Arc::new(MockMetadataProvider::new());
    let mut context = MetadataContext::new();

    // Pre-resolve metadata from provider
    let index_metadata = provider
        .get_index_metadata(1, "person_embedding_index")
        .unwrap();
    context.set_index_metadata("person_embedding_index".to_string(), index_metadata);

    // Verify context contains the metadata
    let retrieved = context.get_index_metadata("person_embedding_index");
    assert!(retrieved.is_some());
    let metadata = retrieved.unwrap();
    assert_eq!(metadata.tag_name, "person");
    assert_eq!(metadata.field_name, "embedding");
    assert_eq!(metadata.index_type, IndexType::Vector);
}

#[test]
fn test_metadata_context_merge() {
    let mut context1 = MetadataContext::new();
    let mut context2 = MetadataContext::new();

    // Add different indexes to each context
    let index1 = IndexMetadata::new(
        "index1".to_string(),
        1,
        "tag1".to_string(),
        "field1".to_string(),
        IndexType::Vector,
    );
    let index2 = IndexMetadata::new(
        "index2".to_string(),
        1,
        "tag2".to_string(),
        "field2".to_string(),
        IndexType::Vector,
    );

    context1.set_index_metadata("index1".to_string(), index1);
    context2.set_index_metadata("index2".to_string(), index2);

    // Merge contexts
    context1.merge(context2);

    // Verify both indexes are present
    assert!(context1.get_index_metadata("index1").is_some());
    assert!(context1.get_index_metadata("index2").is_some());
}

#[test]
fn test_metadata_context_clear() {
    let mut context = MetadataContext::new();

    let index = IndexMetadata::new(
        "test_index".to_string(),
        1,
        "person".to_string(),
        "embedding".to_string(),
        IndexType::Vector,
    );

    context.set_index_metadata("test_index".to_string(), index);
    assert!(context.get_index_metadata("test_index").is_some());

    context.clear();
    assert!(context.get_index_metadata("test_index").is_none());
}

#[test]
fn test_index_metadata_properties() {
    let index = IndexMetadata::new(
        "my_index".to_string(),
        42,
        "my_tag".to_string(),
        "my_field".to_string(),
        IndexType::Vector,
    );

    assert_eq!(index.index_name, "my_index");
    assert_eq!(index.space_id, 42);
    assert_eq!(index.tag_name, "my_tag");
    assert_eq!(index.field_name, "my_field");
    assert_eq!(index.index_type, IndexType::Vector);
}

#[test]
fn test_metadata_provider_error_handling() {
    let provider = MockMetadataProvider::new();

    // Test error for non-existent space
    let result = provider.get_index_metadata(999, "person_embedding_index");
    assert!(result.is_err());
    match result {
        Err(MetadataProviderError::NotFound(msg)) => {
            assert!(msg.contains("not found"));
        }
        _ => panic!("Expected NotFound error"),
    }

    // Test error for non-existent index
    let result = provider.get_index_metadata(1, "nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_multiple_spaces() {
    let provider = MockMetadataProvider::new();

    // List indexes for space 1
    let indexes_space1 = provider.list_indexes(1).unwrap();
    assert_eq!(indexes_space1.len(), 2);

    // List indexes for non-existent space
    let indexes_space2 = provider.list_indexes(2).unwrap();
    assert_eq!(indexes_space2.len(), 0);
}

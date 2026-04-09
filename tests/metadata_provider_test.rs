//! Metadata Provider Tests
//!
//! Tests for the metadata provider functionality

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use graphdb::query::metadata::{
        CachedMetadataProvider, EdgeTypeMetadata, IndexMetadata, IndexType, MetadataContext,
        MetadataProvider, TagMetadata,
    };

    /// Mock metadata provider for testing
    struct MockMetadataProvider {
        indexes: HashMap<String, IndexMetadata>,
        tags: HashMap<String, TagMetadata>,
        edge_types: HashMap<String, EdgeTypeMetadata>,
    }

    impl MockMetadataProvider {
        fn new() -> Self {
            let mut indexes = HashMap::new();
            let mut tags = HashMap::new();
            let mut edge_types = HashMap::new();

            // Add test data
            let index_meta = IndexMetadata::new(
                "test_index".to_string(),
                1,
                "person".to_string(),
                "embedding".to_string(),
                IndexType::Vector,
            );
            indexes.insert("test_index".to_string(), index_meta.clone());

            let tag_meta = TagMetadata::new("person".to_string(), 1);
            tags.insert("person".to_string(), tag_meta.clone());

            let edge_meta = EdgeTypeMetadata::new("knows".to_string(), 1);
            edge_types.insert("knows".to_string(), edge_meta);

            Self {
                indexes,
                tags,
                edge_types,
            }
        }
    }

    impl MetadataProvider for MockMetadataProvider {
        fn get_index_metadata(
            &self,
            _space_id: u64,
            index_name: &str,
        ) -> Result<IndexMetadata, graphdb::query::metadata::provider::MetadataProviderError> {
            self.indexes
                .get(index_name)
                .cloned()
                .ok_or_else(|| {
                    graphdb::query::metadata::provider::MetadataProviderError::NotFound(
                        format!("Index '{}' not found", index_name),
                    )
                })
        }

        fn get_tag_metadata(
            &self,
            _space_id: u64,
            tag_name: &str,
        ) -> Result<TagMetadata, graphdb::query::metadata::provider::MetadataProviderError> {
            self.tags
                .get(tag_name)
                .cloned()
                .ok_or_else(|| {
                    graphdb::query::metadata::provider::MetadataProviderError::NotFound(
                        format!("Tag '{}' not found", tag_name),
                    )
                })
        }

        fn get_edge_type_metadata(
            &self,
            _space_id: u64,
            edge_type: &str,
        ) -> Result<EdgeTypeMetadata, graphdb::query::metadata::provider::MetadataProviderError>
        {
            self.edge_types
                .get(edge_type)
                .cloned()
                .ok_or_else(|| {
                    graphdb::query::metadata::provider::MetadataProviderError::NotFound(
                        format!("Edge type '{}' not found", edge_type),
                    )
                })
        }

        fn list_indexes(
            &self,
            _space_id: u64,
        ) -> Result<Vec<IndexMetadata>, graphdb::query::metadata::provider::MetadataProviderError>
        {
            Ok(self.indexes.values().cloned().collect())
        }

        fn list_tags(
            &self,
            _space_id: u64,
        ) -> Result<Vec<TagMetadata>, graphdb::query::metadata::provider::MetadataProviderError>
        {
            Ok(self.tags.values().cloned().collect())
        }

        fn list_edge_types(
            &self,
            _space_id: u64,
        ) -> Result<Vec<EdgeTypeMetadata>, graphdb::query::metadata::provider::MetadataProviderError>
        {
            Ok(self.edge_types.values().cloned().collect())
        }
    }

    #[test]
    fn test_metadata_context() {
        let mut context = MetadataContext::new();

        let index_meta = IndexMetadata::new(
            "test_index".to_string(),
            1,
            "person".to_string(),
            "embedding".to_string(),
            IndexType::Vector,
        );

        context.set_index_metadata("test_index".to_string(), index_meta.clone());

        let retrieved = context.get_index_metadata("test_index");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().tag_name, "person");
        assert_eq!(retrieved.unwrap().field_name, "embedding");

        assert!(!context.has_index_metadata("nonexistent"));
    }

    #[test]
    fn test_mock_metadata_provider() {
        let provider = MockMetadataProvider::new();

        let index_meta = provider.get_index_metadata(1, "test_index");
        assert!(index_meta.is_ok());
        assert_eq!(index_meta.unwrap().tag_name, "person");

        let tag_meta = provider.get_tag_metadata(1, "person");
        assert!(tag_meta.is_ok());

        let edge_meta = provider.get_edge_type_metadata(1, "knows");
        assert!(edge_meta.is_ok());
    }

    #[test]
    fn test_metadata_provider_not_found() {
        let provider = MockMetadataProvider::new();

        let result = provider.get_index_metadata(1, "nonexistent");
        assert!(result.is_err());

        let result = provider.get_tag_metadata(1, "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_cached_metadata_provider() {
        let inner_provider = Arc::new(MockMetadataProvider::new());
        let cached_provider = CachedMetadataProvider::new(inner_provider);

        let index_meta = cached_provider.get_index_metadata(1, "test_index");
        assert!(index_meta.is_ok());

        let index_meta2 = cached_provider.get_index_metadata(1, "test_index");
        assert!(index_meta2.is_ok());
    }

    #[test]
    fn test_metadata_context_merge() {
        let mut context1 = MetadataContext::new();
        let mut context2 = MetadataContext::new();

        let index1 = IndexMetadata::new(
            "index1".to_string(),
            1,
            "tag1".to_string(),
            "field1".to_string(),
            IndexType::Vector,
        );
        context1.set_index_metadata("index1".to_string(), index1);

        let index2 = IndexMetadata::new(
            "index2".to_string(),
            1,
            "tag2".to_string(),
            "field2".to_string(),
            IndexType::Vector,
        );
        context2.set_index_metadata("index2".to_string(), index2);

        context1.merge(context2);

        assert!(context1.has_index_metadata("index1"));
        assert!(context1.has_index_metadata("index2"));
    }

    #[test]
    fn test_metadata_context_clear() {
        let mut context = MetadataContext::new();

        let index_meta = IndexMetadata::new(
            "test_index".to_string(),
            1,
            "person".to_string(),
            "embedding".to_string(),
            IndexType::Vector,
        );
        context.set_index_metadata("test_index".to_string(), index_meta);

        assert!(context.has_index_metadata("test_index"));

        context.clear();

        assert!(!context.has_index_metadata("test_index"));
    }
}

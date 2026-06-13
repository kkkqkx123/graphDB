use std::sync::Arc;

use crate::query::metadata::provider::MetadataProviderError;
use crate::query::metadata::{
    EdgeTypeMetadata, IndexMetadata, IndexType, MetadataProvider, TagMetadata,
};
use crate::search::manager::FulltextIndexManager;

pub struct FulltextIndexMetadataProvider {
    manager: Arc<FulltextIndexManager>,
}

impl FulltextIndexMetadataProvider {
    pub fn new(manager: Arc<FulltextIndexManager>) -> Self {
        Self { manager }
    }
}

impl MetadataProvider for FulltextIndexMetadataProvider {
    fn get_index_metadata(
        &self,
        space_id: u64,
        index_name: &str,
    ) -> Result<IndexMetadata, MetadataProviderError> {
        let indexes = self.manager.list_indexes();
        for index in &indexes {
            if index.space_id == space_id && index.index_name == index_name {
                return Ok(IndexMetadata::new(
                    index.index_name.clone(),
                    space_id,
                    index.tag_name.clone(),
                    index.field_name.clone(),
                    IndexType::Fulltext,
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
        let indexes = self.manager.list_indexes();
        let tag_indexes: Vec<String> = indexes
            .iter()
            .filter(|idx| idx.space_id == space_id && idx.tag_name == tag_name)
            .map(|idx| idx.index_name.clone())
            .collect();

        let mut metadata = TagMetadata::new(tag_name.to_string(), space_id);
        metadata.indexes = tag_indexes;
        Ok(metadata)
    }

    fn get_edge_type_metadata(
        &self,
        space_id: u64,
        edge_type: &str,
    ) -> Result<EdgeTypeMetadata, MetadataProviderError> {
        Ok(EdgeTypeMetadata::new(edge_type.to_string(), space_id))
    }

    fn list_indexes(&self, space_id: u64) -> Result<Vec<IndexMetadata>, MetadataProviderError> {
        Ok(self
            .manager
            .list_indexes()
            .into_iter()
            .filter(|idx| idx.space_id == space_id)
            .map(|idx| {
                IndexMetadata::new(
                    idx.index_name,
                    space_id,
                    idx.tag_name,
                    idx.field_name,
                    IndexType::Fulltext,
                )
            })
            .collect())
    }

    fn list_tags(&self, space_id: u64) -> Result<Vec<TagMetadata>, MetadataProviderError> {
        let mut tag_map: std::collections::HashMap<String, TagMetadata> =
            std::collections::HashMap::new();

        for index in self.manager.list_indexes() {
            if index.space_id == space_id {
                tag_map
                    .entry(index.tag_name.clone())
                    .or_insert_with(|| TagMetadata::new(index.tag_name.clone(), space_id))
                    .indexes
                    .push(index.index_name.clone());
            }
        }

        Ok(tag_map.into_values().collect())
    }

    fn list_edge_types(
        &self,
        _space_id: u64,
    ) -> Result<Vec<EdgeTypeMetadata>, MetadataProviderError> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
#[cfg(feature = "fulltext-search")]
mod tests {
    use super::*;
    use crate::search::FulltextConfig;

    fn create_isolated_manager() -> Arc<FulltextIndexManager> {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut config = FulltextConfig::default();
        config.index_path = dir.path().to_path_buf();
        // Leak the TempDir so the directory stays alive for Tantivy locks
        std::mem::forget(dir);
        Arc::new(FulltextIndexManager::new(config).unwrap())
    }

    #[test]
    fn test_fulltext_provider_finds_created_index() {
        let manager = create_isolated_manager();

        // Create an index
        futures::executor::block_on(manager.create_index_with_engine_config(
            1,
            "article",
            "content",
            "idx_article_content",
            None,
            None,
        ))
        .unwrap();

        // Verify index exists via list_indexes
        let all = manager.list_indexes();
        assert_eq!(all.len(), 1, "Should have exactly 1 index");

        // Verify metadata provider finds it
        let provider = FulltextIndexMetadataProvider::new(manager);
        let meta = provider.get_index_metadata(1, "idx_article_content");
        assert!(meta.is_ok(), "Provider should find index: {:?}", meta.err());

        // Also test list_indexes on provider
        let indexes = provider.list_indexes(1).unwrap();
        assert_eq!(indexes.len(), 1, "Provider should list 1 index in space 1");
    }
}

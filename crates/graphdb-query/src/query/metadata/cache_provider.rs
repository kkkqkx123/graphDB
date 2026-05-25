use std::collections::HashMap;
use std::sync::Arc;

use crate::query::metadata::provider::MetadataProviderError;
use crate::query::metadata::{EdgeTypeMetadata, IndexMetadata, MetadataProvider, TagMetadata};

pub struct CachedMetadataProvider {
    inner: Arc<dyn MetadataProvider>,
    cache: parking_lot::RwLock<HashMap<String, IndexMetadata>>,
}

impl CachedMetadataProvider {
    pub fn new(inner: Arc<dyn MetadataProvider>) -> Self {
        Self {
            inner,
            cache: parking_lot::RwLock::new(HashMap::new()),
        }
    }
}

impl MetadataProvider for CachedMetadataProvider {
    fn get_index_metadata(
        &self,
        space_id: u64,
        index_name: &str,
    ) -> Result<IndexMetadata, MetadataProviderError> {
        let key = format!("{}_{}", space_id, index_name);

        {
            let cache = self.cache.read();
            if let Some(metadata) = cache.get(&key) {
                return Ok(metadata.clone());
            }
        }

        let metadata = self.inner.get_index_metadata(space_id, index_name)?;

        {
            let mut cache = self.cache.write();
            cache.insert(key, metadata.clone());
        }

        Ok(metadata)
    }

    fn get_tag_metadata(
        &self,
        space_id: u64,
        tag_name: &str,
    ) -> Result<TagMetadata, MetadataProviderError> {
        self.inner.get_tag_metadata(space_id, tag_name)
    }

    fn get_edge_type_metadata(
        &self,
        space_id: u64,
        edge_type: &str,
    ) -> Result<EdgeTypeMetadata, MetadataProviderError> {
        self.inner.get_edge_type_metadata(space_id, edge_type)
    }

    fn list_indexes(&self, space_id: u64) -> Result<Vec<IndexMetadata>, MetadataProviderError> {
        self.inner.list_indexes(space_id)
    }

    fn list_tags(&self, space_id: u64) -> Result<Vec<TagMetadata>, MetadataProviderError> {
        self.inner.list_tags(space_id)
    }

    fn list_edge_types(
        &self,
        space_id: u64,
    ) -> Result<Vec<EdgeTypeMetadata>, MetadataProviderError> {
        self.inner.list_edge_types(space_id)
    }
}

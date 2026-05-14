//! Vector Index Metadata Provider
//!
//! This module provides metadata for vector indexes by querying the VectorCoordinator.

use std::collections::HashMap;
use std::sync::Arc;

use crate::query::metadata::provider::MetadataProviderError;
use crate::query::metadata::{
    EdgeTypeMetadata, IndexMetadata, IndexType, MetadataProvider, TagMetadata,
};
use crate::sync::vector_sync::VectorSyncCoordinator;

/// Vector index metadata provider
pub struct VectorIndexMetadataProvider {
    coordinator: Arc<VectorSyncCoordinator>,
}

impl VectorIndexMetadataProvider {
    /// Create a new vector index metadata provider
    pub fn new(coordinator: Arc<VectorSyncCoordinator>) -> Self {
        Self { coordinator }
    }

    /// Get all vector indexes from the coordinator
    fn get_all_vector_indexes(&self) -> Vec<IndexMetadata> {
        let indexes = self.coordinator.list_indexes();
        indexes
            .iter()
            .map(|idx| {
                IndexMetadata::new(
                    idx.collection_name.clone(),
                    idx.space_id,
                    idx.tag_name.clone(),
                    idx.field_name.clone(),
                    IndexType::Vector,
                )
            })
            .collect()
    }
}

impl MetadataProvider for VectorIndexMetadataProvider {
    fn get_index_metadata(
        &self,
        space_id: u64,
        index_name: &str,
    ) -> Result<IndexMetadata, MetadataProviderError> {
        // Search for index by collection name or index name
        let indexes = self.coordinator.list_indexes();

        for idx in indexes {
            // Match by collection name or index name pattern
            let expected_collection =
                format!("space_{}_{}_{}", space_id, idx.tag_name, idx.field_name);

            if idx.collection_name == index_name || expected_collection == *index_name {
                return Ok(IndexMetadata::new(
                    idx.collection_name.clone(),
                    space_id,
                    idx.tag_name.clone(),
                    idx.field_name.clone(),
                    IndexType::Vector,
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
        // Get all vector indexes for this tag
        let indexes = self.get_all_vector_indexes();
        let tag_indexes: Vec<String> = indexes
            .iter()
            .filter(|idx| idx.space_id == space_id && idx.tag_name == tag_name)
            .map(|idx| idx.index_name.clone())
            .collect();

        // Create tag metadata (simplified - in production would query schema manager)
        let mut metadata = TagMetadata::new(tag_name.to_string(), space_id);
        metadata.indexes = tag_indexes;

        Ok(metadata)
    }

    fn get_edge_type_metadata(
        &self,
        space_id: u64,
        edge_type: &str,
    ) -> Result<EdgeTypeMetadata, MetadataProviderError> {
        // Edge type metadata is not managed by vector coordinator
        // Return empty metadata (in production would query schema manager)
        Ok(EdgeTypeMetadata::new(edge_type.to_string(), space_id))
    }

    fn list_indexes(&self, space_id: u64) -> Result<Vec<IndexMetadata>, MetadataProviderError> {
        let indexes = self.get_all_vector_indexes();
        Ok(indexes
            .into_iter()
            .filter(|idx| idx.space_id == space_id)
            .collect())
    }

    fn list_tags(&self, space_id: u64) -> Result<Vec<TagMetadata>, MetadataProviderError> {
        // Get unique tags from vector indexes
        let indexes = self.get_all_vector_indexes();
        let mut tag_map: HashMap<String, TagMetadata> = HashMap::new();

        for idx in indexes {
            if idx.space_id == space_id {
                tag_map
                    .entry(idx.tag_name.clone())
                    .or_insert_with(|| TagMetadata::new(idx.tag_name.clone(), space_id))
                    .indexes
                    .push(idx.index_name.clone());
            }
        }

        Ok(tag_map.into_values().collect())
    }

    fn list_edge_types(
        &self,
        _space_id: u64,
    ) -> Result<Vec<EdgeTypeMetadata>, MetadataProviderError> {
        // Edge types are not managed by vector coordinator
        Ok(Vec::new())
    }
}

/// Cached metadata provider wrapper
pub struct CachedMetadataProvider {
    inner: Arc<dyn MetadataProvider>,
    cache: parking_lot::RwLock<HashMap<String, IndexMetadata>>,
}

impl CachedMetadataProvider {
    /// Create a new cached metadata provider
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

        // Check cache first
        {
            let cache = self.cache.read();
            if let Some(metadata) = cache.get(&key) {
                return Ok(metadata.clone());
            }
        }

        // Cache miss, query inner provider
        let metadata = self.inner.get_index_metadata(space_id, index_name)?;

        // Update cache
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

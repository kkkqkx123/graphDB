//! Metadata Provider Trait
//!
//! This trait defines the interface for metadata providers, similar to PostgreSQL's FDW.

use super::types::{EdgeTypeMetadata, IndexMetadata, TagMetadata};
use crate::core::error::DBError;
use std::sync::Arc;

/// Metadata provider error
#[derive(Debug, thiserror::Error)]
pub enum MetadataProviderError {
    #[error("Metadata not found: {0}")]
    NotFound(String),

    #[error("Metadata access error: {0}")]
    AccessError(String),

    #[error("Invalid metadata: {0}")]
    InvalidMetadata(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] DBError),

    #[error("Query failed: {0}")]
    QueryFailed(String),
}

/// Metadata provider trait
///
/// Similar to PostgreSQL's FDW callback interface, this trait provides
/// methods for accessing metadata during query planning.
pub trait MetadataProvider: Send + Sync {
    /// Get index metadata
    fn get_index_metadata(
        &self,
        space_id: u64,
        index_name: &str,
    ) -> Result<IndexMetadata, MetadataProviderError>;

    /// Get tag metadata
    fn get_tag_metadata(
        &self,
        space_id: u64,
        tag_name: &str,
    ) -> Result<TagMetadata, MetadataProviderError>;

    /// Get edge type metadata
    fn get_edge_type_metadata(
        &self,
        space_id: u64,
        edge_type: &str,
    ) -> Result<EdgeTypeMetadata, MetadataProviderError>;

    /// List all indexes (for optimization)
    fn list_indexes(&self, space_id: u64) -> Result<Vec<IndexMetadata>, MetadataProviderError>;

    /// List all tags
    fn list_tags(&self, space_id: u64) -> Result<Vec<TagMetadata>, MetadataProviderError>;

    /// List all edge types
    fn list_edge_types(
        &self,
        space_id: u64,
    ) -> Result<Vec<EdgeTypeMetadata>, MetadataProviderError>;
}

/// Composite metadata provider that chains multiple providers.
///
/// Each method tries providers in order, returning the first success.
/// For singular lookups (`get_*`), stops at the first success.
/// For list operations (`list_*`), merges results from all providers.
pub struct CompositeMetadataProvider {
    providers: Vec<Arc<dyn MetadataProvider>>,
}

impl CompositeMetadataProvider {
    pub fn new(providers: Vec<Arc<dyn MetadataProvider>>) -> Self {
        Self { providers }
    }
}

impl MetadataProvider for CompositeMetadataProvider {
    fn get_index_metadata(
        &self,
        space_id: u64,
        index_name: &str,
    ) -> Result<IndexMetadata, MetadataProviderError> {
        let mut last_error = None;
        for provider in &self.providers {
            match provider.get_index_metadata(space_id, index_name) {
                Ok(meta) => return Ok(meta),
                Err(e) => last_error = Some(e),
            }
        }
        Err(last_error.unwrap_or_else(|| {
            MetadataProviderError::NotFound(format!(
                "Index '{}' not found in space {}",
                index_name, space_id
            ))
        }))
    }

    fn get_tag_metadata(
        &self,
        space_id: u64,
        tag_name: &str,
    ) -> Result<TagMetadata, MetadataProviderError> {
        let mut last_error = None;
        for provider in &self.providers {
            match provider.get_tag_metadata(space_id, tag_name) {
                Ok(meta) => return Ok(meta),
                Err(e) => last_error = Some(e),
            }
        }
        Err(last_error.unwrap_or_else(|| {
            MetadataProviderError::NotFound(format!(
                "Tag '{}' not found in space {}",
                tag_name, space_id
            ))
        }))
    }

    fn get_edge_type_metadata(
        &self,
        space_id: u64,
        edge_type: &str,
    ) -> Result<EdgeTypeMetadata, MetadataProviderError> {
        let mut last_error = None;
        for provider in &self.providers {
            match provider.get_edge_type_metadata(space_id, edge_type) {
                Ok(meta) => return Ok(meta),
                Err(e) => last_error = Some(e),
            }
        }
        Err(last_error.unwrap_or_else(|| {
            MetadataProviderError::NotFound(format!(
                "Edge type '{}' not found in space {}",
                edge_type, space_id
            ))
        }))
    }

    fn list_indexes(&self, space_id: u64) -> Result<Vec<IndexMetadata>, MetadataProviderError> {
        let mut all = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for provider in &self.providers {
            if let Ok(indexes) = provider.list_indexes(space_id) {
                for idx in indexes {
                    if seen.insert(idx.index_name.clone()) {
                        all.push(idx);
                    }
                }
            }
        }
        Ok(all)
    }

    fn list_tags(&self, space_id: u64) -> Result<Vec<TagMetadata>, MetadataProviderError> {
        let mut all = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for provider in &self.providers {
            if let Ok(tags) = provider.list_tags(space_id) {
                for tag in tags {
                    if seen.insert(tag.tag_name.clone()) {
                        all.push(tag);
                    }
                }
            }
        }
        Ok(all)
    }

    fn list_edge_types(
        &self,
        space_id: u64,
    ) -> Result<Vec<EdgeTypeMetadata>, MetadataProviderError> {
        let mut all = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for provider in &self.providers {
            if let Ok(types) = provider.list_edge_types(space_id) {
                for et in types {
                    if seen.insert(et.edge_type.clone()) {
                        all.push(et);
                    }
                }
            }
        }
        Ok(all)
    }
}

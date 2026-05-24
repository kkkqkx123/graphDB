//! Metadata Provider Trait
//!
//! This trait defines the interface for metadata providers, similar to PostgreSQL's FDW.

use super::types::{EdgeTypeMetadata, IndexMetadata, TagMetadata};
use crate::core::error::DBError;

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

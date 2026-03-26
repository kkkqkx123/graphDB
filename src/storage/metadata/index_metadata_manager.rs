use crate::core::types::Index;
use crate::core::StorageError;

/// Index Metadata Manager trait
///
/// Metadata management functions that provide both a tag index and a edge index.
/// All operations identify the space using the `space_id` parameter, thereby ensuring the isolation of data from different spaces.
pub trait IndexMetadataManager: Send + Sync + std::fmt::Debug {
    /// Create a tag index
    fn create_tag_index(&self, space_id: u64, index: &Index) -> Result<bool, StorageError>;
    /// Delete the tag index.
    fn drop_tag_index(&self, space_id: u64, index_name: &str) -> Result<bool, StorageError>;
    /// Obtain the tag index.
    fn get_tag_index(&self, space_id: u64, index_name: &str)
        -> Result<Option<Index>, StorageError>;
    /// List all tag indices.
    fn list_tag_indexes(&self, space_id: u64) -> Result<Vec<Index>, StorageError>;
    /// Delete all indexes that contain the specified tag.
    fn drop_tag_indexes_by_tag(&self, space_id: u64, tag_name: &str) -> Result<(), StorageError>;

    /// Create a edge index.
    fn create_edge_index(&self, space_id: u64, index: &Index) -> Result<bool, StorageError>;
    /// Delete the edge index.
    fn drop_edge_index(&self, space_id: u64, index_name: &str) -> Result<bool, StorageError>;
    /// Obtain the edge index.
    fn get_edge_index(
        &self,
        space_id: u64,
        index_name: &str,
    ) -> Result<Option<Index>, StorageError>;
    /// List all edge indices.
    fn list_edge_indexes(&self, space_id: u64) -> Result<Vec<Index>, StorageError>;
    /// Delete all indexes for the specified edge type.
    fn drop_edge_indexes_by_type(&self, space_id: u64, edge_type: &str)
        -> Result<(), StorageError>;
}

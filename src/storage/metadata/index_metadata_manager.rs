use crate::core::StorageError;
use crate::index::Index;

pub trait IndexMetadataManager: Send + Sync + std::fmt::Debug {
    fn create_tag_index(&self, space: &str, index: &Index) -> Result<bool, StorageError>;
    fn drop_tag_index(&self, space: &str, index_name: &str) -> Result<bool, StorageError>;
    fn get_tag_index(&self, space: &str, index_name: &str) -> Result<Option<Index>, StorageError>;
    fn list_tag_indexes(&self, space: &str) -> Result<Vec<Index>, StorageError>;
    fn drop_tag_indexes_by_tag(&self, space: &str, tag_name: &str) -> Result<(), StorageError>;

    fn create_edge_index(&self, space: &str, index: &Index) -> Result<bool, StorageError>;
    fn drop_edge_index(&self, space: &str, index_name: &str) -> Result<bool, StorageError>;
    fn get_edge_index(&self, space: &str, index_name: &str) -> Result<Option<Index>, StorageError>;
    fn list_edge_indexes(&self, space: &str) -> Result<Vec<Index>, StorageError>;
    fn drop_edge_indexes_by_type(&self, space: &str, edge_type: &str) -> Result<(), StorageError>;
}

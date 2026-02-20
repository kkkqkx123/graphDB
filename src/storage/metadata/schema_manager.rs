use crate::core::StorageError;
use crate::core::types::{
    EdgeTypeInfo, SpaceInfo, TagInfo,
};
use crate::storage::Schema;

pub trait SchemaManager: Send + Sync + std::fmt::Debug {
    fn create_space(&self, space: &SpaceInfo) -> Result<bool, StorageError>;
    fn drop_space(&self, space_name: &str) -> Result<bool, StorageError>;
    fn get_space(&self, space_name: &str) -> Result<Option<SpaceInfo>, StorageError>;
    fn get_space_by_id(&self, space_id: u64) -> Result<Option<SpaceInfo>, StorageError>;
    fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError>;

    fn create_tag(&self, space: &str, tag: &TagInfo) -> Result<bool, StorageError>;
    fn get_tag(&self, space: &str, tag_name: &str) -> Result<Option<TagInfo>, StorageError>;
    fn list_tags(&self, space: &str) -> Result<Vec<TagInfo>, StorageError>;
    fn drop_tag(&self, space: &str, tag_name: &str) -> Result<bool, StorageError>;

    fn create_edge_type(&self, space: &str, edge: &EdgeTypeInfo) -> Result<bool, StorageError>;
    fn get_edge_type(&self, space: &str, edge_type_name: &str) -> Result<Option<EdgeTypeInfo>, StorageError>;
    fn list_edge_types(&self, space: &str) -> Result<Vec<EdgeTypeInfo>, StorageError>;
    fn drop_edge_type(&self, space: &str, edge_type_name: &str) -> Result<bool, StorageError>;

    fn get_tag_schema(&self, space: &str, tag: &str) -> Result<Schema, StorageError>;
    fn get_edge_type_schema(&self, space: &str, edge: &str) -> Result<Schema, StorageError>;
}

use crate::core::StorageError;
use crate::index::Index;

/// 索引元数据管理器 trait
///
/// 提供标签索引和边索引的元数据管理功能
/// 所有操作都通过 space_id 来标识空间，实现多空间数据隔离
pub trait IndexMetadataManager: Send + Sync + std::fmt::Debug {
    /// 创建标签索引
    fn create_tag_index(&self, space_id: u64, index: &Index) -> Result<bool, StorageError>;
    /// 删除标签索引
    fn drop_tag_index(&self, space_id: u64, index_name: &str) -> Result<bool, StorageError>;
    /// 获取标签索引
    fn get_tag_index(&self, space_id: u64, index_name: &str) -> Result<Option<Index>, StorageError>;
    /// 列出所有标签索引
    fn list_tag_indexes(&self, space_id: u64) -> Result<Vec<Index>, StorageError>;
    /// 删除指定标签的所有索引
    fn drop_tag_indexes_by_tag(&self, space_id: u64, tag_name: &str) -> Result<(), StorageError>;

    /// 创建边索引
    fn create_edge_index(&self, space_id: u64, index: &Index) -> Result<bool, StorageError>;
    /// 删除边索引
    fn drop_edge_index(&self, space_id: u64, index_name: &str) -> Result<bool, StorageError>;
    /// 获取边索引
    fn get_edge_index(&self, space_id: u64, index_name: &str) -> Result<Option<Index>, StorageError>;
    /// 列出所有边索引
    fn list_edge_indexes(&self, space_id: u64) -> Result<Vec<Index>, StorageError>;
    /// 删除指定边类型的所有索引
    fn drop_edge_indexes_by_type(&self, space_id: u64, edge_type: &str) -> Result<(), StorageError>;
}

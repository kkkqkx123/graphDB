//! 元数据客户端接口 - 定义元数据访问的基本操作

use crate::core::error::ManagerResult;
use super::types::{
    ClusterInfo, EdgeTypeDef, MetadataVersion, SpaceInfo, TagDef,
};

/// 元数据客户端接口 - 定义元数据访问的基本操作
pub trait MetaClient: Send + Sync + std::fmt::Debug {
    /// 获取集群元信息
    fn get_cluster_info(&self) -> ManagerResult<ClusterInfo>;
    /// 获取空间信息
    fn get_space_info(&self, space_id: i32) -> ManagerResult<SpaceInfo>;
    /// 检查连接状态
    fn is_connected(&self) -> bool;

    /// 创建空间
    fn create_space(
        &self,
        space_name: &str,
        partition_num: i32,
        replica_factor: i32,
    ) -> ManagerResult<i32>;
    /// 删除空间
    fn drop_space(&self, space_id: i32) -> ManagerResult<()>;
    /// 列出所有空间
    fn list_spaces(&self) -> ManagerResult<Vec<SpaceInfo>>;
    /// 检查空间是否存在
    fn has_space(&self, space_id: i32) -> bool;

    /// 从磁盘加载元数据
    fn load_from_disk(&self) -> ManagerResult<()>;
    /// 保存元数据到磁盘
    fn save_to_disk(&self) -> ManagerResult<()>;

    /// 创建标签定义
    fn create_tag(&self, space_id: i32, tag_def: TagDef) -> ManagerResult<()>;
    /// 删除标签定义
    fn drop_tag(&self, space_id: i32, tag_name: &str) -> ManagerResult<()>;
    /// 获取标签定义
    fn get_tag(&self, space_id: i32, tag_name: &str) -> ManagerResult<TagDef>;
    /// 列出空间的所有标签
    fn list_tags(&self, space_id: i32) -> ManagerResult<Vec<TagDef>>;

    /// 创建边类型定义
    fn create_edge_type(&self, space_id: i32, edge_type_def: EdgeTypeDef) -> ManagerResult<()>;
    /// 删除边类型定义
    fn drop_edge_type(&self, space_id: i32, edge_name: &str) -> ManagerResult<()>;
    /// 获取边类型定义
    fn get_edge_type(&self, space_id: i32, edge_name: &str) -> ManagerResult<EdgeTypeDef>;
    /// 列出空间的所有边类型
    fn list_edge_types(&self, space_id: i32) -> ManagerResult<Vec<EdgeTypeDef>>;

    /// 获取元数据版本
    fn get_metadata_version(&self, space_id: i32) -> ManagerResult<MetadataVersion>;
    /// 更新元数据版本
    fn update_metadata_version(&self, space_id: i32, description: &str) -> ManagerResult<()>;
}

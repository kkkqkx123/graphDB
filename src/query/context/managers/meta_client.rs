//! 元数据客户端接口 - 定义元数据访问的基本操作

use crate::core::error::{ManagerError, ManagerResult};
use serde::{Deserialize, Serialize};

/// 集群信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterInfo {
    pub cluster_id: String,
    pub meta_servers: Vec<String>,
    pub storage_servers: Vec<String>,
}

/// 空间信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceInfo {
    pub space_id: i32,
    pub space_name: String,
    pub partition_num: i32,
    pub replica_factor: i32,
}

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
}

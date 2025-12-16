//! 元数据客户端接口 - 定义元数据访问的基本操作

/// 集群信息
#[derive(Debug, Clone)]
pub struct ClusterInfo {
    pub cluster_id: String,
    pub meta_servers: Vec<String>,
    pub storage_servers: Vec<String>,
}

/// 空间信息
#[derive(Debug, Clone)]
pub struct SpaceInfo {
    pub space_id: i32,
    pub space_name: String,
    pub partition_num: i32,
    pub replica_factor: i32,
}

/// 元数据客户端接口 - 定义元数据访问的基本操作
pub trait MetaClient: Send + Sync + std::fmt::Debug {
    /// 获取集群元信息
    fn get_cluster_info(&self) -> Result<ClusterInfo, String>;
    /// 获取空间信息
    fn get_space_info(&self, space_id: i32) -> Result<SpaceInfo, String>;
    /// 检查连接状态
    fn is_connected(&self) -> bool;
}

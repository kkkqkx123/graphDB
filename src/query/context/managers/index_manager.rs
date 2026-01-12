//! 索引管理器接口 - 定义索引管理的基本操作

use crate::core::error::ManagerResult;
use crate::core::{Edge, Value, Vertex};
use serde::{Deserialize, Serialize};

/// 索引状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexStatus {
    Creating,
    Active,
    Dropped,
    Failed,
}

/// 索引类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexType {
    TagIndex,
    EdgeIndex,
    FulltextIndex,
}

/// 索引信息 - 表示数据库索引
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub id: i32,
    pub name: String,
    pub space_id: i32,
    pub schema_name: String,
    pub fields: Vec<String>,
    pub index_type: IndexType,
    pub status: IndexStatus,
    pub is_unique: bool,
    pub comment: Option<String>,
}

/// 索引管理器接口 - 定义索引管理的基本操作
pub trait IndexManager: Send + Sync + std::fmt::Debug {
    /// 获取指定名称的索引
    fn get_index(&self, name: &str) -> Option<Index>;
    /// 列出所有索引名称
    fn list_indexes(&self) -> Vec<String>;
    /// 检查索引是否存在
    fn has_index(&self, name: &str) -> bool;

    /// 创建索引
    fn create_index(&self, space_id: i32, index: Index) -> ManagerResult<i32>;
    /// 删除索引
    fn drop_index(&self, space_id: i32, index_id: i32) -> ManagerResult<()>;
    /// 获取索引状态
    fn get_index_status(&self, space_id: i32, index_id: i32) -> Option<IndexStatus>;
    /// 列出指定Space的所有索引
    fn list_indexes_by_space(&self, space_id: i32) -> ManagerResult<Vec<Index>>;

    /// ==================== 索引查询功能 ====================

    /// 基于索引查询顶点
    fn lookup_vertex_by_index(
        &self,
        space_id: i32,
        index_name: &str,
        values: &[Value],
    ) -> ManagerResult<Vec<Vertex>>;

    /// 基于索引查询边
    fn lookup_edge_by_index(
        &self,
        space_id: i32,
        index_name: &str,
        values: &[Value],
    ) -> ManagerResult<Vec<Edge>>;

    /// 基于索引的范围查询顶点
    fn range_lookup_vertex(
        &self,
        space_id: i32,
        index_name: &str,
        start: &Value,
        end: &Value,
    ) -> ManagerResult<Vec<Vertex>>;

    /// 基于索引的范围查询边
    fn range_lookup_edge(
        &self,
        space_id: i32,
        index_name: &str,
        start: &Value,
        end: &Value,
    ) -> ManagerResult<Vec<Edge>>;

    /// ==================== 索引写入操作 ====================

    /// 插入顶点到索引
    fn insert_vertex_to_index(&self, space_id: i32, vertex: &Vertex) -> ManagerResult<()>;

    /// 从索引中删除顶点
    fn delete_vertex_from_index(&self, space_id: i32, vertex: &Vertex) -> ManagerResult<()>;

    /// 更新索引中的顶点
    fn update_vertex_in_index(
        &self,
        space_id: i32,
        old_vertex: &Vertex,
        new_vertex: &Vertex,
    ) -> ManagerResult<()>;

    /// 插入边到索引
    fn insert_edge_to_index(&self, space_id: i32, edge: &Edge) -> ManagerResult<()>;

    /// 从索引中删除边
    fn delete_edge_from_index(&self, space_id: i32, edge: &Edge) -> ManagerResult<()>;

    /// 更新索引中的边
    fn update_edge_in_index(
        &self,
        space_id: i32,
        old_edge: &Edge,
        new_edge: &Edge,
    ) -> ManagerResult<()>;

    /// 从磁盘加载索引
    fn load_from_disk(&self) -> ManagerResult<()>;
    /// 保存索引到磁盘
    fn save_to_disk(&self) -> ManagerResult<()>;
}

//! 索引管理器接口 - 定义索引管理的基本操作

use crate::core::{Value, Vertex, Edge};
use serde::{Deserialize, Serialize};

/// 索引状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexStatus {
    Creating,
    Building,
    Active,
    Dropped,
    Failed,
    Cancelled,
}

/// 索引类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexType {
    TagIndex,
    EdgeIndex,
    FulltextIndex,
}

/// 索引构建进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexBuildProgress {
    pub index_id: i32,
    pub index_name: String,
    pub total_count: u64,
    pub processed_count: u64,
    pub progress_percent: f64,
    pub status: IndexStatus,
    pub error_message: Option<String>,
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
    fn create_index(&self, space_id: i32, index: Index) -> Result<i32, String>;
    /// 删除索引
    fn drop_index(&self, space_id: i32, index_id: i32) -> Result<(), String>;
    /// 获取索引状态
    fn get_index_status(&self, space_id: i32, index_id: i32) -> Option<IndexStatus>;
    /// 列出指定Space的所有索引
    fn list_indexes_by_space(&self, space_id: i32) -> Result<Vec<Index>, String>;
    
    /// 异步构建索引
    fn build_index_async(&self, space_id: i32, index_id: i32) -> Result<(), String>;
    /// 获取索引构建进度
    fn get_build_progress(&self, space_id: i32, index_id: i32) -> Option<IndexBuildProgress>;
    /// 取消索引构建
    fn cancel_build(&self, space_id: i32, index_id: i32) -> Result<(), String>;
    
    /// ==================== 索引查询功能 ====================
    
    /// 基于索引查询顶点
    fn lookup_vertex_by_index(
        &self,
        space_id: i32,
        index_name: &str,
        values: &[Value],
    ) -> Result<Vec<Vertex>, String>;
    
    /// 基于索引查询边
    fn lookup_edge_by_index(
        &self,
        space_id: i32,
        index_name: &str,
        values: &[Value],
    ) -> Result<Vec<Edge>, String>;
    
    /// 基于索引的范围查询顶点
    fn range_lookup_vertex(
        &self,
        space_id: i32,
        index_name: &str,
        start: &Value,
        end: &Value,
    ) -> Result<Vec<Vertex>, String>;
    
    /// 基于索引的范围查询边
    fn range_lookup_edge(
        &self,
        space_id: i32,
        index_name: &str,
        start: &Value,
        end: &Value,
    ) -> Result<Vec<Edge>, String>;
    
    /// 从磁盘加载索引
    fn load_from_disk(&self) -> Result<(), String>;
    /// 保存索引到磁盘
    fn save_to_disk(&self) -> Result<(), String>;
}

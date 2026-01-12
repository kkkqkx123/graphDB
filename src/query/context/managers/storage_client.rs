//! 存储客户端接口 - 定义存储层访问的基本操作

use crate::core::error::ManagerResult;
use crate::core::{Edge, Value, Vertex};
use std::collections::HashMap;

/// 存储操作类型
#[derive(Debug, Clone)]
pub enum StorageOperation {
    Read {
        table: String,
        key: String,
    },
    Write {
        table: String,
        key: String,
        value: Value,
    },
    Delete {
        table: String,
        key: String,
    },
    Scan {
        table: String,
        prefix: String,
    },
}

/// 存储响应
#[derive(Debug, Clone)]
pub struct StorageResponse {
    pub success: bool,
    pub data: Option<Value>,
    pub error_message: Option<String>,
}

/// 边键 - 唯一标识一条边
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EdgeKey {
    pub src: Value,
    pub edge_type: String,
    pub ranking: i64,
    pub dst: Value,
}

impl EdgeKey {
    pub fn new(src: Value, edge_type: String, ranking: i64, dst: Value) -> Self {
        Self {
            src,
            edge_type,
            ranking,
            dst,
        }
    }
}

/// 新建标签 - 用于添加顶点时指定标签和属性
#[derive(Debug, Clone)]
pub struct NewTag {
    pub tag_id: i32,
    pub props: Vec<Value>,
}

impl NewTag {
    pub fn new(tag_id: i32, props: Vec<Value>) -> Self {
        Self { tag_id, props }
    }
}

/// 新建顶点 - 用于批量添加顶点
#[derive(Debug, Clone)]
pub struct NewVertex {
    pub id: Value,
    pub tags: Vec<NewTag>,
}

impl NewVertex {
    pub fn new(id: Value, tags: Vec<NewTag>) -> Self {
        Self { id, tags }
    }
}

/// 新建边 - 用于批量添加边
#[derive(Debug, Clone)]
pub struct NewEdge {
    pub key: EdgeKey,
    pub props: Vec<Value>,
}

impl NewEdge {
    pub fn new(key: EdgeKey, props: Vec<Value>) -> Self {
        Self { key, props }
    }
}

/// 删除标签 - 用于删除顶点的特定标签
#[derive(Debug, Clone)]
pub struct DelTags {
    pub id: Value,
    pub tags: Vec<i32>,
}

impl DelTags {
    pub fn new(id: Value, tags: Vec<i32>) -> Self {
        Self { id, tags }
    }
}

/// 更新属性 - 用于更新顶点或边的属性
#[derive(Debug, Clone)]
pub struct UpdatedProp {
    pub name: String,
    pub value: Value,
}

impl UpdatedProp {
    pub fn new(name: String, value: Value) -> Self {
        Self { name, value }
    }
}

/// 执行响应 - 用于写操作的响应
#[derive(Debug, Clone)]
pub struct ExecResponse {
    pub success: bool,
    pub error_message: Option<String>,
}

impl ExecResponse {
    pub fn ok() -> Self {
        Self {
            success: true,
            error_message: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            error_message: Some(message),
        }
    }
}

/// 更新响应 - 用于更新操作的响应
#[derive(Debug, Clone)]
pub struct UpdateResponse {
    pub success: bool,
    pub inserted: bool,
    pub props: Option<HashMap<String, Value>>,
    pub error_message: Option<String>,
}

impl UpdateResponse {
    pub fn ok(inserted: bool, props: Option<HashMap<String, Value>>) -> Self {
        Self {
            success: true,
            inserted,
            props,
            error_message: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            inserted: false,
            props: None,
            error_message: Some(message),
        }
    }
}

/// 存储客户端接口 - 定义存储层访问的基本操作
pub trait StorageClient: Send + Sync + std::fmt::Debug {
    /// 执行存储操作
    fn execute(&self, operation: StorageOperation) -> ManagerResult<StorageResponse>;
    /// 检查连接状态
    fn is_connected(&self) -> bool;

    /// ==================== Vertex 操作 ====================

    /// 添加单个顶点
    fn add_vertex(&self, space_id: i32, vertex: Vertex) -> ManagerResult<ExecResponse>;

    /// 批量添加顶点
    fn add_vertices(&self, space_id: i32, vertices: Vec<NewVertex>) -> ManagerResult<ExecResponse>;

    /// 获取顶点
    fn get_vertex(&self, space_id: i32, vid: &Value) -> ManagerResult<Option<Vertex>>;

    /// 批量获取顶点
    fn get_vertices(&self, space_id: i32, vids: &[Value]) -> ManagerResult<Vec<Option<Vertex>>>;

    /// 删除单个顶点
    fn delete_vertex(&self, space_id: i32, vid: &Value) -> ManagerResult<ExecResponse>;

    /// 批量删除顶点
    fn delete_vertices(&self, space_id: i32, vids: &[Value]) -> ManagerResult<ExecResponse>;

    /// 删除顶点的特定标签
    fn delete_tags(&self, space_id: i32, del_tags: Vec<DelTags>) -> ManagerResult<ExecResponse>;

    /// 更新顶点属性
    fn update_vertex(
        &self,
        space_id: i32,
        vid: &Value,
        tag_id: i32,
        updated_props: Vec<UpdatedProp>,
        insertable: bool,
        return_props: Vec<String>,
        condition: Option<String>,
    ) -> ManagerResult<UpdateResponse>;

    /// ==================== Edge 操作 ====================

    /// 添加单个边
    fn add_edge(&self, space_id: i32, edge: Edge) -> ManagerResult<ExecResponse>;

    /// 批量添加边
    fn add_edges(&self, space_id: i32, edges: Vec<NewEdge>) -> ManagerResult<ExecResponse>;

    /// 获取边
    fn get_edge(&self, space_id: i32, edge_key: &EdgeKey) -> ManagerResult<Option<Edge>>;

    /// 批量获取边
    fn get_edges(&self, space_id: i32, edge_keys: &[EdgeKey]) -> ManagerResult<Vec<Option<Edge>>>;

    /// 删除单个边
    fn delete_edge(&self, space_id: i32, edge_key: &EdgeKey) -> ManagerResult<ExecResponse>;

    /// 批量删除边
    fn delete_edges(&self, space_id: i32, edge_keys: &[EdgeKey]) -> ManagerResult<ExecResponse>;

    /// 更新边属性
    fn update_edge(
        &self,
        space_id: i32,
        edge_key: &EdgeKey,
        updated_props: Vec<UpdatedProp>,
        insertable: bool,
        return_props: Vec<String>,
        condition: Option<String>,
    ) -> ManagerResult<UpdateResponse>;

    /// ==================== 扫描操作 ====================

    /// 扫描所有顶点
    fn scan_vertices(&self, space_id: i32, limit: Option<usize>) -> ManagerResult<Vec<Vertex>>;

    /// 按标签扫描顶点
    fn scan_vertices_by_tag(
        &self,
        space_id: i32,
        tag_id: i32,
        limit: Option<usize>,
    ) -> ManagerResult<Vec<Vertex>>;

    /// 扫描所有边
    fn scan_edges(&self, space_id: i32, limit: Option<usize>) -> ManagerResult<Vec<Edge>>;

    /// 按边类型扫描边
    fn scan_edges_by_type(
        &self,
        space_id: i32,
        edge_type: &str,
        limit: Option<usize>,
    ) -> ManagerResult<Vec<Edge>>;

    /// 按源顶点扫描边
    fn scan_edges_by_src(
        &self,
        space_id: i32,
        src: &Value,
        limit: Option<usize>,
    ) -> ManagerResult<Vec<Edge>>;

    /// 按目标顶点扫描边
    fn scan_edges_by_dst(
        &self,
        space_id: i32,
        dst: &Value,
        limit: Option<usize>,
    ) -> ManagerResult<Vec<Edge>>;
}

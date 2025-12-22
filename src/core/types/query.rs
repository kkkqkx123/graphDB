//! 查询相关类型定义
//!
//! 定义图数据库查询系统中的核心类型

use serde::{Deserialize, Serialize};
use crate::core::error::{DBError, QueryError as CoreQueryError};

/// 查询类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QueryType {
    /// 数据查询语言
    DataQuery,
    /// 数据操作语言
    DataManipulation,
    /// 数据定义语言
    DataDefinition,
    /// 数据控制语言
    DataControl,
    /// 事务控制语言
    TransactionControl,
}

/// 查询结果类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueryResult {
    /// 成功结果
    Success {
        /// 影响的行数
        affected_rows: usize,
        /// 返回的数据
        data: Option<QueryData>,
        /// 执行时间（毫秒）
        execution_time_ms: u64,
    },
    /// 错误结果
    Error {
        /// 错误信息
        error: QueryError,
        /// 执行时间（毫秒）
        execution_time_ms: u64,
    },
}

/// 查询数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueryData {
    /// 标量值
    Scalar(ScalarValue),
    /// 记录集合
    Records(Vec<Record>),
    /// 图数据
    Graph(GraphData),
    /// 路径集合
    Paths(Vec<Path>),
    /// 统计信息
    Statistics(Statistics),
}

/// 标量值
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScalarValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Null,
}

/// 记录
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Record {
    /// 记录的字段
    pub fields: Vec<(String, FieldValue)>,
}

/// 字段值
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldValue {
    Scalar(ScalarValue),
    List(Vec<FieldValue>),
    Map(Vec<(String, FieldValue)>),
    Vertex(Vertex),
    Edge(Edge),
    Path(Path),
}

/// 顶点
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vertex {
    /// 顶点ID
    pub id: String,
    /// 标签
    pub tags: Vec<String>,
    /// 属性
    pub properties: Vec<(String, ScalarValue)>,
}

/// 边
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    /// 边ID
    pub id: String,
    /// 边类型
    pub edge_type: String,
    /// 源顶点ID
    pub src: String,
    /// 目标顶点ID
    pub dst: String,
    /// 属性
    pub properties: Vec<(String, ScalarValue)>,
    /// 排名
    pub ranking: Option<i64>,
}

/// 路径
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Path {
    /// 路径中的顶点和边
    pub segments: Vec<PathSegment>,
}

/// 路径段
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PathSegment {
    Vertex(Vertex),
    Edge(Edge),
}

/// 图数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphData {
    /// 顶点集合
    pub vertices: Vec<Vertex>,
    /// 边集合
    pub edges: Vec<Edge>,
}

/// 统计信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Statistics {
    /// 顶点数量
    pub vertex_count: usize,
    /// 边数量
    pub edge_count: usize,
    /// 其他统计信息
    pub metadata: Vec<(String, ScalarValue)>,
}

/// 查询错误
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryError {
    /// 错误代码
    pub code: String,
    /// 错误消息
    pub message: String,
    /// 错误详情
    pub details: Option<String>,
    /// 错误位置
    pub position: Option<ErrorPosition>,
}

/// 错误位置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorPosition {
    /// 行号
    pub line: usize,
    /// 列号
    pub column: usize,
    /// 偏移量
    pub offset: usize,
}

impl QueryResult {
    /// 创建成功结果
    pub fn success(affected_rows: usize, data: Option<QueryData>, execution_time_ms: u64) -> Self {
        QueryResult::Success {
            affected_rows,
            data,
            execution_time_ms,
        }
    }
    
    /// 创建错误结果
    pub fn error(error: QueryError, execution_time_ms: u64) -> Self {
        QueryResult::Error {
            error,
            execution_time_ms,
        }
    }
    
    /// 检查是否成功
    pub fn is_success(&self) -> bool {
        matches!(self, QueryResult::Success { .. })
    }
    
    /// 检查是否失败
    pub fn is_error(&self) -> bool {
        matches!(self, QueryResult::Error { .. })
    }
    
    /// 获取成功数据
    pub fn get_success_data(&self) -> Option<&QueryData> {
        match self {
            QueryResult::Success { data, .. } => data.as_ref(),
            _ => None,
        }
    }
    
    /// 获取错误信息
    pub fn get_error(&self) -> Option<&QueryError> {
        match self {
            QueryResult::Error { error, .. } => Some(error),
            _ => None,
        }
    }
    
    /// 获取执行时间
    pub fn execution_time(&self) -> u64 {
        match self {
            QueryResult::Success { execution_time_ms, .. } => *execution_time_ms,
            QueryResult::Error { execution_time_ms, .. } => *execution_time_ms,
        }
    }
}

impl QueryError {
    /// 创建新的查询错误
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
            position: None,
        }
    }
    
    /// 设置错误详情
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
    
    /// 设置错误位置
    pub fn with_position(mut self, line: usize, column: usize, offset: usize) -> Self {
        self.position = Some(ErrorPosition {
            line,
            column,
            offset,
        });
        self
    }
}

impl From<CoreQueryError> for QueryError {
    fn from(query_error: CoreQueryError) -> Self {
        QueryError::new("QUERY_ERROR", query_error.to_string())
    }
}

impl From<DBError> for QueryError {
    fn from(db_error: DBError) -> Self {
        QueryError::new("DB_ERROR", db_error.to_string())
    }
}

impl Record {
    /// 创建新记录
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
        }
    }
    
    /// 添加字段
    pub fn add_field(&mut self, name: impl Into<String>, value: FieldValue) {
        self.fields.push((name.into(), value));
    }
    
    /// 获取字段值
    pub fn get_field(&self, name: &str) -> Option<&FieldValue> {
        self.fields.iter().find(|(n, _)| n == name).map(|(_, v)| v)
    }
    
    /// 获取字段数量
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }
}

impl Vertex {
    /// 创建新顶点
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            tags: Vec::new(),
            properties: Vec::new(),
        }
    }
    
    /// 添加标签
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        self.tags.push(tag.into());
    }
    
    /// 添加属性
    pub fn add_property(&mut self, name: impl Into<String>, value: ScalarValue) {
        self.properties.push((name.into(), value));
    }
    
    /// 获取属性值
    pub fn get_property(&self, name: &str) -> Option<&ScalarValue> {
        self.properties.iter().find(|(n, _)| n == name).map(|(_, v)| v)
    }
}

impl Edge {
    /// 创建新边
    pub fn new(id: impl Into<String>, edge_type: impl Into<String>, src: impl Into<String>, dst: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            edge_type: edge_type.into(),
            src: src.into(),
            dst: dst.into(),
            properties: Vec::new(),
            ranking: None,
        }
    }
    
    /// 添加属性
    pub fn add_property(&mut self, name: impl Into<String>, value: ScalarValue) {
        self.properties.push((name.into(), value));
    }
    
    /// 设置排名
    pub fn set_ranking(&mut self, ranking: i64) {
        self.ranking = Some(ranking);
    }
    
    /// 获取属性值
    pub fn get_property(&self, name: &str) -> Option<&ScalarValue> {
        self.properties.iter().find(|(n, _)| n == name).map(|(_, v)| v)
    }
}

impl Path {
    /// 创建新路径
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }
    
    /// 添加顶点
    pub fn add_vertex(&mut self, vertex: Vertex) {
        self.segments.push(PathSegment::Vertex(vertex));
    }
    
    /// 添加边
    pub fn add_edge(&mut self, edge: Edge) {
        self.segments.push(PathSegment::Edge(edge));
    }
    
    /// 获取路径中的所有顶点
    pub fn get_vertices(&self) -> Vec<&Vertex> {
        self.segments.iter().filter_map(|seg| {
            match seg {
                PathSegment::Vertex(v) => Some(v),
                _ => None,
            }
        }).collect()
    }
    
    /// 获取路径中的所有边
    pub fn get_edges(&self) -> Vec<&Edge> {
        self.segments.iter().filter_map(|seg| {
            match seg {
                PathSegment::Edge(e) => Some(e),
                _ => None,
            }
        }).collect()
    }
    
    /// 获取路径长度（边的数量）
    pub fn length(&self) -> usize {
        self.segments.iter().filter(|seg| matches!(seg, PathSegment::Edge(_))).count()
    }
}

impl GraphData {
    /// 创建新图数据
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            edges: Vec::new(),
        }
    }
    
    /// 添加顶点
    pub fn add_vertex(&mut self, vertex: Vertex) {
        self.vertices.push(vertex);
    }
    
    /// 添加边
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }
    
    /// 获取顶点数量
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
    
    /// 获取边数量
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

impl Default for Record {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for GraphData {
    fn default() -> Self {
        Self::new()
    }
}
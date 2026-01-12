//! 图数据库响应结构定义
//!
//! 包含图数据库查询响应的相关数据结构

use crate::core::{Edge, Path, Value, Vertex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 表示图数据库查询的响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphResponse {
    pub data: GraphData,
    pub execution_time_ms: u64,
    pub message: Option<String>,
    pub success: bool,
}

/// 表示图数据库响应的数据部分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphData {
    /// 单个顶点结果
    Vertex(Vertex),
    /// 顶点列表
    Vertices(Vec<Vertex>),
    /// 单条边结果
    Edge(Edge),
    /// 边列表
    Edges(Vec<Edge>),
    /// 路径结果
    Path(Path),
    /// 路径列表
    Paths(Vec<Path>),
    /// 标量值结果（例如计数）
    Scalar(Value),
    /// 多个标量值
    Scalars(Vec<Value>),
    /// 键值对结果
    KeyValue(HashMap<String, Value>),
    /// 多个键值对
    KeyValues(Vec<HashMap<String, Value>>),
    /// 空结果
    Empty,
}

/// 表示用于API输出的格式化响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: u16,
    pub message: String,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, message: String) -> Self {
        Self {
            code: 200,
            message,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error: String, code: u16) -> Self {
        Self {
            code,
            message: "Error occurred".to_string(),
            data: None,
            error: Some(error),
        }
    }

    pub fn empty(message: String) -> Self {
        Self {
            code: 200,
            message,
            data: None,
            error: None,
        }
    }
}

impl GraphResponse {
    pub fn new(data: GraphData, execution_time_ms: u64, success: bool) -> Self {
        Self {
            data,
            execution_time_ms,
            message: None,
            success,
        }
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    pub fn success_with_data(data: GraphData, execution_time_ms: u64) -> Self {
        Self::new(data, execution_time_ms, true)
    }

    pub fn error_with_message(message: String, execution_time_ms: u64) -> Self {
        Self {
            data: GraphData::Empty,
            execution_time_ms,
            message: Some(message),
            success: false,
        }
    }
}

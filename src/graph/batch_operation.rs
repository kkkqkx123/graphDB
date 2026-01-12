//! 批量操作结构定义
//!
//! 包含批量图操作的相关数据结构

use crate::core::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 表示一批图操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperation {
    pub operations: Vec<GraphOperation>,
    pub atomic: bool, // 是否所有操作都应成功或失败
}

/// 表示批处理中的单个图操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphOperation {
    CreateVertex {
        vertex: crate::core::Vertex,
    },
    CreateEdge {
        edge: crate::core::Edge,
    },
    UpdateVertex {
        vid: Value,
        properties: HashMap<String, Value>,
    },
    UpdateEdge {
        src: Value,
        dst: Value,
        edge_type: String,
        properties: HashMap<String, Value>,
    },
    DeleteVertex {
        vid: Value,
    },
    DeleteEdge {
        src: Value,
        dst: Value,
        edge_type: String,
    },
    ReadVertex {
        vid: Value,
    },
    ReadEdge {
        src: Value,
        dst: Value,
        edge_type: String,
    },
}

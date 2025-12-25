//! 查询相关类型定义
//!
//! 定义图数据库查询系统中的核心类型

use crate::core::Value;
use crate::core::vertex_edge_path::{Edge, Path, Vertex};
use serde::{Deserialize, Serialize};

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

/// 字段值
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum FieldValue {
    Scalar(Value),
    List(Vec<FieldValue>),
    Map(Vec<(String, FieldValue)>),
    Vertex(Vertex),
    Edge(Edge),
    Path(Path),
}

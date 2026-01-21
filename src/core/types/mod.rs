// 核心类型系统模块
//
// 包含图数据库的核心类型定义，包括统一的数据类型、表达式、操作符、查询类型等

use serde::{Deserialize, Serialize};

pub mod expression;
pub mod graph;
pub mod operators;

/// 统一的数据类型枚举
///
/// 用于表示运行时值类型和查询语法层类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataType {
    Empty,
    Null,
    Bool,
    Int,
    Int8,
    Int16,
    Int32,
    Int64,
    Float,
    Double,
    String,
    Date,
    Time,
    DateTime,
    Vertex,
    Edge,
    Path,
    List,
    Map,
    Set,
    Geography,
    Duration,
    DataSet,
}

// 重新导出常用类型
pub use expression::{Expression};
pub use graph::EdgeDirection;
pub use operators::{AggregateFunction, BinaryOperator, UnaryOperator};

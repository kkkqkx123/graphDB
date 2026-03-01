pub mod edge;
pub mod expression;
pub mod graph_schema;
pub mod metadata;
pub mod metadata_version;
pub mod operators;
pub mod property;
pub mod span;
pub mod space;
pub mod query;
pub mod tag;

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
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
    FixedString(usize),
    VID,
    Blob,
    Timestamp,
}

// 从原子模块导出基础Schema类型
pub use self::edge::EdgeTypeInfo;
pub use self::property::PropertyDef;
pub use self::space::{SpaceInfo, generate_space_id, reset_space_id_counter};
pub use self::tag::TagInfo;

// 从metadata_version导出版本类型
pub use self::metadata_version::{MetadataVersion, SchemaVersion, SchemaHistory};

// 从metadata导出其他类型（后续逐步迁移）
pub use self::metadata::{
    SchemaChange, SchemaChangeType,
    ClusterInfo, CharsetInfo,
    InsertVertexInfo, InsertEdgeInfo, UpdateTarget, UpdateOp, UpdateInfo,
    PasswordInfo, UserInfo, UserAlterInfo,
    SchemaExportConfig, SchemaImportResult, ExportFormat,
    SchemaFieldChange, FieldChangeType, SchemaAlterOperation, AlterTargetType,
};

pub use self::expression::{Expression, ExpressionMeta, ExpressionContext, ContextualExpression, SerializableExpression, OptimizationFlags};
pub use self::graph_schema::{EdgeDirection, JoinType, OrderDirection, GraphTypeInference, VertexType, PathInfo, PropertyType, EdgeTypeRef};
pub use self::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
pub use self::span::{Position, Span, ToSpan};

pub use EdgeTypeInfo as EdgeTypeSchema;

/// YIELD列定义
/// 
/// 表示YIELD子句中的一个输出列
#[derive(Debug, Clone)]
pub struct YieldColumn {
    pub expression: crate::core::types::expression::contextual::ContextualExpression,
    pub alias: String,
    pub is_matched: bool,
}

impl YieldColumn {
    pub fn new(expression: crate::core::types::expression::contextual::ContextualExpression, alias: String) -> Self {
        Self {
            expression,
            alias,
            is_matched: false,
        }
    }

    pub fn with_matched(mut self, is_matched: bool) -> Self {
        self.is_matched = is_matched;
        self
    }

    /// 获取列名（别名）
    pub fn name(&self) -> &str {
        &self.alias
    }
}

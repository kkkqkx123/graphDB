pub mod expression;
pub mod graph_schema;
pub mod operators;
pub mod metadata;
pub mod span;

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

pub use self::metadata::{
    SpaceInfo, TagInfo, EdgeTypeInfo, PropertyDef,
    MetadataVersion, SchemaVersion, SchemaHistory, SchemaChange, SchemaChangeType,
    ClusterInfo, CharsetInfo,
    InsertVertexInfo, InsertEdgeInfo, UpdateTarget, UpdateOp, UpdateInfo,
    PasswordInfo,
    SchemaExportConfig, SchemaImportResult, ExportFormat,
    SchemaFieldChange, FieldChangeType, SchemaAlterOperation, AlterTargetType,
};

pub use self::expression::{Expression, ExpressionMeta};
pub use self::graph_schema::{EdgeDirection, JoinType, OrderDirection, GraphTypeInference, VertexType, PathInfo, PropertyType};
pub use self::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
pub use self::span::{Position, Span, ToSpan};

pub use EdgeTypeInfo as EdgeTypeSchema;

/// YIELD列定义
/// 
/// 表示YIELD子句中的一个输出列
#[derive(Debug, Clone)]
pub struct YieldColumn {
    pub expression: Expression,
    pub alias: String,
    pub is_matched: bool,
}

impl YieldColumn {
    pub fn new(expression: Expression, alias: String) -> Self {
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

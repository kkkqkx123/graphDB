pub mod cluster;
pub mod data_modification;
pub mod edge;
pub mod expr;
pub mod graph_schema;
pub mod import_export;
pub mod index;
pub mod metadata_version;
pub mod operators;
pub mod property;
pub mod property_trait;
pub mod query;
pub mod schema_change;
pub mod schema_trait;
pub mod space;
pub mod span;
pub mod tag;
pub mod user;

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
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float,
    Double,
    Decimal128,
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

// Exporting Base Schema Types from Atomic Modules
pub use self::edge::EdgeTypeInfo;
pub use self::index::{Index, IndexConfig, IndexField, IndexStatus, IndexType};
pub use self::property::PropertyDef;
pub use self::space::{generate_space_id, reset_space_id_counter, SpaceInfo};
pub use self::tag::TagInfo;

// Exporting version types from metadata_version
pub use self::metadata_version::{MetadataVersion, SchemaHistory, SchemaVersion};

// Exporting types from split submodules
pub use self::cluster::ClusterInfo;
pub use self::space::CharsetInfo;
pub use self::data_modification::{
    InsertEdgeInfo, InsertVertexInfo, UpdateInfo, UpdateOp, UpdateTarget,
};
pub use self::import_export::{ExportFormat, SchemaExportConfig, SchemaImportResult};
pub use self::schema_change::{
    AlterTargetType, FieldChangeType, SchemaAlterOperation, SchemaChange, SchemaChangeType,
    SchemaFieldChange,
};
pub use self::user::{PasswordInfo, UserAlterInfo, UserInfo};

pub use self::expr::{ContextualExpression, Expression, ExpressionMeta, SerializableExpression};
pub use self::graph_schema::{
    EdgeDirection, EdgeTypeRef, GraphTypeInference, JoinType, OrderDirection, PathInfo,
    PropertyType, VertexType,
};
pub use self::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
pub use self::query::{
    ExecutionMode, PlanType, QueryHint, QueryOptions, QueryStats, QueryStatus, QueryType,
};
pub use self::span::{Position, Span, ToSpan};

pub use EdgeTypeInfo as EdgeTypeSchema;

/// YIELD column definition
///
/// Indicates an output column in the YIELD clause
#[derive(Debug, Clone)]
pub struct YieldColumn {
    pub expression: crate::core::types::expr::contextual::ContextualExpression,
    pub alias: String,
    pub is_matched: bool,
}

impl YieldColumn {
    pub fn new(
        expression: crate::core::types::expr::contextual::ContextualExpression,
        alias: String,
    ) -> Self {
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

    /// Get column name (alias)
    pub fn name(&self) -> &str {
        &self.alias
    }
}

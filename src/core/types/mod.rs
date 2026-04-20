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
// Full-text search types
pub mod fulltext_query;

use oxicode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub enum DataType {
    Empty,
    Null,
    Bool,
    // Integer types: simplified to 3 types (aligned with PostgreSQL)
    SmallInt,   // i16
    Int,        // i32
    BigInt,     // i64
    // Floating point types: 2 types (standard practice)
    Float,      // f32
    Double,     // f64
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
    Vector,
    VectorDense(usize),
    VectorSparse(usize),

    /// JSON text type
    Json,
    /// JSONB binary type
    JsonB,
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Empty => write!(f, "EMPTY"),
            DataType::Null => write!(f, "NULL"),
            DataType::Bool => write!(f, "BOOL"),
            DataType::SmallInt => write!(f, "SMALLINT"),
            DataType::Int => write!(f, "INT"),
            DataType::BigInt => write!(f, "BIGINT"),
            DataType::Float => write!(f, "FLOAT"),
            DataType::Double => write!(f, "DOUBLE"),
            DataType::Decimal128 => write!(f, "DECIMAL128"),
            DataType::String => write!(f, "STRING"),
            DataType::Date => write!(f, "DATE"),
            DataType::Time => write!(f, "TIME"),
            DataType::DateTime => write!(f, "DATETIME"),
            DataType::Vertex => write!(f, "VERTEX"),
            DataType::Edge => write!(f, "EDGE"),
            DataType::Path => write!(f, "PATH"),
            DataType::List => write!(f, "LIST"),
            DataType::Map => write!(f, "MAP"),
            DataType::Set => write!(f, "SET"),
            DataType::Geography => write!(f, "GEOGRAPHY"),
            DataType::Duration => write!(f, "DURATION"),
            DataType::DataSet => write!(f, "DATASET"),
            DataType::FixedString(n) => write!(f, "FIXEDSTRING({})", n),
            DataType::VID => write!(f, "VID"),
            DataType::Blob => write!(f, "BLOB"),
            DataType::Timestamp => write!(f, "TIMESTAMP"),
            DataType::Vector => write!(f, "VECTOR"),
            DataType::VectorDense(n) => write!(f, "VECTOR_DENSE({})", n),
            DataType::VectorSparse(n) => write!(f, "VECTOR_SPARSE({})", n),
            DataType::Json => write!(f, "JSON"),
            DataType::JsonB => write!(f, "JSONB"),
        }
    }
}

// Exporting Base Schema Types from Atomic Modules
pub use self::edge::EdgeTypeInfo;
pub use self::index::{Index, IndexConfig, IndexField, IndexStatus, IndexType};
// Export full-text index types
pub use self::index::{
    BM25IndexConfig, CharsetType, FulltextEngineType, FulltextIndexField, FulltextIndexOptions,
    InversearchIndexConfig, TokenizeMode,
};
// Export full-text query types
pub use self::fulltext_query::{
    FieldQuery, FulltextQuery, FulltextQueryOptions, FulltextSearchResult, HighlightOptions,
    QueryExplanation, SearchResultEntry, ShardFailure, ShardsInfo, SortField, SortMissing,
    SortOrder,
};
pub use self::property::PropertyDef;
pub use self::space::{generate_space_id, reset_space_id_counter, SpaceInfo};
pub use self::tag::TagInfo;

// Exporting version types from metadata_version
pub use self::metadata_version::{MetadataVersion, SchemaHistory, SchemaVersion};

// Exporting types from split submodules
pub use self::cluster::ClusterInfo;
pub use self::data_modification::{
    InsertEdgeInfo, InsertVertexInfo, UpdateInfo, UpdateOp, UpdateTarget,
};
pub use self::import_export::{ExportFormat, SchemaExportConfig, SchemaImportResult};
pub use self::schema_change::{
    AlterTargetType, FieldChangeType, SchemaAlterOperation, SchemaChange, SchemaChangeType,
    SchemaFieldChange,
};
pub use self::space::CharsetInfo;
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

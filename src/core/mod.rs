pub mod error;
pub mod npath;
pub mod result;
pub mod type_system;
pub mod value;
pub mod vertex_edge_path;

// 新增的子模块
pub mod permission;
pub mod stats;
pub mod symbol;
pub mod types;

// 错误和结果类型
pub use error::{
    DBError, DBResult, ErrorCategory, ExpressionError, ExpressionErrorType, ExpressionPosition,
    GraphDBResult, ManagerError, ManagerResult, PermissionError, PermissionResult,
    PlanNodeVisitError, QueryError, QueryResult, SchemaValidationError, SchemaValidationResult,
    SessionError, SessionResult, StorageError, StorageResult, ValidationError, ValidationErrorType,
};

// 对外错误码
pub use error::codes::ErrorCategory as CodeErrorCategory;
pub use error::{ErrorCode, PublicError, ToPublicError};

// Result 系统
pub use result::{DefaultIterator, GetNeighborsIterator, PropIterator, ResultBuilder};

// 核心数据类型
pub use npath::{NPath, NPathEdgeIter, NPathIter, NPathVertexIter};
pub use value::*;
pub use vertex_edge_path::{Edge, Path, Step, Vertex};

// 表达式系统类型
pub use types::expression::Expression;
pub use types::DataType;

pub use types::graph_schema::EdgeDirection;

pub use types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};

pub use types::YieldColumn;

// 符号表类型
pub use symbol::{Symbol, SymbolTable};

// 其他核心类型
pub use type_system::TypeUtils;

// 权限类型
pub use permission::{Permission, RoleType};

// 统计类型
pub use stats::{
    ErrorInfo, ErrorSummary, ErrorType, MetricType, MetricValue, QueryMetrics, QueryPhase,
    QueryProfile, QueryStatus, StatsManager,
};

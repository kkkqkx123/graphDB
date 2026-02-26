pub mod error;
pub mod result;
pub mod type_system;
pub mod value;
pub mod vertex_edge_path;
pub mod npath;

// 新增的子模块
pub mod types;
pub mod symbol;
pub mod permission;
pub mod stats;

// 错误和结果类型
pub use error::{
    DBError, DBResult, GraphDBResult,
    ExpressionError, ExpressionErrorType, ExpressionPosition,
    QueryError, QueryResult,
    StorageError, StorageResult,
    SessionError, SessionResult,
    PermissionError, PermissionResult,
    ManagerError, ManagerResult,
    ValidationError, ValidationErrorType,
    SchemaValidationError, SchemaValidationResult,
    PlanNodeVisitError,
    ErrorCategory,
};

// 对外错误码
pub use error::{ErrorCode, PublicError, ToPublicError};
pub use error::codes::ErrorCategory as CodeErrorCategory;

// Result 系统
pub use result::{ResultBuilder, DefaultIterator, GetNeighborsIterator, PropIterator};

// 核心数据类型
pub use value::*;
pub use vertex_edge_path::{Edge, Path, Step, Vertex};
pub use npath::{NPath, NPathIter, NPathVertexIter, NPathEdgeIter};

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
pub use stats::{StatsManager, QueryMetrics, QueryProfile, MetricType, MetricValue, QueryPhase, ErrorType, ErrorInfo, ErrorSummary, QueryStatus};

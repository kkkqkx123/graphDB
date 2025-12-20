//! 统一错误处理系统 for GraphDB
//!
//! 这个模块提供了统一的错误类型，整合了所有子系统的错误

use std::fmt;
use thiserror::Error;

/// 统一的数据库错误类型
#[derive(Error, Debug)]
pub enum DBError {
    #[error("存储错误: {0}")]
    Storage(#[from] StorageError),

    #[error("查询错误: {0}")]
    Query(#[from] QueryError),

    #[error("表达式错误: {0}")]
    Expression(#[from] ExpressionError),

    #[error("计划节点访问错误: {0}")]
    Plan(#[from] PlanNodeVisitError),

    #[error("锁操作错误: {0}")]
    Lock(#[from] LockError),

    #[error("验证错误: {0}")]
    Validation(String),

    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("类型推导错误: {0}")]
    TypeDeduction(String),

    #[error("序列化错误: {0}")]
    Serialization(String),

    #[error("内部错误: {0}")]
    Internal(String),
}

/// 统一的结果类型
pub type DBResult<T> = Result<T, DBError>;

// 存储错误类型（从 storage/storage_error.rs 移动过来）
#[derive(Error, Debug, Clone)]
pub enum StorageError {
    #[error("数据库错误: {0}")]
    DbError(String),
    #[error("序列化错误: {0}")]
    SerializationError(String),
    #[error("节点未找到: {0:?}")]
    NodeNotFound(crate::core::Value),
    #[error("边未找到: {0:?}")]
    EdgeNotFound(crate::core::Value),
    #[error("事务错误: {0}")]
    TransactionError(String),
}

// 查询错误类型（从 query/mod.rs 移动过来）
#[derive(Error, Debug, Clone)]
pub enum QueryError {
    #[error("存储错误: {0}")]
    StorageError(String),
    #[error("解析错误: {0}")]
    ParseError(String),
    #[error("无效查询: {0}")]
    InvalidQuery(String),
    #[error("执行错误: {0}")]
    ExecutionError(String),
    #[error("表达式错误: {0}")]
    ExpressionError(String),
}

// 表达式错误类型（从 graph/expression/error.rs 移动过来）
#[derive(Error, Debug, Clone)]
pub enum ExpressionError {
    #[error("类型错误: {0}")]
    TypeError(String),
    #[error("属性未找到: {0}")]
    PropertyNotFound(String),
    #[error("函数错误: {0}")]
    FunctionError(String),
    #[error("无效操作: {0}")]
    InvalidOperation(String),
    #[error("未知函数: {0}")]
    UnknownFunction(String),
    #[error("无效的参数个数: {0}")]
    InvalidArgumentCount(String),
}

// 计划节点访问错误类型（从 query/planner/plan/plan_node_visitor.rs 移动过来）
#[derive(Error, Debug, Clone)]
pub enum PlanNodeVisitError {
    #[error("访问错误: {0}")]
    VisitError(String),
    #[error("遍历错误: {0}")]
    TraversalError(String),
    #[error("验证错误: {0}")]
    ValidationError(String),
}

// 锁操作错误类型
#[derive(Error, Debug, Clone)]
pub enum LockError {
    #[error("Mutex锁被污染: {reason}")]
    MutexPoisoned { reason: String },

    #[error("RwLock读锁被污染: {reason}")]
    RwLockReadPoisoned { reason: String },

    #[error("RwLock写锁被污染: {reason}")]
    RwLockWritePoisoned { reason: String },

    #[error("锁操作超时: {reason}")]
    LockTimeout { reason: String },
}

// 为现有错误类型实现转换
impl From<crate::storage::StorageError> for DBError {
    fn from(err: crate::storage::StorageError) -> Self {
        match err {
            crate::storage::StorageError::DbError(msg) => {
                DBError::Storage(StorageError::DbError(msg.to_string()))
            }
            crate::storage::StorageError::SerializationError(msg) => {
                DBError::Storage(StorageError::SerializationError(msg))
            }
            crate::storage::StorageError::NodeNotFound(value) => {
                DBError::Storage(StorageError::NodeNotFound(value))
            }
            crate::storage::StorageError::EdgeNotFound(value) => {
                DBError::Storage(StorageError::EdgeNotFound(value))
            }
            crate::storage::StorageError::InvalidOperation(msg) => {
                DBError::Storage(StorageError::DbError(msg))
            }
        }
    }
}

// 移除冲突的From实现，因为现在QueryError在core模块中定义

// ExpressionError 是本地定义，无需转换实现

impl From<crate::query::planner::plan::core::visitor::PlanNodeVisitError> for DBError {
    fn from(err: crate::query::planner::plan::core::visitor::PlanNodeVisitError) -> Self {
        match err {
            crate::query::planner::plan::core::visitor::PlanNodeVisitError::VisitError(msg) => {
                DBError::Plan(PlanNodeVisitError::VisitError(msg))
            }
            crate::query::planner::plan::core::visitor::PlanNodeVisitError::TraversalError(msg) => {
                DBError::Plan(PlanNodeVisitError::TraversalError(msg))
            }
            crate::query::planner::plan::core::visitor::PlanNodeVisitError::ValidationError(
                msg,
            ) => DBError::Plan(PlanNodeVisitError::ValidationError(msg)),
        }
    }
}

impl From<crate::query::visitor::TypeDeductionError> for DBError {
    fn from(err: crate::query::visitor::TypeDeductionError) -> Self {
        DBError::TypeDeduction(err.to_string())
    }
}

impl From<serde_json::Error> for DBError {
    fn from(err: serde_json::Error) -> Self {
        DBError::Serialization(err.to_string())
    }
}

// 为了向后兼容，保留旧的 Status 类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// 操作成功
    Ok,
    /// 值已插入
    Inserted,
    /// 一般错误
    Error(String),
    /// 文件未找到
    NoSuchFile(String),
    /// 不支持的功能
    NotSupported(String),
    /// 查询语法错误
    SyntaxError(String),
    /// 查询语义错误
    SemanticError(String),
    /// 图内存超出
    GraphMemoryExceeded,
    /// 没有语句可执行
    StatementEmpty,
    /// 存储中未找到键
    KeyNotFound,
    /// 部分成功
    PartialSuccess,
    /// 存储内存超出
    StorageMemoryExceeded,
    /// 空间未找到
    SpaceNotFound,
    /// 主机未找到
    HostNotFound,
    /// 标签未找到
    TagNotFound,
    /// 边未找到
    EdgeNotFound,
    /// 用户未找到
    UserNotFound,
    /// 索引未找到
    IndexNotFound,
    /// 组未找到
    GroupNotFound,
    /// 区域未找到
    ZoneNotFound,
    /// 领导者已更改
    LeaderChanged,
    /// 已平衡
    Balanced,
    /// 分区未找到
    PartNotFound,
    /// 监听器未找到
    ListenerNotFound,
    /// 会话未找到
    SessionNotFound,
    /// 权限错误
    PermissionError,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Ok => write!(f, "OK"),
            Status::Inserted => write!(f, "Inserted"),
            Status::Error(msg) => write!(f, "Error: {}", msg),
            Status::NoSuchFile(path) => write!(f, "No such file: {}", path),
            Status::NotSupported(feature) => write!(f, "Not supported: {}", feature),
            Status::SyntaxError(msg) => write!(f, "Syntax error: {}", msg),
            Status::SemanticError(msg) => write!(f, "Semantic error: {}", msg),
            Status::GraphMemoryExceeded => write!(f, "Graph memory exceeded"),
            Status::StatementEmpty => write!(f, "Statement is empty"),
            Status::KeyNotFound => write!(f, "Key not found"),
            Status::PartialSuccess => write!(f, "Partial success"),
            Status::StorageMemoryExceeded => write!(f, "Storage memory exceeded"),
            Status::SpaceNotFound => write!(f, "Space not found"),
            Status::HostNotFound => write!(f, "Host not found"),
            Status::TagNotFound => write!(f, "Tag not found"),
            Status::EdgeNotFound => write!(f, "Edge not found"),
            Status::UserNotFound => write!(f, "User not found"),
            Status::IndexNotFound => write!(f, "Index not found"),
            Status::GroupNotFound => write!(f, "Group not found"),
            Status::ZoneNotFound => write!(f, "Zone not found"),
            Status::LeaderChanged => write!(f, "Leader changed"),
            Status::Balanced => write!(f, "Balanced"),
            Status::PartNotFound => write!(f, "Part not found"),
            Status::ListenerNotFound => write!(f, "Listener not found"),
            Status::SessionNotFound => write!(f, "Session not found"),
            Status::PermissionError => write!(f, "Permission error"),
        }
    }
}

impl std::error::Error for Status {}

impl From<Status> for DBError {
    fn from(status: Status) -> Self {
        match status {
            Status::Error(msg) => DBError::Internal(msg),
            Status::SyntaxError(msg) => DBError::Query(QueryError::ParseError(msg)),
            Status::SemanticError(msg) => DBError::Query(QueryError::InvalidQuery(msg)),
            Status::KeyNotFound => DBError::Storage(StorageError::NodeNotFound(
                crate::core::Value::Null(crate::core::NullType::Null),
            )),
            _ => DBError::Internal(format!("Status error: {}", status)),
        }
    }
}

/// 类型别名，用于向后兼容
pub type StatusOr<T> = Result<T, Status>;
pub type GraphDBResult<T> = DBResult<T>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dberror_creation() {
        let storage_err = StorageError::NodeNotFound(crate::core::Value::Int(42));
        let db_err: DBError = storage_err.into();
        assert!(matches!(db_err, DBError::Storage(_)));
    }

    #[test]
    fn test_error_conversion() {
        let query_err = QueryError::ParseError("test error".to_string());
        let db_err: DBError = query_err.into();
        assert!(matches!(db_err, DBError::Query(_)));
    }

    #[test]
    fn test_status_to_dberror() {
        let status = Status::SyntaxError("test syntax error".to_string());
        let db_err: DBError = status.into();
        assert!(matches!(db_err, DBError::Query(_)));
    }
}

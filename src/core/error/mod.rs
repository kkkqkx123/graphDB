//! Unified Error Handling System for GraphDB
//!
//! ## Design concepts ##
//!
//! 1. **Design on demand**: selection of appropriate structures based on error complexity
//! - Core errors (e.g., expressions) use a structured design that retains the full error chain and location information
//! - Simple errors (e.g., transactions, indexes) are designed using enumerations for simplicity and efficiency
//!
//! 2. **Layered conversion**:
//!    - 核心错误使用 `#[from]` 注解自动转换，保留完整错误信息
//! - External errors are converted to strings using the custom `From` implementation, reducing module coupling.
//!
//! 3. **Harmonized interface**: `DBResult<T>` Provides harmonized return types to simplify error propagation

use thiserror::Error;

// submodule
pub mod auth;
pub mod codes;
pub mod expression;
pub mod manager;
pub mod optimize;
pub mod permission;
pub mod query;
pub mod session;
pub mod storage;
pub mod validation;

// Re-export the error code
pub use codes::{ErrorCategory as CodeErrorCategory, ErrorCode, PublicError, ToPublicError};

// Re-export all error types
pub use auth::{AuthError, AuthResult};
pub use expression::{ExpressionError, ExpressionErrorType, ExpressionPosition};
pub use manager::{ErrorCategory, ManagerError, ManagerResult};
pub use optimize::{CostError, CostResult, OptimizeError, OptimizeResult};
pub use permission::{PermissionError, PermissionResult};
pub use query::{PlanNodeVisitError, QueryError, QueryResult};
pub use session::{SessionError, SessionResult};
pub use storage::{StorageError, StorageResult};
pub use validation::{
    SchemaValidationError, SchemaValidationResult, ValidationError, ValidationErrorType,
};

pub use crate::core::types::DataType;

/// Harmonized database error types
#[derive(Error, Debug, Clone)]
pub enum DBError {
    #[error("存储错误: {0}")]
    Storage(#[from] StorageError),

    #[error("查询错误: {0}")]
    Query(#[from] QueryError),

    #[error("表达式错误: {0}")]
    Expression(#[from] ExpressionError),

    #[error("计划节点访问错误: {0}")]
    Plan(#[from] PlanNodeVisitError),

    #[error("管理器错误: {0}")]
    Manager(#[from] ManagerError),

    #[error("验证错误: {0}")]
    Validation(String),

    #[error("IO错误: {0}")]
    Io(String),

    #[error("类型推导错误: {0}")]
    TypeDeduction(String),

    #[error("序列化错误: {0}")]
    Serialization(String),

    #[error("索引错误: {0}")]
    Index(String),

    #[error("事务错误: {0}")]
    Transaction(String),

    #[error("内部错误: {0}")]
    Internal(String),

    #[error("会话错误: {0}")]
    Session(#[from] SessionError),

    #[error("认证错误: {0}")]
    Auth(#[from] AuthError),

    #[error("权限错误: {0}")]
    Permission(#[from] PermissionError),

    #[error("内存限制超出: {0}")]
    MemoryLimitExceeded(String),
}

/// Harmonized result types
pub type DBResult<T> = Result<T, DBError>;

/// Type aliases for backward compatibility
pub type GraphDBResult<T> = DBResult<T>;

// ==================== External error conversion implementation ====================

impl ToPublicError for DBError {
    fn to_public_error(&self) -> PublicError {
        PublicError::new(self.to_error_code(), self.to_public_message())
    }

    fn to_error_code(&self) -> ErrorCode {
        match self {
            DBError::Storage(se) => se.to_error_code(),
            DBError::Query(qe) => qe.to_error_code(),
            DBError::Expression(_) => ErrorCode::ExecutionError,
            DBError::Plan(_) => ErrorCode::ExecutionError,
            DBError::Manager(me) => me.to_error_code(),
            DBError::Validation(_) => ErrorCode::ValidationError,
            DBError::Io(_) => ErrorCode::InternalError,
            DBError::TypeDeduction(_) => ErrorCode::ValidationError,
            DBError::Serialization(_) => ErrorCode::InternalError,
            DBError::Index(_) => ErrorCode::InternalError,
            DBError::Transaction(_) => ErrorCode::ExecutionError,
            DBError::Internal(_) => ErrorCode::InternalError,
            DBError::Session(_) => ErrorCode::Unauthorized,
            DBError::Auth(_) => ErrorCode::Unauthorized,
            DBError::Permission(_) => ErrorCode::PermissionDenied,
            DBError::MemoryLimitExceeded(_) => ErrorCode::ResourceExhausted,
        }
    }

    fn to_public_message(&self) -> String {
        match self {
            // Internal errors do not reveal details
            DBError::Internal(_) => "内部服务器错误".to_string(),
            DBError::Io(_) => "IO操作失败".to_string(),
            DBError::Serialization(_) => "数据序列化失败".to_string(),
            DBError::Index(_) => "索引操作失败".to_string(),
            // Other errors return the original message
            _ => self.to_string(),
        }
    }
}

// ==================== External error conversion implementation ====================

impl From<serde_json::Error> for DBError {
    fn from(err: serde_json::Error) -> Self {
        DBError::Serialization(err.to_string())
    }
}

impl From<crate::query::planning::planner::PlannerError> for DBError {
    fn from(err: crate::query::planning::planner::PlannerError) -> Self {
        DBError::Query(QueryError::ExecutionError(err.to_string()))
    }
}

impl From<crate::query::parser::lexing::LexError> for DBError {
    fn from(err: crate::query::parser::lexing::LexError) -> Self {
        DBError::Query(QueryError::ParseError(err.to_string()))
    }
}

impl From<validation::SchemaValidationError> for DBError {
    fn from(err: validation::SchemaValidationError) -> Self {
        DBError::Validation(err.to_string())
    }
}

impl From<validation::ValidationError> for DBError {
    fn from(err: validation::ValidationError) -> Self {
        DBError::Validation(err.to_string())
    }
}

impl From<std::io::Error> for DBError {
    fn from(err: std::io::Error) -> Self {
        DBError::Io(err.to_string())
    }
}

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
}

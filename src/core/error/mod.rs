//! 统一错误处理系统 for GraphDB
//!
//! ## 设计理念
//!
//! 1. **按需设计**：根据错误复杂度选择合适的结构
//!    - 核心错误（如表达式）使用结构化设计，保留完整错误链和位置信息
//!    - 简单错误（如事务、索引）使用枚举设计，简洁高效
//!
//! 2. **分层转换**：
//!    - 核心错误使用 `#[from]` 注解自动转换，保留完整错误信息
//!    - 外部错误使用自定义 `From` 实现转换为字符串，降低模块耦合
//!
//! 3. **统一接口**：`DBResult<T>` 提供统一的返回类型，简化错误传播

use thiserror::Error;

// 子模块
pub mod codes;
pub mod storage;
pub mod query;
pub mod expression;
pub mod manager;
pub mod session;
pub mod permission;
pub mod validation;
pub mod other;

// 重新导出错误码
pub use codes::{ErrorCode, ErrorCategory as CodeErrorCategory, PublicError, ToPublicError};

// 重新导出所有错误类型
pub use storage::{StorageError, StorageResult};
pub use query::{QueryError, QueryResult};
pub use expression::{ExpressionError, ExpressionErrorType, ExpressionPosition};
pub use manager::{ManagerError, ManagerResult, ErrorCategory};
pub use session::{SessionError, SessionResult};
pub use permission::{PermissionError, PermissionResult};
pub use validation::{ValidationError, ValidationErrorType, SchemaValidationError, SchemaValidationResult};
pub use other::{PlanNodeVisitError, LockError};

pub use crate::core::types::DataType;

/// 统一的数据库错误类型
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

    #[error("锁操作错误: {0}")]
    Lock(#[from] LockError),

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

    #[error("权限错误: {0}")]
    Permission(#[from] PermissionError),

    #[error("内存限制超出: {0}")]
    MemoryLimitExceeded(String),
}

/// 统一的结果类型
pub type DBResult<T> = Result<T, DBError>;

/// 类型别名，用于向后兼容
pub type GraphDBResult<T> = DBResult<T>;

// ==================== 对外错误转换实现 ====================

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
            DBError::Lock(_) => ErrorCode::Conflict,
            DBError::Manager(me) => me.to_error_code(),
            DBError::Validation(_) => ErrorCode::ValidationError,
            DBError::Io(_) => ErrorCode::InternalError,
            DBError::TypeDeduction(_) => ErrorCode::ValidationError,
            DBError::Serialization(_) => ErrorCode::InternalError,
            DBError::Index(_) => ErrorCode::InternalError,
            DBError::Transaction(_) => ErrorCode::ExecutionError,
            DBError::Internal(_) => ErrorCode::InternalError,
            DBError::Session(_) => ErrorCode::Unauthorized,
            DBError::Permission(_) => ErrorCode::PermissionDenied,
            DBError::MemoryLimitExceeded(_) => ErrorCode::ResourceExhausted,
        }
    }

    fn to_public_message(&self) -> String {
        match self {
            // 内部错误不暴露细节
            DBError::Internal(_) => "内部服务器错误".to_string(),
            DBError::Io(_) => "IO操作失败".to_string(),
            DBError::Serialization(_) => "数据序列化失败".to_string(),
            DBError::Index(_) => "索引操作失败".to_string(),
            // 其他错误返回原始消息
            _ => self.to_string(),
        }
    }
}

// ==================== 外部错误转换实现 ====================

impl From<crate::index::IndexError> for DBError {
    fn from(err: crate::index::IndexError) -> Self {
        DBError::Index(err.to_string())
    }
}

impl From<serde_json::Error> for DBError {
    fn from(err: serde_json::Error) -> Self {
        DBError::Serialization(err.to_string())
    }
}

impl From<crate::query::planner::planner::PlannerError> for DBError {
    fn from(err: crate::query::planner::planner::PlannerError) -> Self {
        DBError::Query(QueryError::ExecutionError(err.to_string()))
    }
}

impl From<crate::query::parser::lexer::LexError> for DBError {
    fn from(err: crate::query::parser::lexer::LexError) -> Self {
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

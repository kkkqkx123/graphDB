//! 查询层错误类型
//!
//! 涵盖查询解析、验证和执行过程中的错误

use thiserror::Error;

use crate::core::error::codes::{ErrorCode, PublicError, ToPublicError};
use crate::core::error::storage::StorageError;
use crate::core::error::expression::{ExpressionError, ExpressionErrorType};
use crate::core::error::manager::ManagerError;
use crate::core::error::session::SessionError;
use crate::core::error::permission::PermissionError;
use crate::core::error::DBError;

/// 计划节点访问错误类型
///
/// 涵盖查询计划遍历和验证过程中的错误
#[derive(Error, Debug, Clone)]
pub enum PlanNodeVisitError {
    #[error("访问错误: {0}")]
    VisitError(String),
    #[error("遍历错误: {0}")]
    TraversalError(String),
    #[error("验证错误: {0}")]
    ValidationError(String),
}

/// 查询操作结果类型别名
pub type QueryResult<T> = Result<T, QueryError>;

/// 查询层错误类型
#[derive(Error, Debug, Clone, PartialEq)]
pub enum QueryError {
    #[error("存储错误: {0}")]
    StorageError(String),

    #[error("解析错误: {0}")]
    ParseError(String),

    #[error("规划错误: {0}")]
    PlanningError(String),

    #[error("优化错误: {0}")]
    OptimizationError(String),

    #[error("无效查询: {0}")]
    InvalidQuery(String),

    #[error("执行错误: {0}")]
    ExecutionError(String),

    #[error("表达式错误: {0}")]
    ExpressionError(String),

    #[error("计划节点访问错误: {0}")]
    PlanNodeVisitError(String),
}

impl QueryError {
    pub fn pipeline_parse_error<E: std::error::Error>(e: E) -> Self {
        QueryError::ParseError(e.to_string())
    }
    
    pub fn pipeline_validation_error<E: std::error::Error>(e: E) -> Self {
        QueryError::InvalidQuery(e.to_string())
    }
    
    pub fn pipeline_planning_error<E: std::error::Error>(e: E) -> Self {
        QueryError::PlanningError(e.to_string())
    }
    
    pub fn pipeline_optimization_error<E: std::error::Error>(e: E) -> Self {
        QueryError::OptimizationError(e.to_string())
    }
    
    pub fn pipeline_execution_error<E: std::error::Error>(e: E) -> Self {
        QueryError::ExecutionError(e.to_string())
    }
    
    pub fn pipeline_error(phase: &str, message: String) -> Self {
        match phase {
            "parse" => QueryError::ParseError(message),
            "validate" | "validation" => QueryError::InvalidQuery(message),
            "plan" | "planning" => QueryError::PlanningError(message),
            "optimize" | "optimization" => QueryError::OptimizationError(message),
            "execute" | "execution" => QueryError::ExecutionError(message),
            _ => QueryError::ExecutionError(format!("[{}] {}", phase, message)),
        }
    }
}

impl From<StorageError> for QueryError {
    fn from(e: StorageError) -> Self {
        QueryError::StorageError(e.to_string())
    }
}

impl From<DBError> for QueryError {
    fn from(e: DBError) -> Self {
        match e {
            DBError::Query(qe) => qe,
            DBError::Storage(se) => QueryError::StorageError(se.to_string()),
            DBError::Expression(expression) => QueryError::ExpressionError(expression.to_string()),
            DBError::Plan(plan) => QueryError::ExecutionError(plan.to_string()),
            DBError::Manager(manager) => QueryError::ExecutionError(manager.to_string()),
            DBError::Validation(msg) => QueryError::InvalidQuery(msg),
            DBError::Io(io) => QueryError::ExecutionError(io.to_string()),
            DBError::TypeDeduction(msg) => QueryError::ExecutionError(msg),
            DBError::Serialization(msg) => QueryError::ExecutionError(msg),
            DBError::Index(msg) => QueryError::ExecutionError(msg),
            DBError::Transaction(msg) => QueryError::ExecutionError(msg),
            DBError::Internal(msg) => QueryError::ExecutionError(msg),
            DBError::Session(session) => QueryError::ExecutionError(session.to_string()),
            DBError::Auth(auth) => QueryError::ExecutionError(auth.to_string()),
            DBError::Permission(permission) => QueryError::ExecutionError(permission.to_string()),
            DBError::MemoryLimitExceeded(msg) => QueryError::ExecutionError(msg),
        }
    }
}

impl From<std::io::Error> for QueryError {
    fn from(e: std::io::Error) -> Self {
        QueryError::ExecutionError(e.to_string())
    }
}

impl From<PlanNodeVisitError> for QueryError {
    fn from(e: PlanNodeVisitError) -> Self {
        QueryError::PlanNodeVisitError(e.to_string())
    }
}

impl From<ManagerError> for QueryError {
    fn from(e: ManagerError) -> Self {
        QueryError::ExecutionError(e.to_string())
    }
}

impl From<SessionError> for QueryError {
    fn from(e: SessionError) -> Self {
        QueryError::ExecutionError(e.to_string())
    }
}

impl From<PermissionError> for QueryError {
    fn from(e: PermissionError) -> Self {
        QueryError::ExecutionError(e.to_string())
    }
}

impl From<ExpressionError> for QueryError {
    fn from(e: ExpressionError) -> Self {
        QueryError::ExpressionError(e.to_string())
    }
}

impl From<ExpressionErrorType> for QueryError {
    fn from(e: ExpressionErrorType) -> Self {
        QueryError::ExpressionError(e.to_string())
    }
}

impl ToPublicError for QueryError {
    fn to_public_error(&self) -> PublicError {
        PublicError::new(self.to_error_code(), self.to_public_message())
    }

    fn to_error_code(&self) -> ErrorCode {
        match self {
            QueryError::ParseError(_) => ErrorCode::ParseError,
            QueryError::InvalidQuery(_) => ErrorCode::ValidationError,
            QueryError::PlanningError(_) => ErrorCode::ExecutionError,
            QueryError::OptimizationError(_) => ErrorCode::ExecutionError,
            QueryError::ExecutionError(_) => ErrorCode::ExecutionError,
            QueryError::ExpressionError(_) => ErrorCode::ExecutionError,
            QueryError::StorageError(_) => ErrorCode::InternalError,
            QueryError::PlanNodeVisitError(_) => ErrorCode::ExecutionError,
        }
    }

    fn to_public_message(&self) -> String {
        self.to_string()
    }
}

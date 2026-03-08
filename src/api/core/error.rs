//! API 核心层错误类型
//!
//! 与传输层无关的业务逻辑错误

use thiserror::Error;

/// 扩展错误码类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendedErrorCode {
    // 无扩展错误
    None = 0,

    // 解析相关 (1000-1099)
    SyntaxError = 1000,
    SemanticError = 1001,
    UnexpectedToken = 1002,
    UnterminatedLiteral = 1003,

    // 类型相关 (1100-1199)
    TypeMismatch = 1100,
    DivisionByZero = 1101,
    OutOfRange = 1102,

    // 约束相关 (1200-1299)
    DuplicateKey = 1200,
    ForeignKeyConstraint = 1201,
    NotNullConstraint = 1202,
    UniqueConstraint = 1203,
    CheckConstraint = 1204,

    // 并发相关 (1300-1399)
    ConnectionLost = 1300,
    Deadlock = 1301,
    LockTimeout = 1302,

    // 图相关 (1400-1499)
    InvalidVertex = 1400,
    InvalidEdge = 1401,
    PathNotFound = 1402,
}

impl ExtendedErrorCode {
    /// 转换为整数错误码
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }
}

/// 核心层错误类型
#[derive(Error, Debug, Clone)]
pub enum CoreError {
    #[error("Query execution failed: {0}")]
    QueryExecutionFailed(String),

    #[error("Transaction operation failed: {0}")]
    TransactionFailed(String),

    #[error("Schema operation failed: {0}")]
    SchemaOperationFailed(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),

    /// 带详细信息的查询错误
    #[error("Query error: {message}")]
    DetailedQueryError {
        message: String,
        extended_code: ExtendedErrorCode,
        offset: Option<usize>,
    },
}

impl CoreError {
    /// 获取扩展错误码
    pub fn extended_code(&self) -> ExtendedErrorCode {
        match self {
            CoreError::DetailedQueryError { extended_code, .. } => *extended_code,
            _ => ExtendedErrorCode::None,
        }
    }

    /// 获取错误位置偏移量
    pub fn error_offset(&self) -> Option<usize> {
        match self {
            CoreError::DetailedQueryError { offset, .. } => *offset,
            _ => None,
        }
    }

    /// 创建带详细信息的查询错误
    pub fn detailed_query_error(
        message: impl Into<String>,
        extended_code: ExtendedErrorCode,
        offset: Option<usize>,
    ) -> Self {
        CoreError::DetailedQueryError {
            message: message.into(),
            extended_code,
            offset,
        }
    }
}

/// 核心层结果类型
pub type CoreResult<T> = Result<T, CoreError>;

// 从底层错误转换
impl From<crate::core::error::QueryError> for CoreError {
    fn from(err: crate::core::error::QueryError) -> Self {
        CoreError::QueryExecutionFailed(err.to_string())
    }
}

impl From<crate::storage::StorageError> for CoreError {
    fn from(err: crate::storage::StorageError) -> Self {
        CoreError::StorageError(err.to_string())
    }
}

impl From<crate::core::error::DBError> for CoreError {
    fn from(err: crate::core::error::DBError) -> Self {
        match err {
            crate::core::error::DBError::Query(e) => CoreError::QueryExecutionFailed(e.to_string()),
            crate::core::error::DBError::Storage(e) => CoreError::StorageError(e.to_string()),
            crate::core::error::DBError::Transaction(s) => CoreError::TransactionFailed(s),
            _ => CoreError::Internal(err.to_string()),
        }
    }
}

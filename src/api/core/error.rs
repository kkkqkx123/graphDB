//! API 核心层错误类型
//!
//! 与传输层无关的业务逻辑错误

use thiserror::Error;

/// 核心层错误类型
#[derive(Error, Debug, Clone)]
pub enum CoreError {
    #[error("查询执行失败: {0}")]
    QueryExecutionFailed(String),

    #[error("事务操作失败: {0}")]
    TransactionFailed(String),

    #[error("Schema操作失败: {0}")]
    SchemaOperationFailed(String),

    #[error("存储错误: {0}")]
    StorageError(String),

    #[error("无效的参数: {0}")]
    InvalidParameter(String),

    #[error("资源不存在: {0}")]
    NotFound(String),

    #[error("内部错误: {0}")]
    Internal(String),
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

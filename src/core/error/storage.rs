//! 存储层错误类型
//!
//! 涵盖数据库底层存储操作相关的错误

use thiserror::Error;

use crate::core::error::codes::{ErrorCode, PublicError, ToPublicError};

/// 存储层结果类型
pub type StorageResult<T> = Result<T, StorageError>;

/// 存储层错误类型
#[derive(Error, Debug, Clone)]
pub enum StorageError {
    #[error("数据库错误: {0}")]
    DbError(String),
    #[error("存储错误: {0}")]
    StorageError(String),
    #[error("序列化错误: {0}")]
    SerializeError(String),
    #[error("反序列化错误: {0}")]
    DeserializeError(String),
    #[error("节点未找到: {0:?}")]
    NodeNotFound(crate::core::Value),
    #[error("边未找到: {0:?}")]
    EdgeNotFound(crate::core::Value),
    #[error("事务错误: {0}")]
    TransactionError(String),
    #[error("事务未找到: {0}")]
    TransactionNotFound(u64),
    #[error("操作不支持: {0}")]
    NotSupported(String),
    #[error("冲突错误: {0}")]
    Conflict(String),
    #[error("锁错误: {0}")]
    LockError(String),
    #[error("锁超时: {0}")]
    LockTimeout(String),
    #[error("死锁检测")]
    Deadlock,
    #[error("连接错误: {0}")]
    ConnectionError(String),
    #[error("IO错误: {0}")]
    IOError(String),
    #[error("未找到: {0}")]
    NotFound(String),
    #[error("已存在: {0}")]
    AlreadyExists(String),
    #[error("无效输入: {0}")]
    InvalidInput(String),
    #[error("索引错误: {0}")]
    IndexError(String),
    #[error("解析错误: {0}")]
    ParseError(String),
}

impl StorageError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            StorageError::LockTimeout(_) | StorageError::Deadlock | StorageError::ConnectionError(_)
        )
    }
}

impl From<std::io::Error> for StorageError {
    fn from(e: std::io::Error) -> Self {
        StorageError::DbError(e.to_string())
    }
}

impl From<redb::Error> for StorageError {
    fn from(e: redb::Error) -> Self {
        StorageError::DbError(e.to_string())
    }
}

impl From<String> for StorageError {
    fn from(s: String) -> Self {
        StorageError::DbError(s)
    }
}

impl From<&str> for StorageError {
    fn from(s: &str) -> Self {
        StorageError::DbError(s.to_string())
    }
}

impl<T> From<std::sync::PoisonError<T>> for StorageError {
    fn from(e: std::sync::PoisonError<T>) -> Self {
        StorageError::LockError(e.to_string())
    }
}

impl ToPublicError for StorageError {
    fn to_public_error(&self) -> PublicError {
        PublicError::new(self.to_error_code(), self.to_public_message())
    }

    fn to_error_code(&self) -> ErrorCode {
        match self {
            StorageError::NodeNotFound(_) | StorageError::EdgeNotFound(_) | StorageError::NotFound(_) => {
                ErrorCode::ResourceNotFound
            }
            StorageError::AlreadyExists(_) => ErrorCode::ResourceAlreadyExists,
            StorageError::InvalidInput(_) => ErrorCode::InvalidInput,
            StorageError::LockTimeout(_) => ErrorCode::Timeout,
            StorageError::Deadlock => ErrorCode::Deadlock,
            StorageError::Conflict(_) => ErrorCode::Conflict,
            StorageError::NotSupported(_) => ErrorCode::InvalidStatement,
            StorageError::TransactionError(_) => ErrorCode::ExecutionError,
            _ => ErrorCode::InternalError,
        }
    }

    fn to_public_message(&self) -> String {
        match self {
            StorageError::NodeNotFound(_) => "节点不存在".to_string(),
            StorageError::EdgeNotFound(_) => "边不存在".to_string(),
            StorageError::NotFound(name) => format!("资源不存在: {}", name),
            StorageError::AlreadyExists(name) => format!("资源已存在: {}", name),
            _ => "存储操作失败".to_string(),
        }
    }
}

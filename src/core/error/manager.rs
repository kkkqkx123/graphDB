//! 管理器错误类型
//!
//! 涵盖Schema管理器、索引管理器、存储客户端等Manager层的错误

use thiserror::Error;

use crate::core::error::codes::{ErrorCode, PublicError, ToPublicError};

/// Manager操作结果类型
pub type ManagerResult<T> = Result<T, ManagerError>;

/// 错误分类
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// 可重试错误
    Retryable,
    /// 不可重试错误
    NonRetryable,
}

/// 管理器错误类型
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ManagerError {
    #[error("资源未找到: {0}")]
    NotFound(String),

    #[error("资源已存在: {0}")]
    AlreadyExists(String),

    #[error("无效输入: {0}")]
    InvalidInput(String),

    #[error("存储错误: {0}")]
    StorageError(String),

    #[error("Schema错误: {0}")]
    SchemaError(String),

    #[error("事务错误: {0}")]
    TransactionError(String),

    #[error("超时错误: {0}")]
    TimeoutError(String),

    #[error("其他错误: {0}")]
    Other(String),
}

impl ManagerError {
    /// 获取错误分类
    pub fn category(&self) -> ErrorCategory {
        match self {
            ManagerError::StorageError(_)
            | ManagerError::TimeoutError(_) => ErrorCategory::Retryable,
            _ => ErrorCategory::NonRetryable,
        }
    }

    /// 检查是否可重试
    pub fn is_retryable(&self) -> bool {
        matches!(self.category(), ErrorCategory::Retryable)
    }

    /// 创建未找到错误
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// 创建已存在错误
    pub fn already_exists(msg: impl Into<String>) -> Self {
        Self::AlreadyExists(msg.into())
    }

    /// 创建无效输入错误
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }

    /// 创建存储错误
    pub fn storage_error(msg: impl Into<String>) -> Self {
        Self::StorageError(msg.into())
    }

    /// 创建Schema错误
    pub fn schema_error(msg: impl Into<String>) -> Self {
        Self::SchemaError(msg.into())
    }

    /// 创建事务错误
    pub fn transaction_error(msg: impl Into<String>) -> Self {
        Self::TransactionError(msg.into())
    }

    /// 创建超时错误
    pub fn timeout_error(msg: impl Into<String>) -> Self {
        Self::TimeoutError(msg.into())
    }
}

impl ToPublicError for ManagerError {
    fn to_public_error(&self) -> PublicError {
        PublicError::new(self.to_error_code(), self.to_public_message())
    }

    fn to_error_code(&self) -> ErrorCode {
        match self {
            ManagerError::NotFound(_) => ErrorCode::ResourceNotFound,
            ManagerError::AlreadyExists(_) => ErrorCode::ResourceAlreadyExists,
            ManagerError::InvalidInput(_) => ErrorCode::InvalidInput,
            ManagerError::TimeoutError(_) => ErrorCode::Timeout,
            _ => ErrorCode::InternalError,
        }
    }

    fn to_public_message(&self) -> String {
        self.to_string()
    }
}

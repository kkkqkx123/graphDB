//! Storage layer error type
//!
//! Errors related to the underlying storage operations of the database

use thiserror::Error;

use crate::core::error::codes::{ErrorCode, PublicError, ToPublicError};

/// Storage layer result type
pub type StorageResult<T> = Result<T, StorageError>;

/// Storage layer error type
#[derive(Error, Debug, Clone)]
pub enum StorageError {
    #[error("Database error: {0}")]
    DbError(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Serialization error: {0}")]
    SerializeError(String),
    #[error("Deserialization error: {0}")]
    DeserializeError(String),
    #[error("Node not found: {0:?}")]
    NodeNotFound(crate::core::Value),
    #[error("Edge not found: {0:?}")]
    EdgeNotFound(crate::core::Value),
    #[error("Operation not supported: {0}")]
    NotSupported(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Lock timeout: {0}")]
    LockTimeout(String),
    #[error("Deadlock detected")]
    Deadlock,
    #[error("IO error: {0}")]
    IOError(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Parse error: {0}")]
    ParseError(String),
}

impl StorageError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, StorageError::LockTimeout(_) | StorageError::Deadlock)
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
        StorageError::DbError(e.to_string())
    }
}

impl From<oxicode::error::EncodeError> for StorageError {
    fn from(e: oxicoide::error::EncodeError) -> Self {
        StorageError::SerializeError(e.to_string())
    }
}

impl From<oxicode::error::DecodeError> for StorageError {
    fn from(e: oxicoide::error::DecodeError) -> Self {
        StorageError::DeserializeError(e.to_string())
    }
}

impl ToPublicError for StorageError {
    fn to_public_error(&self) -> PublicError {
        PublicError::new(self.to_error_code(), self.to_public_message())
    }

    fn to_error_code(&self) -> ErrorCode {
        match self {
            StorageError::NodeNotFound(_)
            | StorageError::EdgeNotFound(_)
            | StorageError::NotFound(_) => ErrorCode::ResourceNotFound,
            StorageError::AlreadyExists(_) => ErrorCode::ResourceAlreadyExists,
            StorageError::InvalidInput(_) => ErrorCode::InvalidInput,
            StorageError::LockTimeout(_) => ErrorCode::Timeout,
            StorageError::Deadlock => ErrorCode::Deadlock,
            StorageError::Conflict(_) => ErrorCode::Conflict,
            StorageError::NotSupported(_) => ErrorCode::InvalidStatement,
            _ => ErrorCode::InternalError,
        }
    }

    fn to_public_message(&self) -> String {
        match self {
            StorageError::NodeNotFound(_) => "Node does not exist".to_string(),
            StorageError::EdgeNotFound(_) => "Edge does not exist".to_string(),
            StorageError::NotFound(name) => format!("Resource not found: {}", name),
            StorageError::AlreadyExists(name) => format!("Resource already exists: {}", name),
            _ => "Storage operation failed".to_string(),
        }
    }
}

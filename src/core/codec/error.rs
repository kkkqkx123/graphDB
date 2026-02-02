//! Codec 错误类型定义

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CodecError {
    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Type mismatch: {0}")]
    TypeMismatch(String),

    #[error("Field not found: {0}")]
    FieldNotFound(String),

    #[error("Encoding error: {0}")]
    EncodingError(String),

    #[error("Schema version mismatch: expected {expected}, got {actual}")]
    SchemaVersionMismatch { expected: u64, actual: u64 },

    #[error("Unsupported data type: {0}")]
    UnsupportedDataType(String),

    #[error("Buffer overflow: {0}")]
    BufferOverflow(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, CodecError>;
pub type CodecResult<T> = std::result::Result<T, CodecError>;

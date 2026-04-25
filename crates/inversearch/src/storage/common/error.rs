//! Stored Error Type
//!
//! Defines the errors that may be returned by a store operation

use thiserror::Error;

/// storage error
#[derive(Debug, Error)]
pub enum StorageError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Redis errors (only available when using Redis features)
    #[cfg(feature = "store-redis")]
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    /// Storage not open
    #[error("Storage is not open")]
    NotOpen,

    /// Storage is open
    #[error("Storage is already open")]
    AlreadyOpen,

    /// misconfiguration
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// data corruption
    #[error("Data corruption detected: {0}")]
    Corruption(String),
}

/// Store the result type alias
pub type StorageResult<T> = Result<T, StorageError>;

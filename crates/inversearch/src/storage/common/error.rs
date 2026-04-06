//! 存储错误类型
//!
//! 定义存储操作可能返回的错误

use thiserror::Error;

/// 存储错误
#[derive(Debug, Error)]
pub enum StorageError {
    /// IO 错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// 序列化错误
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Redis 错误（仅在使用 Redis 特性时可用）
    #[cfg(feature = "store-redis")]
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    /// 存储未打开
    #[error("Storage is not open")]
    NotOpen,

    /// 存储已打开
    #[error("Storage is already open")]
    AlreadyOpen,

    /// 配置错误
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// 数据损坏
    #[error("Data corruption detected: {0}")]
    Corruption(String),
}

/// 存储结果类型别名
pub type StorageResult<T> = Result<T, StorageError>;

use crate::core::Value;
use sled;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum StorageError {
    #[error("Database error: {0}")]
    DbError(#[from] sled::Error),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Node not found: {0:?}")]
    NodeNotFound(Value),
    #[error("Edge not found: {0:?}")]
    EdgeNotFound(Value),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BatchError {
    #[error("Buffer overflow: {0}")]
    BufferOverflow(String),

    #[error("Queue is full")]
    QueueFull,

    #[error("Queue is closed")]
    QueueClosed,

    #[error("Index error: {0}")]
    IndexError(#[from] crate::sync::external_index::ExternalIndexError),

    #[error("Commit error: {0}")]
    CommitError(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Transaction error: {0}")]
    TransactionError(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

pub type BatchResult<T> = Result<T, BatchError>;

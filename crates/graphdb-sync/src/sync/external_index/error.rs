use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExternalIndexError {
    #[error("Fulltext index error: {0}")]
    FulltextError(String),

    #[error("Vector index error: {0}")]
    VectorError(String),

    #[error("Index not found: {0}")]
    IndexNotFound(String),

    #[error("Index already exists: {0}")]
    IndexAlreadyExists(String),

    #[error("Invalid index data: {0}")]
    InvalidData(String),

    #[error("Insert error: {0}")]
    InsertError(String),

    #[error("Delete error: {0}")]
    DeleteError(String),

    #[error("Commit error: {0}")]
    CommitError(String),

    #[error("Rollback error: {0}")]
    RollbackError(String),

    #[error("Stats error: {0}")]
    StatsError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type IndexResult<T> = Result<T, ExternalIndexError>;

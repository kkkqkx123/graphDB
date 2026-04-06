use thiserror::Error;

pub type Result<T> = std::result::Result<T, Bm25Error>;

#[derive(Error, Debug)]
pub enum Bm25Error {
    #[error("Index not found: {0}")]
    IndexNotFound(String),

    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Tantivy error: {0}")]
    TantivyError(#[from] tantivy::TantivyError),

    #[error("Query parser error: {0}")]
    QueryParserError(#[from] tantivy::query::QueryParserError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),

    #[error("Index creation failed: {0}")]
    IndexCreationFailed(String),

    #[error("Index commit failed: {0}")]
    IndexCommitFailed(String),

    #[error("Index not initialized")]
    IndexNotInitialized,

    #[error("Storage error: {0}")]
    StorageError(String),
}

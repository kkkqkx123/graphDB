use thiserror::Error;

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Engine not found: {0}")]
    EngineNotFound(String),

    #[error("Index not found: {0}")]
    IndexNotFound(String),

    #[error("Index already exists: {0}")]
    IndexAlreadyExists(String),

    #[error("Engine unavailable")]
    EngineUnavailable,

    #[error("Index corrupted: {0}")]
    IndexCorrupted(String),

    #[error("BM25 engine error: {0}")]
    Bm25Error(String),

    #[error("Inversearch engine error: {0}")]
    InversearchError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("Query parse error: {0}")]
    QueryParseError(String),

    #[error("Invalid doc ID format: {0}")]
    InvalidDocId(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, SearchError>;

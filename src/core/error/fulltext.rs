//! Fulltext Index and Coordinator Error Types
//!
//! Provides specialized error types for fulltext search and coordination layer.

use thiserror::Error;

/// Fulltext index operation errors
#[derive(Error, Debug, Clone)]
pub enum FulltextError {
    #[error("Index not found: {0}")]
    IndexNotFound(String),

    #[error("Index already exists: {0}")]
    IndexAlreadyExists(String),

    #[error("Engine not found for space {space_id}, tag {tag_name}, field {field_name}")]
    EngineNotFound {
        space_id: u64,
        tag_name: String,
        field_name: String,
    },

    #[error("Engine unavailable: {0}")]
    EngineUnavailable(String),

    #[error("Index corrupted: {0}")]
    IndexCorrupted(String),

    #[error("BM25 engine error: {0}")]
    Bm25Error(String),

    #[error("Inversearch engine error: {0}")]
    InversearchError(String),

    #[error("Query parse error: {0}")]
    QueryParseError(String),

    #[error("Invalid document ID: {0}")]
    InvalidDocId(String),

    #[error("Index configuration error: {0}")]
    ConfigError(String),

    #[error("Index operation timeout")]
    Timeout,

    #[error("Index is locked: {0}")]
    Locked(String),

    #[error("Index operation cancelled")]
    Cancelled,

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Coordinator operation errors
#[derive(Error, Debug, Clone)]
pub enum CoordinatorError {
    #[error("Fulltext index error: {0}")]
    Fulltext(#[from] FulltextError),

    #[error("Sync error: {0}")]
    Sync(String),

    #[error("Index creation failed for {tag_name}.{field_name}: {reason}")]
    IndexCreationFailed {
        tag_name: String,
        field_name: String,
        reason: String,
    },

    #[error("Index drop failed for {tag_name}.{field_name}: {reason}")]
    IndexDropFailed {
        tag_name: String,
        field_name: String,
        reason: String,
    },

    #[error("Index rebuild failed: {0}")]
    IndexRebuildFailed(String),

    #[error("Vertex change processing failed: {0}")]
    VertexChangeFailed(String),

    #[error("Space not found: {0}")]
    SpaceNotFound(u64),

    #[error("Tag not found: {0}")]
    TagNotFound(String),

    #[error("Field not indexed: {tag_name}.{field_name}")]
    FieldNotIndexed {
        tag_name: String,
        field_name: String,
    },

    #[error("Coordinator not initialized")]
    NotInitialized,

    #[error("Coordinator is shutting down")]
    ShuttingDown,

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type FulltextResult<T> = std::result::Result<T, FulltextError>;
pub type CoordinatorResult<T> = std::result::Result<T, CoordinatorError>;

impl From<crate::search::error::SearchError> for FulltextError {
    fn from(err: crate::search::error::SearchError) -> Self {
        match err {
            crate::search::error::SearchError::EngineNotFound(msg) => {
                FulltextError::EngineNotFound {
                    space_id: 0,
                    tag_name: String::new(),
                    field_name: msg,
                }
            }
            crate::search::error::SearchError::IndexNotFound(msg) => {
                FulltextError::IndexNotFound(msg)
            }
            crate::search::error::SearchError::IndexAlreadyExists(msg) => {
                FulltextError::IndexAlreadyExists(msg)
            }
            crate::search::error::SearchError::SpaceNotFound(space_id) => {
                FulltextError::Internal(format!("Space not found: {}", space_id))
            }
            crate::search::error::SearchError::TagNotFound(tag) => {
                FulltextError::Internal(format!("Tag not found: {}", tag))
            }
            crate::search::error::SearchError::FieldNotFound(field) => {
                FulltextError::Internal(format!("Field not found: {}", field))
            }
            crate::search::error::SearchError::EngineUnavailable => {
                FulltextError::EngineUnavailable("engine unavailable".to_string())
            }
            crate::search::error::SearchError::IndexCorrupted(msg) => {
                FulltextError::IndexCorrupted(msg)
            }
            crate::search::error::SearchError::Bm25Error(msg) => FulltextError::Bm25Error(msg),
            crate::search::error::SearchError::InversearchError(msg) => {
                FulltextError::InversearchError(msg)
            }
            crate::search::error::SearchError::IoError(e) => FulltextError::Internal(e.to_string()),
            crate::search::error::SearchError::SerializationError(msg) => {
                FulltextError::Internal(format!("Serialization error: {}", msg))
            }
            crate::search::error::SearchError::ConfigError(msg) => FulltextError::ConfigError(msg),
            crate::search::error::SearchError::QueryParseError(msg) => {
                FulltextError::QueryParseError(msg)
            }
            crate::search::error::SearchError::InvalidDocId(msg) => {
                FulltextError::InvalidDocId(msg)
            }
            crate::search::error::SearchError::Internal(msg) => FulltextError::Internal(msg),
        }
    }
}

impl From<crate::sync::SyncError> for CoordinatorError {
    fn from(err: crate::sync::SyncError) -> Self {
        CoordinatorError::Sync(err.to_string())
    }
}

impl From<crate::search::error::SearchError> for CoordinatorError {
    fn from(err: crate::search::error::SearchError) -> Self {
        CoordinatorError::Fulltext(FulltextError::from(err))
    }
}

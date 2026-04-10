//! Error types for embedding service

use thiserror::Error;

/// Embedding error type
#[derive(Debug, Error)]
pub enum EmbeddingError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// HTTP request error
    #[error("HTTP error: {0}")]
    Http(String),

    /// API error response
    #[error("API error: {0}")]
    Api(String),

    /// Invalid response format
    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    /// Token limit exceeded
    #[error("Token limit exceeded: {0} > {1}")]
    TokenLimitExceeded(usize, usize),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type alias for embedding operations
pub type Result<T> = std::result::Result<T, EmbeddingError>;

impl From<reqwest::Error> for EmbeddingError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            EmbeddingError::Http("Request timeout".to_string())
        } else if err.is_connect() {
            EmbeddingError::Http("Connection failed".to_string())
        } else {
            EmbeddingError::Http(err.to_string())
        }
    }
}

impl From<serde_json::Error> for EmbeddingError {
    fn from(err: serde_json::Error) -> Self {
        EmbeddingError::InvalidResponse(err.to_string())
    }
}

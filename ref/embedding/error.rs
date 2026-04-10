//! Error types for embedder configuration

use thiserror::Error;

/// Re-export EmbeddingError from base module for backward compatibility
pub use super::base::EmbeddingError;

/// Configuration error type for embedder
#[derive(Error, Debug, PartialEq)]
pub enum ConfigError {
    /// No API key provided
    #[error("No API key provided")]
    MissingApiKey,

    /// Base URL is required
    #[error("base_url is required")]
    MissingBaseUrl,

    /// Model is required
    #[error("model is required")]
    MissingModel,

    /// Invalid max batch tokens
    #[error("max_batch_tokens must be > 0")]
    InvalidMaxBatchTokens,

    /// Invalid max item tokens
    #[error("max_item_tokens must be > 0")]
    InvalidMaxItemTokens,

    /// Invalid URL
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

impl ConfigError {
    /// Create a missing API key error
    pub fn missing_api_key() -> Self {
        Self::MissingApiKey
    }

    /// Create a missing base URL error
    pub fn missing_base_url() -> Self {
        Self::MissingBaseUrl
    }

    /// Create a missing model error
    pub fn missing_model() -> Self {
        Self::MissingModel
    }

    /// Create an invalid max batch tokens error
    pub fn invalid_max_batch_tokens() -> Self {
        Self::InvalidMaxBatchTokens
    }

    /// Create an invalid max item tokens error
    pub fn invalid_max_item_tokens() -> Self {
        Self::InvalidMaxItemTokens
    }

    /// Create an invalid URL error
    pub fn invalid_url(url: impl Into<String>) -> Self {
        Self::InvalidUrl(url.into())
    }
}

// Convert embedding::ConfigError to common ConfigError
impl From<ConfigError> for crate::types::error::ConfigError {
    fn from(err: ConfigError) -> Self {
        Self::new(err.to_string())
    }
}

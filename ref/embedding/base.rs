//! Base traits and types for embedding providers
//!
//! This module defines the `EmbeddingProvider` trait that all embedding providers must implement.
//! This abstraction allows the project to support both HTTP-based providers (OpenAI, Gemini, etc.)
//! and local library-based providers (llama.cpp, candle, ort, etc.).

use async_trait::async_trait;

/// Provider type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderType {
    /// HTTP-based provider (OpenAI compatible)
    Http,
    /// llama.cpp local library
    LlamaCpp,
    /// Other local library (candle, ort, etc.)
    LocalLibrary,
}

/// Embedding provider trait
///
/// This trait defines the interface for all embedding providers,
/// including HTTP-based (OpenAI compatible) and local library-based providers.
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate embeddings for a list of texts
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError>;

    /// Get the embedding dimension for this provider
    fn dimension(&self) -> usize;

    /// Get the model name
    fn model_name(&self) -> &str;

    /// Get the provider type
    fn provider_type(&self) -> ProviderType;
}

/// Embedding error type
#[derive(Debug, thiserror::Error)]
pub enum EmbeddingError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Token limit exceeded: {0} > {1}")]
    TokenLimitExceeded(usize, usize),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<crate::llm::LlmError> for EmbeddingError {
    fn from(err: crate::llm::LlmError) -> Self {
        match err {
            crate::llm::LlmError::Config(msg) => EmbeddingError::Config(msg.to_string()),
            crate::llm::LlmError::Http(msg) => EmbeddingError::Http(msg.to_string()),
            crate::llm::LlmError::Api(msg) => EmbeddingError::Api(msg),
            crate::llm::LlmError::InvalidResponse(msg) => EmbeddingError::InvalidResponse(msg),
            crate::llm::LlmError::TokenLimitExceeded(actual, limit) => {
                EmbeddingError::TokenLimitExceeded(actual, limit)
            }
            _ => EmbeddingError::Internal(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_display() {
        let http = ProviderType::Http;
        let llama = ProviderType::LlamaCpp;
        let local = ProviderType::LocalLibrary;

        assert!(format!("{:?}", http).contains("Http"));
        assert!(format!("{:?}", llama).contains("LlamaCpp"));
        assert!(format!("{:?}", local).contains("LocalLibrary"));
    }

    #[test]
    fn test_provider_type_equality() {
        assert_eq!(ProviderType::Http, ProviderType::Http);
        assert_ne!(ProviderType::Http, ProviderType::LlamaCpp);
        assert_ne!(ProviderType::LlamaCpp, ProviderType::LocalLibrary);
    }

    #[test]
    fn test_embedding_error_display() {
        let err = EmbeddingError::Config("test error".to_string());
        assert!(err.to_string().contains("Configuration error"));
    }

    #[test]
    fn test_embedding_error_from_llm_error() {
        let llm_err = crate::llm::LlmError::api("test".to_string());
        let embed_err: EmbeddingError = llm_err.into();
        assert!(matches!(embed_err, EmbeddingError::Api(_)));
    }
}

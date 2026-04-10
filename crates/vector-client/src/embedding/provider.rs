//! Embedding provider trait

use async_trait::async_trait;

use super::error::EmbeddingError;

/// Provider type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderType {
    /// HTTP-based provider (OpenAI compatible)
    Http,
    /// Local library provider
    Local,
}

/// Embedding provider trait
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate embeddings for a list of texts
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError>;

    /// Get the embedding dimension
    fn dimension(&self) -> usize;

    /// Get the model name
    fn model_name(&self) -> &str;

    /// Get the provider type
    fn provider_type(&self) -> ProviderType;
}

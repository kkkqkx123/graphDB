//! Embedding provider trait and types

use async_trait::async_trait;

use super::error::EmbeddingError;

/// Provider type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderType {
    /// HTTP-based provider (OpenAI compatible)
    Http,
    /// Other local or in-memory provider
    LocalLibrary,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_debug() {
        assert_eq!(format!("{:?}", ProviderType::Http), "Http");
        assert_eq!(format!("{:?}", ProviderType::LocalLibrary), "LocalLibrary");
    }

    #[test]
    fn test_provider_type_eq() {
        assert_eq!(ProviderType::Http, ProviderType::Http);
        assert_ne!(ProviderType::Http, ProviderType::LocalLibrary);
    }

    #[test]
    fn test_provider_type_copy() {
        let a = ProviderType::Http;
        let b = a;
        assert_eq!(a, b);
    }
}

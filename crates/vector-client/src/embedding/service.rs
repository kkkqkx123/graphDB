//! Embedding service wrapper with multi-provider support

use super::config::EmbeddingConfig;
use super::error::{EmbeddingError, Result};
use super::provider::{EmbeddingProvider, ProviderType};

use super::providers::OpenAICompatibleProvider;

/// Embedding service wrapper
///
/// This service provides a unified interface for HTTP-based embedding providers.
pub struct EmbeddingService {
    provider: Box<dyn EmbeddingProvider>,
    config: EmbeddingConfig,
    dimension: usize,
}

impl EmbeddingService {
    /// Create a new embedding service with custom provider
    pub fn new(provider: Box<dyn EmbeddingProvider>, config: EmbeddingConfig) -> Self {
        let dimension = provider.dimension();
        Self {
            provider,
            config,
            dimension,
        }
    }

    /// Create from configuration (HTTP-based provider)
    ///
    /// This creates an OpenAI-compatible HTTP provider.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vector_client::EmbeddingService;
    ///
    /// let config = vector_client::EmbeddingConfig::new(
    ///     "http://localhost:11434/api/embeddings",
    ///     "all-minilm"
    /// );
    /// let service = EmbeddingService::from_config(config)?;
    /// ```
    pub fn from_config(config: EmbeddingConfig) -> Result<Self> {
        config.validate()?;

        let provider = Box::new(OpenAICompatibleProvider::new(config.clone())?);
        let dimension = provider.dimension();

        Ok(Self {
            provider,
            config,
            dimension,
        })
    }

    /// Embed a single text
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.provider.embed(&[text]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| EmbeddingError::InvalidResponse("No embedding returned".to_string()))
    }

    /// Embed multiple texts in batch
    pub async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        self.provider.embed(texts).await
    }

    /// Get the dimension
    pub fn dimension(&self) -> usize {
        self.provider.dimension()
    }

    /// Get the model name
    pub fn model_name(&self) -> &str {
        self.provider.model_name()
    }

    /// Get the provider type
    pub fn provider_type(&self) -> ProviderType {
        self.provider.provider_type()
    }
}

impl std::fmt::Debug for EmbeddingService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmbeddingService")
            .field("model", &self.config.model)
            .field("dimension", &self.dimension)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::provider::ProviderType;

    struct TestProvider {
        dim: usize,
    }

    #[async_trait::async_trait]
    impl EmbeddingProvider for TestProvider {
        async fn embed(
            &self,
            texts: &[&str],
        ) -> std::result::Result<Vec<Vec<f32>>, EmbeddingError> {
            Ok(texts.iter().map(|_| vec![0.0; self.dim]).collect())
        }

        fn dimension(&self) -> usize {
            self.dim
        }

        fn model_name(&self) -> &str {
            "test-model"
        }

        fn provider_type(&self) -> ProviderType {
            ProviderType::LocalLibrary
        }
    }

    #[test]
    fn test_service_creation() {
        let provider = Box::new(TestProvider { dim: 384 });
        let config = EmbeddingConfig::new("http://example.com", "test-model");
        let service = EmbeddingService::new(provider, config);
        assert_eq!(service.dimension(), 384);
        assert_eq!(service.model_name(), "test-model");
        assert_eq!(service.provider_type(), ProviderType::LocalLibrary);
    }

    #[test]
    fn test_from_config_fails_without_dimension() {
        let config = EmbeddingConfig::new("http://localhost:11434/api/embeddings", "model");
        let result = EmbeddingService::from_config(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_service_debug() {
        let provider = Box::new(TestProvider { dim: 768 });
        let config = EmbeddingConfig::new("http://example.com", "my-model").with_dimension(768);
        let service = EmbeddingService::new(provider, config);
        let debug_str = format!("{:?}", service);
        assert!(debug_str.contains("my-model"));
        assert!(debug_str.contains("768"));
    }
}

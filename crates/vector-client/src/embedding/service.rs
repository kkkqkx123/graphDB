//! Embedding service wrapper with multi-provider support

use super::config::EmbeddingConfig;
use super::error::{EmbeddingError, Result};
use super::provider::{EmbeddingProvider, ProviderType};

#[cfg(feature = "llama_cpp")]
use super::providers::local::llama_cpp_provider::{LlamaCppConfig, LlamaCppProvider};
use super::providers::OpenAICompatibleProvider;

/// Embedding service wrapper
///
/// This service provides a unified interface for different embedding providers,
/// including HTTP-based (OpenAI, Gemini, Ollama, etc.) and local libraries (llama-cpp).
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

    /// Create from llama.cpp configuration with GPU support
    ///
    /// # Arguments
    /// * `model_path` - Path to the GGUF model file
    /// * `pooling_type` - Pooling strategy: "cls", "mean", "last", "rank", or "none"
    /// * `dimension` - Embedding dimension (will be inferred from model if None)
    /// * `n_gpu_layers` - Number of layers to offload to GPU (default: 1000 for full offload)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vector_client::EmbeddingService;
    ///
    /// #[cfg(feature = "llama_cpp")]
    /// {
    ///     let service = EmbeddingService::from_llama_cpp_config(
    ///         "models/nomic-embed-text.gguf".to_string(),
    ///         "mean".to_string(),
    ///         Some(768),
    ///         Some(1000),  // Full GPU offload
    ///     ).expect("Failed to create service");
    /// }
    /// ```
    #[cfg(feature = "llama_cpp")]
    pub fn from_llama_cpp_config(
        model_path: String,
        pooling_type: String,
        dimension: Option<usize>,
        n_gpu_layers: Option<u32>,
    ) -> Result<Self> {
        let config = LlamaCppConfig::new(model_path)
            .with_pooling_type(pooling_type)
            .with_dimension(dimension.unwrap_or(768))
            .with_n_gpu_layers(n_gpu_layers.unwrap_or(1000));

        Self::from_llama_cpp_full_config(config)
    }

    /// Create from full llama.cpp configuration
    ///
    /// This method allows full control over all llama.cpp parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vector_client::EmbeddingService;
    /// use vector_client::embedding::providers::local::llama_cpp_provider::LlamaCppConfig;
    ///
    /// #[cfg(feature = "llama_cpp")]
    /// {
    ///     let config = LlamaCppConfig::new("models/nomic-embed-text.gguf")
    ///         .with_pooling_type("mean")
    ///         .with_dimension(768)
    ///         .with_n_ctx(4096)
    ///         .with_n_threads(8)
    ///         .with_n_threads_batch(16)
    ///         .with_n_batch(512)
    ///         .with_n_gpu_layers(1000);
    ///
    ///     let service = EmbeddingService::from_llama_cpp_full_config(config)
    ///         .expect("Failed to create service");
    /// }
    /// ```
    #[cfg(feature = "llama_cpp")]
    pub fn from_llama_cpp_full_config(config: LlamaCppConfig) -> Result<Self> {
        let provider = LlamaCppProvider::from_config(config)?;
        let provider_dimension = provider.dimension();

        let embedding_config = EmbeddingConfig::default();
        Ok(Self {
            provider: Box::new(provider),
            config: embedding_config,
            dimension: provider_dimension,
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

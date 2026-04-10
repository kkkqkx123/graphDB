//! OpenAI-compatible API embedder
//!
//! Supports OpenAI, Gemini, Azure, Ollama, llama.cpp server, and any OpenAI-compatible endpoint.
//!
//! This module uses the unified llm::LlmClient for HTTP operations.

use std::sync::Arc;
use std::time::Instant;

use crate::llm::{EmbeddingConfig, LlmClient, LlmConfig, LlmError};
use crate::metrics::{GenericCollector, MetricCategory, MetricKey, MetricType};

use super::base::{EmbeddingError, EmbeddingProvider, ProviderType};
use super::config::{EmbedderConfig, PreprocessorConfig};
use super::preprocessor::{
    NomicPreprocessor, StellaPreprocessor, TemplatePreprocessor, TextPreprocessor,
};
use super::response::ParsedResponse;

/// OpenAI-compatible API embedder
///
/// This embedder uses the unified llm::LlmClient for HTTP operations
/// while maintaining backward-compatible API.
pub struct OpenAICompatibleProvider {
    llm_client: Arc<LlmClient>,
    embed_config: EmbeddingConfig,
    preprocessor: PreprocessorConfig,
    /// Metrics collector for counter metrics
    counter_metrics: Arc<GenericCollector>,
    /// Metrics collector for histogram metrics
    histogram_metrics: Arc<GenericCollector>,
}

impl std::fmt::Debug for OpenAICompatibleProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenAICompatibleProvider")
            .field("model", &self.embed_config.model)
            .field("max_batch_tokens", &self.embed_config.max_batch_tokens)
            .field("max_item_tokens", &self.embed_config.max_item_tokens)
            .field("vector_dimension", &self.embed_config.vector_dimension)
            .finish_non_exhaustive()
    }
}

/// Create metrics collectors for embedder
fn create_metrics_collectors() -> (Arc<GenericCollector>, Arc<GenericCollector>) {
    let counter_metrics = Arc::new(GenericCollector::new(
        "embedder_counter",
        MetricType::Counter,
    ));

    let histogram_metrics = Arc::new(GenericCollector::new(
        "embedder_histogram",
        MetricType::Histogram,
    ));

    // Register to global registry
    let registry = crate::metrics::init_registry();
    registry.register("embedder_counter", counter_metrics.clone());
    registry.register("embedder_histogram", histogram_metrics.clone());

    (counter_metrics, histogram_metrics)
}

impl OpenAICompatibleProvider {
    /// Create embedder from configuration
    pub fn new(config: EmbedderConfig) -> Result<Self, LlmError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| LlmError::config(e.to_string()))?;

        // Convert EmbedderConfig to LlmConfig
        let llm_config = LlmConfig {
            api_keys: config.api_keys,
            base_url: config.base_url,
            provider: crate::llm::LlmProvider::OpenAi,
            timeout_secs: config.timeout_secs,
            max_retries: config.max_retries,
            retry_delay_ms: config.retry_delay_ms,
            proxy_url: config.proxy_url,
            extra_headers: config.extra_headers,
            extra_params: config.extra_params,
        };

        // Convert to EmbeddingConfig
        let embed_config = EmbeddingConfig {
            model: config.model,
            max_batch_tokens: config.max_batch_tokens,
            max_item_tokens: config.max_item_tokens,
            vector_dimension: None,
            use_base64: config.use_base64,
        };

        let llm_client = Arc::new(
            LlmClient::new(llm_config)
                .map_err(|e| LlmError::config(format!("Failed to create LLM client: {}", e)))?,
        );

        // Create metrics collectors
        let (counter_metrics, histogram_metrics) = create_metrics_collectors();

        Ok(Self {
            llm_client,
            embed_config,
            preprocessor: config.preprocessor,
            counter_metrics,
            histogram_metrics,
        })
    }

    /// Create embeddings for texts
    ///
    /// This method applies the configured preprocessor to all texts before embedding,
    /// and uses the configured response parser to handle the API response.
    pub async fn embed(&self, texts: &[&str]) -> Result<EmbeddingResult, LlmError> {
        if texts.is_empty() {
            return Ok(EmbeddingResult::default());
        }

        let start = Instant::now();
        let batch_size = texts.len();

        // Apply preprocessor
        let processed_texts = self.preprocess_texts(texts);
        let text_refs: Vec<&str> = processed_texts.iter().map(|s| s.as_str()).collect();

        // Use llm client for embedding
        let result = self.llm_client.embed(&text_refs, &self.embed_config).await;

        // Record metrics
        let status = if result.is_ok() { "success" } else { "error" };

        // Record embedding count
        let count_key = MetricKey::new(MetricCategory::Embedding, "embedding_count")
            .with_label("status", status);
        self.counter_metrics.increment(&count_key, 1);

        // Record latency distribution
        let latency_key = MetricKey::new(MetricCategory::Embedding, "embedding_latency_ms");
        let latency_ms = start.elapsed().as_millis() as f64;
        self.histogram_metrics.observe(&latency_key, latency_ms);

        // Record batch size distribution
        let batch_key = MetricKey::new(MetricCategory::Embedding, "embedding_batch_size");
        self.histogram_metrics
            .observe(&batch_key, batch_size as f64);

        result
    }

    /// Preprocess texts using the configured preprocessor
    fn preprocess_texts(&self, texts: &[&str]) -> Vec<String> {
        match &self.preprocessor {
            PreprocessorConfig::None => texts.iter().map(|s| s.to_string()).collect(),
            PreprocessorConfig::Prefix { prefix } => texts
                .iter()
                .map(|text| format!("{}{}", prefix, text))
                .collect(),
            PreprocessorConfig::Template { template } => {
                let preprocessor = TemplatePreprocessor::new(template.clone());
                preprocessor.process_batch(texts)
            }
            PreprocessorConfig::Nomic { task_type } => {
                let preprocessor = NomicPreprocessor::new(*task_type);
                preprocessor.process_batch(texts)
            }
            PreprocessorConfig::Stella { task_type } => {
                let preprocessor = StellaPreprocessor::new(*task_type);
                preprocessor.process_batch(texts)
            }
        }
    }

    /// Embed a single text
    pub async fn embed_one(&self, text: &str) -> Result<Vec<f32>, LlmError> {
        let result = self.embed(&[text]).await?;
        result
            .embeddings
            .into_iter()
            .next()
            .ok_or_else(|| LlmError::invalid_response("No embedding returned".to_string()))
    }

    /// Create embeddings with full response parsing (including sparse and ColBERT vectors)
    ///
    /// This is useful when using BGE-M3 with multi-modal return modes.
    /// Note: This method currently falls back to standard embedding as BGE-M3
    /// support requires additional response parsing logic.
    pub async fn embed_advanced(&self, texts: &[&str]) -> Result<ParsedResponse, LlmError> {
        // For now, delegate to standard embed and wrap result
        let result = self.embed(texts).await?;

        Ok(ParsedResponse {
            embeddings: result.embeddings,
            sparse_embeddings: Vec::new(),
            colbert_embeddings: Vec::new(),
            usage: super::response::TokenUsage {
                prompt_tokens: result.prompt_tokens,
                total_tokens: result.total_tokens,
            },
        })
    }
}

/// Implement EmbeddingProvider trait for OpenAICompatibleProvider
#[async_trait::async_trait]
impl EmbeddingProvider for OpenAICompatibleProvider {
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        // Use the existing embed method and convert the result
        let result = self.embed(texts).await?;
        Ok(result.embeddings)
    }

    fn dimension(&self) -> usize {
        // The dimension is determined by the model, not stored in the config
        // For now, return 0 as it's not directly available
        // TODO: Consider storing dimension in embed_config or deriving from model name
        0
    }

    fn model_name(&self) -> &str {
        &self.embed_config.model
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Http
    }
}

/// Re-export EmbeddingResult for backward compatibility
pub use crate::llm::EmbeddingResult;

/// Type alias for backward compatibility
pub type Embedder = OpenAICompatibleProvider;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::config::EmbedderConfig;
    use crate::metrics::collector::MetricsCollector;

    #[test]
    fn test_create_embedder() {
        let config =
            EmbedderConfig::openai("sk-test".to_string(), "text-embedding-3-small".to_string());
        let embedder = OpenAICompatibleProvider::new(config);
        assert!(embedder.is_ok());
    }

    #[test]
    fn test_preprocess_texts() {
        let config = EmbedderConfig {
            preprocessor: PreprocessorConfig::Prefix {
                prefix: "prefix: ".to_string(),
            },
            ..EmbedderConfig::openai("test".to_string(), "model".to_string())
        };
        let embedder = OpenAICompatibleProvider::new(config).expect("create failed");

        let texts = vec!["hello", "world"];
        let processed = embedder.preprocess_texts(&texts);

        assert_eq!(processed, vec!["prefix: hello", "prefix: world"]);
    }

    #[test]
    fn test_metrics_initialization() {
        let config =
            EmbedderConfig::openai("sk-test".to_string(), "text-embedding-3-small".to_string());
        let embedder = OpenAICompatibleProvider::new(config).expect("Failed to create embedder");

        // Verify metrics collectors are initialized
        assert_eq!(embedder.counter_metrics.name(), "embedder_counter");
        assert_eq!(embedder.histogram_metrics.name(), "embedder_histogram");
    }

    #[test]
    fn test_metrics_registry_registration() {
        // Initialize registry
        let registry = crate::metrics::init_registry();

        // Create embedder (which registers metrics)
        let config =
            EmbedderConfig::openai("sk-test".to_string(), "text-embedding-3-small".to_string());
        let _embedder = OpenAICompatibleProvider::new(config).expect("Failed to create embedder");

        // Verify collectors are registered
        assert!(registry.collector_count() >= 2);
    }

    #[tokio::test]
    async fn test_embed_empty_texts_metrics() {
        let config =
            EmbedderConfig::openai("sk-test".to_string(), "text-embedding-3-small".to_string());
        let embedder = OpenAICompatibleProvider::new(config).expect("Failed to create embedder");

        // Embed empty array should not record metrics
        let result = embedder.embed(&[]).await;
        assert!(result.is_ok());
        assert!(result.expect("result").embeddings.is_empty());

        // No metrics should be recorded for empty input
        assert_eq!(embedder.counter_metrics.len(), 0);
    }

    #[test]
    fn test_backward_compatibility_alias() {
        // Test that Embedder alias works
        let config =
            EmbedderConfig::openai("sk-test".to_string(), "text-embedding-3-small".to_string());
        let embedder: Embedder = OpenAICompatibleProvider::new(config).expect("create failed");
        assert_eq!(embedder.model_name(), "text-embedding-3-small");
    }

    #[test]
    fn test_embedding_provider_trait() {
        let config =
            EmbedderConfig::openai("sk-test".to_string(), "text-embedding-3-small".to_string());
        let provider = OpenAICompatibleProvider::new(config).expect("create failed");

        // Test trait methods
        assert_eq!(provider.model_name(), "text-embedding-3-small");
        assert_eq!(provider.provider_type(), ProviderType::Http);
        assert_eq!(provider.dimension(), 0);
    }
}

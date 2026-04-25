//! OpenAI-compatible HTTP provider for embeddings
//!
//! Supports OpenAI, Gemini, Azure, Ollama, llama.cpp server, and any OpenAI-compatible endpoint.
//!
//! This provider uses reqwest directly for HTTP operations without external LLM client dependencies.

use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::embedding::config::EmbeddingConfig;
use crate::embedding::error::{EmbeddingError, Result};
use crate::embedding::preprocessor::{
    NomicPreprocessor, NoopPreprocessor, PrefixPreprocessor, Preprocessor, StellaPreprocessor,
    TemplatePreprocessor,
};
use crate::embedding::provider::{EmbeddingProvider, ProviderType};

/// OpenAI-compatible HTTP provider
///
/// This provider supports any OpenAI-compatible API endpoint including:
/// - OpenAI API
/// - Google Gemini (via OpenAI compatibility layer)
/// - Azure OpenAI
/// - Ollama
/// - llama.cpp server
/// - Self-hosted embedding services
pub struct OpenAICompatibleProvider {
    client: Client,
    config: EmbeddingConfig,
    preprocessor: Box<dyn Preprocessor>,
    dimension: usize,
}

impl std::fmt::Debug for OpenAICompatibleProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenAICompatibleProvider")
            .field("model", &self.config.model)
            .field("base_url", &self.config.base_url)
            .field("dimension", &self.dimension)
            .finish()
    }
}

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    model: String,
    input: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    encoding_format: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
    #[allow(dead_code)]
    #[serde(default)]
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    index: usize,
    embedding: Vec<f32>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Usage {
    prompt_tokens: usize,
    total_tokens: usize,
}

impl OpenAICompatibleProvider {
    /// Create a new OpenAI-compatible provider
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the provider
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vector_client::embedding::{EmbeddingConfig, providers::OpenAICompatibleProvider};
    ///
    /// let config = EmbeddingConfig::new(
    ///     "https://api.openai.com/v1/embeddings",
    ///     "text-embedding-3-small"
    /// ).with_api_key("sk-xxx");
    ///
    /// let provider = OpenAICompatibleProvider::new(config)?;
    /// ```
    pub fn new(config: EmbeddingConfig) -> Result<Self> {
        config.validate()?;

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| EmbeddingError::Config(format!("Failed to create HTTP client: {}", e)))?;

        // Dimension must be provided by user via config
        let dimension = config.dimension.ok_or_else(|| {
            EmbeddingError::Config(
                "Dimension must be specified in EmbeddingConfig. \
                 Use EmbeddingConfig::with_dimension() to set it."
                    .to_string(),
            )
        })?;

        // Create preprocessor based on config
        let preprocessor = Self::create_preprocessor(&config.preprocessor);

        Ok(Self {
            client,
            config,
            preprocessor,
            dimension,
        })
    }

    /// Create preprocessor from configuration
    fn create_preprocessor(config: &crate::embedding::PreprocessorConfig) -> Box<dyn Preprocessor> {
        use crate::embedding::PreprocessorConfig;

        match config {
            PreprocessorConfig::None => Box::new(NoopPreprocessor),
            PreprocessorConfig::Prefix { prefix } => {
                Box::new(PrefixPreprocessor::new(prefix)) as Box<dyn Preprocessor>
            }
            PreprocessorConfig::Template { template } => {
                Box::new(TemplatePreprocessor::new(template)) as Box<dyn Preprocessor>
            }
            PreprocessorConfig::Nomic { task_type } => {
                Box::new(NomicPreprocessor::new(*task_type)) as Box<dyn Preprocessor>
            }
            PreprocessorConfig::Stella { task_type } => {
                Box::new(StellaPreprocessor::new(*task_type)) as Box<dyn Preprocessor>
            }
        }
    }

    /// Build embedding request
    fn build_request(&self, texts: &[&str]) -> EmbeddingRequest {
        let input = texts
            .iter()
            .map(|&t| self.preprocessor.preprocess(t))
            .collect();
        EmbeddingRequest {
            model: self.config.model.clone(),
            input,
            encoding_format: Some("float".to_string()),
        }
    }

    /// Parse response
    fn parse_response(&self, response: EmbeddingResponse) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = vec![Vec::new(); response.data.len()];

        for item in response.data {
            if item.index >= embeddings.len() {
                return Err(EmbeddingError::InvalidResponse(
                    "Invalid index in response".to_string(),
                ));
            }
            embeddings[item.index] = item.embedding;
        }

        Ok(embeddings)
    }

    /// Create embeddings for texts
    ///
    /// This method applies the configured preprocessor to all texts before embedding.
    pub async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let request = self.build_request(texts);

        let mut req_builder = self.client.post(&self.config.base_url).json(&request);

        // Add authentication if API key is provided
        if let Some(api_key) = &self.config.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req_builder.send().await.map_err(|e| {
            if e.is_timeout() {
                EmbeddingError::Http("Request timeout".to_string())
            } else if e.is_connect() {
                EmbeddingError::Http("Connection failed".to_string())
            } else {
                EmbeddingError::Http(e.to_string())
            }
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(EmbeddingError::Api(format!(
                "API error {}: {}",
                status, error_text
            )));
        }

        let embedding_response: EmbeddingResponse = response.json().await.map_err(|e| {
            EmbeddingError::InvalidResponse(format!("Failed to parse response: {}", e))
        })?;

        self.parse_response(embedding_response)
    }

    /// Embed a single text
    pub async fn embed_one(&self, text: &str) -> Result<Vec<f32>> {
        let result = self.embed(&[text]).await?;
        result
            .into_iter()
            .next()
            .ok_or_else(|| EmbeddingError::InvalidResponse("No embedding returned".to_string()))
    }

    /// Get the configuration
    pub fn config(&self) -> &EmbeddingConfig {
        &self.config
    }

    /// Get the preprocessor
    pub fn preprocessor(&self) -> &dyn Preprocessor {
        &*self.preprocessor
    }
}

#[async_trait::async_trait]
impl EmbeddingProvider for OpenAICompatibleProvider {
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        self.embed(texts).await
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        &self.config.model
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Http
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::preprocessor::NomicTaskType;
    use crate::embedding::PreprocessorConfig;

    #[test]
    fn test_create_provider() {
        let config = EmbeddingConfig::new("http://localhost:11434/api/embeddings", "all-minilm")
            .with_dimension(384);
        let provider = OpenAICompatibleProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_create_provider_with_api_key() {
        let config = EmbeddingConfig::new(
            "https://api.openai.com/v1/embeddings",
            "text-embedding-3-small",
        )
        .with_api_key("sk-test")
        .with_dimension(1536);
        let provider = OpenAICompatibleProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_create_provider_with_preprocessor() {
        let config =
            EmbeddingConfig::new("http://localhost:11434/api/embeddings", "nomic-embed-text")
                .with_preprocessor(PreprocessorConfig::Nomic {
                    task_type: NomicTaskType::SearchDocument,
                })
                .with_dimension(768);
        let provider = OpenAICompatibleProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_dimension_required() {
        let config =
            EmbeddingConfig::new("http://localhost:11434/api/embeddings", "all-MiniLM-L6-v2");
        let result = OpenAICompatibleProvider::new(config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Dimension must be specified"));
    }

    #[test]
    fn test_custom_dimension() {
        let config = EmbeddingConfig::new("http://localhost:11434/api/embeddings", "custom-model")
            .with_dimension(512);
        let provider = OpenAICompatibleProvider::new(config).expect("create failed");
        assert_eq!(provider.dimension(), 512);
    }
}

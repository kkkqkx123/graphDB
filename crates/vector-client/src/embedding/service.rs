//! Embedding service implementation

use std::time::Duration;

use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};

use super::config::EmbeddingConfig;
use super::error::{EmbeddingError, Result};
use super::preprocessor::{NoopPreprocessor, Preprocessor};
use super::provider::{EmbeddingProvider, ProviderType};

/// OpenAI-compatible embedding provider
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
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
    #[allow(dead_code)]
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
    pub fn new(config: EmbeddingConfig) -> Result<Self> {
        config.validate()?;

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| EmbeddingError::Config(format!("Failed to create HTTP client: {}", e)))?;

        // Default dimension based on common models
        let dimension = if let Some(dim) = config.dimension {
            dim
        } else if config.model.contains("all-MiniLM") || config.model.contains("all-mpnet") {
            384
        } else if config.model.contains("text-embedding-3-small") {
            1536
        } else if config.model.contains("text-embedding-3-large") {
            3072
        } else {
            768 // Default fallback
        };

        Ok(Self {
            client,
            config,
            preprocessor: Box::new(NoopPreprocessor),
            dimension,
        })
    }

    /// Set preprocessor
    pub fn with_preprocessor(mut self, preprocessor: Box<dyn Preprocessor>) -> Self {
        self.preprocessor = preprocessor;
        self
    }

    /// Build request
    fn build_request(&self, texts: &[&str]) -> EmbeddingRequest {
        let input = texts.iter().map(|&t| self.preprocessor.preprocess(t)).collect();
        EmbeddingRequest {
            model: self.config.model.clone(),
            input,
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

    /// Build request with authentication
    #[allow(dead_code)]
    fn add_auth(&self, request: RequestBuilder) -> RequestBuilder {
        if let Some(api_key) = &self.config.api_key {
            request.header("Authorization", format!("Bearer {}", api_key))
        } else {
            request
        }
    }
}

#[async_trait::async_trait]
impl EmbeddingProvider for OpenAICompatibleProvider {
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let request = self.build_request(texts);

        let response = self
            .client
            .post(&self.config.base_url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(EmbeddingError::Api(format!(
                "API error {}: {}",
                status, error_text
            )));
        }

        let embedding_response: EmbeddingResponse = response.json().await?;
        self.parse_response(embedding_response)
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

/// Embedding service wrapper
pub struct EmbeddingService {
    provider: Box<dyn EmbeddingProvider>,
    config: EmbeddingConfig,
    dimension: usize,
}

impl EmbeddingService {
    /// Create a new embedding service
    pub fn new(provider: Box<dyn EmbeddingProvider>, config: EmbeddingConfig, dimension: usize) -> Self {
        Self { provider, config, dimension }
    }

    /// Create from configuration
    pub fn from_config(config: EmbeddingConfig) -> Result<Self> {
        let provider = Box::new(OpenAICompatibleProvider::new(config.clone())?);
        let dimension = if let Some(dim) = config.dimension {
            dim
        } else {
            384 // Default fallback
        };
        Ok(Self { provider, config, dimension })
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
}

impl std::fmt::Debug for EmbeddingService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmbeddingService")
            .field("model", &self.config.model)
            .field("dimension", &self.dimension)
            .finish()
    }
}

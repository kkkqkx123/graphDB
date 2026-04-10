//! Configuration for embedding service

use serde::{Deserialize, Serialize};

use super::error::EmbeddingError;

/// Embedding service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// API endpoint URL
    pub base_url: String,
    /// API key (optional for some providers)
    pub api_key: Option<String>,
    /// Model name to use
    pub model: String,
    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    /// Expected vector dimension (optional, will be auto-detected if not set)
    pub dimension: Option<usize>,
}

fn default_timeout() -> u64 {
    30
}

impl EmbeddingConfig {
    /// Create a new configuration
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: None,
            model: model.into(),
            timeout_secs: default_timeout(),
            dimension: None,
        }
    }

    /// Set API key
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Set dimension
    pub fn with_dimension(mut self, dimension: usize) -> Self {
        self.dimension = Some(dimension);
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), EmbeddingError> {
        if self.base_url.is_empty() {
            return Err(EmbeddingError::Config("base_url is required".to_string()));
        }

        if self.model.is_empty() {
            return Err(EmbeddingError::Config("model is required".to_string()));
        }

        // Validate URL format
        if let Err(e) = url::Url::parse(&self.base_url) {
            return Err(EmbeddingError::Config(format!("Invalid base_url: {}", e)));
        }

        Ok(())
    }
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434/api/embeddings".to_string(),
            api_key: None,
            model: "all-minilm".to_string(),
            timeout_secs: default_timeout(),
            dimension: None,
        }
    }
}

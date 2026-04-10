//! Embedder configuration
//!
//! Provides configuration types for the HTTP-based embedder.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

use super::response::BGEM3Mode;

/// Embedder configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedderConfig {
    /// API key(s) - supports multiple keys for rotation
    #[serde(default)]
    pub api_keys: Vec<String>,

    /// Base URL for API endpoint
    pub base_url: String,

    /// Model to use
    pub model: String,

    /// Maximum tokens per batch request
    #[serde(default = "default_max_batch_tokens")]
    pub max_batch_tokens: usize,

    /// Maximum tokens per single text item
    #[serde(default = "default_max_item_tokens")]
    pub max_item_tokens: usize,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Maximum retry attempts
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Initial retry delay in milliseconds
    #[serde(default = "default_retry_delay")]
    pub retry_delay_ms: u64,

    /// Proxy URL (optional)
    #[serde(default)]
    pub proxy_url: Option<String>,

    /// Extra HTTP headers
    #[serde(default)]
    pub extra_headers: HashMap<String, String>,

    /// Extra request parameters
    #[serde(default)]
    pub extra_params: HashMap<String, serde_json::Value>,

    /// Use base64 encoding for embeddings (default: true)
    #[serde(default = "default_true")]
    pub use_base64: bool,

    /// Preprocessor configuration for text transformation
    #[serde(default)]
    pub preprocessor: PreprocessorConfig,

    /// Response parser strategy
    #[serde(default)]
    pub response_parser: ResponseParserConfig,
}

/// Preprocessor configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PreprocessorConfig {
    /// No preprocessing (default)
    #[default]
    None,
    /// Simple prefix
    Prefix { prefix: String },
    /// Template with {text} placeholder
    Template { template: String },
    /// Nomic-Embed task type
    Nomic {
        task_type: super::preprocessor::NomicTaskType,
    },
    /// Stella task type
    Stella {
        task_type: super::preprocessor::StellaTaskType,
    },
}

/// Response parser configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseParserConfig {
    /// Standard OpenAI format (default)
    #[default]
    Standard,
    /// BGE-M3 multi-modal format
    BgeM3 { mode: BGEM3Mode },
}

fn default_max_batch_tokens() -> usize {
    8192
}
fn default_max_item_tokens() -> usize {
    8192
}
fn default_timeout() -> u64 {
    30
}
fn default_max_retries() -> u32 {
    3
}
fn default_retry_delay() -> u64 {
    1000
}
fn default_true() -> bool {
    true
}

impl EmbedderConfig {
    /// Create OpenAI configuration
    pub fn openai(api_key: String, model: String) -> Self {
        Self {
            api_keys: vec![api_key],
            base_url: "https://api.openai.com/v1".to_string(),
            model,
            ..Default::default()
        }
    }

    /// Create Gemini configuration
    pub fn gemini(api_key: String, model: Option<String>) -> Self {
        Self {
            api_keys: vec![api_key],
            base_url: "https://generativelanguage.googleapis.com/v1beta/openai/".to_string(),
            model: model.unwrap_or_else(|| "gemini-embedding-001".to_string()),
            ..Default::default()
        }
    }

    /// Create Ollama configuration
    pub fn ollama(model: String) -> Self {
        Self {
            api_keys: vec!["ollama".to_string()], // Ollama doesn't require API key
            base_url: "http://localhost:11434/v1".to_string(),
            model,
            ..Default::default()
        }
    }

    /// Create BGE-M3 configuration
    ///
    /// BGE-M3 supports dense, sparse (lexical), and ColBERT embeddings.
    /// Use `mode` to control which formats are returned.
    ///
    /// # Example
    ///
    /// ```
    /// use code_context_engine::embedding::config::{EmbedderConfig, BGEM3Mode};
    ///
    /// let config = EmbedderConfig::bge_m3(
    ///     "http://localhost:8000".to_string(),
    ///     "api-key".to_string(),
    ///     BGEM3Mode::All,
    /// );
    /// ```
    pub fn bge_m3(base_url: String, api_key: String, mode: BGEM3Mode) -> Self {
        let mut extra_params = HashMap::new();

        // Add BGE-M3 specific parameters
        extra_params.insert("return_dense".to_string(), json!(mode.has_dense()));
        extra_params.insert("return_sparse".to_string(), json!(mode.has_sparse()));
        extra_params.insert("return_colbert_vecs".to_string(), json!(mode.has_colbert()));

        Self {
            api_keys: vec![api_key],
            base_url,
            model: "bge-m3".to_string(),
            extra_params,
            max_item_tokens: 8192, // BGE-M3 supports long context
            response_parser: ResponseParserConfig::BgeM3 { mode },
            ..Default::default()
        }
    }

    /// Create Nomic-Embed configuration
    ///
    /// Nomic-Embed requires task-specific prefixes for optimal performance.
    /// Use `task_type` to specify the embedding task.
    ///
    /// # Example
    ///
    /// ```
    /// use code_context_engine::embedding::preprocessor::NomicTaskType;
    /// use code_context_engine::embedding::config::EmbedderConfig;
    ///
    /// let config = EmbedderConfig::nomic_embed(
    ///     "api-key".to_string(),
    ///     NomicTaskType::SearchDocument,
    /// );
    /// ```
    pub fn nomic_embed(api_key: String, task_type: super::preprocessor::NomicTaskType) -> Self {
        Self {
            api_keys: vec![api_key],
            base_url: "https://api.nomic.ai/v1".to_string(),
            model: "nomic-embed-text-v1".to_string(),
            preprocessor: PreprocessorConfig::Nomic { task_type },
            ..Default::default()
        }
    }

    /// Create Stella-EN-400M configuration
    ///
    /// Stella models use instruction-based templates for different tasks.
    ///
    /// # Example
    ///
    /// ```
    /// use code_context_engine::embedding::preprocessor::StellaTaskType;
    /// use code_context_engine::embedding::config::EmbedderConfig;
    ///
    /// let config = EmbedderConfig::stella_en_400m(StellaTaskType::S2P);
    /// ```
    pub fn stella_en_400m(task_type: super::preprocessor::StellaTaskType) -> Self {
        Self {
            api_keys: vec!["local".to_string()], // Local model, no API key needed
            base_url: "http://localhost:11434/v1".to_string(),
            model: "stella_en_400m".to_string(),
            preprocessor: PreprocessorConfig::Stella { task_type },
            ..Default::default()
        }
    }

    /// Create Azure OpenAI configuration
    pub fn azure(
        api_key: String,
        resource_name: String,
        deployment_name: String,
        api_version: Option<String>,
    ) -> Self {
        let version = api_version.unwrap_or_else(|| "2024-02-01".to_string());
        Self {
            api_keys: vec![api_key],
            base_url: format!(
                "https://{}.openai.azure.com/openai/deployments/{}?api-version={}",
                resource_name, deployment_name, version
            ),
            model: deployment_name,
            ..Default::default()
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), crate::embedding::ConfigError> {
        if self.api_keys.is_empty() {
            return Err(crate::embedding::ConfigError::missing_api_key());
        }
        if self.base_url.is_empty() {
            return Err(crate::embedding::ConfigError::missing_base_url());
        }
        if self.model.is_empty() {
            return Err(crate::embedding::ConfigError::missing_model());
        }
        if self.max_batch_tokens == 0 {
            return Err(crate::embedding::ConfigError::invalid_max_batch_tokens());
        }
        if self.max_item_tokens == 0 {
            return Err(crate::embedding::ConfigError::invalid_max_item_tokens());
        }
        Ok(())
    }
}

impl Default for EmbedderConfig {
    fn default() -> Self {
        Self {
            api_keys: Vec::new(),
            base_url: String::new(),
            model: String::new(),
            max_batch_tokens: default_max_batch_tokens(),
            max_item_tokens: default_max_item_tokens(),
            timeout_secs: default_timeout(),
            max_retries: default_max_retries(),
            retry_delay_ms: default_retry_delay(),
            proxy_url: None,
            extra_headers: HashMap::new(),
            extra_params: HashMap::new(),
            use_base64: true,
            preprocessor: PreprocessorConfig::default(),
            response_parser: ResponseParserConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_config() {
        let config =
            EmbedderConfig::openai("sk-test".to_string(), "text-embedding-3-small".to_string());
        assert!(config.validate().is_ok());
        assert_eq!(config.base_url, "https://api.openai.com/v1");
        assert_eq!(config.model, "text-embedding-3-small");
    }

    #[test]
    fn test_gemini_config() {
        let config = EmbedderConfig::gemini("api-key".to_string(), None);
        assert!(config.validate().is_ok());
        assert_eq!(config.model, "gemini-embedding-001");
    }

    #[test]
    fn test_ollama_config() {
        let config = EmbedderConfig::ollama("nomic-embed-text".to_string());
        assert!(config.validate().is_ok());
        assert_eq!(config.base_url, "http://localhost:11434/v1");
    }

    #[test]
    fn test_invalid_config() {
        let config = EmbedderConfig::default();
        assert!(config.validate().is_err());
    }
}

//! Embedder module for text vectorization
//!
//! Provides HTTP-based embedding using OpenAI-compatible APIs.
//!
//! # Supported Providers
//!
//! - OpenAI
//! - Google Gemini
//! - Azure OpenAI
//! - Ollama
//! - llama.cpp Server (OpenAI-compatible)
//! - Any OpenAI-compatible endpoint
//!
//! # Example
//!
//! ```rust,no_run
//! use embedding::{Embedder, EmbedderConfig};
//!
//! // OpenAI
//! let config = EmbedderConfig::openai("sk-xxx".to_string(), "text-embedding-3-small".to_string());
//! let embedder = Embedder::new(config)?;
//! let result = embedder.embed(&["Hello", "World"]).await?;
//!
//! // Gemini
//! let config = EmbedderConfig::gemini("api-key".to_string(), None);
//! let embedder = Embedder::new(config)?;
//! let result = embedder.embed(&["你好", "世界"]).await?;
//!
//! // llama.cpp Server (OpenAI-compatible)
//! let config = EmbedderConfig {
//!     api_keys: vec!["no-key".to_string()],
//!     base_url: "http://localhost:8080/v1".to_string(),
//!     model: "nomic-embed-text".to_string(),
//!     ..Default::default()
//! };
//! let embedder = Embedder::new(config)?;
//! let result = embedder.embed(&["Hello", "World"]).await?;
//! # Ok::<(), llm::LlmError>(())
//! ```

mod base;
mod config;
mod error;
#[cfg(feature = "llama_cpp")]
mod llama_cpp_provider;
mod openai_compatible_provider;
mod preprocessor;
mod response;

pub use base::{EmbeddingError, EmbeddingProvider, ProviderType};
pub use config::{EmbedderConfig, PreprocessorConfig, ResponseParserConfig};
pub use error::ConfigError;
#[cfg(feature = "llama_cpp")]
pub use llama_cpp_provider::LlamaCppProvider;
pub use openai_compatible_provider::{Embedder, OpenAICompatibleProvider};
pub use preprocessor::{
    ChainedPreprocessor, NomicPreprocessor, NomicTaskType, NoopPreprocessor, PrefixPreprocessor,
    StellaPreprocessor, StellaTaskType, TemplatePreprocessor, TextPreprocessor,
};
pub use response::{BGEM3Mode, ParsedResponse, ResponseParser, TokenUsage};

// Re-export token estimation from utils for backward compatibility
pub use crate::utils::token_estimation::{estimate_tokens, TokenEstimator};

// Re-export LlmError for convenience (replaces deprecated EmbedError)
pub use crate::llm::LlmError;

//! Embedding Service for vector search
//!
//! Provides text-to-vector embedding capabilities using various providers:
//! - HTTP-based: OpenAI, Gemini, Azure, Ollama, llama.cpp server
//! - Local libraries: llama.cpp, candle, ort, etc.

mod config;
mod error;
mod preprocessor;
mod provider;
mod service;
mod providers;

pub use config::EmbeddingConfig;
pub use error::EmbeddingError;
pub use preprocessor::{
    ChainedPreprocessor, NomicPreprocessor, NomicTaskType, NoopPreprocessor, Preprocessor, PreprocessorConfig,
    PrefixPreprocessor, StellaPreprocessor, StellaTaskType, TemplatePreprocessor,
};
pub use provider::{EmbeddingProvider, ProviderType};
pub use service::EmbeddingService;

// Re-export providers for advanced usage
pub use providers::OpenAICompatibleProvider;

#[cfg(feature = "llama_cpp")]
pub use providers::LlamaCppProvider;

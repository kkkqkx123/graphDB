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

pub use config::EmbeddingConfig;
pub use error::EmbeddingError;
pub use preprocessor::{
    ChainedPreprocessor, NomicPreprocessor, NomicTaskType, NoopPreprocessor, Preprocessor,
    PrefixPreprocessor, StellaPreprocessor, StellaTaskType, TemplatePreprocessor,
};
pub use provider::{EmbeddingProvider, ProviderType};
pub use service::EmbeddingService;

pub mod providers {
    pub use super::service::OpenAICompatibleProvider;
}

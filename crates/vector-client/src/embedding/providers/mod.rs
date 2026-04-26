//! Provider implementations
//!
//! This module contains concrete provider implementations:
//! - HTTP-based providers (OpenAI, Gemini, Ollama, etc.)
//! - Local library providers (llama-cpp, candle, etc.)

pub mod http;
#[cfg(feature = "llama_cpp")]
pub mod local;

pub use http::openai_compatible_provider::OpenAICompatibleProvider;

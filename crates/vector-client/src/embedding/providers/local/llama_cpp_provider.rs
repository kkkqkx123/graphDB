//! llama.cpp local library provider for embeddings with Vulkan support
//!
//! This module provides a provider that uses llama.cpp directly via Rust bindings,
//! avoiding the HTTP overhead of the llama.cpp server.
//!
//! This module is only available when the `llama_cpp` feature is enabled.
//!
//! # Design Notes
//!
//! This implementation uses a simplified design optimized for embedding models:
//!
//! 1. **No Context Pool**: Embedding models are stateless. Each request creates
//!    a fresh context, which is simpler and avoids complex lifetime management.
//!
//! 2. **No Mutex**: Each request gets its own context, eliminating lock contention
//!    and enabling true parallelism.
//!
//! 3. **No KV Cache Management**: Embedding models don't use KV cache (no autoregressive
//!    generation), so we don't need to manage or clear it.
//!
//! 4. **No Streaming**: Embedding models produce a fixed-size vector output,
//!    making streaming unnecessary.
//!
//! See `docs/llama_cpp_embedding_design.md` for detailed analysis.

use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::Arc;

use llama_cpp_2::context::params::{LlamaContextParams, LlamaPoolingType};
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};

use crate::embedding::error::{EmbeddingError, Result};
use crate::embedding::provider::{EmbeddingProvider, ProviderType};

/// Configuration for llama.cpp embedding provider
///
/// This struct encapsulates all configuration parameters for the provider,
/// avoiding the "too many arguments" anti-pattern.
#[derive(Debug, Clone)]
pub struct LlamaCppConfig {
    /// Path to the GGUF model file
    pub model_path: PathBuf,
    /// Pooling strategy: "cls", "mean", "last", "rank", or "none"
    pub pooling_type: String,
    /// Embedding dimension (will be inferred from model if None)
    pub dimension: Option<usize>,
    /// Context window size (default: 4096)
    pub n_ctx: u32,
    /// Number of threads for generation (default: 8)
    pub n_threads: u32,
    /// Number of threads for prompt processing (default: 16)
    pub n_threads_batch: u32,
    /// Physical batch size (default: 512)
    pub n_batch: u32,
    /// Micro-batch size (default: 256)
    pub n_ubatch: u32,
    /// Whether to offload K, Q, V to GPU (default: true)
    pub offload_kqv: bool,
    /// Number of layers to offload to GPU (default: 1000 for full offload)
    pub n_gpu_layers: u32,
}

impl Default for LlamaCppConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::new(),
            pooling_type: "mean".to_string(),
            dimension: None,
            n_ctx: 4096,
            n_threads: 8,
            n_threads_batch: 16,
            n_batch: 512,
            n_ubatch: 256,
            offload_kqv: true,
            n_gpu_layers: 1000,
        }
    }
}

impl LlamaCppConfig {
    /// Create a new config with the specified model path
    pub fn new(model_path: impl Into<PathBuf>) -> Self {
        Self {
            model_path: model_path.into(),
            ..Default::default()
        }
    }

    /// Set the pooling type
    pub fn with_pooling_type(mut self, pooling_type: impl Into<String>) -> Self {
        self.pooling_type = pooling_type.into();
        self
    }

    /// Set the embedding dimension
    pub fn with_dimension(mut self, dimension: usize) -> Self {
        self.dimension = Some(dimension);
        self
    }

    /// Set the context window size
    pub fn with_n_ctx(mut self, n_ctx: u32) -> Self {
        self.n_ctx = n_ctx;
        self
    }

    /// Set the number of threads for generation
    pub fn with_n_threads(mut self, n_threads: u32) -> Self {
        self.n_threads = n_threads;
        self
    }

    /// Set the number of threads for batch processing
    pub fn with_n_threads_batch(mut self, n_threads_batch: u32) -> Self {
        self.n_threads_batch = n_threads_batch;
        self
    }

    /// Set the batch size
    pub fn with_n_batch(mut self, n_batch: u32) -> Self {
        self.n_batch = n_batch;
        self
    }

    /// Set the micro-batch size
    pub fn with_n_ubatch(mut self, n_ubatch: u32) -> Self {
        self.n_ubatch = n_ubatch;
        self
    }

    /// Set whether to offload KQV to GPU
    pub fn with_offload_kqv(mut self, offload_kqv: bool) -> Self {
        self.offload_kqv = offload_kqv;
        self
    }

    /// Set the number of GPU layers
    pub fn with_n_gpu_layers(mut self, n_gpu_layers: u32) -> Self {
        self.n_gpu_layers = n_gpu_layers;
        self
    }
}

/// llama.cpp local library provider with Vulkan GPU acceleration support
///
/// This provider uses llama.cpp directly via Rust bindings for embedding generation.
/// When compiled with Vulkan support (CMAKE_ARGS="-DGGML_VULKAN=on"), it will
/// automatically utilize GPU acceleration for improved performance.
///
/// # Thread Safety
///
/// This provider is thread-safe. Each call to `embed()` creates a fresh context,
/// allowing concurrent requests without lock contention.
pub struct LlamaCppProvider {
    /// Backend must be kept alive as long as the provider exists.
    /// It is used when creating new contexts for each embedding request.
    backend: Arc<LlamaBackend>,
    /// Shared model instance. Thread-safe and can be used across multiple contexts.
    model: Arc<LlamaModel>,
    /// Configuration for the provider
    config: LlamaCppConfig,
    /// Embedding dimension (cached for performance)
    dimension: usize,
    /// Model name extracted from path
    model_name: String,
}

impl std::fmt::Debug for LlamaCppProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LlamaCppProvider")
            .field("config", &self.config)
            .field("dimension", &self.dimension)
            .field("model_name", &self.model_name)
            .finish()
    }
}

impl LlamaCppProvider {
    /// Create a new llama.cpp provider from configuration
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vector_client::embedding::providers::local::llama_cpp_provider::{LlamaCppProvider, LlamaCppConfig};
    ///
    /// let config = LlamaCppConfig::new("models/nomic-embed-text.gguf")
    ///     .with_pooling_type("mean")
    ///     .with_dimension(768)
    ///     .with_n_gpu_layers(1000);
    ///
    /// let provider = LlamaCppProvider::from_config(config)
    ///     .expect("Failed to create provider");
    /// ```
    pub fn from_config(config: LlamaCppConfig) -> Result<Self> {
        if config.model_path.as_os_str().is_empty() {
            return Err(EmbeddingError::Config(
                "Model path cannot be empty".to_string(),
            ));
        }

        // Initialize backend
        let backend = LlamaBackend::init().map_err(|e| {
            EmbeddingError::Internal(format!("Failed to initialize llama.cpp backend: {}", e))
        })?;

        // Configure model parameters with GPU offloading
        let model_params =
            LlamaModelParams::default().with_n_gpu_layers(config.n_gpu_layers);

        // Load model
        let model =
            LlamaModel::load_from_file(&backend, &config.model_path, &model_params).map_err(|e| {
                EmbeddingError::Internal(format!(
                    "Failed to load model from {}: {}",
                    config.model_path.display(),
                    e
                ))
            })?;

        // Use provided dimension or default
        let dimension = config.dimension.unwrap_or(768);

        // Extract model name from path
        let model_name = config
            .model_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Self {
            backend: Arc::new(backend),
            model: Arc::new(model),
            config,
            dimension,
            model_name,
        })
    }

    /// Create a context with the configured parameters
    fn create_context(&self) -> Result<llama_cpp_2::context::LlamaContext<'_>> {
        let pooling = match self.config.pooling_type.as_str() {
            "cls" => LlamaPoolingType::Cls,
            "mean" => LlamaPoolingType::Mean,
            "last" => LlamaPoolingType::Last,
            "rank" => LlamaPoolingType::Rank,
            "none" => LlamaPoolingType::None,
            _ => LlamaPoolingType::Mean,
        };

        let n_ctx = NonZeroU32::new(self.config.n_ctx)
            .ok_or_else(|| EmbeddingError::Config("n_ctx must be > 0".to_string()))?;

        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(Some(n_ctx))
            .with_n_batch(self.config.n_batch)
            .with_n_ubatch(self.config.n_ubatch)
            .with_n_threads(self.config.n_threads as i32)
            .with_n_threads_batch(self.config.n_threads_batch as i32)
            .with_embeddings(true)
            .with_pooling_type(pooling)
            .with_offload_kqv(self.config.offload_kqv);

        self.model
            .new_context(&self.backend, ctx_params)
            .map_err(|e| EmbeddingError::Internal(format!("Failed to create context: {}", e)))
    }

    /// Create embeddings for texts using llama.cpp
    ///
    /// This method creates a fresh context for each call, enabling thread-safe
    /// concurrent processing without lock contention.
    ///
    /// # Performance
    ///
    /// Creating a new context has ~10-50ms overhead. For high-throughput scenarios,
    /// consider increasing batch size to amortize this cost.
    pub fn embed_sync(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Create a fresh context for this request
        let mut ctx = self.create_context()?;

        // Tokenize all texts first
        let mut tokenized_texts: Vec<Vec<llama_cpp_2::token::LlamaToken>> =
            Vec::with_capacity(texts.len());
        for (i, text) in texts.iter().enumerate() {
            let tokens = self.model.str_to_token(text, AddBos::Always).map_err(|e| {
                EmbeddingError::Internal(format!("Failed to tokenize text {}: {}", i, e))
            })?;
            tokenized_texts.push(tokens);
        }

        // Calculate total tokens needed
        let total_tokens: usize = tokenized_texts.iter().map(|t| t.len()).sum();

        // Check if we can fit all in one batch
        let n_ctx = ctx.n_ctx() as usize;

        if total_tokens > n_ctx {
            return Err(EmbeddingError::Internal(format!(
                "Total tokens ({}) exceed context window ({})",
                total_tokens, n_ctx
            )));
        }

        // Process in batches if needed
        let mut embeddings = Vec::with_capacity(texts.len());
        let n_batch = ctx.n_batch() as usize;
        let mut batch = LlamaBatch::new(n_batch, texts.len() as i32);

        for (seq_id, tokens) in tokenized_texts.iter().enumerate() {
            batch.add_sequence(tokens, seq_id as i32, false).map_err(|e| {
                EmbeddingError::Internal(format!(
                    "Failed to add sequence {} to batch: {}",
                    seq_id, e
                ))
            })?;
        }

        // Single decode for all sequences
        ctx.decode(&mut batch)
            .map_err(|e| EmbeddingError::Internal(format!("Failed to decode batch: {}", e)))?;

        // Extract embeddings for all sequences
        for seq_id in 0..texts.len() {
            let embedding = ctx.embeddings_seq_ith(seq_id as i32).map_err(|e| {
                EmbeddingError::Internal(format!(
                    "Failed to get embeddings for text {}: {}",
                    seq_id, e
                ))
            })?;

            let normalized = Self::normalize(embedding);
            embeddings.push(normalized);
        }

        // Note: No need to clear KV cache for embedding models
        // Context is dropped automatically when it goes out of scope

        Ok(embeddings)
    }

    /// Normalize embedding vector using L2 normalization
    fn normalize(embedding: &[f32]) -> Vec<f32> {
        let magnitude: f32 = embedding.iter().map(|&x| x * x).sum::<f32>().sqrt();
        if magnitude == 0.0 {
            embedding.to_vec()
        } else {
            embedding.iter().map(|&x| x / magnitude).collect()
        }
    }
}

#[async_trait::async_trait]
impl EmbeddingProvider for LlamaCppProvider {
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        // Clone Arc pointers to move into the blocking task
        let this = Self {
            backend: Arc::clone(&self.backend),
            model: Arc::clone(&self.model),
            config: self.config.clone(),
            dimension: self.dimension,
            model_name: self.model_name.clone(),
        };

        let texts_owned: Vec<String> = texts.iter().map(|s| (*s).to_string()).collect();

        tokio::task::spawn_blocking(move || {
            let text_refs: Vec<&str> = texts_owned.iter().map(|s| s.as_str()).collect();
            this.embed_sync(&text_refs)
        })
        .await
        .map_err(|e| EmbeddingError::Internal(format!("Task panicked: {}", e)))?
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::LlamaCpp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        let embedding = vec![3.0, 4.0];
        let normalized = LlamaCppProvider::normalize(&embedding);

        // L2 norm of [3, 4] is 5, so normalized should be [0.6, 0.8]
        assert!((normalized[0] - 0.6).abs() < 1e-6);
        assert!((normalized[1] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_zero() {
        let embedding = vec![0.0, 0.0];
        let normalized = LlamaCppProvider::normalize(&embedding);

        // Zero vector should remain unchanged
        assert_eq!(normalized, vec![0.0, 0.0]);
    }

    #[test]
    fn test_config_builder() {
        let config = LlamaCppConfig::new("models/test.gguf")
            .with_pooling_type("cls")
            .with_dimension(512)
            .with_n_ctx(2048)
            .with_n_gpu_layers(33);

        assert_eq!(config.model_path, PathBuf::from("models/test.gguf"));
        assert_eq!(config.pooling_type, "cls");
        assert_eq!(config.dimension, Some(512));
        assert_eq!(config.n_ctx, 2048);
        assert_eq!(config.n_gpu_layers, 33);
    }

    #[test]
    fn test_config_default() {
        let config = LlamaCppConfig::default();

        assert_eq!(config.n_ctx, 4096);
        assert_eq!(config.n_threads, 8);
        assert_eq!(config.n_threads_batch, 16);
        assert_eq!(config.n_batch, 512);
        assert_eq!(config.n_ubatch, 256);
        assert!(config.offload_kqv);
        assert_eq!(config.n_gpu_layers, 1000);
    }
}

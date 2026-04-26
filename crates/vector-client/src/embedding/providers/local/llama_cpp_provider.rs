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
use std::sync::Arc;

use llama_cpp_2::context::params::{LlamaContextParams, LlamaPoolingType};
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};

use crate::embedding::error::{EmbeddingError, Result};
use crate::embedding::provider::{EmbeddingProvider, ProviderType};

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
    /// Context window size for embedding generation
    n_ctx: NonZeroU32,
    /// Pooling type: "cls", "mean", "last", "rank", or "none"
    pooling_type: String,
    /// Embedding dimension
    dimension: usize,
    /// Number of threads for generation
    n_threads: u32,
    /// Number of threads for batch processing
    n_threads_batch: u32,
    /// Physical batch size
    n_batch: u32,
    /// Micro-batch size
    n_ubatch: u32,
    /// Whether to offload K, Q, V to GPU
    offload_kqv: bool,
    /// Number of layers to offload to GPU
    n_gpu_layers: u32,
}

impl std::fmt::Debug for LlamaCppProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LlamaCppProvider")
            .field("n_ctx", &self.n_ctx)
            .field("pooling_type", &self.pooling_type)
            .field("dimension", &self.dimension)
            .field("n_threads", &self.n_threads)
            .field("n_gpu_layers", &self.n_gpu_layers)
            .finish()
    }
}

impl LlamaCppProvider {
    /// Create a new llama.cpp provider with GPU support
    ///
    /// # Arguments
    ///
    /// * `model_path` - Path to the GGUF model file
    /// * `pooling_type` - Pooling strategy: "cls", "mean", "last", "rank", or "none"
    /// * `dimension` - Embedding dimension (will be inferred from model if None)
    /// * `n_ctx` - Context window size (default: 4096)
    /// * `n_threads` - Number of threads for generation (default: 8)
    /// * `n_threads_batch` - Number of threads for prompt processing (default: 16)
    /// * `n_batch` - Physical batch size (default: 512)
    /// * `n_ubatch` - Micro-batch size (default: 256)
    /// * `offload_kqv` - Offload K, Q, V to GPU (default: true)
    /// * `n_gpu_layers` - Number of layers to offload to GPU (default: 1000 for full offload)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vector_client::embedding::providers::local::llama_cpp_provider::LlamaCppProvider;
    ///
    /// // Create provider with full GPU offloading (Vulkan)
    /// let provider = LlamaCppProvider::new(
    ///     "models/nomic-embed-text.gguf".to_string(),
    ///     "mean".to_string(),
    ///     Some(768),
    ///     None,  // n_ctx
    ///     None,  // n_threads
    ///     None,  // n_threads_batch
    ///     None,  // n_batch
    ///     None,  // n_ubatch
    ///     None,  // offload_kqv
    ///     Some(1000),  // n_gpu_layers - full offload to GPU
    /// ).expect("Failed to create provider");
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        model_path: String,
        pooling_type: String,
        dimension: Option<usize>,
        n_ctx: Option<u32>,
        n_threads: Option<u32>,
        n_threads_batch: Option<u32>,
        n_batch: Option<u32>,
        n_ubatch: Option<u32>,
        offload_kqv: Option<bool>,
        n_gpu_layers: Option<u32>,
    ) -> Result<Self> {
        // Initialize backend
        let backend = LlamaBackend::init().map_err(|e| {
            EmbeddingError::Internal(format!("Failed to initialize llama.cpp backend: {}", e))
        })?;

        // Configure model parameters with GPU offloading
        let model_params =
            LlamaModelParams::default().with_n_gpu_layers(n_gpu_layers.unwrap_or(1000));

        // Load model
        let model =
            LlamaModel::load_from_file(&backend, &model_path, &model_params).map_err(|e| {
                EmbeddingError::Internal(format!("Failed to load model from {}: {}", model_path, e))
            })?;

        // Use provided dimension or default
        let dim = dimension.unwrap_or(768);

        Ok(Self {
            backend: Arc::new(backend),
            model: Arc::new(model),
            n_ctx: NonZeroU32::new(n_ctx.unwrap_or(4096)).expect("n_ctx must be > 0"),
            pooling_type,
            dimension: dim,
            n_threads: n_threads.unwrap_or(8),
            n_threads_batch: n_threads_batch.unwrap_or(16),
            n_batch: n_batch.unwrap_or(512),
            n_ubatch: n_ubatch.unwrap_or(256),
            offload_kqv: offload_kqv.unwrap_or(true),
            n_gpu_layers: n_gpu_layers.unwrap_or(1000),
        })
    }

    /// Create a context with the configured parameters
    fn create_context(&self) -> Result<llama_cpp_2::context::LlamaContext<'_>> {
        let pooling = match self.pooling_type.as_str() {
            "cls" => LlamaPoolingType::Cls,
            "mean" => LlamaPoolingType::Mean,
            "last" => LlamaPoolingType::Last,
            "rank" => LlamaPoolingType::Rank,
            "none" => LlamaPoolingType::None,
            _ => LlamaPoolingType::Mean,
        };

        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(Some(self.n_ctx))
            .with_n_batch(self.n_batch)
            .with_n_ubatch(self.n_ubatch)
            .with_n_threads(self.n_threads as i32)
            .with_n_threads_batch(self.n_threads_batch as i32)
            .with_embeddings(true)
            .with_pooling_type(pooling)
            .with_offload_kqv(self.offload_kqv);

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

    /// Get the context window size
    #[allow(dead_code)]
    pub fn context_window(&self) -> u32 {
        self.n_ctx.get()
    }

    /// Get the pooling type used for embeddings
    #[allow(dead_code)]
    pub fn pooling_type(&self) -> &str {
        &self.pooling_type
    }

    /// Get the number of threads used for generation
    #[allow(dead_code)]
    pub fn n_threads(&self) -> u32 {
        self.n_threads
    }

    /// Get the number of threads used for batch processing
    #[allow(dead_code)]
    pub fn n_threads_batch(&self) -> u32 {
        self.n_threads_batch
    }

    /// Check if GPU acceleration is available
    #[allow(dead_code)]
    pub fn is_gpu_accelerated(&self) -> bool {
        self.n_gpu_layers > 0
    }

    /// Get the number of GPU layers
    #[allow(dead_code)]
    pub fn n_gpu_layers(&self) -> u32 {
        self.n_gpu_layers
    }
}

#[async_trait::async_trait]
impl EmbeddingProvider for LlamaCppProvider {
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        // Clone Arc pointers to move into the blocking task
        let this = Self {
            backend: Arc::clone(&self.backend),
            model: Arc::clone(&self.model),
            n_ctx: self.n_ctx,
            pooling_type: self.pooling_type.clone(),
            dimension: self.dimension,
            n_threads: self.n_threads,
            n_threads_batch: self.n_threads_batch,
            n_batch: self.n_batch,
            n_ubatch: self.n_ubatch,
            offload_kqv: self.offload_kqv,
            n_gpu_layers: self.n_gpu_layers,
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
        "llama.cpp embedding model"
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
}

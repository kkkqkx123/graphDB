//! llama.cpp local library provider for embeddings with Vulkan support
//!
//! This module provides a provider that uses llama.cpp directly via Rust bindings,
//! avoiding the HTTP overhead of the llama.cpp server.
//!
//! This module is only available when the `llama_cpp` feature is enabled.

use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::Mutex;

use llama_cpp_2::context::params::{LlamaContextParams, LlamaPoolingType};
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};

use super::error::{EmbeddingError, Result};
use super::provider::{EmbeddingProvider, ProviderType};

/// llama.cpp local library provider with Vulkan GPU acceleration support
///
/// This provider uses llama.cpp directly via Rust bindings for embedding generation.
/// When compiled with Vulkan support (CMAKE_ARGS="-DGGML_VULKAN=on"), it will
/// automatically utilize GPU acceleration for improved performance.
pub struct LlamaCppProvider {
    backend: Arc<LlamaBackend>,
    model: Arc<LlamaModel>,
    ctx_params: LlamaContextParams,
    ctx: Arc<Mutex<llama_cpp_2::context::LlamaContext>>,
    model_path: String,
    pooling_type: String,
    dimension: usize,
}

impl std::fmt::Debug for LlamaCppProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LlamaCppProvider")
            .field("model_path", &self.model_path)
            .field("pooling_type", &self.pooling_type)
            .field("dimension", &self.dimension)
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
    /// use vector_client::embedding::llama_cpp_provider::LlamaCppProvider;
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
        // This is where Vulkan acceleration takes effect if compiled with CMAKE_ARGS="-DGGML_VULKAN=on"
        let model_params =
            LlamaModelParams::default().with_n_gpu_layers(n_gpu_layers.unwrap_or(1000)); // Default: full offload to GPU

        // Load model
        let model =
            LlamaModel::load_from_file(&backend, &model_path, &model_params).map_err(|e| {
                EmbeddingError::Internal(format!("Failed to load model from {}: {}", model_path, e))
            })?;

        // Parse pooling type
        let pooling = match pooling_type.as_str() {
            "cls" => LlamaPoolingType::Cls,
            "mean" => LlamaPoolingType::Mean,
            "last" => LlamaPoolingType::Last,
            "rank" => LlamaPoolingType::Rank,
            "none" => LlamaPoolingType::None,
            _ => LlamaPoolingType::Mean,
        };

        // Configure context parameters
        let n_ctx_value = n_ctx.unwrap_or(4096);
        let n_ctx_nonzero = NonZeroU32::new(n_ctx_value).expect("n_ctx must be greater than 0");

        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(Some(n_ctx_nonzero))
            .with_n_batch(n_batch.unwrap_or(512))
            .with_n_ubatch(n_ubatch.unwrap_or(256))
            .with_n_threads(n_threads.unwrap_or(8))
            .with_n_threads_batch(n_threads_batch.unwrap_or(16))
            .with_embeddings(true)
            .with_pooling_type(pooling)
            .with_offload_kqv(offload_kqv.unwrap_or(true));

        // Create context
        let ctx = model
            .new_context(&backend, ctx_params.clone())
            .map_err(|e| EmbeddingError::Internal(format!("Failed to create context: {}", e)))?;

        // Get dimension from model if not provided
        let dim = dimension.unwrap_or_else(|| {
            // Try to get embedding dimension from model metadata
            // For now, use a default value
            768
        });

        Ok(Self {
            backend: Arc::new(backend),
            model: Arc::new(model),
            ctx_params,
            ctx: Arc::new(Mutex::new(ctx)),
            model_path,
            pooling_type,
            dimension: dim,
        })
    }

    /// Create embeddings for texts using llama.cpp
    pub async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let mut embeddings = Vec::with_capacity(texts.len());

        for (i, text) in texts.iter().enumerate() {
            // Tokenize text
            let tokens = self.model.str_to_token(text, AddBos::Always).map_err(|e| {
                EmbeddingError::Internal(format!("Failed to tokenize text {}: {}", i, e))
            })?;

            // Create batch
            let mut ctx = self
                .ctx
                .lock()
                .map_err(|e| EmbeddingError::Internal(format!("Failed to lock context: {}", e)))?;

            let mut batch = LlamaBatch::new(ctx.n_ctx() as usize, 1);
            batch.add_sequence(&tokens, i, false).map_err(|e| {
                EmbeddingError::Internal(format!("Failed to add sequence to batch: {}", e))
            })?;

            // Decode
            ctx.decode(&mut batch)
                .map_err(|e| EmbeddingError::Internal(format!("Failed to decode batch: {}", e)))?;

            // Get embeddings
            let embedding = ctx.embeddings_seq_ith(i).map_err(|e| {
                EmbeddingError::Internal(format!("Failed to get embeddings for text {}: {}", i, e))
            })?;

            // Normalize embedding (L2 normalization)
            let normalized = Self::normalize(&embedding);
            embeddings.push(normalized);

            // Clear KV cache for next sequence
            ctx.clear_kv_cache();
        }

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

    /// Detect available GPU devices
    ///
    /// This function lists all available backend devices, including GPU devices
    /// when Vulkan or other GPU backends are enabled.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vector_client::embedding::llama_cpp_provider::LlamaCppProvider;
    ///
    /// let devices = LlamaCppProvider::detect_gpu_devices();
    /// for device in devices {
    ///     println!("GPU Device: {}", device);
    /// }
    /// ```
    pub fn detect_gpu_devices() -> Vec<String> {
        use llama_cpp_2::list_llama_ggml_backend_devices;

        let devices = list_llama_ggml_backend_devices();
        devices
            .iter()
            .map(|dev| {
                format!(
                    "Device {}: {} ({} backend, {} MiB total)",
                    dev.name,
                    dev.backend,
                    dev.device_type,
                    dev.memory_total / 1024 / 1024
                )
            })
            .collect()
    }
}

#[async_trait::async_trait]
impl EmbeddingProvider for LlamaCppProvider {
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        self.embed(texts).await
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        &self.model_path
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

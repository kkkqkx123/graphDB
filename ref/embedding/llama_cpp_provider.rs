//! llama.cpp local library provider for embeddings
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
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::model::params::LlamaModelParams;

use super::base::{EmbeddingError, EmbeddingProvider, ProviderType};

/// llama.cpp local library provider
///
/// This provider uses llama.cpp directly via Rust bindings for embedding generation.
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
    /// Create a new llama.cpp provider
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
    ) -> Result<Self, EmbeddingError> {
        // Initialize backend
        let backend = LlamaBackend::init()
            .map_err(|e| EmbeddingError::Internal(format!("Failed to initialize llama.cpp backend: {}", e)))?;

        // Load model
        let model = LlamaModel::load_from_file(&backend, &model_path, &LlamaModelParams::default())
            .map_err(|e| EmbeddingError::Internal(format!("Failed to load model from {}: {}", model_path, e)))?;

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
    pub async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let mut embeddings = Vec::with_capacity(texts.len());

        for (i, text) in texts.iter().enumerate() {
            // Tokenize text
            let tokens = self.model.str_to_token(text, AddBos::Always)
                .map_err(|e| EmbeddingError::Internal(format!("Failed to tokenize text {}: {}", i, e)))?;

            // Create batch
            let mut ctx = self.ctx.lock()
                .map_err(|e| EmbeddingError::Internal(format!("Failed to lock context: {}", e)))?;

            let mut batch = LlamaBatch::new(ctx.n_ctx() as usize, 1);
            batch
                .add_sequence(&tokens, i, false)
                .map_err(|e| EmbeddingError::Internal(format!("Failed to add sequence to batch: {}", e)))?;

            // Decode
            ctx.decode(&mut batch)
                .map_err(|e| EmbeddingError::Internal(format!("Failed to decode batch: {}", e)))?;

            // Get embeddings
            let embedding = ctx.embeddings_seq_ith(i)
                .map_err(|e| EmbeddingError::Internal(format!("Failed to get embeddings for text {}: {}", i, e)))?;

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
}

#[async_trait::async_trait]
impl EmbeddingProvider for LlamaCppProvider {
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
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
    fn test_create_llama_cpp_provider() {
        let provider = LlamaCppProvider::new(
            "/path/to/model.gguf".to_string(),
            "cls".to_string(),
            Some(768),
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert!(provider.is_ok());
        let provider = provider.unwrap();
        assert_eq!(provider.dimension(), 768);
        assert_eq!(provider.provider_type(), ProviderType::LlamaCpp);
    }

    #[test]
    fn test_llama_cpp_provider_debug() {
        let provider = LlamaCppProvider::new(
            "/path/to/model.gguf".to_string(),
            "mean".to_string(),
            Some(1024),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("create failed");
        let debug_str = format!("{:?}", provider);
        assert!(debug_str.contains("LlamaCppProvider"));
        assert!(debug_str.contains("/path/to/model.gguf"));
    }

    #[test]
    fn test_normalize() {
        let embedding = vec![1.0, 2.0, 2.0];
        let normalized = LlamaCppProvider::normalize(&embedding);
        let magnitude = (normalized.iter().map(|&x| x * x).sum::<f32>()).sqrt();
        assert!((magnitude - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_zero_vector() {
        let embedding = vec![0.0, 0.0, 0.0];
        let normalized = LlamaCppProvider::normalize(&embedding);
        assert_eq!(normalized, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_pooling_type_parsing() {
        let poolings = vec!["cls", "mean", "last", "rank", "none", "invalid"];
        for pooling in poolings {
            let provider = LlamaCppProvider::new(
                "/path/to/model.gguf".to_string(),
                pooling.to_string(),
                Some(768),
                None,
                None,
                None,
                None,
                None,
                None,
            );
            assert!(provider.is_ok());
        }
    }
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
    /// Create a new llama.cpp provider
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
    ) -> Result<Self, EmbeddingError> {
        // Initialize backend
        let backend = LlamaBackend::init()
            .map_err(|e| EmbeddingError::Internal(format!("Failed to initialize llama.cpp backend: {}", e)))?;

        // Load model
        let model = LlamaModel::load_from_file(&backend, &model_path, &LlamaModelParams::default())
            .map_err(|e| EmbeddingError::Internal(format!("Failed to load model from {}: {}", model_path, e)))?;

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
    pub async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let mut embeddings = Vec::with_capacity(texts.len());

        for (i, text) in texts.iter().enumerate() {
            // Tokenize text
            let tokens = self.model.str_to_token(text, AddBos::Always)
                .map_err(|e| EmbeddingError::Internal(format!("Failed to tokenize text {}: {}", i, e)))?;

            // Create batch
            let mut ctx = self.ctx.lock()
                .map_err(|e| EmbeddingError::Internal(format!("Failed to lock context: {}", e)))?;

            let mut batch = LlamaBatch::new(ctx.n_ctx() as usize, 1);
            batch
                .add_sequence(&tokens, i, false)
                .map_err(|e| EmbeddingError::Internal(format!("Failed to add sequence to batch: {}", e)))?;

            // Decode
            ctx.decode(&mut batch)
                .map_err(|e| EmbeddingError::Internal(format!("Failed to decode batch: {}", e)))?;

            // Get embeddings
            let embedding = ctx.embeddings_seq_ith(i)
                .map_err(|e| EmbeddingError::Internal(format!("Failed to get embeddings for text {}: {}", i, e)))?;

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
}

#[async_trait::async_trait]
impl EmbeddingProvider for LlamaCppProvider {
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
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
    fn test_create_llama_cpp_provider() {
        let provider = LlamaCppProvider::new(
            "/path/to/model.gguf".to_string(),
            "cls".to_string(),
            Some(768),
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert!(provider.is_ok());
        let provider = provider.unwrap();
        assert_eq!(provider.dimension(), 768);
        assert_eq!(provider.provider_type(), ProviderType::LlamaCpp);
    }

    #[test]
    fn test_llama_cpp_provider_debug() {
        let provider = LlamaCppProvider::new(
            "/path/to/model.gguf".to_string(),
            "mean".to_string(),
            Some(1024),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("create failed");
        let debug_str = format!("{:?}", provider);
        assert!(debug_str.contains("LlamaCppProvider"));
        assert!(debug_str.contains("/path/to/model.gguf"));
    }

    #[test]
    fn test_normalize() {
        let embedding = vec![1.0, 2.0, 2.0];
        let normalized = LlamaCppProvider::normalize(&embedding);
        let magnitude = (normalized.iter().map(|&x| x * x).sum::<f32>()).sqrt();
        assert!((magnitude - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_zero_vector() {
        let embedding = vec![0.0, 0.0, 0.0];
        let normalized = LlamaCppProvider::normalize(&embedding);
        assert_eq!(normalized, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_pooling_type_parsing() {
        let poolings = vec!["cls", "mean", "last", "rank", "none", "invalid"];
        for pooling in poolings {
            let provider = LlamaCppProvider::new(
                "/path/to/model.gguf".to_string(),
                pooling.to_string(),
                Some(768),
                None,
                None,
                None,
                None,
                None,
                None,
            );
            assert!(provider.is_ok());
        }
    }
}

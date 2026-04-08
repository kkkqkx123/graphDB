//! Embedding Service
//!
//! Provides text-to-vector embedding capabilities for vector search.

use std::sync::Arc;

use crate::core::error::VectorResult;

/// Embedding service trait for converting text to vectors
#[async_trait::async_trait]
pub trait EmbeddingService: Send + Sync {
    /// Convert text to vector
    async fn embed(&self, text: &str) -> VectorResult<Vec<f32>>;

    /// Convert multiple texts to vectors in batch
    async fn embed_batch(&self, texts: &[&str]) -> VectorResult<Vec<Vec<f32>>>;

    /// Get the dimension of vectors produced by this service
    fn dimension(&self) -> usize;
}

/// Mock embedding service for testing
pub struct MockEmbeddingService {
    dimension: usize,
}

impl MockEmbeddingService {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

#[async_trait::async_trait]
impl EmbeddingService for MockEmbeddingService {
    async fn embed(&self, text: &str) -> VectorResult<Vec<f32>> {
        // Generate a deterministic mock vector based on text hash
        let hash = text.as_bytes().iter().map(|&b| b as u32).sum::<u32>();
        let mut vector = vec![0.0; self.dimension];
        
        for (i, val) in vector.iter_mut().enumerate().take(self.dimension) {
            *val = ((hash + i as u32) % 1000) as f32 / 1000.0;
        }
        
        Ok(vector)
    }

    async fn embed_batch(&self, texts: &[&str]) -> VectorResult<Vec<Vec<f32>>> {
        let mut vectors = Vec::with_capacity(texts.len());
        for text in texts {
            vectors.push(self.embed(text).await?);
        }
        Ok(vectors)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}

/// Qdrant embedding service configuration
#[derive(Debug, Clone)]
pub struct QdrantEmbeddingConfig {
    pub api_url: String,
    pub api_key: Option<String>,
    pub model_name: String,
}

impl Default for QdrantEmbeddingConfig {
    fn default() -> Self {
        Self {
            api_url: "http://localhost:6333".to_string(),
            api_key: None,
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
        }
    }
}

/// Qdrant embedding service implementation
pub struct QdrantEmbeddingService {
    config: QdrantEmbeddingConfig,
    dimension: usize,
}

impl QdrantEmbeddingService {
    pub async fn new(config: QdrantEmbeddingConfig) -> VectorResult<Self> {
        // Try to connect to Qdrant to verify it's available
        let qdrant_config = crate::vector::config::QdrantConfig {
            host: "localhost".to_string(),
            port: 6333,
            use_tls: false,
            api_key: config.api_key.clone(),
            connect_timeout_secs: 5,
            request_timeout_secs: 30,
            search_timeout_secs: 60,
            upsert_timeout_secs: 30,
        };

        match vector_client::QdrantEngine::new(qdrant_config.to_client_config()).await {
            Ok(_) => {
                log::info!("Connected to Qdrant for embedding service");
            }
            Err(e) => {
                log::warn!("Failed to connect to Qdrant: {}. Text embedding will be unavailable.", e);
            }
        }

        // Default dimension for common models
        let dimension = if config.model_name.contains("all-MiniLM-L6") {
            384
        } else if config.model_name.contains("all-MPNet-base") {
            768
        } else {
            768 // Default fallback
        };

        Ok(Self {
            config,
            dimension,
        })
    }

    pub fn is_available(&self) -> bool {
        // Check if the embedding service is properly configured
        !self.config.api_url.is_empty() && !self.config.model_name.is_empty()
    }
}

#[async_trait::async_trait]
impl EmbeddingService for QdrantEmbeddingService {
    async fn embed(&self, text: &str) -> VectorResult<Vec<f32>> {
        // Use Qdrant's embedding API if available
        // This is a placeholder - actual implementation depends on Qdrant's embedding API
        log::debug!("Embedding text: {}", text);
        
        // For now, return a mock vector based on text hash
        // In production, this would call Qdrant's embedding endpoint
        let hash = text.as_bytes().iter().map(|&b| b as u32).sum::<u32>();
        let mut vector = vec![0.0; self.dimension];
        
        for (i, val) in vector.iter_mut().enumerate().take(self.dimension) {
            *val = ((hash + i as u32) % 1000) as f32 / 1000.0;
        }
        
        Ok(vector)
    }

    async fn embed_batch(&self, texts: &[&str]) -> VectorResult<Vec<Vec<f32>>> {
        let mut vectors = Vec::with_capacity(texts.len());
        for text in texts {
            vectors.push(self.embed(text).await?);
        }
        Ok(vectors)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}

/// Embedding service wrapper that can be used in query execution
#[derive(Clone)]
pub struct EmbeddingServiceHandle {
    service: Arc<dyn EmbeddingService>,
}

impl std::fmt::Debug for EmbeddingServiceHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmbeddingServiceHandle").finish()
    }
}

impl EmbeddingServiceHandle {
    pub fn new(service: Arc<dyn EmbeddingService>) -> Self {
        Self { service }
    }

    pub async fn embed(&self, text: &str) -> VectorResult<Vec<f32>> {
        self.service.embed(text).await
    }

    pub async fn embed_batch(&self, texts: &[&str]) -> VectorResult<Vec<Vec<f32>>> {
        self.service.embed_batch(texts).await
    }

    pub fn dimension(&self) -> usize {
        self.service.dimension()
    }
}

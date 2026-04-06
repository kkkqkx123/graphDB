use std::sync::Arc;

use crate::config::{EngineType, VectorClientConfig};
use crate::engine::VectorEngine;
use crate::error::Result;
use crate::types::{CollectionConfig, HealthStatus};

#[cfg(feature = "qdrant")]
use crate::engine::QdrantEngine;

#[cfg(feature = "mock")]
use crate::engine::MockEngine;

use super::core::{CollectionApi, PointApi, SearchApi};

#[derive(Debug)]
pub struct VectorClient {
    engine: Arc<dyn VectorEngine>,
    config: VectorClientConfig,
}

impl VectorClient {
    #[cfg(feature = "qdrant")]
    pub async fn qdrant(config: VectorClientConfig) -> Result<Self> {
        let engine = QdrantEngine::new(config.clone()).await?;
        Ok(Self {
            engine: Arc::new(engine),
            config,
        })
    }

    #[cfg(feature = "mock")]
    pub fn mock() -> Self {
        let config = VectorClientConfig::mock();
        let engine = MockEngine::new();
        Self {
            engine: Arc::new(engine),
            config,
        }
    }

    #[cfg(feature = "mock")]
    pub fn mock_with_collections(collections: std::collections::HashMap<String, CollectionConfig>) -> Self {
        let config = VectorClientConfig::mock();
        let engine = MockEngine::with_collections(collections);
        Self {
            engine: Arc::new(engine),
            config,
        }
    }

    pub async fn new(config: VectorClientConfig) -> Result<Self> {
        match config.engine {
            #[cfg(feature = "qdrant")]
            EngineType::Qdrant => Self::qdrant(config).await,

            #[cfg(feature = "mock")]
            EngineType::Mock => Ok(Self::mock()),

            #[cfg(not(feature = "qdrant"))]
            EngineType::Qdrant => Err(VectorClientError::EngineNotAvailable("qdrant".to_string())),

            #[cfg(not(feature = "mock"))]
            EngineType::Mock => Err(VectorClientError::EngineNotAvailable("mock".to_string())),
        }
    }

    pub fn engine(&self) -> &dyn VectorEngine {
        self.engine.as_ref()
    }

    pub fn config(&self) -> &VectorClientConfig {
        &self.config
    }

    pub async fn health_check(&self) -> Result<HealthStatus> {
        self.engine.health_check().await
    }

    pub fn collection(&self) -> CollectionApi<'_, dyn VectorEngine> {
        CollectionApi::new(self.engine.as_ref())
    }

    pub fn points(&self, collection: impl Into<String>) -> PointApi<'_, dyn VectorEngine> {
        PointApi::new(self.engine.as_ref(), collection)
    }

    pub fn search(&self, collection: impl Into<String>) -> SearchApi<'_, dyn VectorEngine> {
        SearchApi::new(self.engine.as_ref(), collection)
    }
}

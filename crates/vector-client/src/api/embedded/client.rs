use std::sync::Arc;

use crate::config::{EngineType, VectorClientConfig};
use crate::engine::VectorEngine;
use crate::error::Result;
use crate::types::HealthStatus;

#[cfg(feature = "qdrant")]
use crate::engine::QdrantEngine;

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

    pub async fn new(config: VectorClientConfig) -> Result<Self> {
        match config.engine {
            #[cfg(feature = "qdrant")]
            EngineType::Qdrant => Self::qdrant(config).await,

            #[cfg(not(feature = "qdrant"))]
            EngineType::Qdrant => Err(VectorClientError::EngineNotAvailable("qdrant".to_string())),
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

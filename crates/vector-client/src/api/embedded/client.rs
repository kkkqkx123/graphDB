use std::sync::Arc;

#[cfg(any(feature = "qdrant-http", feature = "qdrant-grpc"))]
use crate::config::EngineType;
use crate::config::VectorClientConfig;
use crate::engine::VectorEngine;
use crate::error::Result;
use crate::types::HealthStatus;

use super::core::{CollectionApi, PointApi, SearchApi};

#[derive(Debug)]
pub struct VectorClient {
    engine: Arc<dyn VectorEngine>,
    config: VectorClientConfig,
}

impl VectorClient {
    pub async fn new(config: VectorClientConfig) -> Result<Self> {
        #[cfg(any(feature = "qdrant-http", feature = "qdrant-grpc"))]
        {
            let engine: Arc<dyn VectorEngine> = match config.engine {
                EngineType::Qdrant => {
                    #[cfg(feature = "qdrant-grpc")]
                    {
                        let e = crate::engine::QdrantGrpcEngine::new(config.clone()).await?;
                        Arc::new(e)
                    }
                    #[cfg(all(not(feature = "qdrant-grpc"), feature = "qdrant-http"))]
                    {
                        let e = crate::engine::QdrantEngine::new(config.clone()).await?;
                        Arc::new(e)
                    }
                }
            };

            Ok(Self { engine, config })
        }

        #[cfg(not(any(feature = "qdrant-http", feature = "qdrant-grpc")))]
        {
            let _ = config;
            Err(crate::error::VectorClientError::EngineNotAvailable(
                "no qdrant engine feature enabled".to_string(),
            ))
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

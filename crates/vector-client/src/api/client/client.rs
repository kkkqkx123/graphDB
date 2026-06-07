use std::sync::Arc;

use async_trait::async_trait;

#[cfg(any(feature = "qdrant-http", feature = "qdrant-grpc"))]
use crate::config::EngineType;
use crate::config::VectorClientConfig;
use crate::embedding::{EmbeddingConfig, EmbeddingService};
use crate::engine::VectorEngine;
use crate::error::{Result, VectorClientError};
use crate::types::*;

use super::core::{CollectionApi, PointApi, SearchApi};

#[derive(Debug)]
struct DisabledEngine;

#[async_trait]
impl VectorEngine for DisabledEngine {
    fn name(&self) -> &str {
        "disabled"
    }
    fn version(&self) -> &str {
        "0.0"
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        Ok(HealthStatus::unhealthy(
            "disabled",
            "0.0",
            "Engine disabled",
        ))
    }

    async fn create_collection(&self, _name: &str, _config: CollectionConfig) -> Result<()> {
        self.err().await
    }
    async fn delete_collection(&self, _name: &str) -> Result<()> {
        self.err().await
    }
    async fn collection_exists(&self, _name: &str) -> Result<bool> {
        self.err().await
    }
    async fn collection_info(&self, _name: &str) -> Result<CollectionInfo> {
        self.err().await
    }
    async fn upsert(&self, _collection: &str, _point: VectorPoint) -> Result<UpsertResult> {
        self.err().await
    }
    async fn upsert_batch(
        &self,
        _collection: &str,
        _points: Vec<VectorPoint>,
    ) -> Result<UpsertResult> {
        self.err().await
    }
    async fn delete(&self, _collection: &str, _point_id: &str) -> Result<DeleteResult> {
        self.err().await
    }
    async fn delete_batch(
        &self,
        _collection: &str,
        _point_ids: Vec<&str>,
    ) -> Result<DeleteResult> {
        self.err().await
    }
    async fn delete_by_filter(
        &self,
        _collection: &str,
        _filter: VectorFilter,
    ) -> Result<DeleteResult> {
        self.err().await
    }
    async fn search(
        &self,
        _collection: &str,
        _query: SearchQuery,
    ) -> Result<Vec<SearchResult>> {
        self.err().await
    }
    async fn search_batch(
        &self,
        _collection: &str,
        _queries: Vec<SearchQuery>,
    ) -> Result<Vec<Vec<SearchResult>>> {
        self.err().await
    }
    async fn get(&self, _collection: &str, _point_id: &str) -> Result<Option<VectorPoint>> {
        self.err().await
    }
    async fn get_batch(
        &self,
        _collection: &str,
        _point_ids: Vec<&str>,
    ) -> Result<Vec<Option<VectorPoint>>> {
        self.err().await
    }
    async fn count(&self, _collection: &str) -> Result<u64> {
        self.err().await
    }
    async fn set_payload(
        &self,
        _collection: &str,
        _point_ids: Vec<&str>,
        _payload: Payload,
    ) -> Result<()> {
        self.err().await
    }
    async fn delete_payload(
        &self,
        _collection: &str,
        _point_ids: Vec<&str>,
        _keys: Vec<&str>,
    ) -> Result<()> {
        self.err().await
    }
    async fn scroll(
        &self,
        _collection: &str,
        _limit: usize,
        _offset: Option<&str>,
        _with_payload: Option<bool>,
        _with_vector: Option<bool>,
    ) -> Result<(Vec<VectorPoint>, Option<String>)> {
        self.err().await
    }
    async fn create_payload_index(
        &self,
        _collection: &str,
        _field: &str,
        _schema: PayloadSchemaType,
    ) -> Result<()> {
        self.err().await
    }
    async fn delete_payload_index(&self, _collection: &str, _field: &str) -> Result<()> {
        self.err().await
    }
    async fn list_payload_indexes(
        &self,
        _collection: &str,
    ) -> Result<Vec<(String, PayloadSchemaType)>> {
        self.err().await
    }
}

impl DisabledEngine {
    async fn err<T>(&self) -> Result<T> {
        Err(VectorClientError::EngineNotAvailable(
            "vector engine disabled".to_string(),
        ))
    }
}

#[derive(Debug)]
pub struct VectorClient {
    engine: Arc<dyn VectorEngine>,
    config: VectorClientConfig,
}

impl VectorClient {
    pub async fn new(config: VectorClientConfig) -> Result<Self> {
        if !config.enabled {
            return Ok(Self {
                engine: Arc::new(DisabledEngine),
                config,
            });
        }

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

    pub async fn search_with_text(
        &self,
        collection: impl Into<String>,
        text: &str,
        embedding_config: &EmbeddingConfig,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let embedding_service = EmbeddingService::from_config(embedding_config.clone())
            .map_err(|e| VectorClientError::InternalError(e.to_string()))?;
        let vector = embedding_service
            .embed(text)
            .await
            .map_err(|e| VectorClientError::InternalError(e.to_string()))?;

        let query = SearchQuery::new(vector, limit);
        self.search(collection).search(query).await
    }
}

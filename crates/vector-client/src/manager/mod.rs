mod index;

pub use index::IndexMetadata;

use std::sync::Arc;

use dashmap::DashMap;
use tracing::{debug, info, warn};

use crate::config::VectorClientConfig;
use crate::engine::VectorEngine;
use crate::error::{Result, VectorClientError};
use crate::types::{CollectionConfig, SearchQuery, SearchResult, VectorPoint};

use super::engine::QdrantEngine;

pub struct VectorManager {
    engine: Arc<dyn VectorEngine>,
    indexes: DashMap<String, IndexMetadata>,
}

impl std::fmt::Debug for VectorManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VectorManager")
            .field("engine", &self.engine.name())
            .field("index_count", &self.indexes.len())
            .finish()
    }
}

impl VectorManager {
    pub async fn new(config: VectorClientConfig) -> Result<Self> {
        let enabled = config.enabled;

        let engine: Arc<dyn VectorEngine> = if enabled {
            info!("Initializing Qdrant HTTP engine");
            let qdrant_engine = QdrantEngine::new(config)
                .await
                .map_err(|e| VectorClientError::ConnectionFailed(e.to_string()))?;
            Arc::new(qdrant_engine) as Arc<dyn VectorEngine>
        } else {
            info!("Vector search is disabled, using no-op engine");
            Arc::new(DisabledEngine) as Arc<dyn VectorEngine>
        };

        if enabled {
            match engine.health_check().await {
                Ok(health) => {
                    if health.is_healthy {
                        info!(
                            "Vector engine health check passed: {} {}",
                            health.engine_name, health.engine_version
                        );
                    } else {
                        warn!("Vector engine health check failed: {:?}", health.message);
                    }
                }
                Err(e) => {
                    warn!("Vector engine health check failed: {}", e);
                }
            }
        }

        Ok(Self {
            engine,
            indexes: DashMap::new(),
        })
    }

    pub fn engine(&self) -> &Arc<dyn VectorEngine> {
        &self.engine
    }

    pub async fn create_index(&self, name: &str, config: CollectionConfig) -> Result<()> {
        if self.indexes.contains_key(name) {
            return Err(VectorClientError::IndexAlreadyExists(name.to_string()));
        }

        debug!("Creating vector collection: {}", name);
        self.engine.create_collection(name, config.clone()).await?;

        let metadata = IndexMetadata::new(name.to_string(), config);
        self.indexes.insert(name.to_string(), metadata);

        info!("Vector index created: {}", name);
        Ok(())
    }

    pub async fn drop_index(&self, name: &str) -> Result<()> {
        if let Some((_, metadata)) = self.indexes.remove(name) {
            debug!("Dropping vector collection: {}", metadata.name);
            self.engine.delete_collection(name).await?;
            info!("Vector index dropped: {}", name);
        }
        Ok(())
    }

    pub fn index_exists(&self, name: &str) -> bool {
        self.indexes.contains_key(name)
    }

    pub fn get_index_metadata(&self, name: &str) -> Option<IndexMetadata> {
        self.indexes.get(name).map(|m| m.clone())
    }

    pub fn list_indexes(&self) -> Vec<IndexMetadata> {
        self.indexes.iter().map(|m| m.value().clone()).collect()
    }

    pub async fn upsert(&self, collection: &str, point: VectorPoint) -> Result<()> {
        self.engine.upsert(collection, point).await?;
        Ok(())
    }

    pub async fn upsert_batch(&self, collection: &str, points: Vec<VectorPoint>) -> Result<()> {
        self.engine.upsert_batch(collection, points).await?;
        Ok(())
    }

    pub async fn delete(&self, collection: &str, point_id: &str) -> Result<()> {
        self.engine.delete(collection, point_id).await?;
        Ok(())
    }

    pub async fn delete_batch(&self, collection: &str, point_ids: Vec<&str>) -> Result<()> {
        self.engine.delete_batch(collection, point_ids).await?;
        Ok(())
    }

    pub async fn search(
        &self,
        collection: &str,
        query: SearchQuery,
    ) -> Result<Vec<SearchResult>> {
        self.engine.search(collection, query).await
    }

    pub async fn get(
        &self,
        collection: &str,
        point_id: &str,
    ) -> Result<Option<VectorPoint>> {
        self.engine.get(collection, point_id).await
    }

    pub async fn count(&self, collection: &str) -> Result<u64> {
        self.engine.count(collection).await
    }
}

#[cfg(feature = "qdrant-http")]
mod disabled {
    use async_trait::async_trait;

    use super::VectorEngine;
    use crate::error::{Result, VectorClientError};
    use crate::types::*;

    pub struct DisabledEngine;

    impl std::fmt::Debug for DisabledEngine {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("DisabledEngine").finish()
        }
    }

    #[async_trait]
    impl VectorEngine for DisabledEngine {
        fn name(&self) -> &str {
            "disabled"
        }

        fn version(&self) -> &str {
            "0.0"
        }

        async fn health_check(&self) -> Result<HealthStatus> {
            Ok(HealthStatus::unhealthy("disabled", "0.0", "Engine disabled"))
        }

        async fn create_collection(&self, _name: &str, _config: CollectionConfig) -> Result<()> {
            Err(VectorClientError::EngineNotAvailable(
                "vector engine disabled".to_string(),
            ))
        }

        async fn delete_collection(&self, _name: &str) -> Result<()> {
            Err(VectorClientError::EngineNotAvailable(
                "vector engine disabled".to_string(),
            ))
        }

        async fn collection_exists(&self, _name: &str) -> Result<bool> {
            Err(VectorClientError::EngineNotAvailable(
                "vector engine disabled".to_string(),
            ))
        }

        async fn collection_info(&self, _name: &str) -> Result<CollectionInfo> {
            Err(VectorClientError::EngineNotAvailable(
                "vector engine disabled".to_string(),
            ))
        }

        async fn upsert(&self, _collection: &str, _point: VectorPoint) -> Result<UpsertResult> {
            Err(VectorClientError::EngineNotAvailable(
                "vector engine disabled".to_string(),
            ))
        }

        async fn upsert_batch(
            &self,
            _collection: &str,
            _points: Vec<VectorPoint>,
        ) -> Result<UpsertResult> {
            Err(VectorClientError::EngineNotAvailable(
                "vector engine disabled".to_string(),
            ))
        }

        async fn delete(&self, _collection: &str, _point_id: &str) -> Result<DeleteResult> {
            Err(VectorClientError::EngineNotAvailable(
                "vector engine disabled".to_string(),
            ))
        }

        async fn delete_batch(
            &self,
            _collection: &str,
            _point_ids: Vec<&str>,
        ) -> Result<DeleteResult> {
            Err(VectorClientError::EngineNotAvailable(
                "vector engine disabled".to_string(),
            ))
        }

        async fn delete_by_filter(
            &self,
            _collection: &str,
            _filter: VectorFilter,
        ) -> Result<DeleteResult> {
            Err(VectorClientError::EngineNotAvailable(
                "vector engine disabled".to_string(),
            ))
        }

        async fn search(
            &self,
            _collection: &str,
            _query: SearchQuery,
        ) -> Result<Vec<SearchResult>> {
            Err(VectorClientError::EngineNotAvailable(
                "vector engine disabled".to_string(),
            ))
        }

        async fn search_batch(
            &self,
            _collection: &str,
            _queries: Vec<SearchQuery>,
        ) -> Result<Vec<Vec<SearchResult>>> {
            Err(VectorClientError::EngineNotAvailable(
                "vector engine disabled".to_string(),
            ))
        }

        async fn get(
            &self,
            _collection: &str,
            _point_id: &str,
        ) -> Result<Option<VectorPoint>> {
            Err(VectorClientError::EngineNotAvailable(
                "vector engine disabled".to_string(),
            ))
        }

        async fn get_batch(
            &self,
            _collection: &str,
            _point_ids: Vec<&str>,
        ) -> Result<Vec<Option<VectorPoint>>> {
            Err(VectorClientError::EngineNotAvailable(
                "vector engine disabled".to_string(),
            ))
        }

        async fn count(&self, _collection: &str) -> Result<u64> {
            Err(VectorClientError::EngineNotAvailable(
                "vector engine disabled".to_string(),
            ))
        }

        async fn set_payload(
            &self,
            _collection: &str,
            _point_ids: Vec<&str>,
            _payload: Payload,
        ) -> Result<()> {
            Err(VectorClientError::EngineNotAvailable(
                "vector engine disabled".to_string(),
            ))
        }

        async fn delete_payload(
            &self,
            _collection: &str,
            _point_ids: Vec<&str>,
            _keys: Vec<&str>,
        ) -> Result<()> {
            Err(VectorClientError::EngineNotAvailable(
                "vector engine disabled".to_string(),
            ))
        }

        async fn scroll(
            &self,
            _collection: &str,
            _limit: usize,
            _offset: Option<&str>,
            _with_payload: Option<bool>,
            _with_vector: Option<bool>,
        ) -> Result<(Vec<VectorPoint>, Option<String>)> {
            Err(VectorClientError::EngineNotAvailable(
                "vector engine disabled".to_string(),
            ))
        }

        async fn create_payload_index(
            &self,
            _collection: &str,
            _field: &str,
            _schema: PayloadSchemaType,
        ) -> Result<()> {
            Err(VectorClientError::EngineNotAvailable(
                "vector engine disabled".to_string(),
            ))
        }

        async fn delete_payload_index(&self, _collection: &str, _field: &str) -> Result<()> {
            Err(VectorClientError::EngineNotAvailable(
                "vector engine disabled".to_string(),
            ))
        }

        async fn list_payload_indexes(
            &self,
            _collection: &str,
        ) -> Result<Vec<(String, PayloadSchemaType)>> {
            Err(VectorClientError::EngineNotAvailable(
                "vector engine disabled".to_string(),
            ))
        }
    }
}

#[cfg(feature = "qdrant-http")]
use disabled::DisabledEngine;

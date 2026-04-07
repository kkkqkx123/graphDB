//! Vector Index Manager
//!
//! Manages vector index lifecycle and operations.

use dashmap::DashMap;
use log::{debug, info, warn};
use std::sync::Arc;

use crate::core::error::{VectorError, VectorResult};
use crate::vector::config::{VectorConfig, VectorIndexConfig, VectorIndexMetadata};

use vector_client::types::{SearchQuery, SearchResult, VectorPoint};
use vector_client::QdrantEngine;
use vector_client::VectorEngine;

type IndexKey = (u64, String, String);

pub struct VectorIndexManager {
    engine: Arc<dyn VectorEngine>,
    metadata: DashMap<IndexKey, VectorIndexMetadata>,
    config: VectorConfig,
}

impl VectorIndexManager {
    pub async fn new(config: VectorConfig) -> VectorResult<Self> {
        if !config.enabled {
            info!("Vector search is disabled");
            let engine = Arc::new(vector_client::MockEngine::new()) as Arc<dyn VectorEngine>;
            return Ok(Self {
                engine,
                metadata: DashMap::new(),
                config,
            });
        }

        let engine: Arc<dyn VectorEngine> = match config.engine {
            crate::vector::config::VectorEngineType::Qdrant => {
                info!("Initializing Qdrant engine");
                let qdrant_config = config.qdrant.to_client_config();
                let qdrant_engine = QdrantEngine::new(qdrant_config)
                    .await
                    .map_err(|e| VectorError::ConnectionFailed(e.to_string()))?;
                Arc::new(qdrant_engine) as Arc<dyn VectorEngine>
            }
        };

        let health = engine
            .health_check()
            .await
            .map_err(|e| VectorError::ConnectionFailed(e.to_string()))?;

        if health.is_healthy {
            info!(
                "Vector engine health check passed: {} {}",
                health.engine_name, health.engine_version
            );
        } else {
            warn!("Vector engine health check failed: {:?}", health.message);
        }

        Ok(Self {
            engine,
            metadata: DashMap::new(),
            config,
        })
    }

    pub async fn create_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        index_config: Option<VectorIndexConfig>,
    ) -> VectorResult<String> {
        let key = (space_id, tag_name.to_string(), field_name.to_string());

        if self.metadata.contains_key(&key) {
            return Err(VectorError::IndexAlreadyExists(format!(
                "{}_{}_{}",
                space_id, tag_name, field_name
            )));
        }

        let config = index_config.unwrap_or(VectorIndexConfig {
            vector_size: self.config.default_vector_size,
            distance: self.config.default_distance,
            hnsw: None,
            quantization: None,
        });

        let collection_name = VectorIndexMetadata::collection_name(space_id, tag_name, field_name);

        debug!("Creating vector collection: {}", collection_name);

        self.engine
            .create_collection(&collection_name, config.to_collection_config())
            .await?;

        let metadata = VectorIndexMetadata {
            space_id,
            tag_name: tag_name.to_string(),
            field_name: field_name.to_string(),
            collection_name: collection_name.clone(),
            config,
            created_at: chrono::Utc::now(),
            vector_count: 0,
        };

        self.metadata.insert(key, metadata);

        info!("Vector index created: {}", collection_name);
        Ok(collection_name)
    }

    pub async fn drop_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> VectorResult<()> {
        let key = (space_id, tag_name.to_string(), field_name.to_string());

        if let Some((_, metadata)) = self.metadata.remove(&key) {
            debug!("Dropping vector collection: {}", metadata.collection_name);
            self.engine
                .delete_collection(&metadata.collection_name)
                .await?;
            info!("Vector index dropped: {}", metadata.collection_name);
        }

        Ok(())
    }

    pub fn get_engine(&self) -> &Arc<dyn VectorEngine> {
        &self.engine
    }

    pub fn get_metadata(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Option<VectorIndexMetadata> {
        let key = (space_id, tag_name.to_string(), field_name.to_string());
        self.metadata.get(&key).map(|m| m.clone())
    }

    pub fn index_exists(&self, space_id: u64, tag_name: &str, field_name: &str) -> bool {
        let key = (space_id, tag_name.to_string(), field_name.to_string());
        self.metadata.contains_key(&key)
    }

    pub fn get_collection_name(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Option<String> {
        let key = (space_id, tag_name.to_string(), field_name.to_string());
        self.metadata.get(&key).map(|m| m.collection_name.clone())
    }

    pub async fn upsert(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        point: VectorPoint,
    ) -> VectorResult<()> {
        let collection_name = self
            .get_collection_name(space_id, tag_name, field_name)
            .ok_or_else(|| VectorError::EngineNotFound {
                space_id,
                tag_name: tag_name.to_string(),
                field_name: field_name.to_string(),
            })?;

        self.engine.upsert(&collection_name, point).await?;
        Ok(())
    }

    pub async fn upsert_batch(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        points: Vec<VectorPoint>,
    ) -> VectorResult<()> {
        let collection_name = self
            .get_collection_name(space_id, tag_name, field_name)
            .ok_or_else(|| VectorError::EngineNotFound {
                space_id,
                tag_name: tag_name.to_string(),
                field_name: field_name.to_string(),
            })?;

        self.engine.upsert_batch(&collection_name, points).await?;
        Ok(())
    }

    pub async fn delete(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        point_id: &str,
    ) -> VectorResult<()> {
        let collection_name = self
            .get_collection_name(space_id, tag_name, field_name)
            .ok_or_else(|| VectorError::EngineNotFound {
                space_id,
                tag_name: tag_name.to_string(),
                field_name: field_name.to_string(),
            })?;

        self.engine.delete(&collection_name, point_id).await?;
        Ok(())
    }

    pub async fn delete_batch(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        point_ids: Vec<&str>,
    ) -> VectorResult<()> {
        let collection_name = self
            .get_collection_name(space_id, tag_name, field_name)
            .ok_or_else(|| VectorError::EngineNotFound {
                space_id,
                tag_name: tag_name.to_string(),
                field_name: field_name.to_string(),
            })?;

        self.engine
            .delete_batch(&collection_name, point_ids)
            .await?;
        Ok(())
    }

    pub async fn search(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        query: SearchQuery,
    ) -> VectorResult<Vec<SearchResult>> {
        let collection_name = self
            .get_collection_name(space_id, tag_name, field_name)
            .ok_or_else(|| VectorError::EngineNotFound {
                space_id,
                tag_name: tag_name.to_string(),
                field_name: field_name.to_string(),
            })?;

        let results = self.engine.search(&collection_name, query).await?;
        Ok(results)
    }

    pub async fn get(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        point_id: &str,
    ) -> VectorResult<Option<VectorPoint>> {
        let collection_name = self
            .get_collection_name(space_id, tag_name, field_name)
            .ok_or_else(|| VectorError::EngineNotFound {
                space_id,
                tag_name: tag_name.to_string(),
                field_name: field_name.to_string(),
            })?;

        let point = self.engine.get(&collection_name, point_id).await?;
        Ok(point)
    }

    pub async fn count(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> VectorResult<u64> {
        let collection_name = self
            .get_collection_name(space_id, tag_name, field_name)
            .ok_or_else(|| VectorError::EngineNotFound {
                space_id,
                tag_name: tag_name.to_string(),
                field_name: field_name.to_string(),
            })?;

        let count = self.engine.count(&collection_name).await?;
        Ok(count)
    }

    pub fn list_indexes(&self) -> Vec<VectorIndexMetadata> {
        self.metadata.iter().map(|m| m.clone()).collect()
    }

    pub async fn health_check(&self) -> VectorResult<bool> {
        let health = self.engine.health_check().await?;
        Ok(health.is_healthy)
    }
}

impl std::fmt::Debug for VectorIndexManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VectorIndexManager")
            .field("config", &self.config)
            .field("index_count", &self.metadata.len())
            .finish()
    }
}

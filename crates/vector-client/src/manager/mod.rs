//! Vector Manager for index lifecycle management
//!
//! Provides high-level API for managing vector indexes and operations.

mod index;

pub use index::IndexMetadata;

use std::sync::Arc;

use dashmap::DashMap;
use tracing::{debug, info, warn};

use crate::config::VectorClientConfig;
use crate::engine::{VectorEngine, QdrantEngine};
use crate::error::{Result, VectorClientError};
use crate::types::{CollectionConfig, SearchQuery, SearchResult, VectorPoint};

/// Vector manager for index lifecycle management
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
    /// Create a new vector manager
    pub async fn new(config: VectorClientConfig) -> Result<Self> {
        let engine: Arc<dyn VectorEngine> = if config.enabled {
            match config.engine {
                crate::config::EngineType::Qdrant => {
                    info!("Initializing Qdrant engine");
                    let qdrant_config = config.to_qdrant_config();
                    let qdrant_engine = QdrantEngine::new(qdrant_config)
                        .await
                        .map_err(|e| VectorClientError::ConnectionFailed(e.to_string()))?;
                    Arc::new(qdrant_engine) as Arc<dyn VectorEngine>
                }
            }
        } else {
            info!("Vector search is disabled, using default Qdrant engine");
            let qdrant_config = VectorClientConfig::default();
            let qdrant_engine = QdrantEngine::new(qdrant_config)
                .await
                .map_err(|e| VectorClientError::ConnectionFailed(e.to_string()))?;
            Arc::new(qdrant_engine) as Arc<dyn VectorEngine>
        };

        // Health check
        if config.enabled {
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

    /// Get the underlying engine
    pub fn engine(&self) -> &Arc<dyn VectorEngine> {
        &self.engine
    }

    // ========== Index Management ==========

    /// Create a new index
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

    /// Drop an index
    pub async fn drop_index(&self, name: &str) -> Result<()> {
        if let Some((_, metadata)) = self.indexes.remove(name) {
            debug!("Dropping vector collection: {}", metadata.name);
            self.engine.delete_collection(name).await?;
            info!("Vector index dropped: {}", name);
        }
        Ok(())
    }

    /// Check if an index exists
    pub fn index_exists(&self, name: &str) -> bool {
        self.indexes.contains_key(name)
    }

    /// Get index metadata
    pub fn get_index_metadata(&self, name: &str) -> Option<IndexMetadata> {
        self.indexes.get(name).map(|m| m.clone())
    }

    /// List all indexes
    pub fn list_indexes(&self) -> Vec<IndexMetadata> {
        self.indexes.iter().map(|m| m.value().clone()).collect()
    }

    // ========== Vector Operations ==========

    /// Upsert a single vector point
    pub async fn upsert(&self, collection: &str, point: VectorPoint) -> Result<()> {
        self.engine.upsert(collection, point).await?;
        Ok(())
    }

    /// Upsert multiple vector points in batch
    pub async fn upsert_batch(&self, collection: &str, points: Vec<VectorPoint>) -> Result<()> {
        self.engine.upsert_batch(collection, points).await?;
        Ok(())
    }

    /// Delete a vector point
    pub async fn delete(&self, collection: &str, point_id: &str) -> Result<()> {
        self.engine.delete(collection, point_id).await?;
        Ok(())
    }

    /// Delete multiple vector points in batch
    pub async fn delete_batch(&self, collection: &str, point_ids: Vec<&str>) -> Result<()> {
        self.engine.delete_batch(collection, point_ids).await?;
        Ok(())
    }

    /// Search for similar vectors
    pub async fn search(&self, collection: &str, query: SearchQuery) -> Result<Vec<SearchResult>> {
        self.engine.search(collection, query).await
    }

    /// Get a vector point by ID
    pub async fn get(&self, collection: &str, point_id: &str) -> Result<Option<VectorPoint>> {
        self.engine.get(collection, point_id).await
    }

    /// Count vectors in a collection
    pub async fn count(&self, collection: &str) -> Result<u64> {
        self.engine.count(collection).await
    }
}

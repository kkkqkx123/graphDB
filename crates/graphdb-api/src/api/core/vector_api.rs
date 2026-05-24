//! Vector Index API – Core Layer
//!
//! Provides transport layer independent vector index management and search operations.

use crate::api::core::error::{CoreError, CoreResult};
use crate::api::core::types::VectorSearchResult;
use crate::sync::vector_sync::{SearchOptions, VectorIndexLocation, VectorSyncCoordinator};
use std::sync::Arc;
use vector_client::manager::IndexMetadata;
use vector_client::{
    CollectionConfig, DistanceMetric, SearchQuery, VectorClientError, VectorManager, VectorPoint,
};

/// Vector Index API – Core Layer
pub struct VectorApi {
    vector_manager: Arc<VectorManager>,
    coordinator: Option<Arc<VectorSyncCoordinator>>,
}

impl VectorApi {
    /// Create a new VectorApi instance
    pub fn new(vector_manager: Arc<VectorManager>) -> Self {
        Self {
            vector_manager,
            coordinator: None,
        }
    }

    /// Create a new VectorApi instance with sync coordinator
    pub fn with_coordinator(
        vector_manager: Arc<VectorManager>,
        coordinator: Arc<VectorSyncCoordinator>,
    ) -> Self {
        Self {
            vector_manager,
            coordinator: Some(coordinator),
        }
    }

    /// Get the vector manager
    pub fn vector_manager(&self) -> &Arc<VectorManager> {
        &self.vector_manager
    }

    /// Get the sync coordinator
    pub fn coordinator(&self) -> Option<&Arc<VectorSyncCoordinator>> {
        self.coordinator.as_ref()
    }

    /// Create a vector index
    pub async fn create_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        vector_size: usize,
        distance: DistanceMetric,
    ) -> CoreResult<String> {
        if let Some(coordinator) = &self.coordinator {
            coordinator
                .create_vector_index(space_id, tag_name, field_name, vector_size, distance)
                .await
                .map_err(|e| CoreError::VectorError(e.to_string()))
        } else {
            let collection_name = format!("space_{}_{}_{}", space_id, tag_name, field_name);
            let config = CollectionConfig {
                vector_size,
                distance,
                ..Default::default()
            };
            self.vector_manager
                .create_index(&collection_name, config)
                .await
                .map_err(|e| CoreError::VectorError(e.to_string()))?;
            Ok(collection_name)
        }
    }

    /// Drop a vector index
    pub async fn drop_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> CoreResult<()> {
        if let Some(coordinator) = &self.coordinator {
            coordinator
                .drop_vector_index(space_id, tag_name, field_name)
                .await
                .map_err(|e| CoreError::VectorError(e.to_string()))
        } else {
            let collection_name = format!("space_{}_{}_{}", space_id, tag_name, field_name);
            self.vector_manager
                .drop_index(&collection_name)
                .await
                .map_err(|e: VectorClientError| CoreError::VectorError(e.to_string()))
        }
    }

    /// Get vector index info
    pub fn get_index_info(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> CoreResult<Option<IndexMetadata>> {
        let collection_name = format!("space_{}_{}_{}", space_id, tag_name, field_name);
        Ok(self.vector_manager.get_index_metadata(&collection_name))
    }

    /// List all vector indexes
    pub fn list_indexes(&self) -> Vec<String> {
        self.vector_manager
            .list_indexes()
            .into_iter()
            .map(|info| info.name)
            .collect()
    }

    /// Insert a vector point
    pub async fn insert_vector(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        point: VectorPoint,
    ) -> CoreResult<()> {
        let collection_name = format!("space_{}_{}_{}", space_id, tag_name, field_name);
        self.vector_manager
            .upsert(&collection_name, point)
            .await
            .map_err(|e| CoreError::VectorError(e.to_string()))?;
        Ok(())
    }

    /// Insert vector points in batch
    pub async fn insert_vector_batch(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        points: Vec<VectorPoint>,
    ) -> CoreResult<()> {
        let collection_name = format!("space_{}_{}_{}", space_id, tag_name, field_name);
        self.vector_manager
            .upsert_batch(&collection_name, points)
            .await
            .map_err(|e| CoreError::VectorError(e.to_string()))?;
        Ok(())
    }

    /// Delete a vector point
    pub async fn delete_vector(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        point_id: &str,
    ) -> CoreResult<()> {
        let collection_name = format!("space_{}_{}_{}", space_id, tag_name, field_name);
        self.vector_manager
            .delete(&collection_name, point_id)
            .await
            .map_err(|e| CoreError::VectorError(e.to_string()))
    }

    /// Delete vector points in batch
    pub async fn delete_vector_batch(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        point_ids: Vec<&str>,
    ) -> CoreResult<()> {
        let collection_name = format!("space_{}_{}_{}", space_id, tag_name, field_name);
        self.vector_manager
            .delete_batch(&collection_name, point_ids)
            .await
            .map_err(|e| CoreError::VectorError(e.to_string()))
    }

    /// Search vectors with options
    pub async fn search_with_options(
        &self,
        options: SearchOptions,
    ) -> CoreResult<Vec<VectorSearchResult>> {
        let collection_name =
            VectorIndexLocation::new(options.space_id, &options.tag_name, &options.field_name)
                .to_collection_name();

        let mut query = SearchQuery::new(options.query_vector, options.limit);

        if let Some(threshold) = options.threshold {
            query = query.with_score_threshold(threshold);
        }

        if let Some(filter) = options.filter {
            query = query.with_filter(filter);
        }

        let results = self
            .vector_manager
            .search(&collection_name, query)
            .await
            .map_err(|e| CoreError::VectorError(e.to_string()))?;

        Ok(results
            .into_iter()
            .map(|r| VectorSearchResult {
                id: r.id,
                score: r.score,
                vector: r.vector.map(|v| v.to_vec()),
                payload: r.payload.map(|p| p.into_iter().collect()),
            })
            .collect())
    }

    /// Get a vector point by ID
    pub async fn get_vector(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        point_id: &str,
    ) -> CoreResult<Option<VectorPoint>> {
        let collection_name = format!("space_{}_{}_{}", space_id, tag_name, field_name);
        self.vector_manager
            .get(&collection_name, point_id)
            .await
            .map_err(|e| CoreError::VectorError(e.to_string()))
    }

    /// Get vector index count
    pub async fn count(&self, space_id: u64, tag_name: &str, field_name: &str) -> CoreResult<u64> {
        let collection_name = format!("space_{}_{}_{}", space_id, tag_name, field_name);
        self.vector_manager
            .count(&collection_name)
            .await
            .map_err(|e| CoreError::VectorError(e.to_string()))
    }
}

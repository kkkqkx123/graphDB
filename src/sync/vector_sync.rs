//! Vector Synchronization Coordinator
//!
//! Coordinates vector index updates with graph data changes.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::core::error::{VectorCoordinatorError, VectorCoordinatorResult};
use crate::core::{Value, Vertex};
pub use crate::sync::task::VectorPointData;
pub use crate::sync::vector_batch::{VectorBatchConfig, VectorBatchManager};

use vector_client::{
    EmbeddingService, SearchQuery, SearchResult, VectorFilter, VectorManager, VectorPoint,
};

/// Search options for vector search
#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub space_id: u64,
    pub tag_name: String,
    pub field_name: String,
    pub query_vector: Vec<f32>,
    pub limit: usize,
    pub threshold: Option<f32>,
    pub filter: Option<VectorFilter>,
}

impl SearchOptions {
    pub fn new(
        space_id: u64,
        tag_name: impl Into<String>,
        field_name: impl Into<String>,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> Self {
        Self {
            space_id,
            tag_name: tag_name.into(),
            field_name: field_name.into(),
            query_vector,
            limit,
            threshold: None,
            filter: None,
        }
    }

    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.threshold = Some(threshold);
        self
    }

    pub fn with_filter(mut self, filter: VectorFilter) -> Self {
        self.filter = Some(filter);
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VectorChangeType {
    Insert,
    Delete,
}



impl From<crate::coordinator::ChangeType> for VectorChangeType {
    fn from(ct: crate::coordinator::ChangeType) -> Self {
        match ct {
            crate::coordinator::ChangeType::Insert => VectorChangeType::Insert,
            crate::coordinator::ChangeType::Delete => VectorChangeType::Delete,
            _ => VectorChangeType::Delete,
        }
    }
}

/// Vector index location identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VectorIndexLocation {
    pub space_id: u64,
    pub tag_name: String,
    pub field_name: String,
}

impl VectorIndexLocation {
    pub fn new(space_id: u64, tag_name: impl Into<String>, field_name: impl Into<String>) -> Self {
        Self {
            space_id,
            tag_name: tag_name.into(),
            field_name: field_name.into(),
        }
    }

    /// Generate collection name (for Qdrant etc.)
    pub fn to_collection_name(&self) -> String {
        format!(
            "space_{}_{}_{}",
            self.space_id, self.tag_name, self.field_name
        )
    }
}

/// Vector change context
#[derive(Debug, Clone)]
pub struct VectorChangeContext {
    pub location: VectorIndexLocation,
    pub change_type: VectorChangeType,
    pub data: VectorPointData,
}

impl VectorChangeContext {
    pub fn new(
        space_id: u64,
        tag_name: impl Into<String>,
        field_name: impl Into<String>,
        change_type: VectorChangeType,
        data: VectorPointData,
    ) -> Self {
        Self {
            location: VectorIndexLocation::new(space_id, tag_name, field_name),
            change_type,
            data,
        }
    }
}

/// Vector synchronization coordinator
pub struct VectorSyncCoordinator {
    vector_manager: Arc<VectorManager>,
    embedding_service: Option<Arc<EmbeddingService>>,
}

impl std::fmt::Debug for VectorSyncCoordinator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VectorSyncCoordinator")
            .field("vector_manager", &self.vector_manager)
            .field("embedding_service", &self.embedding_service.is_some())
            .finish()
    }
}

impl VectorSyncCoordinator {
    /// Create a new vector sync coordinator
    pub fn new(
        vector_manager: Arc<VectorManager>,
        embedding_service: Option<Arc<EmbeddingService>>,
    ) -> Self {
        Self {
            vector_manager,
            embedding_service,
        }
    }

    /// Get the vector manager
    pub fn vector_manager(&self) -> &Arc<VectorManager> {
        &self.vector_manager
    }

    /// Get the embedding service
    pub fn embedding_service(&self) -> Option<&Arc<EmbeddingService>> {
        self.embedding_service.as_ref()
    }

    /// Create a vector index
    pub async fn create_vector_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        vector_size: usize,
        distance: vector_client::DistanceMetric,
    ) -> VectorCoordinatorResult<String> {
        let collection_name =
            VectorIndexLocation::new(space_id, tag_name, field_name).to_collection_name();

        let config = vector_client::CollectionConfig::new(vector_size, distance);

        self.vector_manager
            .create_index(&collection_name, config)
            .await
            .map_err(|e| VectorCoordinatorError::IndexCreationFailed {
                tag_name: tag_name.to_string(),
                field_name: field_name.to_string(),
                reason: e.to_string(),
            })?;

        info!("Vector index created: {}", collection_name);
        Ok(collection_name)
    }

    /// Drop a vector index
    pub async fn drop_vector_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> VectorCoordinatorResult<()> {
        let collection_name =
            VectorIndexLocation::new(space_id, tag_name, field_name).to_collection_name();

        self.vector_manager
            .drop_index(&collection_name)
            .await
            .map_err(|e| VectorCoordinatorError::IndexDropFailed {
                tag_name: tag_name.to_string(),
                field_name: field_name.to_string(),
                reason: e.to_string(),
            })?;

        info!("Vector index dropped: {}", collection_name);
        Ok(())
    }

    /// Handle vertex insertion
    pub async fn on_vertex_inserted(
        &self,
        space_id: u64,
        vertex: &Vertex,
    ) -> VectorCoordinatorResult<()> {
        self.upsert_vertex_vectors(space_id, vertex).await
    }

    /// Upsert vectors for a vertex
    async fn upsert_vertex_vectors(
        &self,
        space_id: u64,
        vertex: &Vertex,
    ) -> VectorCoordinatorResult<()> {
        let mut points_by_collection: HashMap<String, Vec<VectorPoint>> = HashMap::new();

        for tag in &vertex.tags {
            for (field_name, value) in &tag.properties {
                let collection_name =
                    VectorIndexLocation::new(space_id, &tag.name, field_name).to_collection_name();

                if self.vector_manager.index_exists(&collection_name) {
                    if let Some(vector) = value.as_vector() {
                        let point_id = vertex.vid.to_string();
                        let mut payload = HashMap::new();
                        payload.insert(
                            "vertex_id".to_string(),
                            serde_json::to_value(&vertex.vid).unwrap_or(serde_json::Value::Null),
                        );

                        let point = VectorPoint::new(point_id.clone(), vector.clone())
                            .with_payload(payload);

                        points_by_collection
                            .entry(collection_name)
                            .or_default()
                            .push(point);
                    }
                }
            }
        }

        for (collection_name, points) in points_by_collection {
            let points_count = points.len();
            if points_count == 1 {
                self.vector_manager
                    .upsert(&collection_name, points.into_iter().next().unwrap())
                    .await?;
            } else if !points.is_empty() {
                self.vector_manager
                    .upsert_batch(&collection_name, points)
                    .await?;
                debug!(
                    "Batch upserted {} vectors for vertex {} in collection {}",
                    points_count, vertex.vid, collection_name
                );
            }
        }

        Ok(())
    }

    /// Handle vertex update
    pub async fn on_vertex_updated(
        &self,
        space_id: u64,
        vertex: &Vertex,
        changed_fields: &[String],
    ) -> VectorCoordinatorResult<()> {
        let mut points_to_upsert: HashMap<String, Vec<VectorPoint>> = HashMap::new();
        let mut points_to_delete: HashMap<String, Vec<String>> = HashMap::new();

        for tag in &vertex.tags {
            for field_name in changed_fields {
                if let Some(value) = tag.properties.get(field_name) {
                    let collection_name = VectorIndexLocation::new(space_id, &tag.name, field_name)
                        .to_collection_name();

                    if self.vector_manager.index_exists(&collection_name) {
                        let point_id = vertex.vid.to_string();

                        if let Some(vector) = value.as_vector() {
                            let mut payload = HashMap::new();
                            payload.insert(
                                "vertex_id".to_string(),
                                serde_json::to_value(&vertex.vid)
                                    .unwrap_or(serde_json::Value::Null),
                            );

                            let point = VectorPoint::new(point_id.clone(), vector.clone())
                                .with_payload(payload);

                            points_to_upsert
                                .entry(collection_name)
                                .or_default()
                                .push(point);
                        } else {
                            points_to_delete
                                .entry(collection_name)
                                .or_default()
                                .push(point_id);
                        }
                    }
                }
            }
        }

        for (collection_name, points) in points_to_upsert {
            let points_count = points.len();
            if points_count == 1 {
                self.vector_manager
                    .upsert(&collection_name, points.into_iter().next().unwrap())
                    .await?;
            } else if !points.is_empty() {
                self.vector_manager
                    .upsert_batch(&collection_name, points)
                    .await?;
                debug!(
                    "Batch updated {} vectors for vertex {} in collection {}",
                    points_count, vertex.vid, collection_name
                );
            }
        }

        for (collection_name, point_ids) in points_to_delete {
            let point_ids_count = point_ids.len();
            if point_ids_count == 1 {
                self.vector_manager
                    .delete(&collection_name, &point_ids[0])
                    .await?;
            } else if !point_ids.is_empty() {
                let refs: Vec<&str> = point_ids.iter().map(|s| s.as_str()).collect();
                self.vector_manager
                    .delete_batch(&collection_name, refs)
                    .await?;
                debug!(
                    "Batch deleted {} vectors for vertex {} from collection {}",
                    point_ids_count, vertex.vid, collection_name
                );
            }
        }

        Ok(())
    }

    /// Handle vertex deletion
    pub async fn on_vertex_deleted(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
    ) -> VectorCoordinatorResult<()> {
        let point_id = format!("{}", vertex_id);

        let collections_to_delete_from: Vec<String> = self
            .vector_manager
            .list_indexes()
            .iter()
            .filter(|metadata| {
                metadata
                    .name
                    .starts_with(&format!("space_{}_{}_", space_id, tag_name))
            })
            .map(|m| m.name.clone())
            .collect();

        for collection_name in collections_to_delete_from {
            self.vector_manager
                .delete(&collection_name, &point_id)
                .await?;

            debug!(
                "Deleted vector for vertex {} from collection {}",
                vertex_id, collection_name
            );
        }
        Ok(())
    }

    /// Handle vector change
    pub async fn on_vector_change(&self, ctx: VectorChangeContext) -> VectorCoordinatorResult<()> {
        let collection_name = ctx.location.to_collection_name();
        let point_id = ctx.data.id.to_string();

        match ctx.change_type {
            VectorChangeType::Insert => {
                let vector = ctx.data.vector;
                let json_payload: HashMap<String, serde_json::Value> = ctx
                    .data
                    .payload
                    .into_iter()
                    .filter_map(|(k, v)| serde_json::to_value(&v).ok().map(|json| (k, json)))
                    .collect();

                let point = VectorPoint::new(point_id, vector).with_payload(json_payload);

                self.vector_manager.upsert(&collection_name, point).await?;
            }
            VectorChangeType::Delete => {
                self.vector_manager
                    .delete(&collection_name, &point_id)
                    .await?;
            }
        }

        Ok(())
    }

    /// Handle batch vector changes
    pub async fn on_vector_change_batch(
        &self,
        contexts: Vec<VectorChangeContext>,
    ) -> VectorCoordinatorResult<()> {
        let mut upsert_by_collection: HashMap<String, Vec<VectorPoint>> = HashMap::new();
        let mut delete_by_collection: HashMap<String, Vec<String>> = HashMap::new();

        for ctx in contexts {
            let collection_name = ctx.location.to_collection_name();
            let point_id = ctx.data.id.to_string();

            match ctx.change_type {
                VectorChangeType::Insert => {
                    let vector = ctx.data.vector;
                    let json_payload: HashMap<String, serde_json::Value> = ctx
                        .data
                        .payload
                        .into_iter()
                        .filter_map(|(k, v)| serde_json::to_value(&v).ok().map(|json| (k, json)))
                        .collect();

                    let point = VectorPoint::new(point_id, vector).with_payload(json_payload);

                    upsert_by_collection
                        .entry(collection_name)
                        .or_default()
                        .push(point);
                }
                VectorChangeType::Delete => {
                    delete_by_collection
                        .entry(collection_name)
                        .or_default()
                        .push(point_id);
                }
            }
        }

        for (collection_name, points) in upsert_by_collection {
            let points_count = points.len();
            if points_count == 1 {
                self.vector_manager
                    .upsert(&collection_name, points.into_iter().next().unwrap())
                    .await?;
            } else if !points.is_empty() {
                self.vector_manager
                    .upsert_batch(&collection_name, points)
                    .await?;
                debug!(
                    "Batch upserted {} vectors to collection {}",
                    points_count, collection_name
                );
            }
        }

        for (collection_name, point_ids) in delete_by_collection {
            let point_ids_count = point_ids.len();
            if point_ids_count == 1 {
                self.vector_manager
                    .delete(&collection_name, &point_ids[0])
                    .await?;
            } else if !point_ids.is_empty() {
                let refs: Vec<&str> = point_ids.iter().map(|s| s.as_str()).collect();
                self.vector_manager
                    .delete_batch(&collection_name, refs)
                    .await?;
                debug!(
                    "Batch deleted {} vectors from collection {}",
                    point_ids_count, collection_name
                );
            }
        }

        Ok(())
    }

    /// Search for similar vectors
    pub async fn search(
        &self,
        collection: &str,
        query: SearchQuery,
    ) -> VectorCoordinatorResult<Vec<SearchResult>> {
        let results = self.vector_manager.search(collection, query).await?;
        Ok(results)
    }

    /// Search with options
    pub async fn search_with_options(
        &self,
        options: SearchOptions,
    ) -> VectorCoordinatorResult<Vec<SearchResult>> {
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

        let results = self.search(&collection_name, query).await?;
        Ok(results)
    }

    /// Embed text to vector
    pub async fn embed_text(&self, text: &str) -> VectorCoordinatorResult<Vec<f32>> {
        if let Some(embedding) = &self.embedding_service {
            let vector = embedding
                .embed(text)
                .await
                .map_err(|e| VectorCoordinatorError::EmbeddingError(e.to_string()))?;
            Ok(vector)
        } else {
            Err(VectorCoordinatorError::EmbeddingError(
                "Embedding service not available".to_string(),
            ))
        }
    }

    /// Check if index exists
    pub fn index_exists(&self, space_id: u64, tag_name: &str, field_name: &str) -> bool {
        let collection_name =
            VectorIndexLocation::new(space_id, tag_name, field_name).to_collection_name();
        self.vector_manager.index_exists(&collection_name)
    }

    /// List all indexes
    pub fn list_indexes(&self) -> Vec<crate::sync::vector_sync::IndexMetadataWrapper> {
        self.vector_manager
            .list_indexes()
            .into_iter()
            .map(crate::sync::vector_sync::IndexMetadataWrapper::from)
            .collect()
    }

    /// Create vector index with config
    pub async fn create_index_with_config(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        config: vector_client::CollectionConfig,
    ) -> VectorCoordinatorResult<String> {
        let collection_name =
            VectorIndexLocation::new(space_id, tag_name, field_name).to_collection_name();

        self.vector_manager
            .create_index(&collection_name, config)
            .await
            .map_err(|e| VectorCoordinatorError::IndexCreationFailed {
                tag_name: tag_name.to_string(),
                field_name: field_name.to_string(),
                reason: e.to_string(),
            })?;

        info!("Vector index created: {}", collection_name);
        Ok(collection_name)
    }

    /// Search with space_id and tag/field names
    pub async fn search_by_location(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> VectorCoordinatorResult<Vec<SearchResult>> {
        let collection_name =
            VectorIndexLocation::new(space_id, tag_name, field_name).to_collection_name();

        let query = SearchQuery::new(query_vector, limit);
        self.search(&collection_name, query).await
    }

    /// Search with threshold
    pub async fn search_with_threshold(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        query_vector: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> VectorCoordinatorResult<Vec<SearchResult>> {
        let collection_name =
            VectorIndexLocation::new(space_id, tag_name, field_name).to_collection_name();

        let query = SearchQuery::new(query_vector, limit).with_score_threshold(threshold);
        self.search(&collection_name, query).await
    }

    /// Search with filter
    pub async fn search_with_filter(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        query_vector: Vec<f32>,
        limit: usize,
        filter: VectorFilter,
    ) -> VectorCoordinatorResult<Vec<SearchResult>> {
        let collection_name =
            VectorIndexLocation::new(space_id, tag_name, field_name).to_collection_name();

        let query = SearchQuery::new(query_vector, limit).with_filter(filter);
        self.search(&collection_name, query).await
    }

    /// Search with threshold and filter
    pub async fn search_with_threshold_and_filter(
        &self,
        mut options: SearchOptions,
        threshold: f32,
        filter: VectorFilter,
    ) -> VectorCoordinatorResult<Vec<SearchResult>> {
        options.threshold = Some(threshold);
        options.filter = Some(filter);
        self.search_with_options(options).await
    }
}

/// Index metadata wrapper for backward compatibility
#[derive(Debug, Clone)]
pub struct IndexMetadataWrapper {
    pub collection_name: String,
    pub space_id: u64,
    pub tag_name: String,
    pub field_name: String,
}

impl From<vector_client::manager::IndexMetadata> for IndexMetadataWrapper {
    fn from(metadata: vector_client::manager::IndexMetadata) -> Self {
        // Parse collection name to extract space_id, tag_name, field_name
        // Format: "space_{space_id}_{tag}_{field}"
        let parts: Vec<&str> = metadata.name.split('_').collect();
        let (space_id, tag_name, field_name) = if parts.len() >= 4 && parts[0] == "space" {
            let sid: u64 = parts[1].parse().unwrap_or(0);
            (sid, parts[2].to_string(), parts[3].to_string())
        } else {
            (0, String::new(), String::new())
        };

        Self {
            collection_name: metadata.name,
            space_id,
            tag_name,
            field_name,
        }
    }
}

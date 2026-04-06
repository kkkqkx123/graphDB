use async_trait::async_trait;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, PointId, PointStruct,
    SearchPointsBuilder, UpsertPointsBuilder, VectorParamsBuilder,
    GetPointsBuilder, ScrollPointsBuilder, DeletePointsBuilder,
    SetPayloadPointsBuilder, DeletePayloadPointsBuilder, PointsIdsList,
    vectors_output::VectorsOptions,
};
use qdrant_client::{Qdrant, Payload as QdrantPayload};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::config::{ConnectionConfig, VectorClientConfig};
use crate::error::{Result, VectorClientError};
use crate::types::*;
use super::VectorEngine;

const QDRANT_VERSION: &str = "1.17.x";

pub struct QdrantEngine {
    client: Arc<Qdrant>,
    config: VectorClientConfig,
    collections: RwLock<HashMap<String, CollectionConfig>>,
}

impl std::fmt::Debug for QdrantEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QdrantEngine")
            .field("config", &self.config)
            .field("collections", &self.collections)
            .finish()
    }
}

impl QdrantEngine {
    pub async fn new(config: VectorClientConfig) -> Result<Self> {
        let client = Self::create_client(&config.connection).await?;

        Ok(Self {
            client: Arc::new(client),
            config,
            collections: RwLock::new(HashMap::new()),
        })
    }

    async fn create_client(conn_config: &ConnectionConfig) -> Result<Qdrant> {
        let url = conn_config.to_url();

        info!("Connecting to Qdrant at {}", url);

        let mut builder = Qdrant::from_url(&url);

        if let Some(ref api_key) = conn_config.api_key {
            builder = builder.api_key(api_key.clone());
        }

        let client = builder.build()?;

        match client.health_check().await {
            Ok(_) => {
                info!("Successfully connected to Qdrant");
                Ok(client)
            }
            Err(e) => {
                warn!("Failed to connect to Qdrant: {}", e);
                Err(VectorClientError::ConnectionFailed(format!(
                    "Failed to connect to Qdrant at {}: {}",
                    url, e
                )))
            }
        }
    }

    fn convert_distance(distance: DistanceMetric) -> Distance {
        match distance {
            DistanceMetric::Cosine => Distance::Cosine,
            DistanceMetric::Euclid => Distance::Euclid,
            DistanceMetric::Dot => Distance::Dot,
        }
    }

    fn point_id_from_str(id: &str) -> PointId {
        if let Ok(num) = id.parse::<u64>() {
            num.into()
        } else {
            id.into()
        }
    }

    fn point_struct_from_vector_point(point: VectorPoint) -> Result<PointStruct> {
        let id = Self::point_id_from_str(&point.id);
        let payload = Self::payload_to_qdrant_payload(&point.payload)?;
        Ok(PointStruct::new(id, point.vector, payload))
    }

    fn payload_to_qdrant_payload(payload: &Option<Payload>) -> Result<QdrantPayload> {
        let json = match payload {
            Some(p) => serde_json::to_value(p),
            None => Ok(serde_json::Value::Object(Default::default())),
        }
        .map_err(|e| VectorClientError::PayloadError(e.to_string()))?;

        QdrantPayload::try_from(json)
            .map_err(|e| VectorClientError::PayloadError(e.to_string()))
    }

    fn qdrant_payload_to_payload(payload: HashMap<String, qdrant_client::qdrant::Value>) -> Payload {
        let json = serde_json::to_value(payload).unwrap_or(serde_json::Value::Object(Default::default()));
        serde_json::from_value(json).unwrap_or_default()
    }

    fn search_result_from_scored_point(
        point: qdrant_client::qdrant::ScoredPoint,
    ) -> Result<SearchResult> {
        let id = point.id
            .map(|id| format!("{:?}", id))
            .ok_or_else(|| VectorClientError::InvalidPointId("empty id".to_string()))?;

        let payload = if point.payload.is_empty() {
            None
        } else {
            Some(Self::qdrant_payload_to_payload(point.payload))
        };

        Ok(SearchResult {
            id,
            score: point.score,
            payload,
            vector: None,
        })
    }

    fn vector_point_from_retrieved_point(
        point: qdrant_client::qdrant::RetrievedPoint,
    ) -> Result<VectorPoint> {
        let id = point.id
            .map(|id| format!("{:?}", id))
            .ok_or_else(|| VectorClientError::InvalidPointId("empty id".to_string()))?;

        let payload = if point.payload.is_empty() {
            None
        } else {
            Some(Self::qdrant_payload_to_payload(point.payload))
        };

        #[allow(deprecated)]
        let vector = point.vectors.and_then(|v| {
            match v.vectors_options {
                Some(VectorsOptions::Vector(vec)) => Some(vec.data),
                _ => None,
            }
        });

        Ok(VectorPoint {
            id,
            vector: vector.unwrap_or_default(),
            payload,
        })
    }
}

#[async_trait]
impl VectorEngine for QdrantEngine {
    fn name(&self) -> &str {
        "qdrant"
    }

    fn version(&self) -> &str {
        QDRANT_VERSION
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        match self.client.health_check().await {
            Ok(_) => Ok(HealthStatus::healthy(self.name(), self.version())),
            Err(e) => Ok(HealthStatus::unhealthy(
                self.name(),
                self.version(),
                e.to_string(),
            )),
        }
    }

    async fn create_collection(&self, name: &str, config: CollectionConfig) -> Result<()> {
        debug!("Creating collection '{}' with config: {:?}", name, config);

        let distance = Self::convert_distance(config.distance);
        let vector_params = VectorParamsBuilder::new(config.vector_size as u64, distance);

        let builder = CreateCollectionBuilder::new(name)
            .vectors_config(vector_params);

        self.client.create_collection(builder).await?;

        self.collections.write().await.insert(name.to_string(), config);

        info!("Collection '{}' created successfully", name);
        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        debug!("Deleting collection '{}'", name);

        self.client.delete_collection(name).await?;

        self.collections.write().await.remove(name);

        info!("Collection '{}' deleted successfully", name);
        Ok(())
    }

    async fn collection_exists(&self, name: &str) -> Result<bool> {
        let collections = self.client.list_collections().await?;
        Ok(collections
            .collections
            .iter()
            .any(|c| c.name == name))
    }

    async fn collection_info(&self, name: &str) -> Result<CollectionInfo> {
        let response = self.client.collection_info(name).await?;
        let info = response.result.ok_or_else(|| {
            VectorClientError::CollectionNotFound(name.to_string())
        })?;

        let config = self
            .collections
            .read()
            .await
            .get(name)
            .cloned()
            .unwrap_or_default();

        Ok(CollectionInfo {
            name: name.to_string(),
            vector_count: info.points_count.unwrap_or(0),
            indexed_vector_count: info.indexed_vectors_count.unwrap_or(0),
            points_count: info.points_count.unwrap_or(0),
            segments_count: info.segments_count,
            config,
            status: CollectionStatus::Green,
        })
    }

    async fn upsert(&self, collection: &str, point: VectorPoint) -> Result<UpsertResult> {
        debug!("Upserting point '{}' to collection '{}'", point.id, collection);

        let point_struct = Self::point_struct_from_vector_point(point)?;

        self.client
            .upsert_points(UpsertPointsBuilder::new(collection, vec![point_struct]).wait(true))
            .await?;

        Ok(UpsertResult {
            operation_id: None,
            status: UpsertStatus::Completed,
        })
    }

    async fn upsert_batch(&self, collection: &str, points: Vec<VectorPoint>) -> Result<UpsertResult> {
        debug!("Upserting {} points to collection '{}'", points.len(), collection);

        let point_structs: Result<Vec<PointStruct>> = points
            .into_iter()
            .map(Self::point_struct_from_vector_point)
            .collect();

        self.client
            .upsert_points(UpsertPointsBuilder::new(collection, point_structs?).wait(true))
            .await?;

        Ok(UpsertResult {
            operation_id: None,
            status: UpsertStatus::Completed,
        })
    }

    async fn delete(&self, collection: &str, point_id: &str) -> Result<DeleteResult> {
        debug!("Deleting point '{}' from collection '{}'", point_id, collection);

        let id = Self::point_id_from_str(point_id);

        self.client
            .delete_points(DeletePointsBuilder::new(collection).points(vec![id]))
            .await?;

        Ok(DeleteResult {
            operation_id: None,
            deleted_count: 1,
        })
    }

    async fn delete_batch(&self, collection: &str, point_ids: Vec<&str>) -> Result<DeleteResult> {
        debug!("Deleting {} points from collection '{}'", point_ids.len(), collection);

        let ids: Vec<PointId> = point_ids
            .iter()
            .map(|id| Self::point_id_from_str(id))
            .collect();

        self.client
            .delete_points(DeletePointsBuilder::new(collection).points(ids))
            .await?;

        Ok(DeleteResult {
            operation_id: None,
            deleted_count: point_ids.len() as u64,
        })
    }

    async fn delete_by_filter(&self, collection: &str, _filter: VectorFilter) -> Result<DeleteResult> {
        debug!("Deleting points by filter from collection '{}'", collection);

        Err(VectorClientError::FilterError(
            "Filter-based deletion not yet implemented".to_string(),
        ))
    }

    async fn search(&self, collection: &str, query: SearchQuery) -> Result<Vec<SearchResult>> {
        debug!("Searching in collection '{}' with limit {}", collection, query.limit);

        let mut builder = SearchPointsBuilder::new(collection, query.vector, query.limit as u64)
            .with_payload(query.with_payload.unwrap_or(true))
            .with_vectors(query.with_vector.unwrap_or(false));

        if let Some(offset) = query.offset {
            builder = builder.offset(offset as u64);
        }

        if let Some(threshold) = query.score_threshold {
            builder = builder.score_threshold(threshold);
        }

        let result = self.client.search_points(builder).await?;

        let search_results: Result<Vec<SearchResult>> = result
            .result
            .into_iter()
            .map(Self::search_result_from_scored_point)
            .collect();

        search_results
    }

    async fn search_batch(
        &self,
        collection: &str,
        queries: Vec<SearchQuery>,
    ) -> Result<Vec<Vec<SearchResult>>> {
        debug!("Batch searching {} queries in collection '{}'", queries.len(), collection);

        let mut results = Vec::with_capacity(queries.len());

        for query in queries {
            let result = self.search(collection, query).await?;
            results.push(result);
        }

        Ok(results)
    }

    async fn get(&self, collection: &str, point_id: &str) -> Result<Option<VectorPoint>> {
        debug!("Getting point '{}' from collection '{}'", point_id, collection);

        let id = Self::point_id_from_str(point_id);

        let result = self
            .client
            .get_points(
                GetPointsBuilder::new(collection, vec![id])
                    .with_payload(true)
                    .with_vectors(true),
            )
            .await?;

        Ok(result.result.first().map(|p: &qdrant_client::qdrant::RetrievedPoint| {
            Self::vector_point_from_retrieved_point(p.clone())
        }).transpose()?)
    }

    async fn get_batch(&self, collection: &str, point_ids: Vec<&str>) -> Result<Vec<Option<VectorPoint>>> {
        debug!("Getting {} points from collection '{}'", point_ids.len(), collection);

        let ids: Vec<PointId> = point_ids
            .iter()
            .map(|id| Self::point_id_from_str(id))
            .collect();

        let result = self
            .client
            .get_points(
                GetPointsBuilder::new(collection, ids)
                    .with_payload(true)
                    .with_vectors(true),
            )
            .await?;

        let mut points_map: HashMap<String, VectorPoint> = HashMap::new();
        for point in result.result {
            if let Ok(vp) = Self::vector_point_from_retrieved_point(point) {
                points_map.insert(vp.id.clone(), vp);
            }
        }

        Ok(point_ids
            .into_iter()
            .map(|id| points_map.get(id).cloned())
            .collect())
    }

    async fn count(&self, collection: &str) -> Result<u64> {
        let response = self.client.collection_info(collection).await?;
        Ok(response.result.and_then(|r| r.points_count).unwrap_or(0))
    }

    async fn set_payload(&self, collection: &str, point_ids: Vec<&str>, payload: Payload) -> Result<()> {
        debug!("Setting payload for {} points in collection '{}'", point_ids.len(), collection);

        let ids: Vec<PointId> = point_ids
            .iter()
            .map(|id| Self::point_id_from_str(id))
            .collect();

        let qdrant_payload = Self::payload_to_qdrant_payload(&Some(payload))?;

        self.client
            .set_payload(
                SetPayloadPointsBuilder::new(collection, qdrant_payload)
                    .points_selector(PointsIdsList { ids })
                    .wait(true),
            )
            .await?;

        Ok(())
    }

    async fn delete_payload(&self, collection: &str, point_ids: Vec<&str>, keys: Vec<&str>) -> Result<()> {
        debug!("Deleting payload keys {:?} for {} points in collection '{}'", keys, point_ids.len(), collection);

        let ids: Vec<PointId> = point_ids
            .iter()
            .map(|id| Self::point_id_from_str(id))
            .collect();

        let keys_owned: Vec<String> = keys.iter().map(|k| k.to_string()).collect();

        self.client
            .delete_payload(
                DeletePayloadPointsBuilder::new(collection, keys_owned)
                    .points_selector(PointsIdsList { ids })
                    .wait(true),
            )
            .await?;

        Ok(())
    }

    async fn scroll(
        &self,
        collection: &str,
        limit: usize,
        offset: Option<&str>,
        with_payload: Option<bool>,
        with_vector: Option<bool>,
    ) -> Result<(Vec<VectorPoint>, Option<String>)> {
        debug!("Scrolling collection '{}' with limit {}", collection, limit);

        let mut builder = ScrollPointsBuilder::new(collection)
            .limit(limit as u32)
            .with_payload(with_payload.unwrap_or(true))
            .with_vectors(with_vector.unwrap_or(false));

        if let Some(o) = offset {
            let offset_id = Self::point_id_from_str(o);
            builder = builder.offset(offset_id);
        }

        let result = self.client.scroll(builder).await?;

        let points: Result<Vec<VectorPoint>> = result
            .result
            .into_iter()
            .map(Self::vector_point_from_retrieved_point)
            .collect();

        let next_page = result.next_page_offset.map(|id| format!("{:?}", id));

        Ok((points?, next_page))
    }
}

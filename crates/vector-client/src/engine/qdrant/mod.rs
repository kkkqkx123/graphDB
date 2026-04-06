use async_trait::async_trait;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, PointId, PointStruct,
    SearchPointsBuilder, UpsertPointsBuilder, VectorParamsBuilder,
    GetPointsBuilder, ScrollPointsBuilder, DeletePointsBuilder,
    SetPayloadPointsBuilder, DeletePayloadPointsBuilder, PointsIdsList,
    CreateFieldIndexCollectionBuilder, DeleteFieldIndexCollectionBuilder,
};
use qdrant_client::Qdrant;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::config::{ConnectionConfig, VectorClientConfig};
use crate::error::{Result, VectorClientError};
use crate::types::*;
use super::VectorEngine;

mod config;
mod filter;
mod utils;

use config::{convert_distance, build_hnsw_config, build_quantization_config, convert_field_type};
use filter::convert_filter;
use utils::{
    point_id_from_str, point_struct_from_vector_point, payload_to_qdrant_payload,
    search_result_from_scored_point, vector_point_from_retrieved_point,
};

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

        let distance = convert_distance(config.distance);
        let mut vector_params = VectorParamsBuilder::new(config.vector_size as u64, distance);

        if let Some(hnsw_config) = build_hnsw_config(&config.hnsw_config) {
            vector_params = vector_params.hnsw_config(hnsw_config);
        }

        if let Some(quantization) = build_quantization_config(&config.quantization_config) {
            vector_params = vector_params.quantization_config(quantization);
        }

        if let Some(on_disk) = config.on_disk_payload {
            vector_params = vector_params.on_disk(on_disk);
        }

        let mut builder = CreateCollectionBuilder::new(name)
            .vectors_config(vector_params);

        if let Some(shard_number) = config.shard_number {
            builder = builder.shard_number(shard_number as u32);
        }

        if let Some(on_disk_payload) = config.on_disk_payload {
            builder = builder.on_disk_payload(on_disk_payload);
        }

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

        let point_struct = point_struct_from_vector_point(point)?;

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
            .map(point_struct_from_vector_point)
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

        let id = point_id_from_str(point_id);

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
            .map(|id| point_id_from_str(id))
            .collect();

        self.client
            .delete_points(DeletePointsBuilder::new(collection).points(ids))
            .await?;

        Ok(DeleteResult {
            operation_id: None,
            deleted_count: point_ids.len() as u64,
        })
    }

    async fn delete_by_filter(&self, collection: &str, filter: VectorFilter) -> Result<DeleteResult> {
        debug!("Deleting points by filter from collection '{}'", collection);

        let qdrant_filter = convert_filter(&filter)?;

        self.client
            .delete_points(
                DeletePointsBuilder::new(collection)
                    .points(qdrant_filter)
                    .wait(true),
            )
            .await?;

        Ok(DeleteResult {
            operation_id: None,
            deleted_count: 0,
        })
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
            .map(search_result_from_scored_point)
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

        let id = point_id_from_str(point_id);

        let result = self
            .client
            .get_points(
                GetPointsBuilder::new(collection, vec![id])
                    .with_payload(true)
                    .with_vectors(true),
            )
            .await?;

        Ok(result.result.first().map(|p: &qdrant_client::qdrant::RetrievedPoint| {
            vector_point_from_retrieved_point(p.clone())
        }).transpose()?)
    }

    async fn get_batch(&self, collection: &str, point_ids: Vec<&str>) -> Result<Vec<Option<VectorPoint>>> {
        debug!("Getting {} points from collection '{}'", point_ids.len(), collection);

        let ids: Vec<PointId> = point_ids
            .iter()
            .map(|id| point_id_from_str(id))
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
            if let Ok(vp) = vector_point_from_retrieved_point(point) {
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
            .map(|id| point_id_from_str(id))
            .collect();

        let qdrant_payload = payload_to_qdrant_payload(&Some(payload))?;

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
            .map(|id| point_id_from_str(id))
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
            let offset_id = point_id_from_str(o);
            builder = builder.offset(offset_id);
        }

        let result = self.client.scroll(builder).await?;

        let points: Result<Vec<VectorPoint>> = result
            .result
            .into_iter()
            .map(vector_point_from_retrieved_point)
            .collect();

        let next_page = result.next_page_offset.map(|id| format!("{:?}", id));

        Ok((points?, next_page))
    }

    async fn create_payload_index(
        &self,
        collection: &str,
        field: &str,
        schema: PayloadSchemaType,
    ) -> Result<()> {
        debug!("Creating payload index for field '{}' in collection '{}'", field, collection);

        let field_type = convert_field_type(schema);

        self.client
            .create_field_index(
                CreateFieldIndexCollectionBuilder::new(collection, field, field_type)
                    .wait(true),
            )
            .await?;

        info!("Payload index created for field '{}' in collection '{}'", field, collection);
        Ok(())
    }

    async fn delete_payload_index(&self, collection: &str, field: &str) -> Result<()> {
        debug!("Deleting payload index for field '{}' in collection '{}'", field, collection);

        self.client
            .delete_field_index(
                DeleteFieldIndexCollectionBuilder::new(collection, field)
                    .wait(true),
            )
            .await?;

        info!("Payload index deleted for field '{}' in collection '{}'", field, collection);
        Ok(())
    }

    async fn list_payload_indexes(&self, _collection: &str) -> Result<Vec<(String, PayloadSchemaType)>> {
        debug!("Listing payload indexes for collection '{}'", _collection);

        Ok(Vec::new())
    }
}

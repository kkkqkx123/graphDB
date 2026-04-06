use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

use crate::error::{Result, VectorClientError};
use crate::types::*;
use super::VectorEngine;

const MOCK_VERSION: &str = "0.1.0-mock";

type CollectionStore = HashMap<String, VectorPoint>;

#[derive(Debug)]
pub struct MockEngine {
    collections: Arc<RwLock<HashMap<String, (CollectionConfig, CollectionStore)>>>,
    healthy: Arc<RwLock<bool>>,
}

impl MockEngine {
    pub fn new() -> Self {
        Self {
            collections: Arc::new(RwLock::new(HashMap::new())),
            healthy: Arc::new(RwLock::new(true)),
        }
    }

    pub fn with_collections(collections: HashMap<String, CollectionConfig>) -> Self {
        let data: HashMap<String, (CollectionConfig, CollectionStore)> = collections
            .into_iter()
            .map(|(name, config)| (name, (config, HashMap::new())))
            .collect();

        Self {
            collections: Arc::new(RwLock::new(data)),
            healthy: Arc::new(RwLock::new(true)),
        }
    }

    pub async fn set_healthy(&self, healthy: bool) {
        *self.healthy.write().await = healthy;
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot / (norm_a * norm_b)
    }
}

impl Default for MockEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VectorEngine for MockEngine {
    fn name(&self) -> &str {
        "mock"
    }

    fn version(&self) -> &str {
        MOCK_VERSION
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let healthy = *self.healthy.read().await;

        if healthy {
            Ok(HealthStatus::healthy(self.name(), self.version()))
        } else {
            Ok(HealthStatus::unhealthy(
                self.name(),
                self.version(),
                "Mock engine is set to unhealthy",
            ))
        }
    }

    async fn create_collection(&self, name: &str, config: CollectionConfig) -> Result<()> {
        debug!("Mock: Creating collection '{}'", name);

        let mut collections = self.collections.write().await;

        if collections.contains_key(name) {
            return Err(VectorClientError::CollectionAlreadyExists(name.to_string()));
        }

        collections.insert(name.to_string(), (config, HashMap::new()));

        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        debug!("Mock: Deleting collection '{}'", name);

        let mut collections = self.collections.write().await;

        if collections.remove(name).is_none() {
            return Err(VectorClientError::CollectionNotFound(name.to_string()));
        }

        Ok(())
    }

    async fn collection_exists(&self, name: &str) -> Result<bool> {
        let collections = self.collections.read().await;
        Ok(collections.contains_key(name))
    }

    async fn collection_info(&self, name: &str) -> Result<CollectionInfo> {
        let collections = self.collections.read().await;

        let (config, store) = collections
            .get(name)
            .ok_or_else(|| VectorClientError::CollectionNotFound(name.to_string()))?;

        Ok(CollectionInfo {
            name: name.to_string(),
            vector_count: store.len() as u64,
            indexed_vector_count: store.len() as u64,
            points_count: store.len() as u64,
            segments_count: 1,
            config: config.clone(),
            status: CollectionStatus::Green,
        })
    }

    async fn upsert(&self, collection: &str, point: VectorPoint) -> Result<UpsertResult> {
        debug!("Mock: Upserting point '{}' to collection '{}'", point.id, collection);

        let mut collections = self.collections.write().await;

        let (_, store) = collections
            .get_mut(collection)
            .ok_or_else(|| VectorClientError::CollectionNotFound(collection.to_string()))?;

        store.insert(point.id.clone(), point);

        Ok(UpsertResult {
            operation_id: Some(1),
            status: UpsertStatus::Completed,
        })
    }

    async fn upsert_batch(&self, collection: &str, points: Vec<VectorPoint>) -> Result<UpsertResult> {
        debug!("Mock: Upserting {} points to collection '{}'", points.len(), collection);

        let mut collections = self.collections.write().await;

        let (_, store) = collections
            .get_mut(collection)
            .ok_or_else(|| VectorClientError::CollectionNotFound(collection.to_string()))?;

        for point in points {
            store.insert(point.id.clone(), point);
        }

        Ok(UpsertResult {
            operation_id: Some(1),
            status: UpsertStatus::Completed,
        })
    }

    async fn delete(&self, collection: &str, point_id: &str) -> Result<DeleteResult> {
        debug!("Mock: Deleting point '{}' from collection '{}'", point_id, collection);

        let mut collections = self.collections.write().await;

        let (_, store) = collections
            .get_mut(collection)
            .ok_or_else(|| VectorClientError::CollectionNotFound(collection.to_string()))?;

        let existed = store.remove(point_id).is_some();

        Ok(DeleteResult {
            operation_id: Some(1),
            deleted_count: if existed { 1 } else { 0 },
        })
    }

    async fn delete_batch(&self, collection: &str, point_ids: Vec<&str>) -> Result<DeleteResult> {
        debug!("Mock: Deleting {} points from collection '{}'", point_ids.len(), collection);

        let mut collections = self.collections.write().await;

        let (_, store) = collections
            .get_mut(collection)
            .ok_or_else(|| VectorClientError::CollectionNotFound(collection.to_string()))?;

        let mut deleted_count = 0;
        for id in point_ids {
            if store.remove(id).is_some() {
                deleted_count += 1;
            }
        }

        Ok(DeleteResult {
            operation_id: Some(1),
            deleted_count,
        })
    }

    async fn delete_by_filter(&self, collection: &str, _filter: VectorFilter) -> Result<DeleteResult> {
        debug!("Mock: Deleting points by filter from collection '{}'", collection);

        Err(VectorClientError::FilterError(
            "Filter-based deletion not yet implemented in mock".to_string(),
        ))
    }

    async fn search(&self, collection: &str, query: SearchQuery) -> Result<Vec<SearchResult>> {
        debug!("Mock: Searching in collection '{}' with limit {}", collection, query.limit);

        let collections = self.collections.read().await;

        let (_, store) = collections
            .get(collection)
            .ok_or_else(|| VectorClientError::CollectionNotFound(collection.to_string()))?;

        let mut results: Vec<SearchResult> = store
            .values()
            .map(|point| {
                let score = Self::cosine_similarity(&query.vector, &point.vector);
                SearchResult {
                    id: point.id.clone(),
                    score,
                    payload: point.payload.clone(),
                    vector: if query.with_vector.unwrap_or(false) {
                        Some(point.vector.clone())
                    } else {
                        None
                    },
                }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        if let Some(threshold) = query.score_threshold {
            results.retain(|r| r.score >= threshold);
        }

        let offset = query.offset.unwrap_or(0);
        let limit = query.limit;

        results = results.into_iter().skip(offset).take(limit).collect();

        Ok(results)
    }

    async fn search_batch(
        &self,
        collection: &str,
        queries: Vec<SearchQuery>,
    ) -> Result<Vec<Vec<SearchResult>>> {
        debug!("Mock: Batch searching {} queries in collection '{}'", queries.len(), collection);

        let mut results = Vec::with_capacity(queries.len());

        for query in queries {
            let result = self.search(collection, query).await?;
            results.push(result);
        }

        Ok(results)
    }

    async fn get(&self, collection: &str, point_id: &str) -> Result<Option<VectorPoint>> {
        debug!("Mock: Getting point '{}' from collection '{}'", point_id, collection);

        let collections = self.collections.read().await;

        let (_, store) = collections
            .get(collection)
            .ok_or_else(|| VectorClientError::CollectionNotFound(collection.to_string()))?;

        Ok(store.get(point_id).cloned())
    }

    async fn get_batch(&self, collection: &str, point_ids: Vec<&str>) -> Result<Vec<Option<VectorPoint>>> {
        debug!("Mock: Getting {} points from collection '{}'", point_ids.len(), collection);

        let collections = self.collections.read().await;

        let (_, store) = collections
            .get(collection)
            .ok_or_else(|| VectorClientError::CollectionNotFound(collection.to_string()))?;

        Ok(point_ids.into_iter().map(|id| store.get(id).cloned()).collect())
    }

    async fn count(&self, collection: &str) -> Result<u64> {
        let collections = self.collections.read().await;

        let (_, store) = collections
            .get(collection)
            .ok_or_else(|| VectorClientError::CollectionNotFound(collection.to_string()))?;

        Ok(store.len() as u64)
    }

    async fn set_payload(&self, collection: &str, point_ids: Vec<&str>, payload: Payload) -> Result<()> {
        debug!("Mock: Setting payload for {} points in collection '{}'", point_ids.len(), collection);

        let mut collections = self.collections.write().await;

        let (_, store) = collections
            .get_mut(collection)
            .ok_or_else(|| VectorClientError::CollectionNotFound(collection.to_string()))?;

        for id in point_ids {
            if let Some(point) = store.get_mut(id) {
                let point_payload = point.payload.get_or_insert_with(HashMap::new);
                point_payload.extend(payload.clone());
            }
        }

        Ok(())
    }

    async fn delete_payload(&self, collection: &str, point_ids: Vec<&str>, keys: Vec<&str>) -> Result<()> {
        debug!("Mock: Deleting payload keys {:?} for {} points in collection '{}'", keys, point_ids.len(), collection);

        let mut collections = self.collections.write().await;

        let (_, store) = collections
            .get_mut(collection)
            .ok_or_else(|| VectorClientError::CollectionNotFound(collection.to_string()))?;

        for id in point_ids {
            if let Some(point) = store.get_mut(id) {
                if let Some(ref mut payload) = point.payload {
                    for key in &keys {
                        payload.remove(*key);
                    }
                }
            }
        }

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
        debug!("Mock: Scrolling collection '{}' with limit {}", collection, limit);

        let collections = self.collections.read().await;

        let (_, store) = collections
            .get(collection)
            .ok_or_else(|| VectorClientError::CollectionNotFound(collection.to_string()))?;

        let mut points: Vec<VectorPoint> = store.values().cloned().collect();

        points.sort_by(|a, b| a.id.cmp(&b.id));

        let skip = if let Some(offset_id) = offset {
            points.iter().position(|p| p.id == offset_id).map(|i| i + 1).unwrap_or(0)
        } else {
            0
        };

        let result: Vec<VectorPoint> = points
            .into_iter()
            .skip(skip)
            .take(limit)
            .map(|mut p| {
                if !with_payload.unwrap_or(true) {
                    p.payload = None;
                }
                if !with_vector.unwrap_or(false) {
                    p.vector = Vec::new();
                }
                p
            })
            .collect();

        let next_page = if result.len() == limit {
            result.last().map(|p| p.id.clone())
        } else {
            None
        };

        Ok((result, next_page))
    }

    async fn create_payload_index(
        &self,
        collection: &str,
        field: &str,
        _schema: PayloadSchemaType,
    ) -> Result<()> {
        debug!("Mock: Creating payload index for field '{}' in collection '{}'", field, collection);

        let collections = self.collections.read().await;

        if !collections.contains_key(collection) {
            return Err(VectorClientError::CollectionNotFound(collection.to_string()));
        }

        Ok(())
    }

    async fn delete_payload_index(&self, collection: &str, field: &str) -> Result<()> {
        debug!("Mock: Deleting payload index for field '{}' in collection '{}'", field, collection);

        let collections = self.collections.read().await;

        if !collections.contains_key(collection) {
            return Err(VectorClientError::CollectionNotFound(collection.to_string()));
        }

        Ok(())
    }

    async fn list_payload_indexes(&self, collection: &str) -> Result<Vec<(String, PayloadSchemaType)>> {
        debug!("Mock: Listing payload indexes for collection '{}'", collection);

        let collections = self.collections.read().await;

        if !collections.contains_key(collection) {
            return Err(VectorClientError::CollectionNotFound(collection.to_string()));
        }

        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_delete_collection() {
        let engine = MockEngine::new();

        engine
            .create_collection("test", CollectionConfig::default())
            .await
            .unwrap();

        assert!(engine.collection_exists("test").await.unwrap());

        engine.delete_collection("test").await.unwrap();

        assert!(!engine.collection_exists("test").await.unwrap());
    }

    #[tokio::test]
    async fn test_upsert_and_search() {
        let engine = MockEngine::new();

        engine
            .create_collection("test", CollectionConfig::new(3, DistanceMetric::Cosine))
            .await
            .unwrap();

        let point1 = VectorPoint::new("p1", vec![1.0, 0.0, 0.0]);
        let point2 = VectorPoint::new("p2", vec![0.0, 1.0, 0.0]);

        engine.upsert_batch("test", vec![point1, point2]).await.unwrap();

        let query = SearchQuery::new(vec![1.0, 0.0, 0.0], 10);
        let results = engine.search("test", query).await.unwrap();

        assert_eq!(results.len(), 2);
        assert!(results[0].score > results[1].score);
        assert_eq!(results[0].id, "p1");
    }
}

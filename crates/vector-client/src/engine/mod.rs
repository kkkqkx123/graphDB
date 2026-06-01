use async_trait::async_trait;

use crate::error::{Result, VectorClientError};
use crate::types::*;

pub mod common;

#[cfg(feature = "qdrant-http")]
mod http;

#[cfg(feature = "qdrant-http")]
pub use http::QdrantEngine;

#[cfg(feature = "qdrant-grpc")]
pub mod grpc;

#[cfg(feature = "qdrant-grpc")]
pub use grpc::QdrantGrpcEngine;

#[async_trait]
pub trait VectorEngine: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    fn version(&self) -> &str;

    async fn health_check(&self) -> Result<HealthStatus>;

    async fn create_collection(&self, name: &str, config: CollectionConfig) -> Result<()>;
    async fn delete_collection(&self, name: &str) -> Result<()>;
    async fn collection_exists(&self, name: &str) -> Result<bool>;
    async fn collection_info(&self, name: &str) -> Result<CollectionInfo>;

    async fn upsert(&self, collection: &str, point: VectorPoint) -> Result<UpsertResult>;
    async fn upsert_batch(
        &self,
        collection: &str,
        points: Vec<VectorPoint>,
    ) -> Result<UpsertResult>;

    async fn delete(&self, collection: &str, point_id: &str) -> Result<DeleteResult>;
    async fn delete_batch(&self, collection: &str, point_ids: Vec<&str>) -> Result<DeleteResult>;

    async fn delete_by_filter(
        &self,
        collection: &str,
        filter: VectorFilter,
    ) -> Result<DeleteResult> {
        let _ = (collection, filter);
        Err(VectorClientError::NotSupported(
            "delete_by_filter".to_string(),
        ))
    }

    async fn search(&self, collection: &str, query: SearchQuery) -> Result<Vec<SearchResult>>;

    async fn search_batch(
        &self,
        collection: &str,
        queries: Vec<SearchQuery>,
    ) -> Result<Vec<Vec<SearchResult>>> {
        let mut results = Vec::with_capacity(queries.len());
        for query in queries {
            results.push(self.search(collection, query).await?);
        }
        Ok(results)
    }

    async fn get(&self, collection: &str, point_id: &str) -> Result<Option<VectorPoint>>;
    async fn get_batch(
        &self,
        collection: &str,
        point_ids: Vec<&str>,
    ) -> Result<Vec<Option<VectorPoint>>>;
    async fn count(&self, collection: &str) -> Result<u64>;

    async fn set_payload(
        &self,
        collection: &str,
        point_ids: Vec<&str>,
        payload: Payload,
    ) -> Result<()> {
        let _ = (collection, point_ids, payload);
        Err(VectorClientError::NotSupported("set_payload".to_string()))
    }

    async fn delete_payload(
        &self,
        collection: &str,
        point_ids: Vec<&str>,
        keys: Vec<&str>,
    ) -> Result<()> {
        let _ = (collection, point_ids, keys);
        Err(VectorClientError::NotSupported(
            "delete_payload".to_string(),
        ))
    }

    async fn scroll(
        &self,
        collection: &str,
        limit: usize,
        offset: Option<&str>,
        with_payload: Option<bool>,
        with_vector: Option<bool>,
    ) -> Result<(Vec<VectorPoint>, Option<String>)> {
        let _ = (collection, limit, offset, with_payload, with_vector);
        Err(VectorClientError::NotSupported("scroll".to_string()))
    }

    async fn create_payload_index(
        &self,
        collection: &str,
        field: &str,
        schema: PayloadSchemaType,
    ) -> Result<()> {
        let _ = (collection, field, schema);
        Err(VectorClientError::NotSupported(
            "create_payload_index".to_string(),
        ))
    }

    async fn delete_payload_index(&self, collection: &str, field: &str) -> Result<()> {
        let _ = (collection, field);
        Err(VectorClientError::NotSupported(
            "delete_payload_index".to_string(),
        ))
    }

    async fn list_payload_indexes(
        &self,
        collection: &str,
    ) -> Result<Vec<(String, PayloadSchemaType)>> {
        let _ = collection;
        Err(VectorClientError::NotSupported(
            "list_payload_indexes".to_string(),
        ))
    }
}

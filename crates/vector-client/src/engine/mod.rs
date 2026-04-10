use async_trait::async_trait;

use crate::error::Result;
use crate::types::*;

#[cfg(feature = "qdrant")]
mod qdrant;

#[cfg(feature = "qdrant")]
pub use qdrant::QdrantEngine;

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
    async fn upsert_batch(&self, collection: &str, points: Vec<VectorPoint>) -> Result<UpsertResult>;

    async fn delete(&self, collection: &str, point_id: &str) -> Result<DeleteResult>;
    async fn delete_batch(&self, collection: &str, point_ids: Vec<&str>) -> Result<DeleteResult>;
    async fn delete_by_filter(&self, collection: &str, filter: VectorFilter) -> Result<DeleteResult>;

    async fn search(&self, collection: &str, query: SearchQuery) -> Result<Vec<SearchResult>>;
    async fn search_batch(&self, collection: &str, queries: Vec<SearchQuery>) -> Result<Vec<Vec<SearchResult>>>;

    async fn get(&self, collection: &str, point_id: &str) -> Result<Option<VectorPoint>>;
    async fn get_batch(&self, collection: &str, point_ids: Vec<&str>) -> Result<Vec<Option<VectorPoint>>>;
    async fn count(&self, collection: &str) -> Result<u64>;

    async fn set_payload(&self, collection: &str, point_ids: Vec<&str>, payload: Payload) -> Result<()>;
    async fn delete_payload(&self, collection: &str, point_ids: Vec<&str>, keys: Vec<&str>) -> Result<()>;

    async fn scroll(
        &self,
        collection: &str,
        limit: usize,
        offset: Option<&str>,
        with_payload: Option<bool>,
        with_vector: Option<bool>,
    ) -> Result<(Vec<VectorPoint>, Option<String>)>;

    async fn create_payload_index(
        &self,
        collection: &str,
        field: &str,
        schema: PayloadSchemaType,
    ) -> Result<()>;

    async fn delete_payload_index(&self, collection: &str, field: &str) -> Result<()>;

    async fn list_payload_indexes(&self, collection: &str) -> Result<Vec<(String, PayloadSchemaType)>>;
}

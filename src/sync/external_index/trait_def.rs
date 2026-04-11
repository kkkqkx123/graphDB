use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::error::IndexResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexData {
    Fulltext(String),
    Vector(Vec<f32>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    pub space_id: u64,
    pub tag_name: String,
    pub field_name: String,
    pub options: IndexOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IndexOptions {
    pub vector_size: Option<usize>,
    pub distance_metric: Option<String>,
    pub analyzer: Option<String>,
}

#[derive(Debug, Clone)]
pub enum IndexOperation {
    Insert {
        id: String,
        data: IndexData,
        payload: std::collections::HashMap<String, serde_json::Value>,
    },
    Delete {
        id: String,
    },
    Update {
        id: String,
        data: IndexData,
        payload: std::collections::HashMap<String, serde_json::Value>,
    },
}

#[async_trait]
pub trait ExternalIndexClient: Send + Sync + std::fmt::Debug {
    fn client_type(&self) -> &'static str;

    fn index_key(&self) -> (u64, String, String);

    async fn insert(&self, id: &str, data: &IndexData) -> IndexResult<()>;

    async fn insert_batch(&self, items: Vec<(String, IndexData)>) -> IndexResult<()>;

    async fn delete(&self, id: &str) -> IndexResult<()>;

    async fn delete_batch(&self, ids: &[&str]) -> IndexResult<()>;

    async fn commit(&self) -> IndexResult<()>;

    async fn rollback(&self) -> IndexResult<()>;

    async fn stats(&self) -> IndexResult<IndexStats>;

    fn as_any(&self) -> &dyn std::any::Any;
}

#[derive(Debug, Clone, Default)]
pub struct IndexStats {
    pub doc_count: usize,
    pub index_size_bytes: usize,
    pub last_commit_time: Option<std::time::Instant>,
}

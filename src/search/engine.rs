use crate::search::error::SearchError;
use crate::search::result::{IndexStats, SearchResult};
use async_trait::async_trait;

/// Consistency state of a search engine index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ConsistencyState {
    /// Index is consistent with the main storage.
    Consistent,
    /// Index may be inconsistent due to partial commit failure.
    /// Automatic repair should be scheduled.
    Inconsistent,
    /// Rebuild is in progress.
    Rebuilding,
}

#[async_trait]
pub trait SearchEngine: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;

    fn version(&self) -> &str;

    fn is_metrics_wrapped(&self) -> bool {
        false
    }

    async fn index(&self, doc_id: &str, content: &str) -> Result<(), SearchError>;

    async fn index_batch(&self, docs: Vec<(String, String)>) -> Result<(), SearchError>;

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError>;

    async fn delete(&self, doc_id: &str) -> Result<(), SearchError>;

    async fn delete_batch(&self, doc_ids: Vec<&str>) -> Result<(), SearchError>;

    async fn commit(&self) -> Result<(), SearchError>;

    async fn rollback(&self) -> Result<(), SearchError>;

    async fn stats(&self) -> Result<IndexStats, SearchError>;

    async fn close(&self) -> Result<(), SearchError>;

    /// Return the current consistency state of this index.
    fn consistency_state(&self) -> ConsistencyState;

    /// Mark this index as potentially inconsistent.
    fn mark_inconsistent(&self);

    /// Reset consistency state to consistent.
    fn mark_consistent(&self);

    /// Delete all documents from the index.
    async fn clear(&self) -> Result<(), SearchError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum EngineType {
    Bm25,
}

impl std::fmt::Display for EngineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineType::Bm25 => write!(f, "bm25"),
        }
    }
}

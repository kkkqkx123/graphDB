use async_trait::async_trait;
use crate::search::result::{SearchResult, IndexStats};
use crate::search::error::SearchError;

#[async_trait]
pub trait SearchEngine: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    
    fn version(&self) -> &str;
    
    async fn index(&self, doc_id: &str, content: &str) -> Result<(), SearchError>;
    
    async fn index_batch(&self, docs: Vec<(String, String)>) -> Result<(), SearchError>;
    
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError>;
    
    async fn delete(&self, doc_id: &str) -> Result<(), SearchError>;
    
    async fn delete_batch(&self, doc_ids: Vec<&str>) -> Result<(), SearchError>;
    
    async fn commit(&self) -> Result<(), SearchError>;
    
    async fn rollback(&self) -> Result<(), SearchError>;
    
    async fn stats(&self) -> Result<IndexStats, SearchError>;
    
    async fn close(&self) -> Result<(), SearchError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum EngineType {
    Bm25,
    Inversearch,
}

impl std::fmt::Display for EngineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineType::Bm25 => write!(f, "bm25"),
            EngineType::Inversearch => write!(f, "inversearch"),
        }
    }
}

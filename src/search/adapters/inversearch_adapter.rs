use async_trait::async_trait;
use inversearch_service::Index;
use inversearch_service::index::IndexOptions;
use inversearch_service::search::search;
use inversearch_service::r#type::SearchOptions;
use std::path::{Path, PathBuf};
use parking_lot::Mutex;

use crate::core::Value;
use crate::search::engine::SearchEngine;
use crate::search::result::{IndexStats, SearchResult};
use crate::search::error::SearchError;

#[derive(Debug, Clone)]
pub struct InversearchConfig {
    pub tokenize_mode: String,
    pub resolution: usize,
    pub cache_size: Option<usize>,
    pub persistence_path: Option<PathBuf>,
}

impl Default for InversearchConfig {
    fn default() -> Self {
        Self {
            tokenize_mode: "strict".to_string(),
            resolution: 9,
            cache_size: Some(1000),
            persistence_path: None,
        }
    }
}

pub struct InversearchEngine {
    index: Mutex<Index>,
    config: InversearchConfig,
}

impl std::fmt::Debug for InversearchEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InversearchEngine")
            .field("config", &self.config)
            .finish()
    }
}

impl InversearchEngine {
    pub fn new(config: InversearchConfig) -> Result<Self, SearchError> {
        let tokenize_mode: &'static str = match config.tokenize_mode.as_str() {
            "strict" => "strict",
            "forward" => "forward",
            "reverse" => "reverse",
            "full" => "full",
            _ => "strict",
        };

        let options = IndexOptions {
            resolution: Some(config.resolution),
            tokenize_mode: Some(tokenize_mode),
            cache_size: config.cache_size,
            ..Default::default()
        };

        let index = Index::new(options)
            .map_err(|e| SearchError::InversearchError(e.to_string()))?;

        Ok(Self {
            index: Mutex::new(index),
            config,
        })
    }

    pub fn load(_path: &Path, config: InversearchConfig) -> Result<Self, SearchError> {
        Self::new(config)
    }

    fn persist(&self) -> Result<(), SearchError> {
        Ok(())
    }

    fn parse_doc_id(&self, doc_id: &str) -> Result<u64, SearchError> {
        doc_id.parse::<u64>()
            .map_err(|_| SearchError::InvalidDocId(doc_id.to_string()))
    }
}

#[async_trait]
impl SearchEngine for InversearchEngine {
    fn name(&self) -> &str {
        "inversearch"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    async fn index(&self, doc_id: &str, content: &str) -> Result<(), SearchError> {
        let id = self.parse_doc_id(doc_id)?;
        let mut index = self.index.lock();

        index.add(id, content, false)
            .map_err(|e| SearchError::InversearchError(e.to_string()))?;

        Ok(())
    }

    async fn index_batch(&self, docs: Vec<(String, String)>) -> Result<(), SearchError> {
        let mut index = self.index.lock();

        for (doc_id, content) in docs {
            let id = self.parse_doc_id(&doc_id)?;
            index.add(id, &content, false)
                .map_err(|e| SearchError::InversearchError(e.to_string()))?;
        }

        Ok(())
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError> {
        let index = self.index.lock();

        let options = SearchOptions {
            query: Some(query.to_string()),
            limit: Some(limit),
            ..Default::default()
        };

        let results = search(&index, &options)
            .map_err(|e| SearchError::InversearchError(e.to_string()))?;

        Ok(results.results.into_iter().map(|doc_id| SearchResult {
            doc_id: Value::from(doc_id.to_string()),
            score: 1.0,
            highlights: None,
            matched_fields: vec!["content".to_string()],
        }).collect())
    }

    async fn delete(&self, doc_id: &str) -> Result<(), SearchError> {
        let id = self.parse_doc_id(doc_id)?;
        let mut index = self.index.lock();

        index.remove(id, false)
            .map_err(|e| SearchError::InversearchError(e.to_string()))?;

        Ok(())
    }

    async fn delete_batch(&self, doc_ids: Vec<&str>) -> Result<(), SearchError> {
        let mut index = self.index.lock();

        for doc_id in doc_ids {
            let id = self.parse_doc_id(doc_id)?;
            index.remove(id, false)
                .map_err(|e| SearchError::InversearchError(e.to_string()))?;
        }

        Ok(())
    }

    async fn commit(&self) -> Result<(), SearchError> {
        self.persist()
    }

    async fn rollback(&self) -> Result<(), SearchError> {
        Ok(())
    }

    async fn stats(&self) -> Result<IndexStats, SearchError> {
        let index = self.index.lock();

        Ok(IndexStats {
            doc_count: index.document_count(),
            index_size: 0,
            last_updated: None,
            engine_info: None,
        })
    }

    async fn close(&self) -> Result<(), SearchError> {
        self.commit().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inversearch_engine_creation() {
        let config = InversearchConfig::default();
        let engine = InversearchEngine::new(config);
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_inversearch_index_and_search() {
        let config = InversearchConfig::default();
        let engine = InversearchEngine::new(config)
            .expect("Failed to create engine");

        engine.index("1", "Hello world from Rust").await.expect("Failed to index");
        engine.index("2", "Hello GraphDB").await.expect("Failed to index");
        engine.index("3", "Rust programming language").await.expect("Failed to index");

        let results = engine.search("Hello", 10).await.expect("Failed to search");
        assert!(!results.is_empty(), "Should find results for 'Hello'");
    }

    #[tokio::test]
    async fn test_inversearch_delete() {
        let config = InversearchConfig::default();
        let engine = InversearchEngine::new(config)
            .expect("Failed to create engine");

        engine.index("1", "Test document").await.expect("Failed to index");
        engine.delete("1").await.expect("Failed to delete");

        let results = engine.search("Test", 10).await.expect("Failed to search");
        assert!(results.is_empty() || !results.iter().any(|r| matches!(&r.doc_id, Value::String(s) if s == "1")));
    }
}

use async_trait::async_trait;
use inversearch_service::Index;
use inversearch_service::index::IndexOptions;
use std::path::{Path, PathBuf};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use crate::core::Value;
use crate::search::engine::SearchEngine;
use crate::search::result::{IndexStats, SearchResult};
use crate::search::error::SearchError;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        let options = IndexOptions {
            resolution: Some(config.resolution),
            depth: Some(3),
            bidirectional: Some(true),
            fastupdate: Some(true),
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
        let options = IndexOptions {
            resolution: Some(config.resolution),
            depth: Some(3),
            bidirectional: Some(true),
            fastupdate: Some(true),
            ..Default::default()
        };

        let index = Index::new(options)
            .map_err(|e| SearchError::InversearchError(e.to_string()))?;

        Ok(Self {
            index: Mutex::new(index),
            config,
        })
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
        let mut index = self.index.lock();
        let doc_id_u64 = doc_id.parse::<u64>()
            .map_err(|_| SearchError::InvalidDocId(doc_id.to_string()))?;
        index.add(doc_id_u64, content, false)
            .map_err(|e| SearchError::InversearchError(e.to_string()))?;
        Ok(())
    }

    async fn index_batch(&self, documents: Vec<(String, String)>) -> Result<(), SearchError> {
        let mut index = self.index.lock();
        for (doc_id, content) in documents {
            let doc_id_u64 = doc_id.parse::<u64>()
                .map_err(|_| SearchError::InvalidDocId(doc_id.clone()))?;
            index.add(doc_id_u64, &content, false)
                .map_err(|e| SearchError::InversearchError(e.to_string()))?;
        }
        Ok(())
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError> {
        let index = self.index.lock();
        let options = inversearch_service::r#type::SearchOptions {
            query: Some(query.to_string()),
            limit: Some(limit),
            offset: Some(0),
            resolve: Some(true),
            ..Default::default()
        };

        let result = index.search(&options)
            .map_err(|e| SearchError::InversearchError(e.to_string()))?;

        let search_results = result.results.into_iter()
            .map(|r| SearchResult {
                doc_id: Value::Int64(r as i64),
                score: 1.0,
                highlights: None,
                matched_fields: vec![],
            })
            .collect();

        Ok(search_results)
    }

    async fn delete(&self, doc_id: &str) -> Result<(), SearchError> {
        let mut index = self.index.lock();
        let doc_id_u64 = doc_id.parse::<u64>()
            .map_err(|_| SearchError::InvalidDocId(doc_id.to_string()))?;
        index.remove(doc_id_u64, false)
            .map_err(|e| SearchError::InversearchError(e.to_string()))?;
        Ok(())
    }

    async fn delete_batch(&self, doc_ids: Vec<&str>) -> Result<(), SearchError> {
        for doc_id in doc_ids {
            self.delete(doc_id).await?;
        }
        Ok(())
    }

    async fn commit(&self) -> Result<(), SearchError> {
        Ok(())
    }

    async fn rollback(&self) -> Result<(), SearchError> {
        Ok(())
    }

    async fn stats(&self) -> Result<IndexStats, SearchError> {
        let index = self.index.lock();
        let doc_count = index.map.index.values()
            .map(|v| v.values())
            .flatten()
            .map(|v| v.len())
            .sum();

        Ok(IndexStats {
            doc_count,
            index_size: 0,
            last_updated: None,
            engine_info: None,
        })
    }

    async fn close(&self) -> Result<(), SearchError> {
        Ok(())
    }
}

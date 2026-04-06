use async_trait::async_trait;
use inversearch_service::api::embedded::EmbeddedIndex;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::core::Value;
use crate::search::engine::SearchEngine;
use crate::search::error::SearchError;
use crate::search::result::{IndexStats, SearchResult};

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
    index: Mutex<EmbeddedIndex>,
}

impl std::fmt::Debug for InversearchEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InversearchEngine")
            .finish()
    }
}

impl InversearchEngine {
    pub fn new(config: InversearchConfig) -> Result<Self, SearchError> {
        let mut index = EmbeddedIndex::create()
            .map_err(|e| SearchError::InversearchError(e.to_string()))?;

        if let Some(path) = config.persistence_path {
            if path.exists() {
                index
                    .load_from(path)
                    .map_err(|e| SearchError::InversearchError(e.to_string()))?;
            }
        }

        Ok(Self {
            index: Mutex::new(index),
        })
    }

    pub fn load(path: &Path, config: InversearchConfig) -> Result<Self, SearchError> {
        let mut index = EmbeddedIndex::create()
            .map_err(|e| SearchError::InversearchError(e.to_string()))?;

        if path.exists() {
            index
                .load_from(path)
                .map_err(|e| SearchError::InversearchError(e.to_string()))?;
        }

        if let Some(persistence_path) = config.persistence_path {
            if persistence_path.exists() && persistence_path != path {
                index
                    .load_from(persistence_path)
                    .map_err(|e| SearchError::InversearchError(e.to_string()))?;
            }
        }

        Ok(Self {
            index: Mutex::new(index),
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
        let doc_id_u64 = doc_id
            .parse::<u64>()
            .map_err(|_| SearchError::InvalidDocId(doc_id.to_string()))?;
        let mut index = self.index.lock();
        index
            .add(doc_id_u64, content)
            .map_err(|e| SearchError::InversearchError(e.to_string()))?;
        Ok(())
    }

    async fn index_batch(&self, documents: Vec<(String, String)>) -> Result<(), SearchError> {
        let mut index = self.index.lock();
        for (doc_id, content) in documents {
            let doc_id_u64 = doc_id
                .parse::<u64>()
                .map_err(|_| SearchError::InvalidDocId(doc_id.clone()))?;
            index
                .add(doc_id_u64, &content)
                .map_err(|e| SearchError::InversearchError(e.to_string()))?;
        }
        Ok(())
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError> {
        let index = self.index.lock();
        let results = index
            .search(query)
            .map_err(|e| SearchError::InversearchError(e.to_string()))?;

        let search_results = results
            .into_iter()
            .take(limit)
            .map(|r| SearchResult {
                doc_id: Value::Int64(r.id as i64),
                score: r.score,
                highlights: None,
                matched_fields: vec![],
            })
            .collect();

        Ok(search_results)
    }

    async fn delete(&self, doc_id: &str) -> Result<(), SearchError> {
        let doc_id_u64 = doc_id
            .parse::<u64>()
            .map_err(|_| SearchError::InvalidDocId(doc_id.to_string()))?;
        let mut index = self.index.lock();
        index
            .remove(doc_id_u64)
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
        let index = self.index.lock();
        index
            .save()
            .map_err(|e| SearchError::InversearchError(e.to_string()))?;
        Ok(())
    }

    async fn rollback(&self) -> Result<(), SearchError> {
        Ok(())
    }

    async fn stats(&self) -> Result<IndexStats, SearchError> {
        let index = self.index.lock();
        let stats = index.stats();
        let index_size = 0;

        Ok(IndexStats {
            doc_count: stats.document_count,
            index_size,
            last_updated: None,
            engine_info: None,
        })
    }

    async fn close(&self) -> Result<(), SearchError> {
        self.commit().await?;
        Ok(())
    }
}

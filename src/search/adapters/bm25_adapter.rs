use async_trait::async_trait;
use bm25_service::index::{IndexManager, IndexSchema};
use bm25_service::index::document::add_document;
use bm25_service::index::delete::delete_document;
use bm25_service::index::search::{search, SearchOptions};
use bm25_service::index::stats::get_stats;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::core::Value;
use crate::search::engine::SearchEngine;
use crate::search::result::{IndexStats, SearchResult};
use crate::search::error::SearchError;

pub struct Bm25SearchEngine {
    manager: Arc<IndexManager>,
    schema: IndexSchema,
    index_path: std::path::PathBuf,
}

impl std::fmt::Debug for Bm25SearchEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bm25SearchEngine")
            .field("index_path", &self.index_path)
            .field("schema", &self.schema)
            .finish()
    }
}

impl Bm25SearchEngine {
    pub fn open_or_create(path: &Path) -> Result<Self, SearchError> {
        let schema = IndexSchema::new();

        let manager = if path.exists() {
            IndexManager::open(path)
                .map_err(|e| SearchError::Bm25Error(e.to_string()))?
        } else {
            std::fs::create_dir_all(path)
                .map_err(SearchError::IoError)?;
            IndexManager::create(path)
                .map_err(|e| SearchError::Bm25Error(e.to_string()))?
        };

        Ok(Self {
            manager: Arc::new(manager),
            schema,
            index_path: path.to_path_buf(),
        })
    }

    fn get_index_size(&self) -> Result<usize, SearchError> {
        fn calculate_dir_size(path: &Path) -> Result<usize, SearchError> {
            let mut total_size = 0usize;
            for entry in std::fs::read_dir(path)
                .map_err(SearchError::IoError)? {
                let entry = entry.map_err(SearchError::IoError)?;
                let metadata = entry.metadata().map_err(SearchError::IoError)?;
                if metadata.is_file() {
                    total_size += metadata.len() as usize;
                } else if metadata.is_dir() {
                    total_size += calculate_dir_size(&entry.path())?;
                }
            }
            Ok(total_size)
        }
        calculate_dir_size(&self.index_path)
    }
}

#[async_trait]
impl SearchEngine for Bm25SearchEngine {
    fn name(&self) -> &str {
        "bm25"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    async fn index(&self, doc_id: &str, content: &str) -> Result<(), SearchError> {
        let manager = self.manager.clone();
        let schema = self.schema.clone();
        let doc_id = doc_id.to_string();
        let content = content.to_string();

        tokio::task::spawn_blocking(move || {
            let mut fields = HashMap::new();
            fields.insert("content".to_string(), content);
            add_document(&manager, &schema, &doc_id, &fields)
                .map_err(|e| SearchError::Bm25Error(e.to_string()))
        })
        .await
        .map_err(|e| SearchError::Internal(e.to_string()))?
    }

    async fn index_batch(&self, docs: Vec<(String, String)>) -> Result<(), SearchError> {
        let manager = self.manager.clone();
        let schema = self.schema.clone();

        tokio::task::spawn_blocking(move || {
            for (doc_id, content) in docs {
                let mut fields = HashMap::new();
                fields.insert("content".to_string(), content);
                add_document(&manager, &schema, &doc_id, &fields)
                    .map_err(|e| SearchError::Bm25Error(e.to_string()))?;
            }
            Ok(())
        })
        .await
        .map_err(|e| SearchError::Internal(e.to_string()))?
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError> {
        let manager = self.manager.clone();
        let schema = self.schema.clone();
        let query = query.to_string();
        let options = SearchOptions {
            limit,
            offset: 0,
            field_weights: HashMap::new(),
            highlight: false,
        };

        tokio::task::spawn_blocking(move || {
            let (results, _) = search(&manager, &schema, &query, &options)
                .map_err(|e| SearchError::Bm25Error(e.to_string()))?;

            Ok(results.into_iter().map(|r| SearchResult {
                doc_id: Value::from(r.document_id),
                score: r.score,
                highlights: None,
                matched_fields: vec!["content".to_string()],
            }).collect())
        })
        .await
        .map_err(|e| SearchError::Internal(e.to_string()))?
    }

    async fn delete(&self, doc_id: &str) -> Result<(), SearchError> {
        let manager = self.manager.clone();
        let schema = self.schema.clone();
        let doc_id = doc_id.to_string();

        tokio::task::spawn_blocking(move || {
            delete_document(&manager, &schema, &doc_id)
                .map_err(|e| SearchError::Bm25Error(e.to_string()))
        })
        .await
        .map_err(|e| SearchError::Internal(e.to_string()))?
    }

    async fn delete_batch(&self, doc_ids: Vec<&str>) -> Result<(), SearchError> {
        let manager = self.manager.clone();
        let schema = self.schema.clone();
        let doc_ids: Vec<String> = doc_ids.into_iter().map(|s| s.to_string()).collect();

        tokio::task::spawn_blocking(move || {
            for doc_id in doc_ids {
                delete_document(&manager, &schema, &doc_id)
                    .map_err(|e| SearchError::Bm25Error(e.to_string()))?;
            }
            Ok(())
        })
        .await
        .map_err(|e| SearchError::Internal(e.to_string()))?
    }

    async fn commit(&self) -> Result<(), SearchError> {
        Ok(())
    }

    async fn rollback(&self) -> Result<(), SearchError> {
        Ok(())
    }

    async fn stats(&self) -> Result<IndexStats, SearchError> {
        let manager = self.manager.clone();
        let index_size = self.get_index_size()?;

        tokio::task::spawn_blocking(move || {
            let stats = get_stats(&manager)
                .map_err(|e| SearchError::Bm25Error(e.to_string()))?;

            Ok(IndexStats {
                doc_count: stats.total_documents as usize,
                index_size,
                last_updated: None,
                engine_info: None,
            })
        })
        .await
        .map_err(|e| SearchError::Internal(e.to_string()))?
    }

    async fn close(&self) -> Result<(), SearchError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[test]
    fn test_bm25_engine_creation() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let engine = Bm25SearchEngine::open_or_create(temp_dir.path());
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_bm25_index_and_search() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let engine = Bm25SearchEngine::open_or_create(temp_dir.path())
            .expect("Failed to create engine");

        engine.index("1", "Hello world from Rust").await.expect("Failed to index");
        engine.index("2", "Hello GraphDB").await.expect("Failed to index");
        engine.index("3", "Rust programming language").await.expect("Failed to index");

        let results = engine.search("Hello", 10).await.expect("Failed to search");
        assert!(!results.is_empty(), "Should find results for 'Hello'");
    }

    #[tokio::test]
    async fn test_bm25_delete() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let engine = Bm25SearchEngine::open_or_create(temp_dir.path())
            .expect("Failed to create engine");

        engine.index("1", "Test document").await.expect("Failed to index");
        engine.delete("1").await.expect("Failed to delete");

        let results = engine.search("Test", 10).await.expect("Failed to search");
        assert!(results.is_empty() || !results.iter().any(|r| matches!(&r.doc_id, Value::String(s) if s == "1")));
    }
}

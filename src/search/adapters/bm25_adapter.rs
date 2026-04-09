use async_trait::async_trait;
use bm25_service::api::embedded::Bm25Index;
use bm25_service::config::IndexManagerConfig;
use std::path::Path;

use crate::core::Value;
use crate::search::engine::SearchEngine;
use crate::search::error::SearchError;
use crate::search::result::{IndexStats, SearchResult};

pub struct Bm25SearchEngine {
    index: Bm25Index,
    index_path: std::path::PathBuf,
}

impl std::fmt::Debug for Bm25SearchEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bm25SearchEngine")
            .field("index_path", &self.index_path)
            .finish()
    }
}

impl Bm25SearchEngine {
    pub fn open_or_create(path: &Path, config: IndexManagerConfig) -> Result<Self, SearchError> {
        let index = if path.exists() {
            match Bm25Index::open_with_config(path, config) {
                Ok(index) => index,
                Err(_) => {
                    std::fs::create_dir_all(path).map_err(SearchError::IoError)?;
                    Bm25Index::create_with_config(path, IndexManagerConfig::default())
                        .map_err(|e| SearchError::Bm25Error(e.to_string()))?
                }
            }
        } else {
            std::fs::create_dir_all(path).map_err(SearchError::IoError)?;
            Bm25Index::create_with_config(path, config)
                .map_err(|e| SearchError::Bm25Error(e.to_string()))?
        };

        Ok(Self {
            index,
            index_path: path.to_path_buf(),
        })
    }

    fn get_index_size(&self) -> Result<usize, SearchError> {
        fn calculate_dir_size(path: &Path) -> Result<usize, SearchError> {
            let mut total_size = 0usize;
            for entry in std::fs::read_dir(path).map_err(SearchError::IoError)? {
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
        self.index
            .add_document(doc_id, "", content)
            .map_err(|e| SearchError::Bm25Error(e.to_string()))?;
        Ok(())
    }

    async fn index_batch(&self, docs: Vec<(String, String)>) -> Result<(), SearchError> {
        for (doc_id, content) in docs {
            self.index
                .add_document(&doc_id, "", &content)
                .map_err(|e| SearchError::Bm25Error(e.to_string()))?;
        }
        Ok(())
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError> {
        let results = self
            .index
            .search_with_highlights(query, limit)
            .map_err(|e| SearchError::Bm25Error(e.to_string()))?;

        Ok(results
            .into_iter()
            .map(|r| SearchResult {
                doc_id: Value::String(r.document_id),
                score: r.score,
                highlights: r.highlights,
                matched_fields: vec!["content".to_string()],
            })
            .collect())
    }

    async fn delete(&self, doc_id: &str) -> Result<(), SearchError> {
        self.index
            .delete_document(doc_id)
            .map_err(|e| SearchError::Bm25Error(e.to_string()))?;
        Ok(())
    }

    async fn delete_batch(&self, doc_ids: Vec<&str>) -> Result<(), SearchError> {
        for doc_id in doc_ids {
            self.index
                .delete_document(doc_id)
                .map_err(|e| SearchError::Bm25Error(e.to_string()))?;
        }
        Ok(())
    }

    async fn commit(&self) -> Result<(), SearchError> {
        self.index
            .commit()
            .map_err(|e| SearchError::Bm25Error(e.to_string()))?;
        Ok(())
    }

    async fn rollback(&self) -> Result<(), SearchError> {
        Ok(())
    }

    async fn stats(&self) -> Result<IndexStats, SearchError> {
        let count = self
            .index
            .count()
            .map_err(|e| SearchError::Bm25Error(e.to_string()))?;
        let index_size = self.get_index_size()?;

        Ok(IndexStats {
            doc_count: count as usize,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bm25_engine_creation() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let engine =
            Bm25SearchEngine::open_or_create(temp_dir.path(), IndexManagerConfig::default());
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_bm25_index_and_search() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let engine =
            Bm25SearchEngine::open_or_create(temp_dir.path(), IndexManagerConfig::default())
                .expect("Failed to create engine");

        engine
            .index("1", "Hello world from Rust")
            .await
            .expect("Failed to index");
        engine
            .index("2", "Hello GraphDB")
            .await
            .expect("Failed to index");
        engine
            .index("3", "Rust programming language")
            .await
            .expect("Failed to index");

        let results = engine.search("Hello", 10).await.expect("Failed to search");
        assert!(!results.is_empty(), "Should find results for 'Hello'");
    }

    #[tokio::test]
    async fn test_bm25_delete() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let engine =
            Bm25SearchEngine::open_or_create(temp_dir.path(), IndexManagerConfig::default())
                .expect("Failed to create engine");

        engine
            .index("1", "Test document")
            .await
            .expect("Failed to index");
        engine.delete("1").await.expect("Failed to delete");
    }
}

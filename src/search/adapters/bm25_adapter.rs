use async_trait::async_trait;
use bm25_service::index::delete::delete_document_with_writer;
use bm25_service::index::document::add_document_with_writer;
use bm25_service::index::search::{search, SearchOptions};
use bm25_service::index::stats::get_stats;
use bm25_service::index::{IndexManager, IndexSchema};
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tantivy::IndexWriter;
use tokio::sync::Mutex;

use crate::core::Value;
use crate::search::engine::SearchEngine;
use crate::search::error::SearchError;
use crate::search::result::{IndexStats, SearchResult};

pub struct Bm25SearchEngine {
    manager: Arc<IndexManager>,
    schema: IndexSchema,
    index_path: std::path::PathBuf,
    writer: Arc<Mutex<Option<IndexWriter>>>,
    operation_count: Arc<AtomicUsize>,
    batch_size: usize,
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
        Self::open_or_create_with_config(path, 100)
    }

    pub fn open_or_create_with_config(path: &Path, batch_size: usize) -> Result<Self, SearchError> {
        let schema = IndexSchema::new();

        let manager = if path.exists() {
            IndexManager::open(path).map_err(|e| SearchError::Bm25Error(e.to_string()))?
        } else {
            std::fs::create_dir_all(path).map_err(SearchError::IoError)?;
            IndexManager::create(path).map_err(|e| SearchError::Bm25Error(e.to_string()))?
        };

        let writer = manager
            .writer()
            .map_err(|e| SearchError::Bm25Error(format!("Failed to create writer: {}", e)))?;

        Ok(Self {
            manager: Arc::new(manager),
            schema,
            index_path: path.to_path_buf(),
            writer: Arc::new(Mutex::new(Some(writer))),
            operation_count: Arc::new(AtomicUsize::new(0)),
            batch_size,
        })
    }

    fn should_commit(&self) -> bool {
        let count = self.operation_count.fetch_add(1, Ordering::Relaxed);
        count % self.batch_size == 0
    }

    fn reset_counter(&self) {
        self.operation_count.store(0, Ordering::Relaxed);
    }

    async fn get_or_create_writer(&self) -> Result<IndexWriter, SearchError> {
        let mut writer_guard: tokio::sync::MutexGuard<'_, Option<IndexWriter>> =
            self.writer.lock().await;
        if writer_guard.is_none() {
            let writer = self
                .manager
                .writer()
                .map_err(|e| SearchError::Bm25Error(format!("Failed to create writer: {}", e)))?;
            *writer_guard = Some(writer);
        }
        Ok(writer_guard.take().unwrap())
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
        let mut fields = HashMap::new();
        fields.insert("content".to_string(), content.to_string());

        let writer = self.writer.clone();
        let schema = self.schema.clone();
        let doc_id = doc_id.to_string();
        let should_commit = self.should_commit();

        tokio::task::spawn_blocking(move || {
            let mut writer_guard = futures::executor::block_on(writer.lock());
            if writer_guard.is_none() {
                return Err(SearchError::Internal("Writer not initialized".to_string()));
            }

            let writer_ref = writer_guard.as_mut().unwrap();
            add_document_with_writer(writer_ref, &schema, &doc_id, &fields)
                .map_err(|e| SearchError::Bm25Error(e.to_string()))?;

            if should_commit {
                writer_ref
                    .commit()
                    .map_err(|e| SearchError::Bm25Error(format!("Commit failed: {}", e)))?;
            }

            Ok(())
        })
        .await
        .map_err(|e| SearchError::Internal(e.to_string()))?
    }

    async fn index_batch(&self, docs: Vec<(String, String)>) -> Result<(), SearchError> {
        let writer = self.writer.clone();
        let schema = self.schema.clone();

        tokio::task::spawn_blocking(move || {
            let mut writer_guard = futures::executor::block_on(writer.lock());
            if writer_guard.is_none() {
                return Err(SearchError::Internal("Writer not initialized".to_string()));
            }

            let writer_ref = writer_guard.as_mut().unwrap();
            for (doc_id, content) in docs {
                let mut fields = HashMap::new();
                fields.insert("content".to_string(), content);
                add_document_with_writer(writer_ref, &schema, &doc_id, &fields)
                    .map_err(|e| SearchError::Bm25Error(e.to_string()))?;
            }

            writer_ref
                .commit()
                .map_err(|e| SearchError::Bm25Error(format!("Commit failed: {}", e)))?;

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

            Ok(results
                .into_iter()
                .map(|r| SearchResult {
                    doc_id: Value::from(r.document_id),
                    score: r.score,
                    highlights: None,
                    matched_fields: vec!["content".to_string()],
                })
                .collect())
        })
        .await
        .map_err(|e| SearchError::Internal(e.to_string()))?
    }

    async fn delete(&self, doc_id: &str) -> Result<(), SearchError> {
        let writer = self.writer.clone();
        let schema = self.schema.clone();
        let doc_id = doc_id.to_string();
        let should_commit = self.should_commit();

        tokio::task::spawn_blocking(move || {
            let mut writer_guard = futures::executor::block_on(writer.lock());
            if writer_guard.is_none() {
                return Err(SearchError::Internal("Writer not initialized".to_string()));
            }

            let writer_ref = writer_guard.as_mut().unwrap();
            delete_document_with_writer(writer_ref, &schema, &doc_id)
                .map_err(|e| SearchError::Bm25Error(e.to_string()))?;

            if should_commit {
                writer_ref
                    .commit()
                    .map_err(|e| SearchError::Bm25Error(format!("Commit failed: {}", e)))?;
            }

            Ok(())
        })
        .await
        .map_err(|e| SearchError::Internal(e.to_string()))?
    }

    async fn delete_batch(&self, doc_ids: Vec<&str>) -> Result<(), SearchError> {
        let writer = self.writer.clone();
        let schema = self.schema.clone();
        let doc_ids: Vec<String> = doc_ids.into_iter().map(|s| s.to_string()).collect();

        tokio::task::spawn_blocking(move || {
            let mut writer_guard = futures::executor::block_on(writer.lock());
            if writer_guard.is_none() {
                return Err(SearchError::Internal("Writer not initialized".to_string()));
            }

            let writer_ref = writer_guard.as_mut().unwrap();
            for doc_id in doc_ids {
                delete_document_with_writer(writer_ref, &schema, &doc_id)
                    .map_err(|e| SearchError::Bm25Error(e.to_string()))?;
            }

            writer_ref
                .commit()
                .map_err(|e| SearchError::Bm25Error(format!("Commit failed: {}", e)))?;

            Ok(())
        })
        .await
        .map_err(|e| SearchError::Internal(e.to_string()))?
    }

    async fn commit(&self) -> Result<(), SearchError> {
        let mut writer_guard: tokio::sync::MutexGuard<'_, Option<IndexWriter>> =
            self.writer.lock().await;
        if let Some(mut writer) = writer_guard.take() {
            writer
                .commit()
                .map_err(|e| SearchError::Bm25Error(format!("Commit failed: {}", e)))?;
            *writer_guard = Some(writer);
        }
        Ok(())
    }

    async fn rollback(&self) -> Result<(), SearchError> {
        let mut writer_guard: tokio::sync::MutexGuard<'_, Option<IndexWriter>> =
            self.writer.lock().await;
        if let Some(mut writer) = writer_guard.take() {
            writer
                .rollback()
                .map_err(|e| SearchError::Bm25Error(format!("Rollback failed: {}", e)))?;
            *writer_guard = Some(writer);
        }
        Ok(())
    }

    async fn stats(&self) -> Result<IndexStats, SearchError> {
        let manager = self.manager.clone();
        let index_size = self.get_index_size()?;

        tokio::task::spawn_blocking(move || {
            let stats = get_stats(&manager).map_err(|e| SearchError::Bm25Error(e.to_string()))?;

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
        self.commit().await?;

        let mut writer_guard: tokio::sync::MutexGuard<'_, Option<IndexWriter>> =
            self.writer.lock().await;
        *writer_guard = None;

        self.reset_counter();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bm25_engine_creation() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let engine = Bm25SearchEngine::open_or_create(temp_dir.path());
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_bm25_index_and_search() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let engine =
            Bm25SearchEngine::open_or_create(temp_dir.path()).expect("Failed to create engine");

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
            Bm25SearchEngine::open_or_create(temp_dir.path()).expect("Failed to create engine");

        engine
            .index("1", "Test document")
            .await
            .expect("Failed to index");
        engine.delete("1").await.expect("Failed to delete");

        let results = engine.search("Test", 10).await.expect("Failed to search");
        assert!(
            results.is_empty()
                || !results
                    .iter()
                    .any(|r| matches!(&r.doc_id, Value::String(s) if s == "1"))
        );
    }
}

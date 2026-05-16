//! Tantivy Local File Storage Implementation
//!
//! Persistence of BM25 word frequency statistics using Tantivy as the underlying storage

use crate::error::{Bm25Error, Result};
use crate::storage::common::metrics::{StorageMetrics, StorageMetricsCollector};
use crate::storage::common::r#trait::{Bm25Stats, StorageInterface};
use crate::storage::common::types::StorageInfo;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tantivy::schema::{Schema, STORED, STRING, TEXT};
use tantivy::{Index, IndexReader, IndexWriter, Term};
use tokio::sync::RwLock;
use std::time::Instant;

/// Tantivy Storage Configuration
#[derive(Debug, Clone)]
pub struct TantivyStorageConfig {
    pub index_path: PathBuf,
    pub writer_memory_mb: usize,
}

impl Default for TantivyStorageConfig {
    fn default() -> Self {
        Self {
            index_path: PathBuf::from("./index"),
            writer_memory_mb: 50,
        }
    }
}

/// Tantivy Storage Implementation
pub struct TantivyStorage {
    config: TantivyStorageConfig,
    index: Option<Arc<RwLock<Index>>>,
    schema: Schema,
    writer: Option<Arc<RwLock<IndexWriter>>>,
    reader: Option<Arc<RwLock<IndexReader>>>,
    /// Metrics collector for tracking operations and errors
    metrics: Arc<StorageMetricsCollector>,
}

impl std::fmt::Debug for TantivyStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TantivyStorage")
            .field("config", &self.config)
            .field("index", &self.index.is_some())
            .field("writer", &self.writer.is_some())
            .field("reader", &self.reader.is_some())
            .finish()
    }
}

impl TantivyStorage {
    /// Creates a new Tantivy storage instance
    pub fn new(config: TantivyStorageConfig) -> Self {
        Self {
            config,
            index: None,
            schema: Self::build_schema(),
            writer: None,
            reader: None,
            metrics: Arc::new(StorageMetricsCollector::default()),
        }
    }

    fn build_schema() -> Schema {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("document_id", STRING | STORED);
        schema_builder.add_text_field("title", TEXT | STORED);
        schema_builder.add_text_field("content", TEXT | STORED);
        schema_builder.build()
    }

    /// Gets operation statistics and performance metrics
    ///
    /// Returns a snapshot of current storage metrics including
    /// operation counts, latencies, and error statistics.
    pub fn get_operation_stats(&self) -> StorageMetrics {
        // Tantivy memory usage is managed internally and difficult to estimate precisely
        self.metrics.get_metrics(0)
    }
}

#[async_trait::async_trait]
impl StorageInterface for TantivyStorage {
    async fn init(&mut self) -> Result<()> {
        let start = Instant::now();
        
        if self.index.is_none() {
            std::fs::create_dir_all(&self.config.index_path)
                .map_err(|e| Bm25Error::IndexCreationFailed(e.to_string()))?;

            let index = Index::create_in_dir(&self.config.index_path, self.schema.clone())
                .map_err(|e| Bm25Error::IndexCreationFailed(e.to_string()))?;

            let writer = index
                .writer(self.config.writer_memory_mb * 1024 * 1024)
                .map_err(|e| Bm25Error::IndexCreationFailed(e.to_string()))?;

            let reader = index
                .reader_builder()
                .reload_policy(tantivy::ReloadPolicy::OnCommitWithDelay)
                .try_into()
                .map_err(|e| Bm25Error::IndexCreationFailed(e.to_string()))?;

            self.index = Some(Arc::new(RwLock::new(index)));
            self.writer = Some(Arc::new(RwLock::new(writer)));
            self.reader = Some(Arc::new(RwLock::new(reader)));
        }
        
        self.metrics.record_operation(start);
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        let start = Instant::now();
        
        if let Some(writer) = self.writer.take() {
            let mut writer = writer.write().await;
            writer
                .commit()
                .map_err(|e: tantivy::TantivyError| Bm25Error::IndexCommitFailed(e.to_string()))?;
        }
        
        self.metrics.record_operation(start);
        Ok(())
    }

    async fn commit_stats(&mut self, _term: &str, _tf: f32, _df: u64) -> Result<()> {
        let start = Instant::now();
        // Tantivy manages word frequency statistics automatically, eliminating the need for manual submissions
        self.metrics.record_operation(start);
        Ok(())
    }

    async fn commit_batch(&mut self, _stats: &Bm25Stats) -> Result<()> {
        let start = Instant::now();
        // Tantivy manages word frequency statistics automatically, eliminating the need for manual submissions
        self.metrics.record_operation(start);
        Ok(())
    }

    async fn get_stats(&self, term: &str) -> Result<Option<Bm25Stats>> {
        let start = Instant::now();
        
        let reader = self
            .reader
            .as_ref()
            .ok_or_else(|| Bm25Error::IndexNotInitialized)?;
        let reader = reader.read().await;
        let searcher = reader.searcher();

        // Get term frequency from the content field
        let field = self.schema.get_field("content").unwrap();
        let term_obj = Term::from_field_text(field, term);

        // Get document frequency
        let doc_freq = searcher.doc_freq(&term_obj)?;
        let total_docs = searcher.num_docs();

        // Calculate average document length
        let avg_doc_length = if total_docs > 0 {
            let total_terms = searcher.num_docs() * 100; // Approximation
            total_terms as f32 / total_docs as f32
        } else {
            0.0
        };

        self.metrics.record_operation(start);
        
        Ok(Some(Bm25Stats {
            tf: HashMap::new(), // TF is calculated per document during search
            df: HashMap::from([(term.to_string(), doc_freq as u64)]),
            total_docs: total_docs as u64,
            avg_doc_length,
        }))
    }

    async fn get_df(&self, term: &str) -> Result<Option<u64>> {
        let start = Instant::now();
        
        let reader = self
            .reader
            .as_ref()
            .ok_or_else(|| Bm25Error::IndexNotInitialized)?;
        let reader = reader.read().await;
        let searcher = reader.searcher();

        let field = self.schema.get_field("content").unwrap();
        let term_obj = Term::from_field_text(field, term);

        let doc_freq = searcher.doc_freq(&term_obj)?;
        
        self.metrics.record_operation(start);
        
        Ok(Some(doc_freq as u64))
    }

    async fn get_tf(&self, term: &str, _doc_id: &str) -> Result<Option<f32>> {
        let start = Instant::now();
        
        // TF is calculated during search time in Tantivy
        // This is a simplified implementation
        let reader = self
            .reader
            .as_ref()
            .ok_or_else(|| Bm25Error::IndexNotInitialized)?;
        let reader = reader.read().await;
        let searcher = reader.searcher();

        let field = self.schema.get_field("content").unwrap();
        let term_obj = Term::from_field_text(field, term);

        let doc_freq = searcher.doc_freq(&term_obj)?;
        let total_docs = searcher.num_docs();

        // Simple TF calculation (in real BM25, this is more complex)
        let result = if doc_freq > 0 && total_docs > 0 {
            let tf = (doc_freq as f32) / (total_docs as f32);
            Some(tf)
        } else {
            Some(0.0)
        };

        self.metrics.record_operation(start);
        
        Ok(result)
    }

    async fn clear(&mut self) -> Result<()> {
        let start = Instant::now();
        
        if let Some(writer) = self.writer.as_ref() {
            let mut writer = writer.write().await;
            writer
                .commit()
                .map_err(|e: tantivy::TantivyError| Bm25Error::IndexCommitFailed(e.to_string()))?;
        }
        
        self.metrics.record_operation(start);
        Ok(())
    }

    async fn delete_doc_stats(&mut self, _doc_id: &str) -> Result<()> {
        let start = Instant::now();
        
        // Statistics are dynamically calculated in Tantivy and do not need to be explicitly deleted.
        // Document deletion is handled by IndexManager
        self.metrics.record_operation(start);
        Ok(())
    }

    async fn info(&self) -> Result<StorageInfo> {
        let start = Instant::now();
        
        let total_docs = if let Some(reader) = &self.reader {
            let reader = reader.read().await;
            reader.searcher().num_docs() as usize
        } else {
            0
        };

        self.metrics.record_operation(start);
        
        Ok(StorageInfo {
            name: "TantivyStorage".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            size: 0,
            document_count: total_docs,
            term_count: 0,
            is_connected: true,
        })
    }

    async fn health_check(&self) -> Result<bool> {
        let start = Instant::now();
        
        let healthy = self.index.is_some();
        
        self.metrics.record_operation(start);
        
        Ok(healthy)
    }
}

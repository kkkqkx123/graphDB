use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;

use crate::core::stats::StatsManager;
use crate::search::engine::{EngineType, SearchEngine};
use crate::search::error::SearchError;
use crate::search::result::{IndexStats, SearchResult};

/// A decorator that wraps a SearchEngine and records metrics via StatsManager.
///
/// This allows transparent instrumentation of search/index/delete operations
/// without modifying the underlying engine implementations.
pub struct MetricsSearchEngine {
    inner: Arc<dyn SearchEngine>,
    stats_manager: Arc<StatsManager>,
    engine_type: EngineType,
    space_id: u64,
    index_name: String,
}

impl MetricsSearchEngine {
    pub fn new(
        inner: Arc<dyn SearchEngine>,
        stats_manager: Arc<StatsManager>,
        engine_type: EngineType,
        space_id: u64,
        index_name: String,
    ) -> Self {
        Self {
            inner,
            stats_manager,
            engine_type,
            space_id,
            index_name,
        }
    }

    pub fn into_arc(self) -> Arc<dyn SearchEngine> {
        Arc::new(self)
    }
}

#[async_trait]
impl SearchEngine for MetricsSearchEngine {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn version(&self) -> &str {
        self.inner.version()
    }

    async fn index(&self, doc_id: &str, content: &str) -> Result<(), SearchError> {
        let start = Instant::now();
        let result = self.inner.index(doc_id, content).await;
        let latency_ms = start.elapsed().as_millis() as u64;
        self.stats_manager.record_index_operation(
            self.space_id,
            &self.index_name,
            latency_ms,
            result.is_ok(),
        );
        result
    }

    async fn index_batch(&self, docs: Vec<(String, String)>) -> Result<(), SearchError> {
        let start = Instant::now();
        let result = self.inner.index_batch(docs).await;
        let latency_ms = start.elapsed().as_millis() as u64;
        self.stats_manager.record_index_operation(
            self.space_id,
            &self.index_name,
            latency_ms,
            result.is_ok(),
        );
        result
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError> {
        let start = Instant::now();
        let result = self.inner.search(query, limit).await;
        let latency_ms = start.elapsed().as_millis() as u64;

        match &result {
            Ok(results) => {
                self.stats_manager.record_search(
                    self.space_id,
                    &self.index_name,
                    latency_ms,
                    true,
                );
                self.stats_manager.record_search_result_count(
                    self.space_id,
                    results.len() as u64,
                );
            }
            Err(_) => {
                self.stats_manager.record_search(
                    self.space_id,
                    &self.index_name,
                    latency_ms,
                    false,
                );
            }
        }

        result
    }

    async fn delete(&self, doc_id: &str) -> Result<(), SearchError> {
        let start = Instant::now();
        let result = self.inner.delete(doc_id).await;
        let latency_ms = start.elapsed().as_millis() as u64;
        self.stats_manager.record_delete_operation(
            self.space_id,
            &self.index_name,
            latency_ms,
            result.is_ok(),
        );
        result
    }

    async fn delete_batch(&self, doc_ids: Vec<&str>) -> Result<(), SearchError> {
        let start = Instant::now();
        let result = self.inner.delete_batch(doc_ids).await;
        let latency_ms = start.elapsed().as_millis() as u64;
        self.stats_manager.record_delete_operation(
            self.space_id,
            &self.index_name,
            latency_ms,
            result.is_ok(),
        );
        result
    }

    async fn commit(&self) -> Result<(), SearchError> {
        self.inner.commit().await
    }

    async fn rollback(&self) -> Result<(), SearchError> {
        self.inner.rollback().await
    }

    async fn stats(&self) -> Result<IndexStats, SearchError> {
        self.inner.stats().await
    }

    async fn close(&self) -> Result<(), SearchError> {
        self.inner.close().await
    }
}

impl std::fmt::Debug for MetricsSearchEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetricsSearchEngine")
            .field("inner", &self.inner.name())
            .field("engine_type", &self.engine_type)
            .field("space_id", &self.space_id)
            .field("index_name", &self.index_name)
            .finish()
    }
}
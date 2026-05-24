use async_trait::async_trait;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Instant;

use tantivy::collector::TopDocs;
use tantivy::doc;
use tantivy::query::QueryParser;
use tantivy::schema::Value as SchemaValue;
use tantivy::schema::*;
use tantivy::IndexWriter;
use tantivy::TantivyDocument;

use crate::core::Value;
use crate::search::engine::{ConsistencyState, SearchEngine};
use crate::search::error::SearchError;
use crate::search::result::{IndexStats, SearchResult};
use tantivy::tokenizer::JiebaTokenizer;

pub use crate::config::common::fulltext::{TantivyConfig, TokenizerKind};

impl TokenizerKind {
    pub fn name(&self) -> &'static str {
        match self {
            TokenizerKind::Jieba => "jieba",
            TokenizerKind::Raw => "raw",
            TokenizerKind::Default => "default",
            TokenizerKind::Whitespace => "whitespace",
        }
    }
}

fn build_schema(config: &TantivyConfig) -> (Schema, Field, Field) {
    let tokenizer_name = config.tokenizer.name();
    let mut schema_builder = Schema::builder();
    let id_field = schema_builder.add_text_field("id", STRING | STORED);
    let text_options = TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer(tokenizer_name)
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        )
        .set_stored();
    let text_field = schema_builder.add_text_field("text", text_options);
    let schema = schema_builder.build();
    (schema, id_field, text_field)
}

pub struct TantivySearchEngine {
    index: tantivy::Index,
    index_path: PathBuf,
    id_field: Field,
    text_field: Field,
    writer: Arc<Mutex<IndexWriter>>,
    reader: Arc<tantivy::IndexReader>,
    /// Consistency tracking:
    ///   0 = Consistent, 1 = Inconsistent, 2 = Rebuilding
    consistency_state: AtomicU8,
    /// Cached stats to avoid directory I/O on every stats() call.
    cached_doc_count: AtomicU64,
    cached_index_size: AtomicU64,
    last_stats_update: std::sync::Mutex<Option<Instant>>,
}

impl std::fmt::Debug for TantivySearchEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TantivySearchEngine").finish()
    }
}

impl TantivySearchEngine {
    /// Execute a writer operation on the blocking thread pool.
    /// This prevents tantivy's CPU/IO-bound IndexWriter operations
    /// from starving tokio's async worker threads.
    async fn with_writer<F, T>(&self, f: F) -> Result<T, SearchError>
    where
        F: FnOnce(&mut IndexWriter) -> Result<T, tantivy::TantivyError> + Send + 'static,
        T: Send + 'static,
    {
        let writer = self.writer.clone();
        tokio::task::spawn_blocking(move || {
            let mut guard = writer.lock();
            f(&mut guard)
        })
        .await
        .map_err(|e| SearchError::Internal(format!("Blocking task failed: {}", e)))?
        .map_err(SearchError::from)
    }

    /// Recompute cached stats from the index reader and directory.
    fn refresh_stats_cache(&self) {
        {
            let searcher = self.reader.searcher();
            let doc_count = searcher.num_docs();
            self.cached_doc_count.store(doc_count, Ordering::Release);
        }

        let index_size = self
            .index_path
            .read_dir()
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().ok().is_some_and(|t| t.is_file()))
            .filter_map(|entry| entry.metadata().ok())
            .map(|meta| meta.len())
            .sum::<u64>();
        self.cached_index_size.store(index_size, Ordering::Release);

        if let Ok(mut last) = self.last_stats_update.lock() {
            *last = Some(Instant::now());
        }
    }

    pub fn open_or_create(path: &Path, config: TantivyConfig) -> Result<Self, SearchError> {
        let (schema, id_field, text_field) = build_schema(&config);

        if !path.exists() {
            std::fs::create_dir_all(path)?;
        }

        let index = if path.join("meta.json").exists() {
            tantivy::Index::open_in_dir(path)?
        } else {
            tantivy::Index::create_in_dir(path, schema.clone())?
        };

        // Register jieba unconditionally for backward compatibility with existing
        // indexes whose schema may reference "jieba". Tantivy's default TokenizerManager
        // auto-registers "raw", "default", and "whitespace".
        index.tokenizers().register("jieba", JiebaTokenizer::default());

        let writer = index.writer(config.writer_memory_budget)?;

        // Create a cached IndexReader with OnCommitWithDelay policy.
        // This reader auto-refreshes when meta.json changes (i.e., after a commit),
        // so we don't need to create a new reader on every search call.
        let reader = index
            .reader_builder()
            .reload_policy(tantivy::ReloadPolicy::OnCommitWithDelay)
            .doc_store_cache_num_blocks(config.doc_store_cache_num_blocks)
            .try_into()?;

        let index_path = path.to_path_buf();

        Ok(Self {
            index,
            index_path,
            id_field,
            text_field,
            writer: Arc::new(Mutex::new(writer)),
            reader: Arc::new(reader),
            consistency_state: AtomicU8::new(0),
            cached_doc_count: AtomicU64::new(0),
            cached_index_size: AtomicU64::new(0),
            last_stats_update: std::sync::Mutex::new(None),
        })
    }
}

#[async_trait]
impl SearchEngine for TantivySearchEngine {
    fn name(&self) -> &str {
        "tantivy"
    }

    fn version(&self) -> &str {
        "0.26.0"
    }

    async fn index(&self, doc_id: &str, content: &str) -> Result<(), SearchError> {
        let id_field = self.id_field;
        let text_field = self.text_field;
        let doc_id = doc_id.to_string();
        let content = content.to_string();
        self.with_writer(move |writer| {
            let doc = doc!(id_field => doc_id.as_str(), text_field => content.as_str());
            writer.add_document(doc)?;
            Ok(())
        })
        .await
    }

    async fn index_batch(&self, docs: Vec<(String, String)>) -> Result<(), SearchError> {
        let id_field = self.id_field;
        let text_field = self.text_field;
        let docs_clone = docs.clone();
        self.with_writer(move |writer| {
            for (doc_id, content) in &docs_clone {
                let doc = doc!(id_field => doc_id.as_str(), text_field => content.as_str());
                writer.add_document(doc)?;
            }
            Ok(())
        })
        .await
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError> {
        let searcher = self.reader.searcher();

        let query_parser = QueryParser::for_index(&self.index, vec![self.text_field]);
        let query = query_parser
            .parse_query(query)
            .map_err(|e| SearchError::QueryParseError(e.to_string()))?;

        let top_docs = searcher.search(
            &query,
            &TopDocs::with_limit(limit.max(1)).order_by_score(),
        )?;

        // Create snippet generator for highlight extraction.
        let snippet_generator = tantivy::snippet::SnippetGenerator::create(
            &searcher,
            &*query,
            self.text_field,
        )?;

        let mut results = Vec::with_capacity(top_docs.len());
        for (score, doc_address) in top_docs {
            let doc = searcher.doc::<TantivyDocument>(doc_address)?;
            let doc_id: String = doc
                .get_first(self.id_field)
                .and_then(|v| SchemaValue::as_str(&v))
                .unwrap_or("")
                .to_string();

            // Generate highlight snippet from the stored text field.
            let highlights = doc
                .get_first(self.text_field)
                .and_then(|v| SchemaValue::as_str(&v))
                .map(|text| vec![snippet_generator.snippet(text).to_html()]);

            results.push(SearchResult {
                doc_id: Value::String(doc_id),
                score,
                highlights,
                matched_fields: vec![],
            });
        }

        Ok(results)
    }

    async fn delete(&self, doc_id: &str) -> Result<(), SearchError> {
        let id_field = self.id_field;
        let doc_id = doc_id.to_string();
        self.with_writer(move |writer| {
            writer.delete_term(tantivy::Term::from_field_text(id_field, &doc_id));
            Ok(())
        })
        .await
    }

    async fn delete_batch(&self, doc_ids: Vec<&str>) -> Result<(), SearchError> {
        let id_field = self.id_field;
        let ids: Vec<String> = doc_ids.into_iter().map(|s| s.to_string()).collect();
        self.with_writer(move |writer| {
            for doc_id in &ids {
                writer.delete_term(tantivy::Term::from_field_text(id_field, doc_id));
            }
            Ok(())
        })
        .await
    }

    async fn commit(&self) -> Result<(), SearchError> {
        self.with_writer(move |writer| {
            writer.commit()?;
            Ok(())
        })
        .await?;
        self.refresh_stats_cache();
        Ok(())
    }

    async fn rollback(&self) -> Result<(), SearchError> {
        self.with_writer(move |_writer| {
            // tantivy does not support rollback.
            // The transaction buffer (TransactionBatchBuffer) provides
            // in-memory protection before commit. If a commit has already
            // occurred, the index state is intentionally unchanged.
            Ok(())
        })
        .await
    }

    async fn stats(&self) -> Result<IndexStats, SearchError> {
        const STATS_CACHE_TTL_SECS: u64 = 5;

        let needs_refresh = self
            .last_stats_update
            .lock()
            .ok()
            .and_then(|last| *last)
            .map(|t| t.elapsed().as_secs() > STATS_CACHE_TTL_SECS)
            .unwrap_or(true);

        if needs_refresh {
            self.refresh_stats_cache();
        }

        Ok(IndexStats {
            doc_count: self.cached_doc_count.load(Ordering::Acquire) as usize,
            index_size: self.cached_index_size.load(Ordering::Acquire) as usize,
            last_updated: None,
            engine_info: None,
        })
    }

    fn consistency_state(&self) -> ConsistencyState {
        match self.consistency_state.load(Ordering::Acquire) {
            0 => ConsistencyState::Consistent,
            1 => ConsistencyState::Inconsistent,
            _ => ConsistencyState::Rebuilding,
        }
    }

    fn mark_inconsistent(&self) {
        self.consistency_state.store(1, Ordering::Release);
    }

    fn mark_consistent(&self) {
        self.consistency_state.store(0, Ordering::Release);
    }

    async fn clear(&self) -> Result<(), SearchError> {
        self.with_writer(move |writer| {
            writer.delete_all_documents()?;
            writer.commit()?;
            Ok(())
        })
        .await
    }

    async fn close(&self) -> Result<(), SearchError> {
        self.commit().await?;
        Ok(())
    }
}

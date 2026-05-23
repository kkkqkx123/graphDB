use async_trait::async_trait;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tantivy::collector::TopDocs;
use tantivy::doc;
use tantivy::query::QueryParser;
use tantivy::schema::Value as SchemaValue;
use tantivy::schema::*;
use tantivy::IndexWriter;
use tantivy::TantivyDocument;

use crate::core::Value;
use crate::search::engine::SearchEngine;
use crate::search::error::SearchError;
use crate::search::result::{IndexStats, SearchResult};
use tantivy::tokenizer::JiebaTokenizer;

/// Supported tokenizer kinds.
///
/// Tokenizers are always from the project itself (tantivy built-in or jieba),
/// so we use a closed enum instead of a dynamic string-based approach.
/// This eliminates runtime validation and ensures type safety.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenizerKind {
    /// Jieba Chinese text segmentation (default).
    #[default]
    Jieba,
    /// Tantivy's raw tokenizer (no processing).
    Raw,
    /// Tantivy's default tokenizer (SimpleTokenizer + LowerCaser).
    Default,
    /// Tantivy's whitespace tokenizer.
    Whitespace,
}

impl TokenizerKind {
    fn name(&self) -> &'static str {
        match self {
            TokenizerKind::Jieba => "jieba",
            TokenizerKind::Raw => "raw",
            TokenizerKind::Default => "default",
            TokenizerKind::Whitespace => "whitespace",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TantivyConfig {
    pub writer_memory_budget: usize,
    #[serde(default)]
    pub tokenizer: TokenizerKind,
}

impl Default for TantivyConfig {
    fn default() -> Self {
        Self {
            writer_memory_budget: 50_000_000,
            tokenizer: TokenizerKind::default(),
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
}

impl std::fmt::Debug for TantivySearchEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TantivySearchEngine").finish()
    }
}

impl TantivySearchEngine {
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
            .try_into()?;

        let index_path = path.to_path_buf();

        Ok(Self {
            index,
            index_path,
            id_field,
            text_field,
            writer: Arc::new(Mutex::new(writer)),
            reader: Arc::new(reader),
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
        let writer = self.writer.lock();
        let doc = doc!(
            self.id_field => doc_id,
            self.text_field => content,
        );
        writer.add_document(doc)?;
        Ok(())
    }

    async fn index_batch(&self, docs: Vec<(String, String)>) -> Result<(), SearchError> {
        let writer = self.writer.lock();
        for (doc_id, content) in docs {
            let doc = doc!(
                self.id_field => doc_id.as_str(),
                self.text_field => content.as_str(),
            );
            writer.add_document(doc)?;
        }
        Ok(())
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
        let writer = self.writer.lock();
        writer.delete_term(
            tantivy::Term::from_field_text(self.id_field, doc_id),
        );
        Ok(())
    }

    async fn delete_batch(&self, doc_ids: Vec<&str>) -> Result<(), SearchError> {
        let writer = self.writer.lock();
        for doc_id in doc_ids {
            writer.delete_term(
                tantivy::Term::from_field_text(self.id_field, doc_id),
            );
        }
        Ok(())
    }

    async fn commit(&self) -> Result<(), SearchError> {
        let mut writer = self.writer.lock();
        writer.commit()?;
        Ok(())
    }

    async fn rollback(&self) -> Result<(), SearchError> {
        Ok(())
    }

    async fn stats(&self) -> Result<IndexStats, SearchError> {
        let searcher = self.reader.searcher();
        let doc_count = searcher.num_docs() as usize;

        // Calculate on-disk index size by summing all files in the index directory.
        let index_size = self
            .index_path
            .read_dir()
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().ok().is_some_and(|t| t.is_file()))
            .filter_map(|entry| entry.metadata().ok())
            .map(|meta| meta.len() as usize)
            .sum();

        Ok(IndexStats {
            doc_count,
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

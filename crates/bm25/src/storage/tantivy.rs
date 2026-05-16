use crate::api::core::IndexManager;
use crate::error::{Bm25Error, Result};
use crate::storage::common::r#trait::{Bm25Stats, StorageInterface};
use crate::storage::common::types::StorageInfo;
use crate::storage::stats_store::StatsStore;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tantivy::postings::Postings;
use tantivy::{DocSet, Term};

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

pub struct TantivyStorage {
    config: TantivyStorageConfig,
    index_manager: Option<Arc<IndexManager>>,
    stats_store: Option<StatsStore>,
}

impl std::fmt::Debug for TantivyStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TantivyStorage")
            .field("config", &self.config)
            .field("index_manager", &self.index_manager.is_some())
            .field("stats_store", &self.stats_store.is_some())
            .finish()
    }
}

impl TantivyStorage {
    pub fn new(config: TantivyStorageConfig) -> Self {
        let stats_path = config.index_path.join("stats.bin");
        Self {
            config,
            index_manager: None,
            stats_store: Some(StatsStore::new(stats_path)),
        }
    }

    pub fn with_index_manager(config: TantivyStorageConfig, index_manager: Arc<IndexManager>) -> Self {
        let stats_path = config.index_path.join("stats.bin");
        Self {
            config,
            index_manager: Some(index_manager),
            stats_store: Some(StatsStore::new(stats_path)),
        }
    }
}

#[async_trait::async_trait]
impl StorageInterface for TantivyStorage {
    async fn init(&mut self) -> Result<()> {
        if self.index_manager.is_none() {
            std::fs::create_dir_all(&self.config.index_path)
                .map_err(|e| Bm25Error::IndexCreationFailed(e.to_string()))?;

            let manager = IndexManager::create(&self.config.index_path)?;
            self.index_manager = Some(Arc::new(manager));
        }

        if let Some(ref store) = self.stats_store {
            store.load().await?;
        }

        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        if let Some(ref manager) = self.index_manager {
            let mut writer = manager.writer()?;
            writer
                .commit()
                .map_err(|e: tantivy::TantivyError| Bm25Error::IndexCommitFailed(e.to_string()))?;
        }
        Ok(())
    }

    async fn commit_stats(&mut self, term: &str, tf: f32, df: u64) -> Result<()> {
        let stats = Bm25Stats {
            tf: HashMap::from([(term.to_string(), tf)]),
            df: HashMap::from([(term.to_string(), df)]),
            total_docs: 0,
            avg_doc_length: 0.0,
        };
        self.commit_batch(&stats).await
    }

    async fn commit_batch(&mut self, stats: &Bm25Stats) -> Result<()> {
        if let Some(ref store) = self.stats_store {
            store.commit_batch(stats).await?;
        }
        Ok(())
    }

    async fn get_stats(&self, term: &str) -> Result<Option<Bm25Stats>> {
        if let Some(ref store) = self.stats_store {
            if let Some(stats) = store.get_stats(term).await? {
                return Ok(Some(stats));
            }
        }

        let manager = self
            .index_manager
            .as_ref()
            .ok_or_else(|| Bm25Error::IndexNotInitialized)?;
        let reader = manager.reader()?;
        let searcher = reader.searcher();
        let schema = manager.schema();

        let field = schema.get_field("content").unwrap();
        let term_obj = Term::from_field_text(field, term);

        let doc_freq = searcher.doc_freq(&term_obj)?;
        let total_docs = searcher.num_docs();

        let avg_doc_length = if total_docs > 0 {
            if let Some(ref store) = self.stats_store {
                let stats = store.get_stats(term).await?;
                stats.map(|s| s.avg_doc_length).unwrap_or(100.0)
            } else {
                100.0
            }
        } else {
            0.0
        };

        Ok(Some(Bm25Stats {
            tf: HashMap::new(),
            df: HashMap::from([(term.to_string(), doc_freq)]),
            total_docs,
            avg_doc_length,
        }))
    }

    async fn get_df(&self, term: &str) -> Result<Option<u64>> {
        if let Some(ref store) = self.stats_store {
            if let Some(df) = store.get_df(term).await? {
                return Ok(Some(df));
            }
        }

        let manager = self
            .index_manager
            .as_ref()
            .ok_or_else(|| Bm25Error::IndexNotInitialized)?;
        let reader = manager.reader()?;
        let searcher = reader.searcher();
        let schema = manager.schema();

        let field = schema.get_field("content").unwrap();
        let term_obj = Term::from_field_text(field, term);

        let doc_freq = searcher.doc_freq(&term_obj)?;
        Ok(Some(doc_freq))
    }

    async fn get_tf(&self, term: &str, doc_id: &str) -> Result<Option<f32>> {
        if let Some(ref store) = self.stats_store {
            if let Some(tf) = store.get_tf(term).await? {
                return Ok(Some(tf));
            }
        }

        let manager = self
            .index_manager
            .as_ref()
            .ok_or_else(|| Bm25Error::IndexNotInitialized)?;
        let reader = manager.reader()?;
        let searcher = reader.searcher();
        let schema = manager.schema();

        let doc_id_field = schema.get_field("document_id")?;
        let term_query = tantivy::query::TermQuery::new(
            Term::from_field_text(doc_id_field, doc_id),
            tantivy::schema::IndexRecordOption::Basic,
        );

        let top_docs = searcher.search(&term_query, &tantivy::collector::TopDocs::with_limit(1))?;
        let (_score, doc_address) = match top_docs.into_iter().next() {
            Some(result) => result,
            None => return Ok(Some(0.0)),
        };

        let content_field = schema.get_field("content")?;
        let term_obj = Term::from_field_text(content_field, term);

        let segment_reader = &searcher.segment_readers()[doc_address.segment_ord as usize];
        let inverted_index = segment_reader.inverted_index(content_field)?;
        if let Some(mut postings) = inverted_index.read_postings(
            &term_obj,
            tantivy::schema::IndexRecordOption::Basic,
        )? {
            while postings.advance() != 0 {
                if postings.doc() == doc_address.doc_id {
                    return Ok(Some(postings.term_freq() as f32));
                }
            }
        }

        Ok(Some(0.0))
    }

    async fn clear(&mut self) -> Result<()> {
        if let Some(ref manager) = self.index_manager {
            let mut writer = manager.writer()?;
            writer
                .commit()
                .map_err(|e: tantivy::TantivyError| Bm25Error::IndexCommitFailed(e.to_string()))?;
        }
        Ok(())
    }

    async fn delete_doc_stats(&mut self, doc_id: &str) -> Result<()> {
        if let Some(ref store) = self.stats_store {
            store.delete_doc_stats(doc_id).await?;
        }
        Ok(())
    }

    async fn info(&self) -> Result<StorageInfo> {
        let total_docs = if let Some(ref manager) = self.index_manager {
            if let Ok(reader) = manager.reader() {
                reader.searcher().num_docs() as usize
            } else {
                0
            }
        } else {
            0
        };

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
        Ok(self.index_manager.is_some())
    }
}
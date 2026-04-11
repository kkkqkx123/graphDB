use std::sync::Arc;

use async_trait::async_trait;

use super::error::{ExternalIndexError, IndexResult};
use super::trait_def::{IndexData, IndexStats, ExternalIndexClient};

pub struct FulltextClient {
    space_id: u64,
    tag_name: String,
    field_name: String,
    search_engine: Arc<dyn crate::search::engine::SearchEngine>,
}

impl std::fmt::Debug for FulltextClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FulltextClient")
            .field("space_id", &self.space_id)
            .field("tag_name", &self.tag_name)
            .field("field_name", &self.field_name)
            .finish()
    }
}

impl FulltextClient {
    pub fn new(
        space_id: u64,
        tag_name: String,
        field_name: String,
        search_engine: Arc<dyn crate::search::engine::SearchEngine>,
    ) -> Self {
        Self {
            space_id,
            tag_name,
            field_name,
            search_engine,
        }
    }
}

#[async_trait]
impl ExternalIndexClient for FulltextClient {
    fn client_type(&self) -> &'static str {
        "fulltext"
    }

    fn index_key(&self) -> (u64, String, String) {
        (self.space_id, self.tag_name.clone(), self.field_name.clone())
    }

    async fn insert(&self, id: &str, data: &IndexData) -> IndexResult<()> {
        if let IndexData::Fulltext(text) = data {
            self.search_engine
                .index(id, text)
                .await
                .map_err(|e| ExternalIndexError::InsertError(e.to_string()))
        } else {
            Err(ExternalIndexError::InvalidData(
                "Expected fulltext data".to_string(),
            ))
        }
    }

    async fn insert_batch(&self, items: Vec<(String, IndexData)>) -> IndexResult<()> {
        let fulltext_items: Vec<(String, String)> = items
            .into_iter()
            .filter_map(|(id, data)| {
                if let IndexData::Fulltext(text) = data {
                    Some((id, text))
                } else {
                    None
                }
            })
            .collect();

        if fulltext_items.is_empty() {
            return Ok(());
        }

        self.search_engine
            .index_batch(fulltext_items)
            .await
            .map_err(|e| ExternalIndexError::InsertError(e.to_string()))
    }

    async fn delete(&self, id: &str) -> IndexResult<()> {
        self.search_engine
            .delete(id)
            .await
            .map_err(|e| ExternalIndexError::DeleteError(e.to_string()))
    }

    async fn delete_batch(&self, ids: &[&str]) -> IndexResult<()> {
        let id_vec: Vec<&str> = ids.to_vec();
        self.search_engine
            .delete_batch(id_vec)
            .await
            .map_err(|e| ExternalIndexError::DeleteError(e.to_string()))
    }

    async fn commit(&self) -> IndexResult<()> {
        self.search_engine
            .commit()
            .await
            .map_err(|e| ExternalIndexError::CommitError(e.to_string()))
    }

    async fn rollback(&self) -> IndexResult<()> {
        self.search_engine
            .rollback()
            .await
            .map_err(|e| ExternalIndexError::RollbackError(e.to_string()))
    }

    async fn stats(&self) -> IndexResult<IndexStats> {
        let stats = self
            .search_engine
            .stats()
            .await
            .map_err(|e| ExternalIndexError::StatsError(e.to_string()))?;

        Ok(IndexStats {
            doc_count: stats.doc_count,
            index_size_bytes: stats.index_size,
            last_commit_time: None,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

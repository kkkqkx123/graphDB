//! File-based storage implementation
//!
//! Provides persistent storage using the local filesystem.
//! Data is serialized to disk using postcard format.

use crate::error::{Result, StorageError};
use crate::r#type::{DocId, EnrichedSearchResult, EnrichedSearchResults, SearchResults};
use crate::storage::common::io::{atomic_write, load_from_file};
use crate::storage::common::types::FileStorageData;
use crate::storage::common::StorageInfo;
use crate::storage::common::StorageInterface;
use crate::Index;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;

pub struct FileStorage {
    path: PathBuf,
    data: RwLock<FileStorageData>,
}

impl FileStorage {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            data: RwLock::new(FileStorageData {
                version: "1.0".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                data: HashMap::new(),
                context_data: HashMap::new(),
                documents: HashMap::new(),
            }),
        }
    }

    pub fn get_file_size(&self) -> u64 {
        std::fs::metadata(&self.path).map(|m| m.len()).unwrap_or(0)
    }
}

#[async_trait::async_trait]
impl StorageInterface for FileStorage {
    async fn open(&self) -> Result<()> {
        if self.path.exists() {
            let file_data = load_from_file(&self.path).await?;
            let mut data = self.data.write().await;
            *data = file_data;
        } else {
            if let Some(parent) = self.path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| StorageError::Generic(e.to_string()))?;
            }
        }
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        let data = self.data.read().await;
        let bytes = postcard::to_allocvec(&*data)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        atomic_write(&self.path, &bytes).await
    }

    async fn mount(&self, _index: &Index) -> Result<()> {
        Ok(())
    }

    async fn commit(&self, index: &Index, _replace: bool, _append: bool) -> Result<()> {
        let mut data = self.data.write().await;

        for (term, doc_ids) in index.map.index.values().flat_map(|m| m.iter()) {
            data.data.insert(term.clone(), doc_ids.clone());
        }
        for (id, doc) in &index.documents {
            data.documents.insert(*id, doc.clone());
        }
        data.timestamp = chrono::Utc::now().to_rfc3339();

        Ok(())
    }

    async fn get(
        &self,
        key: &str,
        ctx: Option<&str>,
        _limit: usize,
        _offset: usize,
        _resolve: bool,
        _enrich: bool,
    ) -> Result<SearchResults> {
        let data = self.data.read().await;

        if let Some(ctx_name) = ctx {
            if let Some(ctx_map) = data.context_data.get(ctx_name) {
                if let Some(doc_ids) = ctx_map.get(key) {
                    return Ok(doc_ids.clone());
                }
            }
            return Ok(Vec::new());
        }

        Ok(data.data.get(key).cloned().unwrap_or_default())
    }

    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults> {
        let data = self.data.read().await;

        let results: EnrichedSearchResults = ids
            .iter()
            .filter_map(|id| {
                data.documents.get(id).map(|content| EnrichedSearchResult {
                    id: *id,
                    doc: Some(serde_json::Value::String(content.clone())),
                    highlight: None,
                })
            })
            .collect();

        Ok(results)
    }

    async fn has(&self, id: DocId) -> Result<bool> {
        let data = self.data.read().await;
        Ok(data.documents.contains_key(&id)
            || data.data.values().any(|ids| ids.contains(&id)))
    }

    async fn remove(&self, ids: &[DocId]) -> Result<()> {
        let mut data = self.data.write().await;

        for id in ids {
            data.documents.remove(id);
            for doc_ids in data.data.values_mut() {
                doc_ids.retain(|&doc_id| doc_id != *id);
            }
            for ctx_map in data.context_data.values_mut() {
                for doc_ids in ctx_map.values_mut() {
                    doc_ids.retain(|&doc_id| doc_id != *id);
                }
            }
        }

        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        let mut data = self.data.write().await;
        data.data.clear();
        data.context_data.clear();
        data.documents.clear();
        Ok(())
    }

    async fn destroy(&self) -> Result<()> {
        self.clear().await?;
        if self.path.exists() {
            std::fs::remove_file(&self.path)
                .map_err(|e| StorageError::Generic(e.to_string()))?;
        }
        Ok(())
    }

    async fn info(&self) -> Result<StorageInfo> {
        let data = self.data.read().await;
        Ok(StorageInfo {
            name: "FileStorage".to_string(),
            version: "1.0".to_string(),
            size: self.get_file_size(),
            document_count: data.documents.len(),
            index_count: data.data.len(),
            is_connected: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::IndexOptions;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_storage_basic() {
        let temp_dir = TempDir::new().expect("create temp dir should succeed");
        let file_path = temp_dir.path().join("index.bin");
        let storage = FileStorage::new(&file_path);

        storage.open().await.expect("open should succeed");

        let mut index = Index::new(IndexOptions::default()).expect("create index should succeed");
        index
            .add(1, "hello world", false)
            .expect("add should succeed");
        index
            .add(2, "rust programming", false)
            .expect("add should succeed");

        storage
            .commit(&index, false, false)
            .await
            .expect("commit should succeed");

        let results = storage
            .get("hello", None, 10, 0, true, false)
            .await
            .expect("get should succeed");
        assert_eq!(results.len(), 1);
        assert!(results.contains(&1));

        storage.close().await.expect("close should succeed");
    }

    #[tokio::test]
    async fn test_file_storage_persistence() {
        let temp_dir = TempDir::new().expect("create temp dir should succeed");
        let file_path = temp_dir.path().join("persist.bin");

        {
            let storage = FileStorage::new(&file_path);
            storage.open().await.expect("open should succeed");

            let mut index = Index::new(IndexOptions::default()).expect("create index should succeed");
            index
                .add(1, "persistent data", false)
                .expect("add should succeed");
            storage
                .commit(&index, false, false)
                .await
                .expect("commit should succeed");

            storage.close().await.expect("close should succeed");
        }

        {
            let storage = FileStorage::new(&file_path);
            storage.open().await.expect("open should succeed");

            let results = storage
                .get("persistent", None, 10, 0, true, false)
                .await
                .expect("get should succeed");
            assert_eq!(results.len(), 1);
            assert!(results.contains(&1));

            storage.close().await.expect("close should succeed");
        }
    }

    #[tokio::test]
    async fn test_file_storage_size() {
        let temp_dir = TempDir::new().expect("create temp dir should succeed");
        let file_path = temp_dir.path().join("size_test.bin");
        let storage = FileStorage::new(&file_path);

        storage.open().await.expect("open should succeed");

        let mut index = Index::new(IndexOptions::default()).expect("create index should succeed");
        index
            .add(1, "test content", false)
            .expect("add should succeed");
        storage
            .commit(&index, false, false)
            .await
            .expect("commit should succeed");

        storage.close().await.expect("close should succeed");

        let size = storage.get_file_size();
        assert!(size > 0, "File size should be positive");
    }
}
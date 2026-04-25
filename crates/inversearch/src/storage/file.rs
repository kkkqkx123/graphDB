//! File Storage Implementation
//!
//! Provides a file-based persistent storage backend

use crate::error::Result;
use crate::r#type::{DocId, EnrichedSearchResults, SearchResults};
use crate::storage::common::base::StorageBase;
use crate::storage::common::io::{load_from_file, save_to_file};
use crate::storage::common::{FileStorageData, StorageInfo, StorageInterface, StorageMetrics};
use crate::Index;
use std::path::PathBuf;
use tokio::sync::RwLock;

/// File Storage
pub struct FileStorage {
    base: RwLock<StorageBase>,
    base_path: PathBuf,
    is_open: RwLock<bool>,
}

impl FileStorage {
    /// Creating a new file store
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base: RwLock::new(StorageBase::new()),
            base_path: base_path.into(),
            is_open: RwLock::new(false),
        }
    }

    /// Getting Memory Usage
    pub fn get_memory_usage(&self) -> usize {
        self.base.blocking_read().get_memory_usage()
    }

    /// Get Operation Statistics
    pub fn get_operation_stats(&self) -> StorageMetrics {
        let base = self.base.blocking_read();
        StorageMetrics {
            operation_count: base.get_operation_count(),
            average_latency: base.get_average_latency(),
            memory_usage: base.get_memory_usage(),
            error_count: 0,
        }
    }

    /// Get file size
    pub fn get_file_size(&self) -> u64 {
        let data_file = self.base_path.join("data.bin");
        crate::storage::common::io::get_file_size(&data_file)
    }

    /// Save to file
    pub async fn save_to_file(&self) -> Result<()> {
        let data = {
            let base = self.base.read().await;
            FileStorageData {
                version: "1.0.0".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                data: base.data.clone(),
                context_data: base.context_data.clone(),
                documents: base.documents.clone(),
            }
        };

        let data_file = self.base_path.join("data.bin");
        save_to_file(&data_file, &data).await
    }

    /// Load from file
    pub async fn load_from_file(&self) -> Result<()> {
        let data_file = self.base_path.join("data.bin");
        let data = load_from_file(&data_file).await?;

        let mut base = self.base.write().await;
        base.data = data.data;
        base.context_data = data.context_data;
        base.documents = data.documents;
        base.update_memory_usage();

        Ok(())
    }
}

#[async_trait::async_trait]
impl StorageInterface for FileStorage {
    async fn mount(&self, _index: &Index) -> Result<()> {
        tokio::fs::create_dir_all(&self.base_path).await?;

        if let Err(e) = self.load_from_file().await {
            eprintln!("Failed to load from file: {}", e);
        }
        Ok(())
    }

    async fn open(&self) -> Result<()> {
        *self.is_open.write().await = true;
        self.load_from_file().await
    }

    async fn close(&self) -> Result<()> {
        self.save_to_file().await?;
        *self.is_open.write().await = false;
        Ok(())
    }

    async fn destroy(&self) -> Result<()> {
        let mut base = self.base.write().await;
        base.clear();

        let data_file = self.base_path.join("data.bin");
        crate::storage::common::io::remove_file_safe(&data_file).await?;

        base.update_memory_usage();
        *self.is_open.write().await = false;
        Ok(())
    }

    async fn commit(&self, index: &Index, _replace: bool, _append: bool) -> Result<()> {
        {
            let mut base = self.base.write().await;
            let start_time = base.record_operation_start();
            base.commit_from_index(index);
            base.update_memory_usage();
            base.record_operation_completion(start_time);
        }
        self.save_to_file().await?;
        Ok(())
    }

    async fn get(
        &self,
        key: &str,
        ctx: Option<&str>,
        limit: usize,
        offset: usize,
        _resolve: bool,
        _enrich: bool,
    ) -> Result<SearchResults> {
        let base = self.base.read().await;
        let start_time = base.record_operation_start();
        let results = base.get(key, ctx, limit, offset);
        base.record_operation_completion(start_time);
        Ok(results)
    }

    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults> {
        let base = self.base.read().await;
        let start_time = base.record_operation_start();
        let results = base.enrich(ids);
        base.record_operation_completion(start_time);
        Ok(results)
    }

    async fn has(&self, id: DocId) -> Result<bool> {
        let base = self.base.read().await;
        let start_time = base.record_operation_start();
        let result = base.has(id);
        base.record_operation_completion(start_time);
        Ok(result)
    }

    async fn remove(&self, ids: &[DocId]) -> Result<()> {
        {
            let mut base = self.base.write().await;
            let start_time = base.record_operation_start();
            base.remove(ids);
            base.update_memory_usage();
            base.record_operation_completion(start_time);
        }
        self.save_to_file().await?;
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        {
            let mut base = self.base.write().await;
            let start_time = base.record_operation_start();
            base.clear();
            base.update_memory_usage();
            base.record_operation_completion(start_time);
        }
        self.save_to_file().await?;
        Ok(())
    }

    async fn info(&self) -> Result<StorageInfo> {
        let base = self.base.read().await;
        Ok(StorageInfo {
            name: "FileStorage".to_string(),
            version: "1.0.0".to_string(),
            size: self.get_file_size(),
            document_count: base.get_document_count(),
            index_count: base.get_index_count(),
            is_connected: *self.is_open.read().await,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Index;

    #[tokio::test]
    async fn test_file_storage() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        let storage = FileStorage::new(dir_path.to_str().unwrap().to_string());
        storage.open().await.unwrap();

        let mut index = Index::default();
        index.add(1, "test document", false).unwrap();
        index.add(2, "another test", false).unwrap();

        // Commit to Storage
        storage.commit(&index, false, false).await.unwrap();

        // Test Acquisition
        let results = storage.get("test", None, 10, 0, true, false).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.contains(&1));
        assert!(results.contains(&2));

        // Turn off storage (will save to file)
        storage.close().await.unwrap();

        // Reopen and verify that the data is still there
        let storage2 = FileStorage::new(dir_path.to_str().unwrap().to_string());
        storage2.open().await.unwrap();

        let results2 = storage2
            .get("test", None, 10, 0, true, false)
            .await
            .unwrap();
        assert_eq!(results2.len(), 2);

        storage2.destroy().await.unwrap();
    }
}

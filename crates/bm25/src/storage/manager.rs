use crate::error::Result;
use crate::storage::common::r#trait::StorageInterface;
use crate::storage::common::types::{Bm25Stats, StorageInfo};
use crate::storage::tantivy::{TantivyStorage, TantivyStorageConfig};
use std::sync::Arc;
use tokio::sync::RwLock;

pub type DefaultStorage = TantivyStorage;

#[derive(Clone)]
pub struct StorageManager {
    storage: Arc<DefaultStorage>,
}

impl StorageManager {
    pub fn new(storage: Arc<DefaultStorage>) -> Self {
        Self { storage }
    }

    pub fn storage(&self) -> Arc<DefaultStorage> {
        self.storage.clone()
    }

    pub async fn get_stats(&self, term: &str) -> Result<Option<Bm25Stats>> {
        self.storage.get_stats(term).await
    }

    pub async fn get_df(&self, term: &str) -> Result<Option<u64>> {
        self.storage.get_df(term).await
    }

    pub async fn get_tf(&self, term: &str, doc_id: &str) -> Result<Option<f32>> {
        self.storage.get_tf(term, doc_id).await
    }

    pub async fn info(&self) -> Result<StorageInfo> {
        self.storage.info().await
    }

    pub async fn health_check(&self) -> Result<bool> {
        self.storage.health_check().await
    }
}

pub struct MutableStorageManager {
    storage: Arc<RwLock<DefaultStorage>>,
}

impl Clone for MutableStorageManager {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
        }
    }
}

impl MutableStorageManager {
    pub fn new(storage: DefaultStorage) -> Self {
        Self {
            storage: Arc::new(RwLock::new(storage)),
        }
    }

    pub fn from_arc(storage: Arc<DefaultStorage>) -> Self {
        Self {
            storage: Arc::new(RwLock::new(
                Arc::try_unwrap(storage).unwrap_or_else(|_| {
                    panic!("Cannot create MutableStorageManager from shared Arc<TantivyStorage>")
                }),
            )),
        }
    }

    pub fn storage_arc(&self) -> Arc<RwLock<DefaultStorage>> {
        self.storage.clone()
    }

    pub async fn init(&self) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.init().await
    }

    pub async fn commit_stats(&self, term: &str, tf: f32, df: u64) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.commit_stats(term, tf, df).await
    }

    pub async fn commit_batch(&self, stats: &Bm25Stats) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.commit_batch(stats).await
    }

    pub async fn get_stats(&self, term: &str) -> Result<Option<Bm25Stats>> {
        let storage = self.storage.read().await;
        storage.get_stats(term).await
    }

    pub async fn get_df(&self, term: &str) -> Result<Option<u64>> {
        let storage = self.storage.read().await;
        storage.get_df(term).await
    }

    pub async fn get_tf(&self, term: &str, doc_id: &str) -> Result<Option<f32>> {
        let storage = self.storage.read().await;
        storage.get_tf(term, doc_id).await
    }

    pub async fn clear(&self) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.clear().await
    }

    pub async fn info(&self) -> Result<StorageInfo> {
        let storage = self.storage.read().await;
        storage.info().await
    }

    pub async fn health_check(&self) -> Result<bool> {
        let storage = self.storage.read().await;
        storage.health_check().await
    }

    pub async fn delete_doc_stats(&self, doc_id: &str) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.delete_doc_stats(doc_id).await
    }

    pub async fn close(&self) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.close().await
    }
}

pub struct StorageManagerBuilder;

impl StorageManagerBuilder {
    pub fn build_tantivy(config: TantivyStorageConfig) -> Result<StorageManager> {
        let storage = TantivyStorage::new(config);
        Ok(StorageManager::new(Arc::new(storage)))
    }

    pub fn build_mutable_tantivy(config: TantivyStorageConfig) -> Result<MutableStorageManager> {
        let storage = TantivyStorage::new(config);
        Ok(MutableStorageManager::new(storage))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_storage_manager_creation() {
        let dir = tempdir().expect("Failed to create temp dir");
        let config = TantivyStorageConfig {
            index_path: dir.path().to_path_buf(),
            writer_memory_mb: 50,
        };
        let manager = StorageManagerBuilder::build_tantivy(config);
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_mutable_storage_manager() {
        let dir = tempdir().expect("Failed to create temp dir");
        let config = TantivyStorageConfig {
            index_path: dir.path().to_path_buf(),
            writer_memory_mb: 50,
        };
        let manager = StorageManagerBuilder::build_mutable_tantivy(config).unwrap();

        assert!(manager.init().await.is_ok());
        assert!(manager.health_check().await.unwrap_or(false));
        assert!(manager.clear().await.is_ok());
    }
}

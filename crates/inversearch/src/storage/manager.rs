use crate::error::Result;
use crate::storage::common::types::StorageInfo;
use crate::storage::common::StorageInterface;
use crate::{DocId, Index};
use std::sync::Arc;

#[derive(Clone)]
pub struct StorageManager {
    storage: Arc<dyn StorageInterface>,
}

impl StorageManager {
    pub fn new(storage: Arc<dyn StorageInterface>) -> Self {
        Self { storage }
    }

    pub fn storage(&self) -> Arc<dyn StorageInterface> {
        self.storage.clone()
    }

    pub async fn open(&self) -> Result<()> {
        self.storage.open().await
    }

    pub async fn close(&self) -> Result<()> {
        self.storage.close().await
    }

    pub async fn mount(&self, index: &Index) -> Result<()> {
        self.storage.mount(index).await
    }

    pub async fn commit(&self, index: &Index, replace: bool, append: bool) -> Result<()> {
        self.storage.commit(index, replace, append).await
    }

    pub async fn get(
        &self,
        key: &str,
        ctx: Option<&str>,
        limit: usize,
        offset: usize,
        resolve: bool,
        enrich: bool,
    ) -> Result<crate::r#type::SearchResults> {
        self.storage.get(key, ctx, limit, offset, resolve, enrich).await
    }

    pub async fn enrich(&self, ids: &[DocId]) -> Result<crate::r#type::EnrichedSearchResults> {
        self.storage.enrich(ids).await
    }

    pub async fn has(&self, id: DocId) -> Result<bool> {
        self.storage.has(id).await
    }

    pub async fn remove(&self, ids: &[DocId]) -> Result<()> {
        self.storage.remove(ids).await
    }

    pub async fn remove_documents(&self, ids: &[DocId]) -> Result<()> {
        self.storage.remove(ids).await
    }

    pub async fn mount_index(&self, index: &Index) -> Result<()> {
        self.storage.mount(index).await
    }

    pub async fn clear(&self) -> Result<()> {
        self.storage.clear().await
    }

    pub async fn destroy(&self) -> Result<()> {
        self.storage.destroy().await
    }

    pub async fn info(&self) -> Result<StorageInfo> {
        self.storage.info().await
    }
}

pub struct StorageManagerBuilder;

impl StorageManagerBuilder {
    pub async fn build_default() -> Result<StorageManager> {
        let storage = crate::storage::memory::MemoryStorage::new();
        Ok(StorageManager::new(Arc::new(storage)))
    }

    pub async fn build_from_config(config: &crate::config::Config) -> Result<StorageManager> {
        let storage = crate::storage::factory::StorageFactory::from_config(config).await?;
        Ok(StorageManager::new(storage))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_manager_creation() {
        let manager = StorageManagerBuilder::build_default().await;
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        assert!(manager.open().await.is_ok());

        let info = manager.info().await.unwrap();
        assert!(!info.name.is_empty());
    }

    #[tokio::test]
    async fn test_storage_operations() {
        let manager = StorageManagerBuilder::build_default().await.unwrap();

        assert!(manager.has(1).await.is_ok());
        assert!(manager.remove(&[1, 2, 3]).await.is_ok());
        assert!(manager.clear().await.is_ok());

        let results = manager.get("test", None, 10, 0, true, false).await;
        assert!(results.is_ok());
    }
}
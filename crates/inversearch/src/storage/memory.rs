use crate::error::Result;
use crate::r#type::{DocId, EnrichedSearchResults, SearchResults};
use crate::storage::common::base::StorageBase;
use crate::storage::common::{StorageInfo, StorageInterface};
use crate::Index;
use tokio::sync::RwLock;

pub struct MemoryStorage {
    base: RwLock<StorageBase>,
    is_open: RwLock<bool>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            base: RwLock::new(StorageBase::new()),
            is_open: RwLock::new(false),
        }
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl StorageInterface for MemoryStorage {
    async fn mount(&self, _index: &Index) -> Result<()> {
        Ok(())
    }

    async fn open(&self) -> Result<()> {
        *self.is_open.write().await = true;
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        *self.is_open.write().await = false;
        Ok(())
    }

    async fn destroy(&self) -> Result<()> {
        let mut base = self.base.write().await;
        base.clear();
        *self.is_open.write().await = false;
        Ok(())
    }

    async fn commit(&self, index: &Index, _replace: bool, _append: bool) -> Result<()> {
        let mut base = self.base.write().await;
        base.commit_from_index(index);
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
        let results = base.get(key, ctx, limit, offset);
        Ok(results)
    }

    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults> {
        let base = self.base.read().await;
        let results = base.enrich(ids);
        Ok(results)
    }

    async fn has(&self, id: DocId) -> Result<bool> {
        let base = self.base.read().await;
        let result = base.has(id);
        Ok(result)
    }

    async fn remove(&self, ids: &[DocId]) -> Result<()> {
        let mut base = self.base.write().await;
        base.remove(ids);
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        let mut base = self.base.write().await;
        base.clear();
        Ok(())
    }

    async fn info(&self) -> Result<StorageInfo> {
        let base = self.base.read().await;
        Ok(StorageInfo {
            name: "MemoryStorage".to_string(),
            version: "0.1.0".to_string(),
            size: (base.get_index_count() + base.get_document_count()) as u64,
            document_count: base.get_document_count(),
            index_count: base.get_index_count(),
            is_connected: *self.is_open.read().await,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_storage() {
        let storage = MemoryStorage::new();
        storage.open().await.unwrap();

        let mut index = Index::default();
        index.add(1, "hello world", false).unwrap();
        index.add(2, "rust programming", false).unwrap();

        storage.commit(&index, false, false).await.unwrap();

        let results = storage
            .get("hello", None, 10, 0, true, false)
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert!(results.contains(&1));

        let has_result = storage.has(1).await.unwrap();
        assert!(has_result);
        assert!(!storage.has(3).await.unwrap());

        storage.remove(&[1]).await.unwrap();
        assert!(!storage.has(1).await.unwrap());

        storage.close().await.unwrap();
    }
}

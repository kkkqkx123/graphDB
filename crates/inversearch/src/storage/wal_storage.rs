//! WAL 存储实现
//!
//! 提供基于预写日志的持久化存储后端

use crate::r#type::{SearchResults, EnrichedSearchResults, DocId};
use crate::error::Result;
use crate::Index;
use crate::storage::common::{StorageInterface, StorageInfo};
use crate::storage::wal::{WALManager, WALConfig, IndexChange};
use std::collections::HashMap;

/// WAL 存储
pub struct WALStorage {
    wal_manager: WALManager,
    documents: HashMap<DocId, String>,
    is_open: bool,
}

impl WALStorage {
    /// 创建新的 WAL 存储
    pub async fn new(config: WALConfig) -> Result<Self> {
        let wal_manager = WALManager::new(config).await?;

        Ok(Self {
            wal_manager,
            documents: HashMap::new(),
            is_open: false,
        })
    }

    /// 创建快照
    pub async fn create_snapshot(&self, index: &Index) -> Result<()> {
        self.wal_manager.create_snapshot(index).await
    }
}

#[async_trait::async_trait]
impl StorageInterface for WALStorage {
    async fn mount(&mut self, _index: &Index) -> Result<()> {
        Ok(())
    }

    async fn open(&mut self) -> Result<()> {
        self.is_open = true;
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        self.is_open = false;
        Ok(())
    }

    async fn destroy(&mut self) -> Result<()> {
        self.documents.clear();
        self.wal_manager.clear().await?;
        self.is_open = false;
        Ok(())
    }

    async fn commit(&mut self, index: &Index, _replace: bool, _append: bool) -> Result<()> {
        // 使用 WAL 创建快照
        self.wal_manager.create_snapshot(index).await
    }

    async fn get(&self, _key: &str, _ctx: Option<&str>, _limit: usize, _offset: usize, _resolve: bool, _enrich: bool) -> Result<SearchResults> {
        // WAL 存储需要通过加载索引来获取数据
        // 这里简化处理，返回空结果
        // 实际应用中应该维护一个内存索引
        Ok(Vec::new())
    }

    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults> {
        let mut results = Vec::new();

        for &id in ids {
            if let Some(content) = self.documents.get(&id) {
                results.push(crate::r#type::EnrichedSearchResult {
                    id,
                    doc: Some(serde_json::json!({
                        "content": content,
                        "id": id
                    })),
                    highlight: None,
                });
            }
        }

        Ok(results)
    }

    async fn has(&self, id: DocId) -> Result<bool> {
        Ok(self.documents.contains_key(&id))
    }

    async fn remove(&mut self, ids: &[DocId]) -> Result<()> {
        for &id in ids {
            self.documents.remove(&id);
            self.wal_manager.record_change(IndexChange::Remove { doc_id: id }).await?;
        }
        Ok(())
    }

    async fn clear(&mut self) -> Result<()> {
        self.documents.clear();
        self.wal_manager.clear().await?;
        Ok(())
    }

    async fn info(&self) -> Result<StorageInfo> {
        let wal_size = self.wal_manager.wal_size() as u64;
        let snapshot_size = self.wal_manager.snapshot_size().await?;

        Ok(StorageInfo {
            name: "WALStorage".to_string(),
            version: "0.1.0".to_string(),
            size: wal_size + snapshot_size,
            document_count: self.documents.len(),
            index_count: 0,
            is_connected: self.is_open,
        })
    }
}

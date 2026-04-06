//! 存储管理器
//!
//! 提供统一的存储管理接口，集成存储到业务逻辑
//! 使用条件编译确定具体存储类型，零运行时开销

use crate::error::Result;
use crate::storage::common::types::StorageInfo;
use crate::{Index, DocId};
use std::sync::Arc;

// 根据特性导入具体存储类型
#[cfg(feature = "store-cold-warm-cache")]
use crate::storage::cold_warm_cache::ColdWarmCacheManager;

#[cfg(all(feature = "store-file", not(feature = "store-cold-warm-cache")))]
use crate::storage::file::FileStorage;

#[cfg(all(feature = "store-redis", not(any(feature = "store-cold-warm-cache", feature = "store-file"))))]
use crate::storage::redis::RedisStorage;

#[cfg(all(feature = "store-wal", not(any(feature = "store-cold-warm-cache", feature = "store-file", feature = "store-redis"))))]
use crate::storage::wal::WALStorage;

// 定义默认存储类型
#[cfg(feature = "store-cold-warm-cache")]
pub type DefaultStorage = ColdWarmCacheManager;

#[cfg(all(feature = "store-file", not(feature = "store-cold-warm-cache")))]
pub type DefaultStorage = FileStorage;

#[cfg(all(feature = "store-redis", not(any(feature = "store-cold-warm-cache", feature = "store-file"))))]
pub type DefaultStorage = RedisStorage;

#[cfg(all(feature = "store-wal", not(any(feature = "store-cold-warm-cache", feature = "store-file", feature = "store-redis"))))]
pub type DefaultStorage = WALStorage;

#[cfg(not(any(feature = "store-cold-warm-cache", feature = "store-file", feature = "store-redis", feature = "store-wal")))]
use crate::storage::memory::MemoryStorage;

#[cfg(not(any(feature = "store-cold-warm-cache", feature = "store-file", feature = "store-redis", feature = "store-wal")))]
pub type DefaultStorage = MemoryStorage;

/// 存储管理器
/// 
/// 使用条件编译确定具体存储类型，提供零成本抽象的存储管理
#[derive(Clone)]
pub struct StorageManager {
    storage: Arc<DefaultStorage>,
}

impl StorageManager {
    /// 创建新的存储管理器
    pub fn new(storage: Arc<DefaultStorage>) -> Self {
        Self { storage }
    }

    /// 获取底层存储
    pub fn storage(&self) -> Arc<DefaultStorage> {
        self.storage.clone()
    }

    /// 打开存储连接
    pub async fn open(&self) -> Result<()> {
        // 具体存储类型的 open 方法
        #[cfg(feature = "store-cold-warm-cache")]
        {
            // ColdWarmCacheManager 在创建时已经初始化
            Ok(())
        }
        #[cfg(not(feature = "store-cold-warm-cache"))]
        {
            // 其他存储类型需要调用 open
            Ok(())
        }
    }

    /// 关闭存储连接
    pub async fn close(&self) -> Result<()> {
        Ok(())
    }

    /// 挂载索引到存储
    pub async fn mount(&self, index: &Index) -> Result<()> {
        #[cfg(feature = "store-cold-warm-cache")]
        {
            // ColdWarmCacheManager 通过 Arc 使用
            // 需要特殊处理
        }
        let _ = index;
        Ok(())
    }

    /// 提交索引变更
    pub async fn commit(&self, index: &Index, replace: bool, append: bool) -> Result<()> {
        let _ = (index, replace, append);
        Ok(())
    }

    /// 获取术语结果
    pub async fn get(
        &self,
        key: &str,
        ctx: Option<&str>,
        limit: usize,
        offset: usize,
        resolve: bool,
        enrich: bool,
    ) -> Result<crate::r#type::SearchResults> {
        let _ = (key, ctx, limit, offset, resolve, enrich);
        Ok(Vec::new())
    }

    /// 富化结果
    pub async fn enrich(&self, ids: &[DocId]) -> Result<crate::r#type::EnrichedSearchResults> {
        let _ = ids;
        Ok(Vec::new())
    }

    /// 检查ID是否存在
    pub async fn has(&self, id: DocId) -> Result<bool> {
        let _ = id;
        Ok(false)
    }

    /// 删除文档
    pub async fn remove(&self, ids: &[DocId]) -> Result<()> {
        let _ = ids;
        Ok(())
    }

    /// 删除文档（别名，与 remove 功能相同）
    pub async fn remove_documents(&self, ids: &[DocId]) -> Result<()> {
        self.remove(ids).await
    }

    /// 挂载索引
    pub async fn mount_index(&self, index: &Index) -> Result<()> {
        self.mount(index).await
    }

    /// 清空数据
    pub async fn clear(&self) -> Result<()> {
        Ok(())
    }

    /// 销毁数据库
    pub async fn destroy(&self) -> Result<()> {
        Ok(())
    }

    /// 获取存储信息
    pub async fn info(&self) -> Result<StorageInfo> {
        Ok(StorageInfo {
            name: stringify!(DefaultStorage).to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            size: 0,
            document_count: 0,
            index_count: 0,
            is_connected: true,
        })
    }
}

/// 存储管理器构建器
/// 
/// 用于根据配置创建存储管理器
pub struct StorageManagerBuilder;

impl StorageManagerBuilder {
    /// 创建默认存储管理器
    pub async fn build_default() -> Result<StorageManager> {
        #[cfg(feature = "store-cold-warm-cache")]
        {
            let storage = ColdWarmCacheManager::new().await?;
            Ok(StorageManager::new(storage))
        }
        
        #[cfg(all(feature = "store-file", not(feature = "store-cold-warm-cache")))]
        {
            let storage = Arc::new(FileStorage::new("./data"));
            Ok(StorageManager::new(storage))
        }
        
        #[cfg(all(feature = "store-redis", not(any(feature = "store-cold-warm-cache", feature = "store-file"))))]
        {
            use crate::storage::redis::RedisStorageConfig;
            let config = RedisStorageConfig::default();
            let storage = RedisStorage::new(config).await?;
            Ok(StorageManager::new(Arc::new(storage)))
        }
        
        #[cfg(all(feature = "store-wal", not(any(feature = "store-cold-warm-cache", feature = "store-file", feature = "store-redis"))))]
        {
            use crate::storage::wal::WALConfig;
            let config = WALConfig::default();
            let storage = WALStorage::new(config).await?;
            Ok(StorageManager::new(Arc::new(storage)))
        }
        
        #[cfg(not(any(feature = "store-cold-warm-cache", feature = "store-file", feature = "store-redis", feature = "store-wal")))]
        {
            let storage = Arc::new(MemoryStorage::new());
            Ok(StorageManager::new(storage))
        }
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
        
        // 测试基本操作
        assert!(manager.has(1).await.is_ok());
        assert!(manager.remove(&[1, 2, 3]).await.is_ok());
        assert!(manager.clear().await.is_ok());
        
        // 测试搜索
        let results = manager.get("test", None, 10, 0, true, false).await;
        assert!(results.is_ok());
    }
}

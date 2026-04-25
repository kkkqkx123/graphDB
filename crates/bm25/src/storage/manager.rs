//! 存储管理器
//!
//! 提供统一的存储管理接口，集成存储到业务逻辑
//! 使用条件编译确定具体存储类型，零运行时开销

use crate::error::Result;
use crate::storage::common::r#trait::StorageInterface;
use crate::storage::common::types::{Bm25Stats, StorageInfo};
use std::sync::Arc;
use tokio::sync::RwLock;

// 根据特性导入具体存储类型
#[cfg(feature = "storage-tantivy")]
use crate::storage::tantivy::TantivyStorage;

#[cfg(all(feature = "storage-redis", not(feature = "storage-tantivy")))]
use crate::storage::redis::RedisStorage;

// 定义默认存储类型 - 当两个特性都启用时，优先使用 TantivyStorage
#[cfg(feature = "storage-tantivy")]
pub type DefaultStorage = TantivyStorage;

#[cfg(all(feature = "storage-redis", not(feature = "storage-tantivy")))]
pub type DefaultStorage = RedisStorage;

#[cfg(not(any(feature = "storage-tantivy", feature = "storage-redis")))]
compile_error!(
    "At least one storage backend must be enabled: 'storage-tantivy' or 'storage-redis'"
);

/// 存储管理器（只读操作）
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

    /// 获取词项统计
    pub async fn get_stats(&self, term: &str) -> Result<Option<Bm25Stats>> {
        self.storage.get_stats(term).await
    }

    /// 获取文档频率
    pub async fn get_df(&self, term: &str) -> Result<Option<u64>> {
        self.storage.get_df(term).await
    }

    /// 获取词项频率
    pub async fn get_tf(&self, term: &str, doc_id: &str) -> Result<Option<f32>> {
        self.storage.get_tf(term, doc_id).await
    }

    /// 获取存储信息
    pub async fn info(&self) -> Result<StorageInfo> {
        self.storage.info().await
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<bool> {
        self.storage.health_check().await
    }
}

/// 可变的存储管理器，支持修改操作
///
/// 使用条件编译确定具体存储类型
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
    /// 创建新的可变存储管理器
    pub fn new(storage: DefaultStorage) -> Self {
        Self {
            storage: Arc::new(RwLock::new(storage)),
        }
    }

    /// 从 Arc 创建可变存储管理器
    pub fn from_arc(storage: Arc<DefaultStorage>) -> Self {
        Self {
            storage: Arc::new(RwLock::new(Arc::try_unwrap(storage).unwrap_or_else(
                |_arc| {
                    // 如果 Arc 有多个引用，克隆内部数据
                    #[cfg(feature = "storage-tantivy")]
                    {
                        // TantivyStorage 需要特殊处理，这里简化处理
                        panic!(
                            "Cannot create MutableStorageManager from shared Arc<TantivyStorage>"
                        )
                    }
                    #[cfg(all(feature = "storage-redis", not(feature = "storage-tantivy")))]
                    {
                        panic!("Cannot create MutableStorageManager from shared Arc<RedisStorage>")
                    }
                },
            ))),
        }
    }

    /// 获取存储 Arc（用于共享）
    pub fn storage_arc(&self) -> Arc<RwLock<DefaultStorage>> {
        self.storage.clone()
    }

    /// 初始化存储
    pub async fn init(&self) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.init().await
    }

    /// 提交词项统计
    pub async fn commit_stats(&self, term: &str, tf: f32, df: u64) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.commit_stats(term, tf, df).await
    }

    /// 批量提交统计
    pub async fn commit_batch(&self, stats: &Bm25Stats) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.commit_batch(stats).await
    }

    /// 获取词项统计
    pub async fn get_stats(&self, term: &str) -> Result<Option<Bm25Stats>> {
        let storage = self.storage.read().await;
        storage.get_stats(term).await
    }

    /// 获取文档频率
    pub async fn get_df(&self, term: &str) -> Result<Option<u64>> {
        let storage = self.storage.read().await;
        storage.get_df(term).await
    }

    /// 获取词项频率
    pub async fn get_tf(&self, term: &str, doc_id: &str) -> Result<Option<f32>> {
        let storage = self.storage.read().await;
        storage.get_tf(term, doc_id).await
    }

    /// 清空所有数据
    pub async fn clear(&self) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.clear().await
    }

    /// 获取存储信息
    pub async fn info(&self) -> Result<StorageInfo> {
        let storage = self.storage.read().await;
        storage.info().await
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<bool> {
        let storage = self.storage.read().await;
        storage.health_check().await
    }

    /// 删除特定文档的统计信息
    pub async fn delete_doc_stats(&self, doc_id: &str) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.delete_doc_stats(doc_id).await
    }

    /// 关闭存储
    pub async fn close(&self) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.close().await
    }
}

/// 存储管理器构建器
///
/// 用于根据配置创建存储管理器
pub struct StorageManagerBuilder;

impl StorageManagerBuilder {
    /// 创建默认存储管理器（只读）
    #[cfg(feature = "storage-tantivy")]
    pub fn build_tantivy(
        config: crate::storage::tantivy::TantivyStorageConfig,
    ) -> Result<StorageManager> {
        let storage = TantivyStorage::new(config);
        Ok(StorageManager::new(Arc::new(storage)))
    }

    /// 创建默认存储管理器（只读）- 仅在 Redis 作为默认存储时可用
    #[cfg(all(feature = "storage-redis", not(feature = "storage-tantivy")))]
    pub async fn build_redis(
        config: crate::storage::redis::RedisStorageConfig,
    ) -> Result<StorageManager> {
        let storage = RedisStorage::new(config).await?;
        Ok(StorageManager::new(Arc::new(storage)))
    }

    /// 创建可变存储管理器
    #[cfg(feature = "storage-tantivy")]
    pub fn build_mutable_tantivy(
        config: crate::storage::tantivy::TantivyStorageConfig,
    ) -> Result<MutableStorageManager> {
        let storage = TantivyStorage::new(config);
        Ok(MutableStorageManager::new(storage))
    }

    /// 创建可变存储管理器 - 仅在 Redis 作为默认存储时可用
    #[cfg(all(feature = "storage-redis", not(feature = "storage-tantivy")))]
    pub async fn build_mutable_redis(
        config: crate::storage::redis::RedisStorageConfig,
    ) -> Result<MutableStorageManager> {
        let storage = RedisStorage::new(config).await?;
        Ok(MutableStorageManager::new(storage))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_manager_creation() {
        #[cfg(feature = "storage-tantivy")]
        {
            let config = crate::storage::tantivy::TantivyStorageConfig::default();
            let manager = StorageManagerBuilder::build_tantivy(config);
            assert!(manager.is_ok());
        }
    }

    #[tokio::test]
    async fn test_mutable_storage_manager() {
        #[cfg(feature = "storage-tantivy")]
        {
            let config = crate::storage::tantivy::TantivyStorageConfig::default();
            let manager = StorageManagerBuilder::build_mutable_tantivy(config).unwrap();

            assert!(manager.init().await.is_ok());
            assert!(manager.health_check().await.unwrap_or(false));
            assert!(manager.clear().await.is_ok());
        }
    }
}

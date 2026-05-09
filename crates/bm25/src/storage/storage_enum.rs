//! Storage Enum - Static dispatch alternative to dynamic dispatch
//!
//! This module provides a concrete enum type for storage implementations,
//! following the project's guideline to minimize dynamic dispatch.

use crate::error::Result;
use crate::storage::common::r#trait::{Bm25Stats, StorageInfo, StorageInterface};

#[cfg(feature = "storage-tantivy")]
use crate::storage::tantivy::TantivyStorage;

#[cfg(feature = "storage-redis")]
use crate::storage::redis::RedisStorage;

#[cfg(all(feature = "storage-tantivy", feature = "storage-redis"))]
#[derive(Debug)]
pub enum StorageEnum {
    Tantivy(TantivyStorage),
    Redis(RedisStorage),
}

#[cfg(all(feature = "storage-tantivy", not(feature = "storage-redis")))]
#[derive(Debug)]
pub enum StorageEnum {
    Tantivy(TantivyStorage),
}

#[cfg(all(not(feature = "storage-tantivy"), feature = "storage-redis"))]
#[derive(Debug)]
pub enum StorageEnum {
    Redis(RedisStorage),
}

#[cfg(any(feature = "storage-tantivy", feature = "storage-redis"))]
#[async_trait::async_trait]
impl StorageInterface for StorageEnum {
    async fn init(&mut self) -> Result<()> {
        match self {
            #[cfg(feature = "storage-tantivy")]
            StorageEnum::Tantivy(storage) => storage.init().await,
            #[cfg(feature = "storage-redis")]
            StorageEnum::Redis(storage) => storage.init().await,
        }
    }

    async fn close(&mut self) -> Result<()> {
        match self {
            #[cfg(feature = "storage-tantivy")]
            StorageEnum::Tantivy(storage) => storage.close().await,
            #[cfg(feature = "storage-redis")]
            StorageEnum::Redis(storage) => storage.close().await,
        }
    }

    async fn commit_stats(&mut self, term: &str, tf: f32, df: u64) -> Result<()> {
        match self {
            #[cfg(feature = "storage-tantivy")]
            StorageEnum::Tantivy(storage) => storage.commit_stats(term, tf, df).await,
            #[cfg(feature = "storage-redis")]
            StorageEnum::Redis(storage) => storage.commit_stats(term, tf, df).await,
        }
    }

    async fn commit_batch(&mut self, stats: &Bm25Stats) -> Result<()> {
        match self {
            #[cfg(feature = "storage-tantivy")]
            StorageEnum::Tantivy(storage) => storage.commit_batch(stats).await,
            #[cfg(feature = "storage-redis")]
            StorageEnum::Redis(storage) => storage.commit_batch(stats).await,
        }
    }

    async fn get_stats(&self, term: &str) -> Result<Option<Bm25Stats>> {
        match self {
            #[cfg(feature = "storage-tantivy")]
            StorageEnum::Tantivy(storage) => storage.get_stats(term).await,
            #[cfg(feature = "storage-redis")]
            StorageEnum::Redis(storage) => storage.get_stats(term).await,
        }
    }

    async fn get_df(&self, term: &str) -> Result<Option<u64>> {
        match self {
            #[cfg(feature = "storage-tantivy")]
            StorageEnum::Tantivy(storage) => storage.get_df(term).await,
            #[cfg(feature = "storage-redis")]
            StorageEnum::Redis(storage) => storage.get_df(term).await,
        }
    }

    async fn get_tf(&self, term: &str, doc_id: &str) -> Result<Option<f32>> {
        match self {
            #[cfg(feature = "storage-tantivy")]
            StorageEnum::Tantivy(storage) => storage.get_tf(term, doc_id).await,
            #[cfg(feature = "storage-redis")]
            StorageEnum::Redis(storage) => storage.get_tf(term, doc_id).await,
        }
    }

    async fn clear(&mut self) -> Result<()> {
        match self {
            #[cfg(feature = "storage-tantivy")]
            StorageEnum::Tantivy(storage) => storage.clear().await,
            #[cfg(feature = "storage-redis")]
            StorageEnum::Redis(storage) => storage.clear().await,
        }
    }

    async fn delete_doc_stats(&mut self, doc_id: &str) -> Result<()> {
        match self {
            #[cfg(feature = "storage-tantivy")]
            StorageEnum::Tantivy(storage) => storage.delete_doc_stats(doc_id).await,
            #[cfg(feature = "storage-redis")]
            StorageEnum::Redis(storage) => storage.delete_doc_stats(doc_id).await,
        }
    }

    async fn info(&self) -> Result<StorageInfo> {
        match self {
            #[cfg(feature = "storage-tantivy")]
            StorageEnum::Tantivy(storage) => storage.info().await,
            #[cfg(feature = "storage-redis")]
            StorageEnum::Redis(storage) => storage.info().await,
        }
    }

    async fn health_check(&self) -> Result<bool> {
        match self {
            #[cfg(feature = "storage-tantivy")]
            StorageEnum::Tantivy(storage) => storage.health_check().await,
            #[cfg(feature = "storage-redis")]
            StorageEnum::Redis(storage) => storage.health_check().await,
        }
    }
}

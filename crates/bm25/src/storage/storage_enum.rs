use crate::error::Result;
use crate::storage::common::r#trait::{Bm25Stats, StorageInfo, StorageInterface};
use crate::storage::tantivy::TantivyStorage;

#[derive(Debug)]
pub enum StorageEnum {
    Tantivy(TantivyStorage),
}

#[async_trait::async_trait]
impl StorageInterface for StorageEnum {
    async fn init(&mut self) -> Result<()> {
        match self {
            StorageEnum::Tantivy(storage) => storage.init().await,
        }
    }

    async fn close(&mut self) -> Result<()> {
        match self {
            StorageEnum::Tantivy(storage) => storage.close().await,
        }
    }

    async fn commit_stats(&mut self, term: &str, tf: f32, df: u64) -> Result<()> {
        match self {
            StorageEnum::Tantivy(storage) => storage.commit_stats(term, tf, df).await,
        }
    }

    async fn commit_batch(&mut self, stats: &Bm25Stats) -> Result<()> {
        match self {
            StorageEnum::Tantivy(storage) => storage.commit_batch(stats).await,
        }
    }

    async fn get_stats(&self, term: &str) -> Result<Option<Bm25Stats>> {
        match self {
            StorageEnum::Tantivy(storage) => storage.get_stats(term).await,
        }
    }

    async fn get_df(&self, term: &str) -> Result<Option<u64>> {
        match self {
            StorageEnum::Tantivy(storage) => storage.get_df(term).await,
        }
    }

    async fn get_tf(&self, term: &str, doc_id: &str) -> Result<Option<f32>> {
        match self {
            StorageEnum::Tantivy(storage) => storage.get_tf(term, doc_id).await,
        }
    }

    async fn clear(&mut self) -> Result<()> {
        match self {
            StorageEnum::Tantivy(storage) => storage.clear().await,
        }
    }

    async fn delete_doc_stats(&mut self, doc_id: &str) -> Result<()> {
        match self {
            StorageEnum::Tantivy(storage) => storage.delete_doc_stats(doc_id).await,
        }
    }

    async fn info(&self) -> Result<StorageInfo> {
        match self {
            StorageEnum::Tantivy(storage) => storage.info().await,
        }
    }

    async fn health_check(&self) -> Result<bool> {
        match self {
            StorageEnum::Tantivy(storage) => storage.health_check().await,
        }
    }
}

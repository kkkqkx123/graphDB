//! Statistical information storage
//!
//! Persist BM25 TF/DF statistics to disk using postcard format.
//! Independent of the Tantivy index, avoiding the complexity of introducing a second Schema.

use crate::error::{Bm25Error, Result};
use crate::storage::common::Bm25Stats;
use postcard::{from_bytes, to_allocvec};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;

/// Statistical information storage
///
/// Persist BM25 statistics to a separate file in postcard format.
/// Does not rely on the Tantivy index, avoiding the complexity of introducing a second Schema.
pub struct StatsStore {
    /// Statistics file path
    path: PathBuf,
    /// In-memory statistics cache
    stats: RwLock<Bm25Stats>,
}

impl StatsStore {
    /// Create a new StatsStore
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            stats: RwLock::new(Bm25Stats::default()),
        }
    }

    /// Load statistics from disk
    pub async fn load(&self) -> Result<Bm25Stats> {
        if !self.path.exists() {
            return Ok(Bm25Stats::default());
        }
        let bytes = tokio::fs::read(&self.path).await?;
        let stats = from_bytes::<Bm25Stats>(&bytes)
            .map_err(|e| Bm25Error::StorageError(format!("Deserialization failed: {}", e)))?;
        *self.stats.write().await = stats.clone();
        Ok(stats)
    }

    /// Batch submit statistics
    pub async fn commit_batch(&self, new_stats: &Bm25Stats) -> Result<()> {
        let mut stats = self.stats.write().await;
        // Merge statistics
        for (term, df) in &new_stats.df {
            stats.df.insert(term.clone(), *df);
        }
        for (term, tf) in &new_stats.tf {
            *stats.tf.entry(term.clone()).or_insert(0.0) += tf;
        }
        stats.total_docs = new_stats.total_docs;
        stats.avg_doc_length = new_stats.avg_doc_length;
        // Persist to disk
        self.flush(&stats).await
    }

    /// Get statistics for a specific term
    pub async fn get_stats(&self, term: &str) -> Result<Option<Bm25Stats>> {
        let stats = self.stats.read().await;
        let df = stats.df.get(term).copied();
        let tf = stats.tf.get(term).copied();
        match df {
            Some(df) => Ok(Some(Bm25Stats {
                tf: tf.map(|v| HashMap::from([(term.to_string(), v)])).unwrap_or_default(),
                df: HashMap::from([(term.to_string(), df)]),
                total_docs: stats.total_docs,
                avg_doc_length: stats.avg_doc_length,
            })),
            None => Ok(None),
        }
    }

    /// Get document frequency
    pub async fn get_df(&self, term: &str) -> Result<Option<u64>> {
        let stats = self.stats.read().await;
        Ok(stats.df.get(term).copied())
    }

    /// Get term frequency
    pub async fn get_tf(&self, term: &str) -> Result<Option<f32>> {
        let stats = self.stats.read().await;
        Ok(stats.tf.get(term).copied())
    }

    /// Delete statistics for a specific document
    pub async fn delete_doc_stats(&self, _doc_id: &str) -> Result<()> {
        let mut stats = self.stats.write().await;
        // Simplified: clear all TF data (actual implementation needs to filter by doc_id)
        stats.tf.clear();
        self.flush(&stats).await
    }

    /// Flush to disk
    async fn flush(&self, stats: &Bm25Stats) -> Result<()> {
        let bytes = to_allocvec(stats)
            .map_err(|e| Bm25Error::StorageError(format!("Serialization failed: {}", e)))?;
        tokio::fs::write(&self.path, &bytes).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_stats_store_commit_and_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("stats.bin");
        let store = StatsStore::new(path.clone());

        let stats = Bm25Stats {
            tf: HashMap::from([("hello".to_string(), 3.0)]),
            df: HashMap::from([("hello".to_string(), 2)]),
            total_docs: 10,
            avg_doc_length: 50.0,
        };

        store.commit_batch(&stats).await.unwrap();

        let loaded = store.load().await.unwrap();
        assert_eq!(loaded.df.get("hello"), Some(&2));
        assert_eq!(loaded.total_docs, 10);
        assert!((loaded.avg_doc_length - 50.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_stats_store_get_df() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("stats.bin");
        let store = StatsStore::new(path);

        let stats = Bm25Stats {
            tf: HashMap::new(),
            df: HashMap::from([("world".to_string(), 5)]),
            total_docs: 20,
            avg_doc_length: 100.0,
        };

        store.commit_batch(&stats).await.unwrap();
        let df = store.get_df("world").await.unwrap();
        assert_eq!(df, Some(5));

        let df = store.get_df("nonexistent").await.unwrap();
        assert_eq!(df, None);
    }
}
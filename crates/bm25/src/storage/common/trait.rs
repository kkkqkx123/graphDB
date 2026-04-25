//! Storage interface definition
//!
//! Define the core traits and abstract interfaces of the BM25 storage module

use crate::error::Result;
use std::collections::HashMap;

/// Term statistics
#[derive(Debug, Clone, Default)]
pub struct Bm25Stats {
    /// Term Frequency
    pub tf: HashMap<String, f32>,
    /// Document Frequency
    pub df: HashMap<String, u64>,
    /// Total number of documents
    pub total_docs: u64,
    /// Average document length
    pub avg_doc_length: f32,
}

/// store information
#[derive(Debug, Clone)]
pub struct StorageInfo {
    pub name: String,
    pub version: String,
    pub size: u64,
    pub document_count: usize,
    pub term_count: usize,
    pub is_connected: bool,
}

/// Storage interface-BM25 word frequency statistical storage
#[async_trait::async_trait]
pub trait StorageInterface: Send + Sync {
    /// initialize the storage
    async fn init(&mut self) -> Result<()>;

    /// shut down the storage
    async fn close(&mut self) -> Result<()>;

    /// Submission term statistics
    async fn commit_stats(&mut self, term: &str, tf: f32, df: u64) -> Result<()>;

    /// Batch submission statistics
    async fn commit_batch(&mut self, stats: &Bm25Stats) -> Result<()>;

    /// Get term statistics
    async fn get_stats(&self, term: &str) -> Result<Option<Bm25Stats>>;

    /// Frequency of obtaining documents
    async fn get_df(&self, term: &str) -> Result<Option<u64>>;

    /// Get term frequencies
    async fn get_tf(&self, term: &str, doc_id: &str) -> Result<Option<f32>>;

    /// Clear all data
    async fn clear(&mut self) -> Result<()>;

    /// Delete statistics for specific documents
    async fn delete_doc_stats(&mut self, doc_id: &str) -> Result<()>;

    /// Get storage information
    async fn info(&self) -> Result<StorageInfo>;

    /// health check
    async fn health_check(&self) -> Result<bool>;
}

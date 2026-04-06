//! 存储接口定义
//!
//! 定义 BM25 存储模块的核心 trait 和抽象接口

use crate::error::Result;
use std::collections::HashMap;

/// 词项统计信息
#[derive(Debug, Clone, Default)]
pub struct Bm25Stats {
    /// 词项频率 (Term Frequency)
    pub tf: HashMap<String, f32>,
    /// 文档频率 (Document Frequency)
    pub df: HashMap<String, u64>,
    /// 总文档数
    pub total_docs: u64,
    /// 平均文档长度
    pub avg_doc_length: f32,
}

/// 存储信息
#[derive(Debug, Clone)]
pub struct StorageInfo {
    pub name: String,
    pub version: String,
    pub size: u64,
    pub document_count: usize,
    pub term_count: usize,
    pub is_connected: bool,
}

/// 存储接口 - BM25 词频统计存储
#[async_trait::async_trait]
pub trait StorageInterface: Send + Sync {
    /// 初始化存储
    async fn init(&mut self) -> Result<()>;

    /// 关闭存储
    async fn close(&mut self) -> Result<()>;

    /// 提交词项统计
    async fn commit_stats(&mut self, term: &str, tf: f32, df: u64) -> Result<()>;

    /// 批量提交统计
    async fn commit_batch(&mut self, stats: &Bm25Stats) -> Result<()>;

    /// 获取词项统计
    async fn get_stats(&self, term: &str) -> Result<Option<Bm25Stats>>;

    /// 获取文档频率
    async fn get_df(&self, term: &str) -> Result<Option<u64>>;

    /// 获取词项频率
    async fn get_tf(&self, term: &str, doc_id: &str) -> Result<Option<f32>>;

    /// 清空所有数据
    async fn clear(&mut self) -> Result<()>;

    /// 删除特定文档的统计信息
    async fn delete_doc_stats(&mut self, doc_id: &str) -> Result<()>;

    /// 获取存储信息
    async fn info(&self) -> Result<StorageInfo>;

    /// 健康检查
    async fn health_check(&self) -> Result<bool>;
}

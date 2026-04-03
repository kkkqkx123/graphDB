//! 存储接口定义
//!
//! 定义存储模块的核心 trait 和抽象接口

use crate::error::Result;
use crate::r#type::{DocId, EnrichedSearchResults, SearchResults};
use crate::storage::common::types::StorageInfo;
use crate::Index;

/// 存储接口 - 类似JavaScript版本的StorageInterface
#[async_trait::async_trait]
pub trait StorageInterface: Send + Sync {
    /// 挂载索引到存储
    async fn mount(&mut self, index: &Index) -> Result<()>;

    /// 打开连接
    async fn open(&mut self) -> Result<()>;

    /// 关闭连接
    async fn close(&mut self) -> Result<()>;

    /// 销毁数据库
    async fn destroy(&mut self) -> Result<()>;

    /// 提交索引变更
    async fn commit(&mut self, index: &Index, replace: bool, append: bool) -> Result<()>;

    /// 获取术语结果
    async fn get(
        &self,
        key: &str,
        ctx: Option<&str>,
        limit: usize,
        offset: usize,
        resolve: bool,
        enrich: bool,
    ) -> Result<SearchResults>;

    /// 富化结果
    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults>;

    /// 检查ID是否存在
    async fn has(&self, id: DocId) -> Result<bool>;

    /// 删除ID
    async fn remove(&mut self, ids: &[DocId]) -> Result<()>;

    /// 清空数据
    async fn clear(&mut self) -> Result<()>;

    /// 获取存储信息
    async fn info(&self) -> Result<StorageInfo>;
}

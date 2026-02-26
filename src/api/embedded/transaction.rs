//! 事务管理模块
//!
//! 提供完整的事务管理功能，包括保存点支持

use crate::api::core::{CoreError, CoreResult, QueryContext, TransactionHandle};
use crate::api::embedded::result::QueryResult;
use crate::api::embedded::session::Session;
use crate::core::Value;
use crate::storage::StorageClient;
use crate::transaction::{SavepointId, SavepointInfo, TransactionOptions, DurabilityLevel};
use std::collections::HashMap;
use std::time::Duration;

/// 事务配置选项
///
/// 用于配置事务的行为，如超时、只读模式、持久性级别等
///
/// # 示例
///
/// ```rust
/// use graphdb::api::embedded::{GraphDatabase, DatabaseConfig, TransactionConfig};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let db = GraphDatabase::open("my_db")?;
/// let session = db.session()?;
///
/// // 创建只读事务配置
/// let config = TransactionConfig::new()
///     .read_only()
///     .with_timeout(Duration::from_secs(60));
///
/// let txn = session.begin_transaction_with_config(config)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct TransactionConfig {
    /// 事务超时时间
    pub timeout: Option<Duration>,
    /// 是否只读
    pub read_only: bool,
    /// 持久性级别
    pub durability: DurabilityLevel,
    /// 是否启用两阶段提交
    pub two_phase_commit: bool,
}

impl Default for TransactionConfig {
    fn default() -> Self {
        Self {
            timeout: None,
            read_only: false,
            durability: DurabilityLevel::Immediate,
            two_phase_commit: false,
        }
    }
}

impl TransactionConfig {
    /// 创建默认配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置超时时间
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// 设置为只读模式
    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self
    }

    /// 设置持久性级别
    pub fn with_durability(mut self, durability: DurabilityLevel) -> Self {
        self.durability = durability;
        self
    }

    /// 启用两阶段提交
    pub fn with_two_phase_commit(mut self) -> Self {
        self.two_phase_commit = true;
        self
    }

    /// 转换为内部 TransactionOptions
    pub(crate) fn into_options(self) -> TransactionOptions {
        TransactionOptions {
            timeout: self.timeout,
            read_only: self.read_only,
            durability: self.durability,
            two_phase_commit: self.two_phase_commit,
        }
    }
}

/// 事务句柄
///
/// 封装事务的生命周期管理，确保事务正确提交或回滚
/// 支持保存点功能，允许部分回滚
///
/// # 示例
///
/// ```rust
/// use graphdb::api::embedded::{GraphDatabase, DatabaseConfig};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let db = GraphDatabase::open("my_db")?;
/// let session = db.session()?;
///
/// // 开始事务
/// let txn = session.begin_transaction()?;
///
/// // 在事务中执行查询
/// txn.execute("CREATE TAG user(name string)")?;
/// txn.execute("INSERT VERTEX user(name) VALUES \"1\":(\"Alice\")")?;
///
/// // 提交事务
/// txn.commit()?;
/// # Ok(())
/// # }
/// ```
pub struct Transaction<'sess, S: StorageClient + Clone + 'static> {
    session: &'sess Session<S>,
    txn_handle: TransactionHandle,
    committed: bool,
    rolled_back: bool,
}

impl<'sess, S: StorageClient + Clone + 'static> Transaction<'sess, S> {
    /// 创建新的事务
    pub(crate) fn new(
        session: &'sess Session<S>,
        txn_handle: TransactionHandle,
    ) -> Self {
        Self {
            session,
            txn_handle,
            committed: false,
            rolled_back: false,
        }
    }

    /// 在事务中执行查询
    ///
    /// # 参数
    /// - `query` - 查询语句字符串
    ///
    /// # 返回
    /// - 成功时返回查询结果
    /// - 失败时返回错误
    ///
    /// # 错误
    /// 如果事务已提交或回滚，返回错误
    pub fn execute(&self, query: &str) -> CoreResult<QueryResult> {
        self.check_active()?;

        let ctx = QueryContext {
            space_id: self.session.space_id(),
            auto_commit: false,
            transaction_id: Some(self.txn_handle.0),
            parameters: None,
        };

        let mut query_api = self.session.query_api();
        let result = query_api.execute(query, ctx)?;
        Ok(QueryResult::from_core(result))
    }

    /// 在事务中执行参数化查询
    ///
    /// # 参数
    /// - `query` - 查询语句字符串
    /// - `params` - 查询参数
    ///
    /// # 返回
    /// - 成功时返回查询结果
    /// - 失败时返回错误
    pub fn execute_with_params(
        &self,
        query: &str,
        params: HashMap<String, Value>,
    ) -> CoreResult<QueryResult> {
        self.check_active()?;

        let ctx = QueryContext {
            space_id: self.session.space_id(),
            auto_commit: false,
            transaction_id: Some(self.txn_handle.0),
            parameters: Some(params),
        };

        let mut query_api = self.session.query_api();
        let result = query_api.execute(query, ctx)?;
        Ok(QueryResult::from_core(result))
    }

    /// 提交事务
    ///
    /// # 返回
    /// - 成功时返回 ()
    /// - 失败时返回错误
    ///
    /// # 注意
    /// 事务提交后不能再使用
    pub fn commit(mut self) -> CoreResult<()> {
        self.check_active()?;

        self.session.txn_api().commit(self.txn_handle)?;
        self.committed = true;
        Ok(())
    }

    /// 回滚事务
    ///
    /// # 返回
    /// - 成功时返回 ()
    /// - 失败时返回错误
    ///
    /// # 注意
    /// 事务回滚后不能再使用
    pub fn rollback(mut self) -> CoreResult<()> {
        self.check_active()?;

        self.session.txn_api().rollback(self.txn_handle)?;
        self.rolled_back = true;
        Ok(())
    }

    /// 创建保存点
    ///
    /// 保存点允许在事务内部创建一个标记点，可以回滚到该点而不影响整个事务
    ///
    /// # 参数
    /// - `name` - 保存点名称（可选）
    ///
    /// # 返回
    /// - 成功时返回保存点ID
    /// - 失败时返回错误
    ///
    /// # 示例
    ///
    /// ```rust
    /// use graphdb::api::embedded::GraphDatabase;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = GraphDatabase::open("my_db")?;
    /// let session = db.session()?;
    /// let txn = session.begin_transaction()?;
    ///
    /// // 创建保存点
    /// let sp = txn.create_savepoint(Some("checkpoint1".to_string()))?;
    ///
    /// // 执行一些操作...
    /// txn.execute("INSERT VERTEX user(name) VALUES \"1\":(\"Alice\")")?;
    ///
    /// // 如果需要，可以回滚到保存点
    /// txn.rollback_to_savepoint(sp)?;
    ///
    /// // 提交事务
    /// txn.commit()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn create_savepoint(&self, name: Option<String>) -> CoreResult<SavepointId> {
        self.check_active()?;

        let savepoint_manager = self.session.savepoint_manager();
        savepoint_manager
            .create_savepoint(self.txn_handle.0, name)
            .map_err(|e| CoreError::TransactionFailed(e.to_string()))
    }

    /// 回滚到保存点
    ///
    /// 回滚到指定保存点，该保存点之后的所有操作都会被撤销
    /// 但保存点本身仍然有效，可以继续使用
    ///
    /// # 参数
    /// - `savepoint_id` - 保存点ID
    ///
    /// # 返回
    /// - 成功时返回 ()
    /// - 失败时返回错误
    pub fn rollback_to_savepoint(&self, savepoint_id: SavepointId) -> CoreResult<()> {
        self.check_active()?;

        let savepoint_manager = self.session.savepoint_manager();
        savepoint_manager
            .rollback_to_savepoint(savepoint_id)
            .map_err(|e| CoreError::TransactionFailed(e.to_string()))
    }

    /// 释放保存点
    ///
    /// 释放保存点后，不能再回滚到该保存点，但也不会回滚任何更改
    ///
    /// # 参数
    /// - `savepoint_id` - 保存点ID
    ///
    /// # 返回
    /// - 成功时返回 ()
    /// - 失败时返回错误
    pub fn release_savepoint(&self, savepoint_id: SavepointId) -> CoreResult<()> {
        self.check_active()?;

        let savepoint_manager = self.session.savepoint_manager();
        savepoint_manager
            .release_savepoint(savepoint_id)
            .map_err(|e| CoreError::TransactionFailed(e.to_string()))
    }

    /// 通过名称查找保存点
    ///
    /// # 参数
    /// - `name` - 保存点名称
    ///
    /// # 返回
    /// - 找到时返回 Some(SavepointId)
    /// - 未找到时返回 None
    pub fn find_savepoint(&self, name: &str) -> Option<SavepointId> {
        if !self.is_active() {
            return None;
        }

        let savepoint_manager = self.session.savepoint_manager();
        savepoint_manager.find_savepoint_by_name(self.txn_handle.0, name)
    }

    /// 获取所有活跃保存点
    ///
    /// # 返回
    /// 活跃保存点信息列表
    pub fn list_savepoints(&self) -> Vec<SavepointInfo> {
        if !self.is_active() {
            return Vec::new();
        }

        let savepoint_manager = self.session.savepoint_manager();
        savepoint_manager.get_active_savepoints(self.txn_handle.0)
    }

    /// 获取事务信息
    ///
    /// # 返回
    /// - 成功时返回事务信息
    /// - 失败时返回错误
    pub fn info(&self) -> CoreResult<TransactionInfo> {
        let txn_manager = self.session.txn_manager();
        txn_manager
            .get_transaction_info(self.txn_handle.0)
            .map(|info| TransactionInfo {
                id: info.id,
                state: format!("{:?}", info.state),
                is_read_only: info.is_read_only,
                elapsed_ms: info.elapsed.as_millis() as u64,
                savepoint_count: info.savepoint_count,
            })
            .ok_or_else(|| CoreError::TransactionFailed("事务未找到".to_string()))
    }

    /// 检查事务是否处于活动状态
    fn check_active(&self) -> CoreResult<()> {
        if self.committed {
            return Err(CoreError::TransactionFailed(
                "事务已提交，不能执行操作".to_string()
            ));
        }
        if self.rolled_back {
            return Err(CoreError::TransactionFailed(
                "事务已回滚，不能执行操作".to_string()
            ));
        }
        Ok(())
    }

    /// 检查事务是否已提交
    pub fn is_committed(&self) -> bool {
        self.committed
    }

    /// 检查事务是否已回滚
    pub fn is_rolled_back(&self) -> bool {
        self.rolled_back
    }

    /// 检查事务是否仍处于活动状态
    pub fn is_active(&self) -> bool {
        !self.committed && !self.rolled_back
    }

    /// 获取事务句柄
    ///
    /// 返回此事务的唯一句柄，可用于跨 API 追踪事务状态
    pub fn handle(&self) -> TransactionHandle {
        self.txn_handle
    }

    /// 获取事务ID
    pub fn id(&self) -> u64 {
        self.txn_handle.0
    }
}

impl<'sess, S: StorageClient + Clone + 'static> Drop for Transaction<'sess, S> {
    fn drop(&mut self) {
        // 如果事务仍处于活动状态，自动回滚
        if self.is_active() {
            let _ = self.session.txn_api().rollback(self.txn_handle);
        }
    }
}

/// 事务信息
///
/// 提供事务的详细信息和状态
#[derive(Debug, Clone)]
pub struct TransactionInfo {
    /// 事务ID
    pub id: u64,
    /// 事务状态
    pub state: String,
    /// 是否只读
    pub is_read_only: bool,
    /// 已运行时间（毫秒）
    pub elapsed_ms: u64,
    /// 保存点数量
    pub savepoint_count: usize,
}

impl TransactionInfo {
    /// 获取事务ID
    pub fn id(&self) -> u64 {
        self.id
    }

    /// 获取事务状态
    pub fn state(&self) -> &str {
        &self.state
    }

    /// 检查是否只读
    pub fn is_read_only(&self) -> bool {
        self.is_read_only
    }

    /// 获取已运行时间（毫秒）
    pub fn elapsed_ms(&self) -> u64 {
        self.elapsed_ms
    }

    /// 获取保存点数量
    pub fn savepoint_count(&self) -> usize {
        self.savepoint_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_config_default() {
        let config = TransactionConfig::default();
        assert!(!config.read_only);
        assert!(config.timeout.is_none());
    }

    #[test]
    fn test_transaction_config_builder() {
        let config = TransactionConfig::new()
            .read_only()
            .with_timeout(Duration::from_secs(60))
            .with_durability(DurabilityLevel::None)
            .with_two_phase_commit();

        assert!(config.read_only);
        assert_eq!(config.timeout, Some(Duration::from_secs(60)));
        assert_eq!(config.durability, DurabilityLevel::None);
        assert!(config.two_phase_commit);
    }

    #[test]
    fn test_transaction_config_into_options() {
        let config = TransactionConfig::new()
            .read_only()
            .with_timeout(Duration::from_secs(30));

        let options = config.into_options();
        assert!(options.read_only);
        assert_eq!(options.timeout, Some(Duration::from_secs(30)));
    }
}

//! 事务 trait - 定义事务接口
//!
//! 对应原C++中的Transaction.h
//! 提供：
//! - Transaction: 事务 trait
//! - TransactionId: 事务标识符
//! - TransactionState: 事务状态
//! - TransactionResult: 事务结果
use serde::{Deserialize, Serialize};
use bincode::{Decode, Encode};
use crate::core::StorageError;
use std::sync::Arc;

/// 事务标识符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Encode, Decode)]
pub struct TransactionId(pub u64);

impl TransactionId {
    pub fn new(id: u64) -> Self {
        TransactionId(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Default for TransactionId {
    fn default() -> Self {
        TransactionId(rand::random())
    }
}

impl std::fmt::Display for TransactionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::AddAssign<u64> for TransactionId {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

impl std::ops::Add<u64> for TransactionId {
    type Output = Self;
    fn add(self, rhs: u64) -> Self {
        TransactionId(self.0 + rhs)
    }
}

impl From<u64> for TransactionId {
    fn from(val: u64) -> Self {
        TransactionId(val)
    }
}

impl From<TransactionId> for u64 {
    fn from(val: TransactionId) -> Self {
        val.0
    }
}

/// 事务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    /// 活跃状态
    Active,
    /// 正在提交
    Committing,
    /// 已提交
    Committed,
    /// 正在回滚
    Aborting,
    /// 已回滚
    Aborted,
    /// 部分提交（失败状态）
    PartialCommitted,
}

impl std::fmt::Display for TransactionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionState::Active => write!(f, "ACTIVE"),
            TransactionState::Committing => write!(f, "COMMITTING"),
            TransactionState::Committed => write!(f, "COMMITTED"),
            TransactionState::Aborting => write!(f, "ABORTING"),
            TransactionState::Aborted => write!(f, "ABORTED"),
            TransactionState::PartialCommitted => write!(f, "PARTIAL_COMMITTED"),
        }
    }
}

/// 事务结果
#[derive(Debug, Clone)]
pub enum TransactionResult {
    /// 成功
    Success,
    /// 失败并回滚
    Failure(StorageError),
    /// 冲突失败
    Conflict(String),
    /// 超时
    Timeout,
}

impl TransactionResult {
    pub fn is_success(&self) -> bool {
        matches!(self, TransactionResult::Success)
    }

    pub fn is_failure(&self) -> bool {
        matches!(self, TransactionResult::Failure(_))
    }

    pub fn is_conflict(&self) -> bool {
        matches!(self, TransactionResult::Conflict(_))
    }

    pub fn error(&self) -> Option<&StorageError> {
        match self {
            TransactionResult::Failure(err) => Some(err),
            _ => None,
        }
    }
}

impl From<StorageError> for TransactionResult {
    fn from(err: StorageError) -> Self {
        TransactionResult::Failure(err)
    }
}

/// 事务配置
#[derive(Debug, Clone)]
pub struct TransactionConfig {
    /// 隔离级别
    pub isolation_level: super::IsolationLevel,
    /// 超时时间（毫秒）
    pub timeout_ms: u64,
    /// 最大重试次数
    pub max_retries: u32,
    /// 是否启用写冲突检测
    pub conflict_detection: bool,
    /// 是否启用日志
    pub enable_logging: bool,
}

impl Default for TransactionConfig {
    fn default() -> Self {
        Self {
            isolation_level: super::IsolationLevel::Snapshot,
            timeout_ms: 30000,
            max_retries: 3,
            conflict_detection: true,
            enable_logging: true,
        }
    }
}

/// 事务上下文
#[derive(Debug, Clone)]
pub struct TransactionContext {
    /// 事务 ID
    pub tx_id: TransactionId,
    /// 开始时间戳
    pub start_time: u64,
    /// 空间名称
    pub space: String,
    /// 是否为只读事务
    pub read_only: bool,
    /// 客户端信息
    pub client_info: Option<String>,
    /// 用户自定义数据
    pub user_data: Option<Arc<dyn std::any::Any + Send + Sync>>,
}

impl TransactionContext {
    pub fn new(tx_id: TransactionId, space: &str) -> Self {
        let start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::ZERO)
            .as_millis() as u64;
        Self {
            tx_id,
            start_time,
            space: space.to_string(),
            read_only: false,
            client_info: None,
            user_data: None,
        }
    }

    pub fn with_read_only(tx_id: TransactionId, space: &str, read_only: bool) -> Self {
        let mut ctx = Self::new(tx_id, space);
        ctx.read_only = read_only;
        ctx
    }

    pub fn set_client_info(&mut self, info: &str) {
        self.client_info = Some(info.to_string());
    }

    pub fn set_user_data<T: std::any::Any + Send + Sync>(&mut self, data: Arc<T>) {
        self.user_data = Some(data as Arc<dyn std::any::Any + Send + Sync>);
    }

    pub fn get_user_data<T: std::any::Any + Send + Sync + Clone>(&self) -> Option<Arc<T>> {
        self.user_data.as_ref().and_then(|d| {
            d.downcast_ref::<T>().cloned().map(Arc::new)
        })
    }
}

/// 事务 trait
///
/// 定义事务的完整生命周期接口
pub trait Transaction: Send + Sync {
    /// 获取事务 ID
    fn id(&self) -> TransactionId;

    /// 获取当前状态
    fn state(&self) -> TransactionState;

    /// 获取上下文
    fn context(&self) -> &TransactionContext;

    /// 获取配置
    fn config(&self) -> &TransactionConfig;

    /// 开始事务
    fn begin(&mut self) -> Result<(), StorageError>;

    /// 提交事务
    fn commit(&mut self) -> TransactionResult;

    /// 回滚事务
    fn rollback(&mut self) -> TransactionResult;

    /// 获取快照版本
    fn snapshot_version(&self) -> Option<super::VersionVec>;

    /// 获取事务日志
    fn transaction_log(&self) -> Option<&super::TransactionLog>;

    /// 检查是否活跃
    fn is_active(&self) -> bool {
        self.state() == TransactionState::Active
    }

    /// 检查是否为只读
    fn is_read_only(&self) -> bool {
        self.context().read_only
    }

    /// 设置保存点
    fn set_savepoint(&self, name: &str) -> Result<(), StorageError>;

    /// 回滚到保存点
    fn rollback_to_savepoint(&self, name: &str) -> Result<(), StorageError>;

    /// 获取所有保存点
    fn savepoints(&self) -> Vec<String>;
}

/// 事务工厂 trait
pub trait TransactionFactory: Send + Sync {
    /// 创建新事务
    fn create_transaction(
        &self,
        space: &str,
        config: Option<TransactionConfig>,
    ) -> Result<Box<dyn Transaction>, StorageError>;

    /// 创建只读事务
    fn create_read_only_transaction(
        &self,
        space: &str,
    ) -> Result<Box<dyn Transaction>, StorageError>;
}

/// 事务管理器 trait
pub trait TransactionManager: Send + Sync {
    /// 获取当前活跃事务数
    fn active_transaction_count(&self) -> usize;

    /// 获取事务统计
    fn get_statistics(&self) -> TransactionStatistics;

    /// 清理过期事务
    fn cleanup_expired_transactions(&self);

    /// 获取所有活跃事务
    fn get_active_transactions(&self) -> Vec<TransactionId>;
}

/// 事务统计信息
#[derive(Debug, Clone, Default)]
pub struct TransactionStatistics {
    pub active_count: u64,
    pub committed_count: u64,
    pub aborted_count: u64,
    pub conflict_count: u64,
    pub average_commit_time_ms: f64,
    pub average_rollback_time_ms: f64,
}

impl TransactionStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_commit(&mut self, duration_ms: f64) {
        self.committed_count += 1;
        self.active_count = self.active_count.saturating_sub(1);
        self.average_commit_time_ms =
            (self.average_commit_time_ms * (self.committed_count - 1) as f64 + duration_ms)
                / self.committed_count as f64;
    }

    pub fn record_abort(&mut self, duration_ms: f64) {
        self.aborted_count += 1;
        self.active_count = self.active_count.saturating_sub(1);
        self.average_rollback_time_ms =
            (self.average_rollback_time_ms * (self.aborted_count - 1) as f64 + duration_ms)
                / self.aborted_count as f64;
    }

    pub fn record_conflict(&mut self) {
        self.conflict_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::transaction::IsolationLevel;

    #[test]
    fn test_transaction_id() {
        let id = TransactionId::new(100);
        assert_eq!(id.as_u64(), 100);
    }

    #[test]
    fn test_transaction_state_display() {
        assert_eq!(format!("{}", TransactionState::Active), "ACTIVE");
        assert_eq!(format!("{}", TransactionState::Committed), "COMMITTED");
    }

    #[test]
    fn test_transaction_result() {
        let success = TransactionResult::Success;
        assert!(success.is_success());

        let failure = TransactionResult::Failure(StorageError::DbError("test".to_string()));
        assert!(failure.is_failure());
        assert!(failure.error().is_some());
    }

    #[test]
    fn test_transaction_config_default() {
        let config = TransactionConfig::default();
        assert_eq!(config.isolation_level, IsolationLevel::Snapshot);
        assert_eq!(config.timeout_ms, 30000);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_transaction_context() {
        let tx_id = TransactionId::new(1);
        let ctx = TransactionContext::new(tx_id, "test_space");

        assert_eq!(ctx.tx_id, tx_id);
        assert_eq!(ctx.space, "test_space");
        assert!(!ctx.read_only);
    }

    #[test]
    fn test_transaction_context_read_only() {
        let tx_id = TransactionId::new(1);
        let ctx = TransactionContext::with_read_only(tx_id, "test_space", true);

        assert!(ctx.read_only);
    }
}

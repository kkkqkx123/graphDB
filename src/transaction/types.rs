//! 事务管理类型定义
//!
//! 提供事务管理所需的核心类型和结构

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use crate::transaction::savepoint::SavepointId;

/// 事务ID
pub type TransactionId = u64;

/// 事务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionState {
    /// 活跃状态，可执行读写操作
    Active,
    /// 已准备（2PC阶段1完成）
    Prepared,
    /// 提交中
    Committing,
    /// 已提交
    Committed,
    /// 中止中
    Aborting,
    /// 已中止
    Aborted,
}

impl TransactionState {
    /// 检查是否可以执行操作
    pub fn can_execute(&self) -> bool {
        matches!(self, TransactionState::Active)
    }

    /// 检查是否可以提交
    pub fn can_commit(&self) -> bool {
        matches!(self, TransactionState::Active | TransactionState::Prepared)
    }

    /// 检查是否可以中止
    pub fn can_abort(&self) -> bool {
        matches!(self, TransactionState::Active | TransactionState::Prepared)
    }

    /// 检查是否已结束
    pub fn is_terminal(&self) -> bool {
        matches!(self, TransactionState::Committed | TransactionState::Aborted)
    }
}

impl fmt::Display for TransactionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionState::Active => write!(f, "Active"),
            TransactionState::Prepared => write!(f, "Prepared"),
            TransactionState::Committing => write!(f, "Committing"),
            TransactionState::Committed => write!(f, "Committed"),
            TransactionState::Aborting => write!(f, "Aborting"),
            TransactionState::Aborted => write!(f, "Aborted"),
        }
    }
}

/// 事务错误类型
#[derive(Error, Debug, Clone)]
pub enum TransactionError {
    #[error("事务开始失败: {0}")]
    BeginFailed(String),

    #[error("事务提交失败: {0}")]
    CommitFailed(String),

    #[error("事务中止失败: {0}")]
    AbortFailed(String),

    #[error("事务未找到: {0}")]
    TransactionNotFound(TransactionId),

    #[error("事务未准备: {0}")]
    TransactionNotPrepared(TransactionId),

    #[error("无效的状态转换: 从 {from} 到 {to}")]
    InvalidStateTransition {
        from: TransactionState,
        to: TransactionState,
    },

    #[error("无效的状态用于提交: {0}")]
    InvalidStateForCommit(TransactionState),

    #[error("无效的状态用于中止: {0}")]
    InvalidStateForAbort(TransactionState),

    #[error("事务超时")]
    TransactionTimeout,

    #[error("事务已过期")]
    TransactionExpired,

    #[error("保存点创建失败: {0}")]
    SavepointFailed(String),

    #[error("保存点未找到: {0}")]
    SavepointNotFound(crate::transaction::savepoint::SavepointId),

    #[error("保存点未激活: {0}")]
    SavepointNotActive(crate::transaction::savepoint::SavepointId),

    #[error("事务中无保存点")]
    NoSavepointsInTransaction,

    #[error("2PC事务未找到: {0}")]
    TwoPhaseNotFound(crate::transaction::two_phase::TwoPhaseId),

    #[error("回滚失败: {0}")]
    RollbackFailed(String),

    #[error("并发事务数过多")]
    TooManyTransactions,

    #[error("写事务冲突，已有活跃的写事务")]
    WriteTransactionConflict,

    #[error("只读事务")]
    ReadOnlyTransaction,

    #[error("恢复失败: {0}")]
    RecoveryFailed(String),

    #[error("持久化失败: {0}")]
    PersistenceFailed(String),

    #[error("序列化失败: {0}")]
    SerializationFailed(String),

    #[error("内部错误: {0}")]
    Internal(String),
}

/// 事务选项
#[derive(Debug, Clone, PartialEq)]
pub struct TransactionOptions {
    /// 事务超时时间
    pub timeout: Option<Duration>,
    /// 是否只读
    pub read_only: bool,
    /// 持久性级别
    pub durability: DurabilityLevel,
    /// 是否启用两阶段提交
    pub two_phase_commit: bool,
}

impl Default for TransactionOptions {
    fn default() -> Self {
        Self {
            timeout: None,
            read_only: false,
            durability: DurabilityLevel::Immediate,
            two_phase_commit: false,
        }
    }
}

impl TransactionOptions {
    /// 创建默认选项
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置超时
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// 设置为只读
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
}

/// 持久性级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurabilityLevel {
    /// 不保证立即持久化（高性能）
    None,
    /// 立即持久化（默认）
    Immediate,
}

impl From<DurabilityLevel> for redb::Durability {
    fn from(level: DurabilityLevel) -> Self {
        match level {
            DurabilityLevel::None => redb::Durability::None,
            DurabilityLevel::Immediate => redb::Durability::Immediate,
        }
    }
}

/// 事务管理器配置
#[derive(Debug, Clone)]
pub struct TransactionManagerConfig {
    /// 默认事务超时时间
    pub default_timeout: Duration,
    /// 最大并发事务数
    pub max_concurrent_transactions: usize,
    /// 是否启用2PC
    pub enable_2pc: bool,
    /// 死锁检测间隔
    pub deadlock_detection_interval: Duration,
    /// 是否自动清理过期事务
    pub auto_cleanup: bool,
    /// 清理间隔
    pub cleanup_interval: Duration,
}

impl Default for TransactionManagerConfig {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(30),
            max_concurrent_transactions: 1000,
            enable_2pc: false,
            deadlock_detection_interval: Duration::from_secs(5),
            auto_cleanup: true,
            cleanup_interval: Duration::from_secs(10),
        }
    }
}

/// 事务统计信息
#[derive(Debug, Default)]
pub struct TransactionStats {
    /// 总事务数
    pub total_transactions: AtomicU64,
    /// 活跃事务数
    pub active_transactions: AtomicU64,
    /// 已提交事务数
    pub committed_transactions: AtomicU64,
    /// 已中止事务数
    pub aborted_transactions: AtomicU64,
    /// 超时事务数
    pub timeout_transactions: AtomicU64,
}

impl TransactionStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment_total(&self) {
        self.total_transactions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_active(&self) {
        self.active_transactions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_active(&self) {
        self.active_transactions.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn increment_committed(&self) {
        self.committed_transactions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_aborted(&self) {
        self.aborted_transactions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_timeout(&self) {
        self.timeout_transactions.fetch_add(1, Ordering::Relaxed);
    }
}

/// 操作日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationLog {
    /// 插入顶点
    InsertVertex {
        space: String,
        vertex_id: Vec<u8>,
    },
    /// 更新顶点
    UpdateVertex {
        space: String,
        vertex_id: Vec<u8>,
    },
    /// 删除顶点
    DeleteVertex {
        space: String,
        vertex_id: Vec<u8>,
    },
    /// 插入边
    InsertEdge {
        space: String,
        edge_key: Vec<u8>,
    },
    /// 删除边
    DeleteEdge {
        space: String,
        edge_key: Vec<u8>,
    },
    /// 更新索引
    UpdateIndex {
        space: String,
        index_name: String,
        key: Vec<u8>,
    },
    /// 删除索引
    DeleteIndex {
        space: String,
        index_name: String,
        key: Vec<u8>,
    },
}

/// 保存点信息
#[derive(Debug, Clone)]
pub struct SavepointInfo {
    pub id: SavepointId,
    pub name: String,
    pub created_at: Instant,
    pub operation_log_index: usize,
}

/// 事务信息（用于监控）
#[derive(Debug, Clone)]
pub struct TransactionInfo {
    pub id: TransactionId,
    pub state: TransactionState,
    pub start_time: Instant,
    pub elapsed: Duration,
    pub is_read_only: bool,
    pub modified_tables: Vec<String>,
    pub savepoint_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_state_transitions() {
        assert!(TransactionState::Active.can_execute());
        assert!(TransactionState::Active.can_commit());
        assert!(TransactionState::Active.can_abort());
        assert!(!TransactionState::Active.is_terminal());

        assert!(!TransactionState::Committed.can_execute());
        assert!(!TransactionState::Committed.can_commit());
        assert!(!TransactionState::Committed.can_abort());
        assert!(TransactionState::Committed.is_terminal());
    }

    #[test]
    fn test_transaction_options_builder() {
        let options = TransactionOptions::new()
            .with_timeout(Duration::from_secs(60))
            .read_only()
            .with_durability(DurabilityLevel::None)
            .with_two_phase_commit();

        assert_eq!(options.timeout, Some(Duration::from_secs(60)));
        assert!(options.read_only);
        assert_eq!(options.durability, DurabilityLevel::None);
        assert!(options.two_phase_commit);
    }

    #[test]
    fn test_transaction_stats() {
        let stats = TransactionStats::new();

        stats.increment_total();
        stats.increment_active();

        assert_eq!(stats.total_transactions.load(Ordering::Relaxed), 1);
        assert_eq!(stats.active_transactions.load(Ordering::Relaxed), 1);

        stats.decrement_active();
        stats.increment_committed();

        assert_eq!(stats.active_transactions.load(Ordering::Relaxed), 0);
        assert_eq!(stats.committed_transactions.load(Ordering::Relaxed), 1);
    }
}

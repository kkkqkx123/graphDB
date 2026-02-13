//! 快照隔离 - 实现快照隔离级别
//!
//! 实现快照隔离（Snapshot Isolation）：
//! - Snapshot: 快照
//! - IsolationLevel: 隔离级别
//! - 读写冲突检测
//! - 版本可见性判断

use super::{TransactionId, MvccManager, Version, VersionVec};
use crate::core::StorageError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime};

/// 隔离级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsolationLevel {
    /// 读未提交（Read Uncommitted）
    ReadUncommitted,
    /// 读已提交（Read Committed）
    ReadCommitted,
    /// 可重复读（Repeatable Read）
    RepeatableRead,
    /// 快照隔离（Snapshot Isolation）
    Snapshot,
    /// 串行化（Serializable）
    Serializable,
}

impl IsolationLevel {
    pub fn allows_dirty_read(&self) -> bool {
        matches!(self, IsolationLevel::ReadUncommitted)
    }

    pub fn allows_non_repeatable_read(&self) -> bool {
        matches!(self, IsolationLevel::ReadUncommitted | IsolationLevel::ReadCommitted)
    }

    pub fn allows_phantom(&self) -> bool {
        matches!(
            self,
            IsolationLevel::ReadUncommitted | IsolationLevel::ReadCommitted | IsolationLevel::RepeatableRead
        )
    }

    pub fn uses_snapshot(&self) -> bool {
        matches!(
            self,
            IsolationLevel::RepeatableRead | IsolationLevel::Snapshot | IsolationLevel::Serializable
        )
    }

    pub fn from_u8(level: u8) -> Self {
        match level {
            0 => IsolationLevel::ReadUncommitted,
            1 => IsolationLevel::ReadCommitted,
            2 => IsolationLevel::RepeatableRead,
            3 => IsolationLevel::Snapshot,
            4 => IsolationLevel::Serializable,
            _ => IsolationLevel::Snapshot,
        }
    }
}

impl Default for IsolationLevel {
    fn default() -> Self {
        IsolationLevel::Snapshot
    }
}

impl std::fmt::Display for IsolationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IsolationLevel::ReadUncommitted => write!(f, "READ UNCOMMITTED"),
            IsolationLevel::ReadCommitted => write!(f, "READ COMMITTED"),
            IsolationLevel::RepeatableRead => write!(f, "REPEATABLE READ"),
            IsolationLevel::Snapshot => write!(f, "SNAPSHOT"),
            IsolationLevel::Serializable => write!(f, "SERIALIZABLE"),
        }
    }
}

/// 快照
///
/// 存储事务开始时的版本信息
#[derive(Debug, Clone)]
pub struct Snapshot {
    /// 快照版本
    pub version: Version,
    /// 事务 ID
    pub tx_id: TransactionId,
    /// 活跃事务列表
    pub active_transactions: Vec<TransactionId>,
    /// 版本向量
    pub version_vector: VersionVec,
    /// 创建时间
    pub create_time: SystemTime,
    /// 隔离级别
    pub isolation_level: IsolationLevel,
}

impl Snapshot {
    pub fn new(
        tx_id: TransactionId,
        version: Version,
        version_vector: VersionVec,
        isolation_level: IsolationLevel,
    ) -> Self {
        Self {
            version,
            tx_id,
            active_transactions: Vec::new(),
            version_vector,
            create_time: SystemTime::now(),
            isolation_level,
        }
    }

    pub fn with_active_transactions(
        mut self,
        active_transactions: Vec<TransactionId>,
    ) -> Self {
        self.active_transactions = active_transactions;
        self
    }

    pub fn age(&self) -> Duration {
        SystemTime::now().duration_since(self.create_time).unwrap_or(Duration::ZERO)
    }

    pub fn is_expired(&self, max_age: Duration) -> bool {
        self.age() > max_age
    }
}

/// 读写冲突类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadWriteConflict {
    /// 写后读（Write-After-Read）
    WriteAfterRead,
    /// 读后写（Read-After-Write）
    ReadAfterWrite,
    /// 写后写（Write-After-Write）
    WriteAfterWrite,
}

/// 冲突信息
#[derive(Debug, Clone)]
pub struct ConflictInfo {
    pub conflict_type: ReadWriteConflict,
    pub tx_id: TransactionId,
    pub other_tx_id: TransactionId,
    pub key: String,
    pub timestamp: SystemTime,
}

impl ConflictInfo {
    pub fn new(
        conflict_type: ReadWriteConflict,
        tx_id: TransactionId,
        other_tx_id: TransactionId,
        key: String,
    ) -> Self {
        Self {
            conflict_type,
            tx_id,
            other_tx_id,
            key,
            timestamp: SystemTime::now(),
        }
    }
}

/// 快照管理器
///
/// 管理快照的创建和版本可见性判断
#[derive(Debug)]
pub struct SnapshotManager {
    /// MVCC 管理器
    mvcc: Arc<MvccManager>,
    /// 当前活跃快照
    snapshots: RwLock<HashMap<TransactionId, Snapshot>>,
    /// 配置
    config: SnapshotConfig,
    /// 统计信息
    stats: Arc<Mutex<SnapshotStats>>,
}

impl SnapshotManager {
    pub fn new(
        mvcc: Arc<MvccManager>,
        config: SnapshotConfig,
    ) -> Self {
        Self {
            mvcc,
            snapshots: RwLock::new(HashMap::new()),
            config,
            stats: Arc::new(Mutex::new(SnapshotStats::default())),
        }
    }

    /// 创建新快照
    pub fn create_snapshot(
        &self,
        tx_id: TransactionId,
        isolation_level: IsolationLevel,
    ) -> Result<Snapshot, StorageError> {
        let version = self.mvcc.next_version()?;
        let version_vector = self.mvcc.get_global_version_vec()?;
        let active_transactions = self.mvcc.get_active_transactions()?;

        let snapshot = Snapshot::new(tx_id, version, version_vector, isolation_level)
            .with_active_transactions(active_transactions);

        if isolation_level.uses_snapshot() {
            let mut snapshots = self.snapshots.write().map_err(|e| {
                StorageError::DbError(format!("Failed to acquire write lock: {}", e))
            })?;
            snapshots.insert(tx_id, snapshot.clone());
        }

        let mut stats = self.stats.lock().map_err(|e| {
            StorageError::DbError(format!("Failed to acquire stats lock: {}", e))
        })?;
        stats.snapshots_created += 1;

        Ok(snapshot)
    }

    /// 获取事务快照
    pub fn get_snapshot(&self, tx_id: TransactionId) -> Result<Option<Snapshot>, StorageError> {
        let snapshots = self.snapshots.read().map_err(|e| {
            StorageError::DbError(format!("Failed to acquire snapshots read lock: {}", e))
        })?;
        Ok(snapshots.get(&tx_id).cloned())
    }

    /// 删除快照
    pub fn remove_snapshot(&self, tx_id: TransactionId) -> Result<(), StorageError> {
        let mut snapshots = self.snapshots.write().map_err(|e| {
            StorageError::DbError(format!("Failed to acquire snapshots write lock: {}", e))
        })?;
        snapshots.remove(&tx_id);
        Ok(())
    }

    /// 检查版本是否对快照可见
    pub fn is_visible(
        &self,
        snapshot: &Snapshot,
        version_tx_id: TransactionId,
        version: Version,
    ) -> Result<bool, StorageError> {
        if version_tx_id == snapshot.tx_id {
            return Ok(true);
        }

        match snapshot.isolation_level {
            IsolationLevel::ReadUncommitted => Ok(true),
            IsolationLevel::ReadCommitted => {
                Ok(!self.is_active(version_tx_id)?)
            }
            IsolationLevel::RepeatableRead | IsolationLevel::Snapshot | IsolationLevel::Serializable => {
                if self.is_active(version_tx_id)? {
                    return Ok(false);
                }

                if let Some(active_version) = snapshot.version_vector.get(&version_tx_id) {
                    return Ok(active_version.as_u64() >= version.as_u64());
                }

                Ok(version < snapshot.version)
            }
        }
    }

    /// 检查事务是否活跃
    fn is_active(&self, tx_id: TransactionId) -> Result<bool, StorageError> {
        let snapshots = self.snapshots.read().map_err(|e| {
            StorageError::DbError(format!("Failed to acquire snapshots read lock: {}", e))
        })?;
        Ok(snapshots.contains_key(&tx_id))
    }

    /// 清理过期快照
    pub fn cleanup_expired_snapshots(&self) -> Result<(), StorageError> {
        let mut snapshots = self.snapshots.write().map_err(|e| {
            StorageError::DbError(format!("Failed to acquire snapshots write lock: {}", e))
        })?;
        let mut expired = Vec::new();

        for (tx_id, snapshot) in snapshots.iter() {
            if snapshot.is_expired(self.config.max_snapshot_age) {
                expired.push(*tx_id);
            }
        }

        for tx_id in &expired {
            snapshots.remove(tx_id);
        }

        let mut stats = self.stats.lock().map_err(|e| {
            StorageError::DbError(format!("Failed to acquire stats lock: {}", e))
        })?;
        stats.snapshots_cleaned += expired.len() as u64;
        Ok(())
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> Result<SnapshotStats, StorageError> {
        let stats = self.stats.lock().map_err(|e| {
            StorageError::DbError(format!("Failed to acquire stats lock: {}", e))
        })?;
        Ok(stats.clone())
    }
}

/// 快照配置
#[derive(Debug, Clone)]
pub struct SnapshotConfig {
    /// 快照最大保留时间
    pub max_snapshot_age: Duration,
    /// 是否启用冲突检测
    pub enable_conflict_detection: bool,
    /// 是否启用可串行化验证
    pub enable_serializable_validation: bool,
    /// 活跃事务阈值
    pub active_tx_threshold: usize,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            max_snapshot_age: Duration::from_secs(300),
            enable_conflict_detection: true,
            enable_serializable_validation: false,
            active_tx_threshold: 1000,
        }
    }
}

/// 快照统计信息
#[derive(Debug, Clone, Default)]
pub struct SnapshotStats {
    pub snapshots_created: u64,
    pub snapshots_cleaned: u64,
    pub conflicts_detected: u64,
    pub serializable_validations: u64,
    pub serializable_failures: u64,
}

impl SnapshotStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_conflict(&mut self) {
        self.conflicts_detected += 1;
    }

    pub fn record_validation(&mut self) {
        self.serializable_validations += 1;
    }

    pub fn record_validation_failure(&mut self) {
        self.serializable_failures += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isolation_level() {
        assert!(IsolationLevel::ReadUncommitted.allows_dirty_read());
        assert!(!IsolationLevel::Snapshot.allows_dirty_read());

        assert!(IsolationLevel::ReadCommitted.allows_non_repeatable_read());
        assert!(!IsolationLevel::RepeatableRead.allows_non_repeatable_read());

        assert!(IsolationLevel::ReadCommitted.allows_phantom());
        assert!(!IsolationLevel::Snapshot.allows_phantom());

        assert!(IsolationLevel::Snapshot.uses_snapshot());
        assert!(!IsolationLevel::ReadCommitted.uses_snapshot());
    }

    #[test]
    fn test_isolation_level_from_u8() {
        assert_eq!(IsolationLevel::from_u8(0), IsolationLevel::ReadUncommitted);
        assert_eq!(IsolationLevel::from_u8(1), IsolationLevel::ReadCommitted);
        assert_eq!(IsolationLevel::from_u8(2), IsolationLevel::RepeatableRead);
        assert_eq!(IsolationLevel::from_u8(3), IsolationLevel::Snapshot);
        assert_eq!(IsolationLevel::from_u8(4), IsolationLevel::Serializable);
    }

    #[test]
    fn test_isolation_level_display() {
        assert_eq!(format!("{}", IsolationLevel::Snapshot), "SNAPSHOT");
        assert_eq!(format!("{}", IsolationLevel::Serializable), "SERIALIZABLE");
    }

    #[test]
    fn test_snapshot() {
        let tx_id = TransactionId::new(1);
        let version = Version::new(100);
        let vv = VersionVec::new();
        let snapshot = Snapshot::new(tx_id, version, vv, IsolationLevel::Snapshot);

        assert_eq!(snapshot.tx_id, tx_id);
        assert_eq!(snapshot.version, version);
        assert!(snapshot.isolation_level.uses_snapshot());
    }

    #[test]
    fn test_conflict_info() {
        let tx1 = TransactionId::new(1);
        let tx2 = TransactionId::new(2);

        let conflict = ConflictInfo::new(
            ReadWriteConflict::WriteAfterRead,
            tx1,
            tx2,
            "vertex:space1:v1".to_string(),
        );

        assert_eq!(conflict.conflict_type, ReadWriteConflict::WriteAfterRead);
        assert_eq!(conflict.tx_id, tx1);
        assert_eq!(conflict.other_tx_id, tx2);
    }
}

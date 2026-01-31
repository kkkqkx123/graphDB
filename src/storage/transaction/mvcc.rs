//! MVCC - 多版本并发控制
//!
//! 实现多版本并发控制机制：
//! - Version: 版本标识
//! - VersionVec: 版本向量
//! - MvccManager: MVCC 管理器
//! - 版本链维护
//! - 垃圾回收

use serde::{Deserialize, Serialize};
use super::TransactionId;
use crate::core::{StorageError, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime};

/// 版本号
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Version(pub u64);

impl Version {
    pub fn new(v: u64) -> Self {
        Version(v)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn max() -> Self {
        Version(u64::MAX)
    }

    pub fn min() -> Self {
        Version(0)
    }
}

impl Default for Version {
    fn default() -> Self {
        Version(0)
    }
}

/// 版本向量 - 用于检测读写冲突
///
/// 使用向量时钟追踪各事务的版本信息
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionVec(HashMap<TransactionId, Version>);

impl VersionVec {
    pub fn new() -> Self {
        VersionVec(HashMap::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        VersionVec(HashMap::with_capacity(capacity))
    }

    pub fn increment(&mut self, tx_id: TransactionId) {
        let new_version = self.get(&tx_id).unwrap_or(Version::min()).0 + 1;
        self.0.insert(tx_id, Version(new_version));
    }

    pub fn get(&self, tx_id: &TransactionId) -> Option<Version> {
        self.0.get(tx_id).copied()
    }

    pub fn set(&mut self, tx_id: TransactionId, version: Version) {
        self.0.insert(tx_id, version);
    }

    pub fn merge(&mut self, other: &VersionVec) {
        for (tx_id, version) in &other.0 {
            let current = self.get(tx_id).unwrap_or(Version::min());
            if *version > current {
                self.0.insert(*tx_id, *version);
            }
        }
    }

    pub fn contains(&self, tx_id: &TransactionId) -> bool {
        self.0.contains_key(tx_id)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn max_version(&self) -> Version {
        self.0.values().max().copied().unwrap_or(Version::min())
    }

    pub fn to_vec(&self) -> Vec<(TransactionId, Version)> {
        self.0.iter().map(|(k, v)| (*k, *v)).collect()
    }
}

/// 版本类型
#[derive(Debug, Clone, PartialEq)]
pub enum VersionType {
    /// 读版本
    Read,
    /// 写版本
    Write,
    /// 事务版本
    Transaction,
}

/// 版本记录
#[derive(Debug, Clone)]
pub struct VersionRecord {
    /// 版本号
    pub version: Version,
    /// 所属事务 ID
    pub tx_id: TransactionId,
    /// 版本类型
    pub version_type: VersionType,
    /// 创建时间
    pub create_time: SystemTime,
    /// 过期时间（用于 GC）
    pub expire_time: Option<SystemTime>,
    /// 是否可见
    pub visible: bool,
    /// 下一个版本的链接
    pub next_version: Option<Arc<VersionRecord>>,
    /// 前一个版本的链接
    pub prev_version: Option<Arc<VersionRecord>>,
    /// 数据值（如果是写版本）
    pub value: Option<Value>,
}

impl VersionRecord {
    pub fn new(
        version: Version,
        tx_id: TransactionId,
        version_type: VersionType,
    ) -> Self {
        Self {
            version,
            tx_id,
            version_type,
            create_time: SystemTime::now(),
            expire_time: None,
            visible: true,
            next_version: None,
            prev_version: None,
            value: None,
        }
    }

    pub fn with_value(mut self, value: Value) -> Self {
        self.value = Some(value);
        self
    }

    pub fn set_expire_time(&mut self, duration: Duration) {
        self.expire_time = Some(SystemTime::now() + duration);
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expire) = &self.expire_time {
            SystemTime::now() > *expire
        } else {
            false
        }
    }
}

/// 数据项的版本链
#[derive(Debug, Clone)]
pub struct VersionChain {
    /// 键
    pub key: String,
    /// 所有版本的链表（从新到旧）
    pub versions: Vec<Arc<VersionRecord>>,
    /// 当前活跃写入事务
    pub active_writer: Option<TransactionId>,
}

impl VersionChain {
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
            versions: Vec::new(),
            active_writer: None,
        }
    }

    pub fn add_version(&mut self, record: Arc<VersionRecord>) {
        self.versions.insert(0, record);
    }

    pub fn latest_version(&self) -> Option<&Arc<VersionRecord>> {
        self.versions.first()
    }

    pub fn get_version(&self, version: Version) -> Option<&Arc<VersionRecord>> {
        self.versions.iter().find(|v| v.version == version)
    }

    pub fn is_being_written(&self) -> bool {
        self.active_writer.is_some()
    }
}

/// MVCC 管理器
///
/// 管理多版本并发控制的所有版本信息
#[derive(Debug)]
pub struct MvccManager {
    /// 全局版本号
    global_version: Arc<Mutex<u64>>,
    /// 版本向量（每个数据项一个）
    version_chains: RwLock<HashMap<String, VersionChain>>,
    /// 活跃事务的读取时间戳
    active_reads: RwLock<HashMap<TransactionId, u64>>,
    /// 垃圾回收配置
    gc_config: GcConfig,
    /// 统计信息
    stats: Arc<Mutex<MvccStats>>,
}

impl Default for MvccManager {
    fn default() -> Self {
        Self::new(GcConfig::default())
    }
}

impl MvccManager {
    pub fn new(gc_config: GcConfig) -> Self {
        Self {
            global_version: Arc::new(Mutex::new(0)),
            version_chains: RwLock::new(HashMap::new()),
            active_reads: RwLock::new(HashMap::new()),
            gc_config,
            stats: Arc::new(Mutex::new(MvccStats::default())),
        }
    }

    /// 创建新版本
    pub fn create_version(
        &self,
        key: &str,
        tx_id: TransactionId,
        value: Value,
    ) -> Result<Version, StorageError> {
        let version = self.next_version();

        let mut record = VersionRecord::new(version, tx_id, VersionType::Write);
        record = record.with_value(value);

        let mut chains = self.version_chains.write().map_err(|e| {
            StorageError::DbError(format!("Failed to acquire write lock: {}", e))
        })?;

        let chain = chains.entry(key.to_string()).or_insert_with(|| VersionChain::new(key));

        chain.add_version(Arc::new(record));

        Ok(version)
    }

    /// 获取下一版本号
    pub fn next_version(&self) -> Version {
        let mut global = self.global_version.lock().map_err(|e| {
            StorageError::DbError(format!("Failed to acquire lock: {}", e))
        }).unwrap();
        *global += 1;
        Version(*global)
    }

    /// 读取数据（MVCC）
    pub fn read(
        &self,
        key: &str,
        tx_id: TransactionId,
        read_version: Version,
    ) -> Result<Option<Value>, StorageError> {
        self.register_read(tx_id, read_version);

        let chains = self.version_chains.read().map_err(|e| {
            StorageError::DbError(format!("Failed to acquire read lock: {}", e))
        })?;

        if let Some(chain) = chains.get(key) {
            if chain.is_being_written() {
                return Err(StorageError::Conflict(
                    "Data is being written by another transaction".to_string(),
                ));
            }

            for version_record in &chain.versions {
                if self.is_visible(version_record.as_ref(), tx_id, read_version) {
                    return Ok(version_record.value.clone());
                }
            }
        }

        Ok(None)
    }

    /// 注册读操作
    fn register_read(&self, tx_id: TransactionId, read_version: Version) {
        let mut reads = self.active_reads.write().unwrap();
        reads.insert(tx_id, read_version.as_u64());
    }

    /// 检查版本是否对事务可见
    fn is_visible(&self, record: &VersionRecord, tx_id: TransactionId, read_version: Version) -> bool {
        if !record.visible {
            return false;
        }

        if record.version_type == VersionType::Write && record.tx_id == tx_id {
            return true;
        }

        if record.version <= read_version {
            return true;
        }

        false
    }

    /// 获取数据的版本链
    pub fn get_version_chain(&self, key: &str) -> Option<VersionChain> {
        let chains = self.version_chains.read().unwrap();
        chains.get(key).cloned()
    }

    /// 获取全局版本向量
    pub fn get_global_version_vec(&self) -> VersionVec {
        let mut vv = VersionVec::new();
        let chains = self.version_chains.read().unwrap();

        for chain in chains.values() {
            if let Some(version) = chain.latest_version() {
                vv.set(version.tx_id, version.version);
            }
        }

        vv
    }

    /// 提交事务
    pub fn commit(&self, tx_id: TransactionId) {
        let mut reads = self.active_reads.write().unwrap();
        reads.remove(&tx_id);

        let mut stats = self.stats.lock().unwrap();
        stats.committed_versions += 1;
    }

    /// 中止事务
    pub fn abort(&self, tx_id: TransactionId) {
        let mut reads = self.active_reads.write().unwrap();
        reads.remove(&tx_id);

        let mut chains = self.version_chains.write().unwrap();

        for chain in chains.values_mut() {
            if chain.active_writer == Some(tx_id) {
                chain.active_writer = None;
            }

            chain.versions.retain(|v| v.tx_id != tx_id);
        }

        let mut stats = self.stats.lock().unwrap();
        stats.aborted_versions += 1;
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> MvccStats {
        let stats = self.stats.lock().unwrap();
        stats.clone()
    }

    /// 获取活跃事务列表
    pub fn get_active_transactions(&self) -> Vec<TransactionId> {
        let reads = self.active_reads.read().unwrap();
        reads.keys().cloned().collect()
    }
}

/// GC 配置
#[derive(Debug, Clone)]
pub struct GcConfig {
    /// 保留的最小版本数
    pub min_versions: usize,
    /// 版本保留时间
    pub retention_duration: Duration,
    /// GC 间隔
    pub gc_interval: Duration,
    /// 是否启用自动 GC
    pub auto_gc: bool,
}

impl Default for GcConfig {
    fn default() -> Self {
        Self {
            min_versions: 3,
            retention_duration: Duration::from_secs(3600),
            gc_interval: Duration::from_secs(300),
            auto_gc: true,
        }
    }
}

/// MVCC 统计信息
#[derive(Debug, Clone, Default)]
pub struct MvccStats {
    pub total_versions: u64,
    pub committed_versions: u64,
    pub aborted_versions: u64,
    pub garbage_collected: u64,
    pub gc_runs: u64,
}

impl MvccStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_gc(&mut self, count: u64) {
        self.garbage_collected += count;
        self.gc_runs += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let v = Version::new(100);
        assert_eq!(v.as_u64(), 100);
        assert!(v > Version::min());
        assert!(v < Version::max());
    }

    #[test]
    fn test_version_vec() {
        let mut vv = VersionVec::new();
        let tx_id = TransactionId::new(1);

        assert!(!vv.contains(&tx_id));

        vv.set(tx_id, Version::new(5));
        assert!(vv.contains(&tx_id));
        assert_eq!(vv.get(&tx_id), Some(Version::new(5)));

        vv.increment(tx_id);
        assert_eq!(vv.get(&tx_id), Some(Version::new(6)));
    }

    #[test]
    fn test_version_vec_merge() {
        let mut vv1 = VersionVec::new();
        let tx1 = TransactionId::new(1);
        let tx2 = TransactionId::new(2);

        vv1.set(tx1, Version::new(5));

        let mut vv2 = VersionVec::new();
        vv2.set(tx2, Version::new(10));

        vv1.merge(&vv2);

        assert_eq!(vv1.get(&tx1), Some(Version::new(5)));
        assert_eq!(vv1.get(&tx2), Some(Version::new(10)));
    }

    #[test]
    fn test_version_chain() {
        let mut chain = VersionChain::new("test_key");

        let v1 = Arc::new(VersionRecord::new(Version::new(1), TransactionId::new(1), VersionType::Write));
        let v2 = Arc::new(VersionRecord::new(Version::new(2), TransactionId::new(2), VersionType::Write));

        chain.add_version(v1.clone());
        chain.add_version(v2.clone());

        assert_eq!(chain.versions.len(), 2);
        assert_eq!(chain.latest_version().map(|v| v.version), Some(Version::new(2)));

        chain.add_version(Arc::new(VersionRecord::new(Version::new(3), TransactionId::new(3), VersionType::Write)));
        assert_eq!(chain.versions.len(), 3);
    }
}

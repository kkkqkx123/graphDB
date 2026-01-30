//! 锁管理器 - 提供事务锁机制
//!
//! 实现细粒度的行级锁：
//! - LockManager: 锁管理器
//! - LockType: 锁类型
//! - LockRequest: 锁请求
//! - LockResult: 锁操作结果
//! - 死锁检测

use super::TransactionId;
use crate::core::StorageError;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime};
use std::hash::{Hash, Hasher};
use std::fmt;

/// 锁类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LockType {
    /// 共享锁（读锁）
    Shared,
    /// 排他锁（写锁）
    Exclusive,
    /// 意向共享锁
    IntentionShared,
    /// 意向排他锁
    IntentionExclusive,
    /// 共享排他锁
    SharedExclusive,
}

impl fmt::Display for LockType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LockType::Shared => write!(f, "S"),
            LockType::Exclusive => write!(f, "X"),
            LockType::IntentionShared => write!(f, "IS"),
            LockType::IntentionExclusive => write!(f, "IX"),
            LockType::SharedExclusive => write!(f, "SIX"),
        }
    }
}

impl LockType {
    /// 检查是否可以与另一锁兼容
    pub fn is_compatible_with(&self, other: LockType) -> bool {
        match (self, other) {
            (LockType::Shared, LockType::Shared) => true,
            (LockType::Shared, LockType::IntentionShared) => true,
            (LockType::Shared, LockType::IntentionExclusive) => true,
            (LockType::Exclusive, _) => false,
            (LockType::IntentionShared, LockType::Shared) => true,
            (LockType::IntentionShared, LockType::IntentionShared) => true,
            (LockType::IntentionExclusive, LockType::Shared) => true,
            (LockType::IntentionExclusive, LockType::IntentionShared) => true,
            (LockType::SharedExclusive, LockType::IntentionShared) => true,
            _ => false,
        }
    }

    /// 检查是否是排他锁
    pub fn is_exclusive(&self) -> bool {
        matches!(self, LockType::Exclusive)
    }

    /// 检查是否是共享锁
    pub fn is_shared(&self) -> bool {
        matches!(self, LockType::Shared)
    }
}

/// 锁粒度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockGranularity {
    /// 行级锁
    Row,
    /// 页级锁
    Page,
    /// 表级锁
    Table,
    /// 数据库级锁
    Database,
}

impl fmt::Display for LockGranularity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LockGranularity::Row => write!(f, "ROW"),
            LockGranularity::Page => write!(f, "PAGE"),
            LockGranularity::Table => write!(f, "TABLE"),
            LockGranularity::Database => write!(f, "DATABASE"),
        }
    }
}

/// 锁标识符
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockKey {
    pub resource_type: String,
    pub resource_name: String,
}

impl LockKey {
    pub fn new(resource_type: &str, resource_name: &str) -> Self {
        Self {
            resource_type: resource_type.to_string(),
            resource_name: resource_name.to_string(),
        }
    }

    pub fn vertex_key(space: &str, vid: &str) -> Self {
        Self::new("vertex", &format!("{}:{}", space, vid))
    }

    pub fn edge_key(space: &str, src: &str, edge_type: &str, dst: &str) -> Self {
        Self::new("edge", &format!("{}:{}:{}:{}", space, src, edge_type, dst))
    }

    pub fn tag_key(space: &str, tag: &str) -> Self {
        Self::new("tag", &format!("{}:{}", space, tag))
    }

    pub fn edge_type_key(space: &str, edge_type: &str) -> Self {
        Self::new("edge_type", &format!("{}:{}", space, edge_type))
    }
}

impl Hash for LockKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.resource_type.hash(state);
        ":".hash(state);
        self.resource_name.hash(state);
    }
}

/// 锁等待信息
#[derive(Debug, Clone)]
pub struct WaitInfo {
    pub tx_id: TransactionId,
    pub lock_type: LockType,
    pub wait_start: SystemTime,
    pub timeout: Duration,
}

impl WaitInfo {
    pub fn new(tx_id: TransactionId, lock_type: LockType, timeout: Duration) -> Self {
        Self {
            tx_id,
            lock_type,
            wait_start: SystemTime::now(),
            timeout,
        }
    }

    pub fn is_timeout(&self) -> bool {
        SystemTime::now().duration_since(self.wait_start).unwrap_or(Duration::ZERO) > self.timeout
    }
}

/// 锁记录
#[derive(Debug, Clone)]
pub struct LockRecord {
    pub key: LockKey,
    pub lock_type: LockType,
    pub tx_id: TransactionId,
    pub granulation: LockGranularity,
    pub hold_start: SystemTime,
    pub wait_queue: Vec<WaitInfo>,
}

impl LockRecord {
    pub fn new(
        key: LockKey,
        lock_type: LockType,
        tx_id: TransactionId,
        granulation: LockGranularity,
    ) -> Self {
        Self {
            key,
            lock_type,
            tx_id,
            granulation,
            hold_start: SystemTime::now(),
            wait_queue: Vec::new(),
        }
    }

    pub fn add_waiter(&mut self, waiter: WaitInfo) {
        self.wait_queue.push(waiter);
    }

    pub fn remove_waiter(&mut self, tx_id: TransactionId) {
        self.wait_queue.retain(|w| w.tx_id != tx_id);
    }

    pub fn is_waiting(&self, tx_id: TransactionId) -> bool {
        self.wait_queue.iter().any(|w| w.tx_id == tx_id)
    }

    pub fn waiting_transactions(&self) -> Vec<TransactionId> {
        self.wait_queue.iter().map(|w| w.tx_id).collect()
    }
}

/// 锁请求
#[derive(Debug, Clone)]
pub struct LockRequest {
    pub key: LockKey,
    pub lock_type: LockType,
    pub granulation: LockGranularity,
    pub timeout: Duration,
    pub no_wait: bool,
    pub skip_locked: bool,
}

impl LockRequest {
    pub fn new(key: LockKey, lock_type: LockType) -> Self {
        Self {
            key,
            lock_type,
            granulation: LockGranularity::Row,
            timeout: Duration::from_secs(30),
            no_wait: false,
            skip_locked: false,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_granularity(mut self, granulation: LockGranularity) -> Self {
        self.granulation = granulation;
        self
    }

    pub fn no_wait(mut self) -> Self {
        self.no_wait = true;
        self
    }

    pub fn skip_locked(mut self) -> Self {
        self.skip_locked = true;
        self
    }
}

/// 锁操作结果
#[derive(Debug, Clone, PartialEq)]
pub enum LockResult {
    /// 成功获取锁
    Acquired,
    /// 锁等待中
    Waiting,
    /// 锁超时
    Timeout,
    /// 死锁检测导致回滚
    Deadlock,
    /// 跳过（skip_locked）
    Skipped,
    /// 失败
    Failed(String),
}

impl LockResult {
    pub fn is_success(&self) -> bool {
        matches!(self, LockResult::Acquired)
    }

    pub fn is_waiting(&self) -> bool {
        matches!(self, LockResult::Waiting)
    }

    pub fn is_timeout(&self) -> bool {
        matches!(self, LockResult::Timeout)
    }

    pub fn is_deadlock(&self) -> bool {
        matches!(self, LockResult::Deadlock)
    }
}

/// 锁管理器
///
/// 管理所有锁的获取、释放和死锁检测
#[derive(Debug)]
pub struct LockManager {
    /// 锁表
    lock_table: RwLock<HashMap<LockKey, LockRecord>>,
    /// 事务拥有的锁
    tx_locks: RwLock<HashMap<TransactionId, HashSet<LockKey>>>,
    /// 等待图（用于死锁检测）
    wait_graph: RwLock<BTreeMap<TransactionId, HashSet<TransactionId>>>,
    /// 配置
    config: LockManagerConfig,
    /// 统计信息
    stats: Arc<Mutex<LockStats>>,
}

impl Default for LockManager {
    fn default() -> Self {
        Self::new(LockManagerConfig::default())
    }
}

impl LockManager {
    pub fn new(config: LockManagerConfig) -> Self {
        Self {
            lock_table: RwLock::new(HashMap::new()),
            tx_locks: RwLock::new(HashMap::new()),
            wait_graph: RwLock::new(BTreeMap::new()),
            config,
            stats: Arc::new(Mutex::new(LockStats::default())),
        }
    }

    /// 尝试获取锁
    pub fn try_lock(&self, tx_id: TransactionId, request: LockRequest) -> LockResult {
        let key = request.key;

        let mut lock_table = self.lock_table.write().unwrap();
        let mut tx_locks = self.tx_locks.write().unwrap();

        let record = lock_table.entry(key.clone()).or_insert_with(|| {
            LockRecord::new(
                key.clone(),
                LockType::IntentionShared,
                tx_id,
                request.granulation,
            )
        });

        if record.tx_id == tx_id {
            if request.lock_type == LockType::Exclusive && record.lock_type != LockType::Exclusive {
                record.lock_type = LockType::Exclusive;
            }
            return LockResult::Acquired;
        }

        if !request.lock_type.is_compatible_with(record.lock_type) {
            if request.no_wait {
                return LockResult::Failed("Lock conflict".to_string());
            }

            if request.skip_locked {
                return LockResult::Skipped;
            }

            let waiter = WaitInfo::new(tx_id, request.lock_type, request.timeout);
            record.add_waiter(waiter);

            self.add_wait_edge(tx_id, record.tx_id);

            if let Some(cycle) = self.detect_deadlock() {
                self.remove_wait_edges(&cycle);
                return LockResult::Deadlock;
            }

            return LockResult::Waiting;
        }

        if request.lock_type == LockType::Exclusive {
            record.lock_type = LockType::Exclusive;
        }

        tx_locks.entry(tx_id).or_insert_with(HashSet::new).insert(key);

        let mut stats = self.stats.lock().unwrap();
        stats.lock_acquired += 1;

        LockResult::Acquired
    }

    /// 释放事务的所有锁
    pub fn release_transaction_locks(&self, tx_id: TransactionId) {
        let mut lock_table = self.lock_table.write().unwrap();
        let mut tx_locks = self.tx_locks.write().unwrap();

        let locks_count = if let Some(locks) = tx_locks.remove(&tx_id) {
            let count = locks.len();
            for key in locks {
                if let Some(record) = lock_table.get_mut(&key) {
                    record.remove_waiter(tx_id);

                    if record.wait_queue.is_empty() {
                        lock_table.remove(&key);
                    } else {
                        let next_waiter = record.wait_queue.first().unwrap();
                        record.tx_id = next_waiter.tx_id;
                        record.lock_type = next_waiter.lock_type;
                        record.wait_queue.remove(0);
                    }
                }
            }
            count
        } else {
            0
        };

        self.remove_wait_edges_for(tx_id);

        let mut stats = self.stats.lock().unwrap();
        stats.lock_released += locks_count as u64;
    }

    /// 检查事务是否持有锁
    pub fn is_locked_by(&self, key: &LockKey, tx_id: TransactionId) -> bool {
        let lock_table = self.lock_table.read().unwrap();
        if let Some(record) = lock_table.get(key) {
            record.tx_id == tx_id
        } else {
            false
        }
    }

    /// 获取事务持有的所有锁
    pub fn get_locks_held_by(&self, tx_id: TransactionId) -> Vec<LockKey> {
        let tx_locks = self.tx_locks.read().unwrap();
        tx_locks.get(&tx_id).map(|s| s.iter().cloned().collect()).unwrap_or_default()
    }

    /// 获取等待锁的事务
    pub fn get_waiting_transactions(&self) -> Vec<TransactionId> {
        let lock_table = self.lock_table.read().unwrap();
        let mut waiters = Vec::new();
        for record in lock_table.values() {
            for waiter in &record.wait_queue {
                if !waiters.contains(&waiter.tx_id) {
                    waiters.push(waiter.tx_id);
                }
            }
        }
        waiters
    }

    fn add_wait_edge(&self, waiter: TransactionId, holder: TransactionId) {
        let mut graph = self.wait_graph.write().unwrap();
        graph.entry(waiter).or_insert_with(HashSet::new).insert(holder);
    }

    fn remove_wait_edges(&self, cycle: &[TransactionId]) {
        let mut graph = self.wait_graph.write().unwrap();
        for &tx in cycle {
            graph.remove(&tx);
        }
    }

    fn remove_wait_edges_for(&self, tx_id: TransactionId) {
        let mut graph = self.wait_graph.write().unwrap();
        graph.remove(&tx_id);

        for holders in graph.values_mut() {
            holders.remove(&tx_id);
        }
    }

    fn detect_deadlock(&self) -> Option<Vec<TransactionId>> {
        let graph = self.wait_graph.read().unwrap();

        for &tx in graph.keys() {
            if let Some(cycle) = self.find_cycle(tx, &graph, &mut HashSet::new(), &mut Vec::new()) {
                return Some(cycle);
            }
        }

        None
    }

    fn find_cycle(
        &self,
        tx: TransactionId,
        graph: &BTreeMap<TransactionId, HashSet<TransactionId>>,
        visited: &mut HashSet<TransactionId>,
        path: &mut Vec<TransactionId>,
    ) -> Option<Vec<TransactionId>> {
        if visited.contains(&tx) {
            if let Some(idx) = path.iter().position(|&t| t == tx) {
                return Some(path[idx..].to_vec());
            }
            return None;
        }

        visited.insert(tx);
        path.push(tx);

        if let Some(waiters) = graph.get(&tx) {
            for &waiter in waiters {
                if let Some(cycle) = self.find_cycle(waiter, graph, visited, path) {
                    return Some(cycle);
                }
            }
        }

        path.pop();
        None
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> LockStats {
        let stats = self.stats.lock().unwrap();
        stats.clone()
    }
}

/// 锁管理器配置
#[derive(Debug, Clone)]
pub struct LockManagerConfig {
    /// 默认超时时间
    pub default_timeout: Duration,
    /// 死锁检测间隔
    pub deadlock_check_interval: Duration,
    /// 是否启用死锁检测
    pub enable_deadlock_detection: bool,
    /// 最大等待队列长度
    pub max_wait_queue_length: usize,
    /// 是否使用公平锁
    pub fair_locking: bool,
}

impl Default for LockManagerConfig {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(30),
            deadlock_check_interval: Duration::from_secs(5),
            enable_deadlock_detection: true,
            max_wait_queue_length: 1000,
            fair_locking: false,
        }
    }
}

/// 锁统计信息
#[derive(Debug, Clone, Default)]
pub struct LockStats {
    pub lock_acquired: u64,
    pub lock_released: u64,
    pub lock_waited: u64,
    pub lock_timeout: u64,
    pub deadlocks_detected: u64,
    pub deadlocks_resolved: u64,
}

impl LockStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_acquire(&mut self) {
        self.lock_acquired += 1;
    }

    pub fn record_wait(&mut self) {
        self.lock_waited += 1;
    }

    pub fn record_timeout(&mut self) {
        self.lock_timeout += 1;
    }

    pub fn record_deadlock(&mut self) {
        self.deadlocks_detected += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_type_compatible() {
        assert!(LockType::Shared.is_compatible_with(LockType::Shared));
        assert!(LockType::Exclusive.is_compatible_with(LockType::IntentionShared));
        assert!(!LockType::Exclusive.is_compatible_with(LockType::Shared));
        assert!(!LockType::Exclusive.is_compatible_with(LockType::Exclusive));
    }

    #[test]
    fn test_lock_key() {
        let key = LockKey::vertex_key("space1", "v1");
        assert_eq!(key.resource_type, "vertex");
        assert!(key.resource_name.contains("space1"));
        assert!(key.resource_name.contains("v1"));
    }

    #[test]
    fn test_lock_manager_basic() {
        let manager = LockManager::default();
        let tx_id = TransactionId::new(1);
        let key = LockKey::vertex_key("space1", "v1");

        let request = LockRequest::new(key.clone(), LockType::Shared);
        let result = manager.try_lock(tx_id, request);

        assert_eq!(result, LockResult::Acquired);
        assert!(manager.is_locked_by(&key, tx_id));
    }

    #[test]
    fn test_lock_manager_exclusive() {
        let manager = LockManager::default();
        let tx1 = TransactionId::new(1);
        let tx2 = TransactionId::new(2);
        let key = LockKey::vertex_key("space1", "v1");

        let request1 = LockRequest::new(key.clone(), LockType::Shared);
        assert_eq!(manager.try_lock(tx1, request1), LockResult::Acquired);

        let request2 = LockRequest::new(key.clone(), LockType::Exclusive);
        let result = manager.try_lock(tx2, request2);
        assert!(result.is_waiting());

        manager.release_transaction_locks(tx1);
        assert!(!manager.is_locked_by(&key, tx1));
    }

    #[test]
    fn test_lock_manager_no_wait() {
        let manager = LockManager::default();
        let tx1 = TransactionId::new(1);
        let tx2 = TransactionId::new(2);
        let key = LockKey::vertex_key("space1", "v1");

        let request1 = LockRequest::new(key.clone(), LockType::Exclusive);
        assert_eq!(manager.try_lock(tx1, request1), LockResult::Acquired);

        let request2 = LockRequest::new(key.clone(), LockType::Exclusive).no_wait();
        let result = manager.try_lock(tx2, request2);
        assert!(result != LockResult::Acquired);
    }

    #[test]
    fn test_lock_manager_release() {
        let manager = LockManager::default();
        let tx_id = TransactionId::new(1);
        let key1 = LockKey::vertex_key("space1", "v1");
        let key2 = LockKey::vertex_key("space1", "v2");

        let request1 = LockRequest::new(key1.clone(), LockType::Shared);
        let request2 = LockRequest::new(key2.clone(), LockType::Shared);
        assert_eq!(manager.try_lock(tx_id, request1), LockResult::Acquired);
        assert_eq!(manager.try_lock(tx_id, request2), LockResult::Acquired);

        let locks = manager.get_locks_held_by(tx_id);
        assert_eq!(locks.len(), 2);

        manager.release_transaction_locks(tx_id);
        assert!(manager.get_locks_held_by(tx_id).is_empty());
    }
}

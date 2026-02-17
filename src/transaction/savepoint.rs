//! 保存点管理模块
//!
//! 提供嵌套事务支持，允许在事务内部创建保存点，实现部分回滚功能

use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use parking_lot::RwLock;

use crate::transaction::{TransactionError, TransactionId};

/// 保存点ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SavepointId(u64);

impl SavepointId {
    /// 创建新的保存点ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// 获取原始ID值
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for SavepointId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for SavepointId {
    fn default() -> Self {
        Self(0)
    }
}

/// 保存点状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SavepointState {
    /// 活跃状态，可以回滚到该保存点
    Active,
    /// 已释放，不能再回滚
    Released,
    /// 已回滚，保存点已失效
    RolledBack,
}

/// 保存点信息
#[derive(Debug, Clone)]
pub struct Savepoint {
    /// 保存点ID
    pub id: SavepointId,
    /// 所属事务ID
    pub txn_id: TransactionId,
    /// 保存点名称（可选）
    pub name: Option<String>,
    /// 保存点状态
    pub state: SavepointState,
    /// 创建时间戳
    pub created_at: std::time::Instant,
    /// 保存点序号（用于确定回滚顺序）
    pub sequence: u64,
}

impl Savepoint {
    /// 创建新的保存点
    pub fn new(
        id: SavepointId,
        txn_id: TransactionId,
        name: Option<String>,
        sequence: u64,
    ) -> Self {
        Self {
            id,
            txn_id,
            name,
            state: SavepointState::Active,
            created_at: std::time::Instant::now(),
            sequence,
        }
    }

    /// 检查保存点是否活跃
    pub fn is_active(&self) -> bool {
        self.state == SavepointState::Active
    }
}

/// 保存点统计信息
#[derive(Debug, Default)]
pub struct SavepointStats {
    /// 创建的保存点总数
    pub total_created: AtomicU64,
    /// 释放的保存点数量
    pub released_count: AtomicU64,
    /// 回滚的保存点数量
    pub rollback_count: AtomicU64,
}

impl SavepointStats {
    /// 增加创建计数
    pub fn increment_created(&self) {
        self.total_created.fetch_add(1, Ordering::Relaxed);
    }

    /// 增加释放计数
    pub fn increment_released(&self) {
        self.released_count.fetch_add(1, Ordering::Relaxed);
    }

    /// 增加回滚计数
    pub fn increment_rollback(&self) {
        self.rollback_count.fetch_add(1, Ordering::Relaxed);
    }
}

/// 保存点管理器
///
/// 管理事务内的保存点，支持嵌套保存点和部分回滚
pub struct SavepointManager {
    /// 保存点ID生成器
    id_generator: AtomicU64,
    /// 保存点序列号生成器（每个事务独立）
    sequence_generator: RwLock<HashMap<TransactionId, AtomicU64>>,
    /// 活跃保存点映射
    savepoints: RwLock<HashMap<SavepointId, Arc<RwLock<Savepoint>>>>,
    /// 事务到保存点的映射
    txn_savepoints: RwLock<HashMap<TransactionId, Vec<SavepointId>>>,
    /// 统计信息
    stats: SavepointStats,
}

impl SavepointManager {
    /// 创建新的保存点管理器
    pub fn new() -> Self {
        Self {
            id_generator: AtomicU64::new(1),
            sequence_generator: RwLock::new(HashMap::new()),
            savepoints: RwLock::new(HashMap::new()),
            txn_savepoints: RwLock::new(HashMap::new()),
            stats: SavepointStats::default(),
        }
    }

    /// 创建保存点
    ///
    /// # Arguments
    /// * `txn_id` - 事务ID
    /// * `name` - 保存点名称（可选）
    ///
    /// # Returns
    /// * `Ok(SavepointId)` - 保存点ID
    /// * `Err(TransactionError)` - 创建失败
    pub fn create_savepoint(
        &self,
        txn_id: TransactionId,
        name: Option<String>,
    ) -> Result<SavepointId, TransactionError> {
        let id = SavepointId::new(self.id_generator.fetch_add(1, Ordering::SeqCst));

        // 获取或创建该事务的序列号生成器
        let mut seq_map = self.sequence_generator.write();
        let seq_gen = seq_map
            .entry(txn_id)
            .or_insert_with(|| AtomicU64::new(1));
        let sequence = seq_gen.fetch_add(1, Ordering::SeqCst);
        drop(seq_map);

        let savepoint = Arc::new(RwLock::new(Savepoint::new(
            id,
            txn_id,
            name,
            sequence,
        )));

        // 保存保存点
        self.savepoints.write().insert(id, savepoint);

        // 添加到事务的保存点列表
        self.txn_savepoints
            .write()
            .entry(txn_id)
            .or_insert_with(Vec::new)
            .push(id);

        self.stats.increment_created();

        Ok(id)
    }

    /// 回滚到保存点
    ///
    /// # Arguments
    /// * `savepoint_id` - 保存点ID
    ///
    /// # Returns
    /// * `Ok(())` - 回滚成功
    /// * `Err(TransactionError)` - 回滚失败
    pub fn rollback_to_savepoint(&self, savepoint_id: SavepointId) -> Result<(), TransactionError> {
        let savepoint = self
            .savepoints
            .read()
            .get(&savepoint_id)
            .cloned()
            .ok_or(TransactionError::SavepointNotFound(savepoint_id))?;

        {
            let sp = savepoint.read();
            if !sp.is_active() {
                return Err(TransactionError::SavepointNotActive(savepoint_id));
            }
        }

        // 获取该事务的所有保存点
        let txn_id = savepoint.read().txn_id;
        let txn_sps = self
            .txn_savepoints
            .read()
            .get(&txn_id)
            .cloned()
            .unwrap_or_default();

        // 找到目标保存点的位置
        let target_sequence = savepoint.read().sequence;

        // 标记所有在该保存点之后创建的保存点为已回滚
        for sp_id in &txn_sps {
            if let Some(sp_arc) = self.savepoints.read().get(sp_id) {
                let sp_seq = sp_arc.read().sequence;
                if sp_seq > target_sequence {
                    sp_arc.write().state = SavepointState::RolledBack;
                }
            }
        }

        // 从事务保存点列表中移除被回滚的保存点
        let mut txn_sps_write = self.txn_savepoints.write();
        if let Some(list) = txn_sps_write.get_mut(&txn_id) {
            list.retain(|id| {
                self.savepoints
                    .read()
                    .get(id)
                    .map(|sp| sp.read().sequence <= target_sequence)
                    .unwrap_or(false)
            });
        }
        drop(txn_sps_write);

        self.stats.increment_rollback();

        Ok(())
    }

    /// 释放保存点
    ///
    /// 释放保存点后，不能再回滚到该保存点，但也不会回滚任何更改
    ///
    /// # Arguments
    /// * `savepoint_id` - 保存点ID
    pub fn release_savepoint(&self, savepoint_id: SavepointId) -> Result<(), TransactionError> {
        let savepoint = self
            .savepoints
            .read()
            .get(&savepoint_id)
            .cloned()
            .ok_or(TransactionError::SavepointNotFound(savepoint_id))?;

        {
            let mut sp = savepoint.write();
            if !sp.is_active() {
                return Err(TransactionError::SavepointNotActive(savepoint_id));
            }
            sp.state = SavepointState::Released;
        }

        self.stats.increment_released();

        Ok(())
    }

    /// 获取事务的所有活跃保存点
    ///
    /// # Arguments
    /// * `txn_id` - 事务ID
    pub fn get_active_savepoints(&self, txn_id: TransactionId) -> Vec<SavepointInfo> {
        self.txn_savepoints
            .read()
            .get(&txn_id)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|id| {
                self.savepoints.read().get(&id).and_then(|sp| {
                    let sp_read = sp.read();
                    if sp_read.is_active() {
                        Some(SavepointInfo {
                            id: sp_read.id,
                            name: sp_read.name.clone(),
                            sequence: sp_read.sequence,
                        })
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    /// 通过名称查找保存点
    ///
    /// # Arguments
    /// * `txn_id` - 事务ID
    /// * `name` - 保存点名称
    pub fn find_savepoint_by_name(
        &self,
        txn_id: TransactionId,
        name: &str,
    ) -> Option<SavepointId> {
        self.txn_savepoints
            .read()
            .get(&txn_id)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .find(|id| {
                self.savepoints
                    .read()
                    .get(id)
                    .map(|sp| {
                        let sp_read = sp.read();
                        sp_read.is_active() && sp_read.name.as_deref() == Some(name)
                    })
                    .unwrap_or(false)
            })
    }

    /// 清理事务的所有保存点
    ///
    /// 当事务提交或中止时调用
    ///
    /// # Arguments
    /// * `txn_id` - 事务ID
    pub fn cleanup_transaction(&self, txn_id: TransactionId) {
        // 获取该事务的所有保存点ID
        let sp_ids = self
            .txn_savepoints
            .write()
            .remove(&txn_id)
            .unwrap_or_default();

        // 从保存点映射中移除
        let mut savepoints = self.savepoints.write();
        for id in sp_ids {
            savepoints.remove(&id);
        }
        drop(savepoints);

        // 移除序列号生成器
        self.sequence_generator.write().remove(&txn_id);
    }

    /// 获取统计信息
    pub fn stats(&self) -> &SavepointStats {
        &self.stats
    }
}

impl Default for SavepointManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 保存点信息（用于API返回）
#[derive(Debug, Clone)]
pub struct SavepointInfo {
    /// 保存点ID
    pub id: SavepointId,
    /// 保存点名称
    pub name: Option<String>,
    /// 保存点序号
    pub sequence: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_savepoint() {
        let manager = SavepointManager::new();
        let txn_id: TransactionId = 1;

        let sp_id = manager
            .create_savepoint(txn_id, Some("test".to_string()))
            .expect("创建保存点失败");

        assert_eq!(sp_id.value(), 1);
        assert_eq!(
            manager.stats().total_created.load(Ordering::Relaxed),
            1
        );
    }

    #[test]
    fn test_create_multiple_savepoints() {
        let manager = SavepointManager::new();
        let txn_id: TransactionId = 1;

        let sp1 = manager.create_savepoint(txn_id, None).expect("创建保存点1失败");
        let sp2 = manager.create_savepoint(txn_id, None).expect("创建保存点2失败");
        let sp3 = manager.create_savepoint(txn_id, None).expect("创建保存点3失败");

        assert_eq!(sp1.value(), 1);
        assert_eq!(sp2.value(), 2);
        assert_eq!(sp3.value(), 3);

        let active_sps = manager.get_active_savepoints(txn_id);
        assert_eq!(active_sps.len(), 3);
    }

    #[test]
    fn test_rollback_to_savepoint() {
        let manager = SavepointManager::new();
        let txn_id: TransactionId = 1;

        let sp1 = manager.create_savepoint(txn_id, None).expect("创建保存点1失败");
        let sp2 = manager.create_savepoint(txn_id, None).expect("创建保存点2失败");
        let _sp3 = manager.create_savepoint(txn_id, None).expect("创建保存点3失败");

        // 回滚到sp1，sp2和sp3应该被标记为已回滚
        manager.rollback_to_savepoint(sp1).expect("回滚失败");

        let active_sps = manager.get_active_savepoints(txn_id);
        assert_eq!(active_sps.len(), 1);
        assert_eq!(active_sps[0].id, sp1);

        // sp2应该不能再回滚
        let result = manager.rollback_to_savepoint(sp2);
        assert!(result.is_err());
    }

    #[test]
    fn test_release_savepoint() {
        let manager = SavepointManager::new();
        let txn_id: TransactionId = 1;

        let sp = manager.create_savepoint(txn_id, None).expect("创建保存点失败");

        manager.release_savepoint(sp).expect("释放保存点失败");

        // 释放后不能再回滚
        let result = manager.rollback_to_savepoint(sp);
        assert!(result.is_err());

        assert_eq!(
            manager.stats().released_count.load(Ordering::Relaxed),
            1
        );
    }

    #[test]
    fn test_find_savepoint_by_name() {
        let manager = SavepointManager::new();
        let txn_id: TransactionId = 1;

        let _sp1 = manager
            .create_savepoint(txn_id, Some("first".to_string()))
            .expect("创建保存点1失败");
        let sp2 = manager
            .create_savepoint(txn_id, Some("second".to_string()))
            .expect("创建保存点2失败");

        let found = manager.find_savepoint_by_name(txn_id, "second");
        assert_eq!(found, Some(sp2));

        let not_found = manager.find_savepoint_by_name(txn_id, "third");
        assert_eq!(not_found, None);
    }

    #[test]
    fn test_cleanup_transaction() {
        let manager = SavepointManager::new();
        let txn_id: TransactionId = 1;

        let _sp1 = manager.create_savepoint(txn_id, None).expect("创建保存点1失败");
        let _sp2 = manager.create_savepoint(txn_id, None).expect("创建保存点2失败");

        manager.cleanup_transaction(txn_id);

        let active_sps = manager.get_active_savepoints(txn_id);
        assert!(active_sps.is_empty());
    }

    #[test]
    fn test_multiple_transactions() {
        let manager = SavepointManager::new();
        let txn1: TransactionId = 1;
        let txn2: TransactionId = 2;

        let sp1 = manager.create_savepoint(txn1, None).expect("创建保存点1失败");
        let sp2 = manager.create_savepoint(txn2, None).expect("创建保存点2失败");

        // 验证每个事务的保存点是独立的
        let active_sps1 = manager.get_active_savepoints(txn1);
        let active_sps2 = manager.get_active_savepoints(txn2);

        assert_eq!(active_sps1.len(), 1);
        assert_eq!(active_sps2.len(), 1);
        assert_eq!(active_sps1[0].id, sp1);
        assert_eq!(active_sps2[0].id, sp2);
    }
}

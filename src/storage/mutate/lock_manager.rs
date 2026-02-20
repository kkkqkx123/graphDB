//! 内存锁管理器
//!
//! 提供类似 NebulaGraph 的内存锁机制，用于并发控制
//! 支持顶点锁和边锁

use crate::core::{StorageError, Value};
use std::collections::HashSet;
use std::sync::Arc;
use parking_lot::Mutex;

/// 锁类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LockType {
    /// 顶点锁: (space_id, vertex_id)
    Vertex(u64, Value),
    /// 边锁: (space_id, src_id, edge_type, rank, dst_id)
    Edge(u64, Value, String, i64, Value),
    /// 标签锁: (space_id, vertex_id, tag_id)
    Tag(u64, Value, i32),
}

/// 内存锁管理器
///
/// 管理 DML 操作的内存锁，防止并发冲突
pub struct MemoryLockManager {
    /// 已锁定的资源集合
    locked_resources: HashSet<LockType>,
}

impl MemoryLockManager {
    /// 创建新的锁管理器
    pub fn new() -> Self {
        Self {
            locked_resources: HashSet::new(),
        }
    }

    /// 尝试获取锁
    ///
    /// # Arguments
    /// * `lock_type` - 锁类型
    ///
    /// # Returns
    /// * `Ok(true)` - 获取锁成功
    /// * `Ok(false)` - 资源已被锁定
    pub fn try_lock(&mut self, lock_type: LockType) -> Result<bool, StorageError> {
        if self.locked_resources.contains(&lock_type) {
            Ok(false)
        } else {
            self.locked_resources.insert(lock_type);
            Ok(true)
        }
    }

    /// 释放锁
    ///
    /// # Arguments
    /// * `lock_type` - 锁类型
    pub fn unlock(&mut self, lock_type: &LockType) {
        self.locked_resources.remove(lock_type);
    }

    /// 批量获取锁
    ///
    /// # Arguments
    /// * `locks` - 锁类型列表
    ///
    /// # Returns
    /// * `Ok(())` - 所有锁获取成功
    /// * `Err(StorageError)` - 部分锁获取失败，已获取的锁会被释放
    pub fn try_lock_batch(&mut self, locks: &[LockType]) -> Result<(), StorageError> {
        let mut acquired_locks = Vec::new();

        for lock in locks {
            if self.locked_resources.contains(lock) {
                // 释放已获取的锁
                for acquired in &acquired_locks {
                    self.locked_resources.remove(acquired);
                }
                return Err(StorageError::DbError(format!(
                    "资源已被锁定: {:?}",
                    lock
                )));
            }
            self.locked_resources.insert(lock.clone());
            acquired_locks.push(lock.clone());
        }

        Ok(())
    }

    /// 批量释放锁
    ///
    /// # Arguments
    /// * `locks` - 锁类型列表
    pub fn unlock_batch(&mut self, locks: &[LockType]) {
        for lock in locks {
            self.locked_resources.remove(lock);
        }
    }

    /// 检查资源是否被锁定
    ///
    /// # Arguments
    /// * `lock_type` - 锁类型
    ///
    /// # Returns
    /// * `true` - 资源已被锁定
    /// * `false` - 资源未被锁定
    pub fn is_locked(&self, lock_type: &LockType) -> bool {
        self.locked_resources.contains(lock_type)
    }

    /// 创建顶点锁
    pub fn create_vertex_lock(space_id: u64, vertex_id: Value) -> LockType {
        LockType::Vertex(space_id, vertex_id)
    }

    /// 创建边锁
    pub fn create_edge_lock(
        space_id: u64,
        src_id: Value,
        edge_type: String,
        rank: i64,
        dst_id: Value,
    ) -> LockType {
        LockType::Edge(space_id, src_id, edge_type, rank, dst_id)
    }

    /// 创建标签锁
    pub fn create_tag_lock(space_id: u64, vertex_id: Value, tag_id: i32) -> LockType {
        LockType::Tag(space_id, vertex_id, tag_id)
    }
}

impl Default for MemoryLockManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 锁守卫
///
/// 自动管理锁的生命周期，在 Drop 时自动释放锁
pub struct LockGuard {
    lock_manager: Arc<Mutex<MemoryLockManager>>,
    locks: Vec<LockType>,
}

impl LockGuard {
    /// 创建锁守卫
    pub fn new(
        lock_manager: Arc<Mutex<MemoryLockManager>>,
        locks: Vec<LockType>,
    ) -> Result<Self, StorageError> {
        let mut manager = lock_manager.lock();
        manager.try_lock_batch(&locks)?;
        drop(manager);
        
        Ok(Self {
            lock_manager,
            locks,
        })
    }

    /// 获取锁列表
    pub fn locks(&self) -> &[LockType] {
        &self.locks
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let mut manager = self.lock_manager.lock();
        manager.unlock_batch(&self.locks);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_lock_success() {
        let mut manager = MemoryLockManager::new();
        let lock = LockType::Vertex(1, Value::String("vid1".to_string()));

        assert!(manager.try_lock(lock.clone()).unwrap());
        assert!(manager.is_locked(&lock));
    }

    #[test]
    fn test_try_lock_conflict() {
        let mut manager = MemoryLockManager::new();
        let lock = LockType::Vertex(1, Value::String("vid1".to_string()));

        assert!(manager.try_lock(lock.clone()).unwrap());
        assert!(!manager.try_lock(lock.clone()).unwrap());
    }

    #[test]
    fn test_unlock() {
        let mut manager = MemoryLockManager::new();
        let lock = LockType::Vertex(1, Value::String("vid1".to_string()));

        manager.try_lock(lock.clone()).unwrap();
        assert!(manager.is_locked(&lock));

        manager.unlock(&lock);
        assert!(!manager.is_locked(&lock));
    }

    #[test]
    fn test_try_lock_batch_success() {
        let mut manager = MemoryLockManager::new();
        let locks = vec![
            LockType::Vertex(1, Value::String("vid1".to_string())),
            LockType::Vertex(1, Value::String("vid2".to_string())),
        ];

        assert!(manager.try_lock_batch(&locks).is_ok());
        assert!(manager.is_locked(&locks[0]));
        assert!(manager.is_locked(&locks[1]));
    }

    #[test]
    fn test_try_lock_batch_partial_failure() {
        let mut manager = MemoryLockManager::new();
        let lock1 = LockType::Vertex(1, Value::String("vid1".to_string()));
        let locks = vec![
            lock1.clone(),
            LockType::Vertex(1, Value::String("vid2".to_string())),
        ];

        // 先锁定第一个资源
        manager.try_lock(lock1.clone()).unwrap();

        // 批量获取锁应该失败
        assert!(manager.try_lock_batch(&locks).is_err());

        // 第二个锁不应该被获取
        assert!(!manager.is_locked(&locks[1]));
    }

    #[test]
    fn test_lock_guard() {
        let manager = Arc::new(Mutex::new(MemoryLockManager::new()));
        let lock = LockType::Vertex(1, Value::String("vid1".to_string()));

        {
            let guard = LockGuard::new(manager.clone(), vec![lock.clone()]).expect("创建锁守卫失败");
            assert!(manager.lock().is_locked(&lock));
            assert_eq!(guard.locks().len(), 1);
        }

        // 守卫被 Drop 后，锁应该被释放
        assert!(!manager.lock().is_locked(&lock));
    }
}

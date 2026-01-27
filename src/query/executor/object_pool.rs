//! 对象池模块
//!
//! 提供执行器对象池，减少频繁的内存分配和释放
//! 提高查询执行性能

use crate::query::executor::executor_enum::ExecutorEnum;
use crate::storage::StorageEngine;
use crate::utils::error_handling::safe_lock;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 对象池配置
#[derive(Debug, Clone)]
pub struct ObjectPoolConfig {
    /// 每种类型执行器的最大缓存数量
    pub max_pool_size: usize,
    /// 对象池是否启用
    pub enabled: bool,
}

impl Default for ObjectPoolConfig {
    fn default() -> Self {
        Self {
            max_pool_size: 10,
            enabled: true,
        }
    }
}

/// 对象池 - 缓存执行器实例
///
/// 使用对象池模式重用执行器实例，减少内存分配开销
pub struct ExecutorObjectPool<S: StorageEngine + 'static> {
    config: ObjectPoolConfig,
    pools: HashMap<String, Vec<ExecutorEnum<S>>>,
    stats: PoolStats,
}

/// 对象池统计信息
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// 总获取次数
    pub total_acquires: usize,
    /// 总释放次数
    pub total_releases: usize,
    /// 缓存命中次数
    pub cache_hits: usize,
    /// 缓存未命中次数
    pub cache_misses: usize,
}

impl<S: StorageEngine + 'static> ExecutorObjectPool<S> {
    /// 创建新的对象池
    pub fn new(config: ObjectPoolConfig) -> Self {
        Self {
            config,
            pools: HashMap::new(),
            stats: PoolStats::default(),
        }
    }

    /// 创建默认配置的对象池
    pub fn default_pool() -> Self {
        Self::new(ObjectPoolConfig::default())
    }

    /// 从对象池获取执行器
    ///
    /// 如果池中有可用的执行器，则返回缓存的实例
    /// 否则返回None，调用者需要创建新实例
    pub fn acquire(&mut self, executor_type: &str) -> Option<ExecutorEnum<S>> {
        if !self.config.enabled {
            return None;
        }

        self.stats.total_acquires += 1;

        let pool = self.pools.get_mut(executor_type);
        if let Some(executors) = pool {
            if let Some(executor) = executors.pop() {
                self.stats.cache_hits += 1;
                return Some(executor);
            }
        }

        self.stats.cache_misses += 1;
        None
    }

    /// 将执行器释放回对象池
    ///
    /// 如果池未满，则将执行器放回池中
    /// 否则丢弃执行器
    pub fn release(&mut self, executor_type: &str, executor: ExecutorEnum<S>) {
        if !self.config.enabled {
            return;
        }

        self.stats.total_releases += 1;

        let pool = self
            .pools
            .entry(executor_type.to_string())
            .or_insert_with(Vec::new);

        if pool.len() < self.config.max_pool_size {
            pool.push(executor);
        }
    }

    /// 清空对象池
    pub fn clear(&mut self) {
        self.pools.clear();
    }

    /// 获取对象池统计信息
    pub fn stats(&self) -> &PoolStats {
        &self.stats
    }

    /// 获取对象池配置
    pub fn config(&self) -> &ObjectPoolConfig {
        &self.config
    }

    /// 更新对象池配置
    pub fn set_config(&mut self, config: ObjectPoolConfig) {
        self.config = config;
    }

    /// 获取指定类型的池大小
    pub fn pool_size(&self, executor_type: &str) -> usize {
        self.pools
            .get(executor_type)
            .map(|pool| pool.len())
            .unwrap_or(0)
    }

    /// 获取总池大小
    pub fn total_size(&self) -> usize {
        self.pools.values().map(|pool| pool.len()).sum()
    }
}

/// 对象池包装器 - 提供线程安全的对象池
pub struct ThreadSafeExecutorPool<S: StorageEngine + 'static> {
    inner: Arc<Mutex<ExecutorObjectPool<S>>>,
}

impl<S: StorageEngine + 'static> ThreadSafeExecutorPool<S> {
    /// 创建新的线程安全对象池
    pub fn new(config: ObjectPoolConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(ExecutorObjectPool::new(config))),
        }
    }

    /// 创建默认配置的线程安全对象池
    pub fn default_pool() -> Self {
        Self::new(ObjectPoolConfig::default())
    }

    /// 从对象池获取执行器
    pub fn acquire(&self, executor_type: &str) -> Option<ExecutorEnum<S>> {
        let mut pool = safe_lock(&self.inner).ok()?;
        pool.acquire(executor_type)
    }

    /// 将执行器释放回对象池
    pub fn release(&self, executor_type: &str, executor: ExecutorEnum<S>) {
        if let Ok(mut pool) = safe_lock(&self.inner) {
            pool.release(executor_type, executor);
        }
    }

    /// 清空对象池
    pub fn clear(&self) {
        if let Ok(mut pool) = safe_lock(&self.inner) {
            pool.clear();
        }
    }

    /// 获取对象池统计信息
    pub fn stats(&self) -> PoolStats {
        safe_lock(&self.inner)
            .map(|pool| pool.stats().clone())
            .unwrap_or_else(|_| PoolStats::default())
    }

    /// 获取对象池配置
    pub fn config(&self) -> ObjectPoolConfig {
        safe_lock(&self.inner)
            .map(|pool| pool.config().clone())
            .unwrap_or_else(|_| ObjectPoolConfig::default())
    }

    /// 更新对象池配置
    pub fn set_config(&self, config: ObjectPoolConfig) {
        if let Ok(mut pool) = safe_lock(&self.inner) {
            pool.set_config(config);
        }
    }

    /// 获取指定类型的池大小
    pub fn pool_size(&self, executor_type: &str) -> usize {
        safe_lock(&self.inner)
            .map(|pool| pool.pool_size(executor_type))
            .unwrap_or(0)
    }

    /// 获取总池大小
    pub fn total_size(&self) -> usize {
        safe_lock(&self.inner)
            .map(|pool| pool.total_size())
            .unwrap_or(0)
    }
}

impl<S: StorageEngine + 'static> Clone for ThreadSafeExecutorPool<S> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::test_mock::MockStorage;

    #[test]
    fn test_object_pool_config_default() {
        let config = ObjectPoolConfig::default();
        assert_eq!(config.max_pool_size, 10);
        assert!(config.enabled);
    }

    #[test]
    fn test_object_pool_creation() {
        let pool = ExecutorObjectPool::<MockStorage>::default_pool();
        assert_eq!(pool.total_size(), 0);
    }

    #[test]
    fn test_object_pool_acquire_empty() {
        let mut pool = ExecutorObjectPool::<MockStorage>::default_pool();
        let executor = pool.acquire("TestExecutor");
        assert!(executor.is_none());
        assert_eq!(pool.stats().cache_misses, 1);
    }

    #[test]
    fn test_object_pool_release_and_acquire() {
        let pool = ExecutorObjectPool::<MockStorage>::default_pool();

        // 由于没有实际的执行器实现，这里只测试接口
        // 在实际使用中，会释放真实的执行器实例
        assert_eq!(pool.pool_size("TestExecutor"), 0);
    }

    #[test]
    fn test_thread_safe_pool() {
        let pool = ThreadSafeExecutorPool::<MockStorage>::default_pool();
        assert_eq!(pool.total_size(), 0);

        let executor = pool.acquire("TestExecutor");
        assert!(executor.is_none());
    }

    #[test]
    fn test_pool_stats() {
        let mut pool = ExecutorObjectPool::<MockStorage>::default_pool();
        pool.acquire("TestExecutor");
        pool.acquire("TestExecutor");

        assert_eq!(pool.stats().total_acquires, 2);
        assert_eq!(pool.stats().cache_misses, 2);
        assert_eq!(pool.stats().cache_hits, 0);
    }

    #[test]
    fn test_pool_clear() {
        let mut pool = ExecutorObjectPool::<MockStorage>::default_pool();
        pool.acquire("TestExecutor");
        pool.clear();
        assert_eq!(pool.total_size(), 0);
    }
}

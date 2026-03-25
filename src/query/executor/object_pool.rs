//! Object Pool Module
//!
//! Provide an executor object pool to reduce the frequent allocation and release of memory.
//! Improving the performance of query execution

use crate::query::executor::executor_enum::ExecutorEnum;
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

/// Object pool configuration
#[derive(Debug, Clone)]
pub struct ObjectPoolConfig {
    /// The maximum number of caches for each type of executor
    pub max_pool_size: usize,
    /// Is the object pool enabled?
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

/// Object pool: A cache for executor instances
///
/// Reuse executor instances by using the object pool pattern to reduce the overhead associated with memory allocation.
pub struct ExecutorObjectPool<S: StorageClient + 'static> {
    config: ObjectPoolConfig,
    pools: HashMap<String, Vec<ExecutorEnum<S>>>,
    stats: PoolStats,
}

/// Object pool statistics
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// Total number of acquisitions
    pub total_acquires: usize,
    /// Total number of releases
    pub total_releases: usize,
    /// Number of cache hits
    pub cache_hits: usize,
    /// Number of cache misses
    pub cache_misses: usize,
}

impl<S: StorageClient + 'static> ExecutorObjectPool<S> {
    /// Create a new object pool.
    pub fn new(config: ObjectPoolConfig) -> Self {
        Self {
            config,
            pools: HashMap::new(),
            stats: PoolStats::default(),
        }
    }

    /// Create an object pool with default configuration.
    pub fn default_pool() -> Self {
        Self::new(ObjectPoolConfig::default())
    }

    /// Obtain an executor from the object pool.
    ///
    /// If there are available executors in the pool, the cached instance is returned.
    /// Otherwise, return `None`. The caller will need to create a new instance.
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

    /// Release the executor back to the object pool.
    ///
    /// If the pool is not full, the executor will be returned to the pool.
    /// Otherwise, discard the actuator.
    pub fn release(&mut self, executor_type: &str, executor: ExecutorEnum<S>) {
        if !self.config.enabled {
            return;
        }

        self.stats.total_releases += 1;

        let pool = self.pools.entry(executor_type.to_string()).or_default();

        if pool.len() < self.config.max_pool_size {
            pool.push(executor);
        }
    }

    /// Clear the object pool.
    pub fn clear(&mut self) {
        self.pools.clear();
    }

    /// Obtain object pool statistics information
    pub fn stats(&self) -> &PoolStats {
        &self.stats
    }

    /// Obtaining the object pool configuration
    pub fn config(&self) -> &ObjectPoolConfig {
        &self.config
    }

    /// Update the object pool configuration.
    pub fn set_config(&mut self, config: ObjectPoolConfig) {
        self.config = config;
    }

    /// Obtain the pool size of the specified type.
    pub fn pool_size(&self, executor_type: &str) -> usize {
        self.pools
            .get(executor_type)
            .map(|pool| pool.len())
            .unwrap_or(0)
    }

    /// Obtain the total size of the pool.
    pub fn total_size(&self) -> usize {
        self.pools.values().map(|pool| pool.len()).sum()
    }
}

/// Object pool wrapper – Provides a thread-safe object pool.
pub struct ThreadSafeExecutorPool<S: StorageClient + 'static> {
    inner: Arc<Mutex<ExecutorObjectPool<S>>>,
}

impl<S: StorageClient + 'static> ThreadSafeExecutorPool<S> {
    /// Create a new thread-safe object pool.
    pub fn new(config: ObjectPoolConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(ExecutorObjectPool::new(config))),
        }
    }

    /// Create a thread-safe object pool with default configuration.
    pub fn default_pool() -> Self {
        Self::new(ObjectPoolConfig::default())
    }

    /// 从对象池获取执行器
    pub fn acquire(&self, executor_type: &str) -> Option<ExecutorEnum<S>> {
        let mut pool = self.inner.lock();
        pool.acquire(executor_type)
    }

    /// 将执行器释放回对象池
    pub fn release(&self, executor_type: &str, executor: ExecutorEnum<S>) {
        let mut pool = self.inner.lock();
        pool.release(executor_type, executor);
    }

    /// 清空对象池
    pub fn clear(&self) {
        let mut pool = self.inner.lock();
        pool.clear();
    }

    /// 获取对象池统计信息
    pub fn stats(&self) -> PoolStats {
        let pool = self.inner.lock();
        pool.stats().clone()
    }

    /// 获取对象池配置
    pub fn config(&self) -> ObjectPoolConfig {
        let pool = self.inner.lock();
        pool.config().clone()
    }

    /// 更新对象池配置
    pub fn set_config(&self, config: ObjectPoolConfig) {
        let mut pool = self.inner.lock();
        pool.set_config(config);
    }

    /// 获取指定类型的池大小
    pub fn pool_size(&self, executor_type: &str) -> usize {
        let pool = self.inner.lock();
        pool.pool_size(executor_type)
    }

    /// 获取总池大小
    pub fn total_size(&self) -> usize {
        let pool = self.inner.lock();
        pool.total_size()
    }
}

impl<S: StorageClient + 'static> Clone for ThreadSafeExecutorPool<S> {
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

        // Since there is no actual implementation of the actuator, only the interface is being tested here.
        // In practical use, real executor instances will be released.
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

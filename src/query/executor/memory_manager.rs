//! 内存管理模块
//!
//! 提供查询执行过程中的内存使用监控和限制功能

use crate::core::error::{DBError, DBResult};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// 内存使用配置
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// 单个查询最大内存使用（字节）
    pub max_query_memory: usize,
    /// 内存检查间隔（行数）
    pub check_interval: usize,
    /// 是否启用内存溢出到磁盘
    pub spill_enabled: bool,
    /// 内存溢出阈值（百分比，0-100）
    pub spill_threshold: u8,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_query_memory: 100 * 1024 * 1024, // 100MB 默认限制
            check_interval: 1000,                // 每1000行检查一次
            spill_enabled: true,
            spill_threshold: 80, // 80% 时开始溢出
        }
    }
}

/// 内存使用跟踪器
pub struct MemoryTracker {
    /// 当前内存使用量
    current_usage: AtomicUsize,
    /// 内存限制
    limit: usize,
    /// 配置
    config: MemoryConfig,
}

impl MemoryTracker {
    /// 创建新的内存跟踪器
    pub fn new(limit: usize, config: MemoryConfig) -> Self {
        Self {
            current_usage: AtomicUsize::new(0),
            limit,
            config,
        }
    }

    /// 分配内存
    pub fn allocate(&self, size: usize) -> DBResult<()> {
        let current = self.current_usage.fetch_add(size, Ordering::AcqRel);

        // 检查是否超出限制
        if current + size > self.limit {
            // 回滚分配
            self.current_usage.fetch_sub(size, Ordering::AcqRel);
            return Err(DBError::Internal(format!(
                "Memory limit exceeded: current={}, limit={}",
                current + size,
                self.limit
            )));
        }

        Ok(())
    }

    /// 释放内存
    pub fn deallocate(&self, size: usize) {
        self.current_usage.fetch_sub(size, Ordering::AcqRel);
    }

    /// 获取当前内存使用量
    pub fn current_usage(&self) -> usize {
        self.current_usage.load(Ordering::Acquire)
    }

    /// 检查是否应该溢出
    pub fn should_spill(&self) -> bool {
        let usage = self.current_usage();
        let threshold = (self.limit * self.config.spill_threshold as usize) / 100;
        usage >= threshold
    }

    /// 重置内存计数器
    pub fn reset(&self) {
        self.current_usage.store(0, Ordering::Release);
    }

    /// 获取内存使用比例（0-100）
    pub fn usage_ratio(&self) -> u8 {
        let current = self.current_usage();
        ((current as f64 / self.limit as f64) * 100.0) as u8
    }
}

/// 可追踪内存的数据结构包装器
pub struct TrackedVec<T> {
    inner: Vec<T>,
    tracker: Arc<MemoryTracker>,
    element_size: usize,
}

impl<T> TrackedVec<T>
where
    T: Sized,
{
    /// 创建新的跟踪向量
    pub fn new(tracker: Arc<MemoryTracker>) -> Self {
        Self {
            inner: Vec::new(),
            tracker,
            element_size: std::mem::size_of::<T>(),
        }
    }

    /// 创建指定容量的跟踪向量
    pub fn with_capacity(capacity: usize, tracker: Arc<MemoryTracker>) -> DBResult<Self> {
        let element_size = std::mem::size_of::<T>();
        let estimated_size = capacity * element_size;

        // 预分配内存检查
        tracker.allocate(estimated_size)?;

        Ok(Self {
            inner: Vec::with_capacity(capacity),
            tracker,
            element_size,
        })
    }

    /// 添加元素
    pub fn push(&mut self, value: T) -> DBResult<()> {
        // 为新元素分配内存
        self.tracker.allocate(self.element_size)?;
        self.inner.push(value);
        Ok(())
    }

    /// 获取长度
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// 获取内部引用
    pub fn as_slice(&self) -> &[T] {
        self.inner.as_slice()
    }

    /// 获取可变引用
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.inner.as_mut_slice()
    }

    /// 清空向量（释放内存跟踪）
    pub fn clear(&mut self) {
        let size = self.inner.len() * self.element_size;
        self.inner.clear();
        self.tracker.deallocate(size);
    }
}

impl<T> Drop for TrackedVec<T> {
    fn drop(&mut self) {
        let size = self.inner.len() * self.element_size;
        self.tracker.deallocate(size);
    }
}

/// 内存使用统计
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub current_usage: usize,
    pub peak_usage: usize,
    pub allocation_count: usize,
    pub deallocation_count: usize,
}

/// 内存管理器
pub struct MemoryManager {
    tracker: Arc<MemoryTracker>,
    stats: Arc<std::sync::Mutex<MemoryStats>>,
}

impl MemoryManager {
    /// 创建新的内存管理器
    pub fn new(config: MemoryConfig) -> Self {
        let tracker = Arc::new(MemoryTracker::new(config.max_query_memory, config.clone()));
        let stats = Arc::new(std::sync::Mutex::new(MemoryStats {
            current_usage: 0,
            peak_usage: 0,
            allocation_count: 0,
            deallocation_count: 0,
        }));

        Self { tracker, stats }
    }

    /// 获取内存跟踪器
    pub fn tracker(&self) -> Arc<MemoryTracker> {
        self.tracker.clone()
    }

    /// 获取内存统计
    pub fn get_stats(&self) -> MemoryStats {
        self.stats.lock().expect("Failed to acquire lock on memory stats").clone()
    }

    /// 记录分配
    pub fn record_allocation(&self, size: usize) {
        let mut stats = self.stats.lock().expect("Failed to acquire lock on memory stats");
        stats.allocation_count += 1;
        stats.current_usage += size;
        if stats.current_usage > stats.peak_usage {
            stats.peak_usage = stats.current_usage;
        }
    }

    /// 记录释放
    pub fn record_deallocation(&self, size: usize) {
        let mut stats = self.stats.lock().expect("Failed to acquire lock on memory stats");
        stats.deallocation_count += 1;
        stats.current_usage = stats.current_usage.saturating_sub(size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tracker_basic() {
        let config = MemoryConfig::default();
        let tracker = MemoryTracker::new(1024, config);

        // 测试基本分配和释放
        assert!(tracker.allocate(100).is_ok());
        assert_eq!(tracker.current_usage(), 100);
        assert_eq!(tracker.usage_ratio(), 9); // ~9.7%

        tracker.deallocate(50);
        assert_eq!(tracker.current_usage(), 50);

        // 测试超出限制
        assert!(tracker.allocate(1000).is_err());
        assert_eq!(tracker.current_usage(), 50); // 应该回滚
    }

    #[test]
    fn test_memory_tracker_spill_detection() {
        let mut config = MemoryConfig::default();
        config.spill_threshold = 70;
        let tracker = MemoryTracker::new(1000, config);

        assert!(!tracker.should_spill());

        tracker.allocate(600).expect("allocation should succeed");
        assert!(!tracker.should_spill()); // 60% < 70%，不应该溢出

        tracker.allocate(100).expect("allocation should succeed");
        assert!(tracker.should_spill()); // 70% >= 70%，应该溢出
    }

    #[test]
    fn test_tracked_vec() {
        let config = MemoryConfig::default();
        let tracker = Arc::new(MemoryTracker::new(1024, config));
        let mut vec = TrackedVec::<i32>::new(tracker.clone());

        // 测试基本操作
        vec.push(42).expect("push should succeed");
        vec.push(43).expect("push should succeed");

        assert_eq!(vec.len(), 2);
        assert_eq!(vec.as_slice(), &[42, 43]);

        // 测试内存跟踪（跟踪容量而不是长度）
        let expected_size = 2 * std::mem::size_of::<i32>();
        assert_eq!(tracker.current_usage(), expected_size);

        // 测试清空
        vec.clear();
        assert_eq!(tracker.current_usage(), 0);
    }

    #[test]
    fn test_tracked_vec_capacity() {
        let config = MemoryConfig::default();
        let tracker = Arc::new(MemoryTracker::new(1024, config));

        // 测试带容量的创建
        let vec = TrackedVec::<i32>::with_capacity(10, tracker.clone());
        assert!(vec.is_ok());

        // 测试超出内存限制
        let tracker_small = Arc::new(MemoryTracker::new(10, MemoryConfig::default()));
        let vec_large = TrackedVec::<i32>::with_capacity(10, tracker_small);
        assert!(vec_large.is_err());
    }
}

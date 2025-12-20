//! 内存管理器模块 - 提供内存使用监控和管理功能

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

/// 内存管理器trait
pub trait MemoryManager: Send + Sync {
    fn check_memory(&self, bytes: u64) -> Result<bool, String>;
    fn register_allocation(&self, bytes: u64);
    fn register_deallocation(&self, bytes: u64);
    fn get_current_usage(&self) -> u64;
    fn get_limit(&self) -> u64;
    fn get_peak_usage(&self) -> u64;
}

/// 简单的内存管理器实现
#[derive(Debug, Clone)]
pub struct SimpleMemoryManager {
    current_usage: Arc<AtomicU64>,
    peak_usage: Arc<AtomicU64>,
    limit: u64,
}

impl SimpleMemoryManager {
    pub fn new(limit: u64) -> Self {
        Self {
            current_usage: Arc::new(AtomicU64::new(0)),
            peak_usage: Arc::new(AtomicU64::new(0)),
            limit,
        }
    }

    pub fn with_default_limit() -> Self {
        Self::new(100 * 1024 * 1024) // 默认100MB
    }
}

impl MemoryManager for SimpleMemoryManager {
    fn check_memory(&self, bytes: u64) -> Result<bool, String> {
        let current = self.current_usage.load(Ordering::Relaxed);
        if current + bytes > self.limit {
            Err(format!(
                "Memory limit exceeded: {} + {} > {}",
                current, bytes, self.limit
            ))
        } else {
            Ok(true)
        }
    }

    fn register_allocation(&self, bytes: u64) {
        let old_usage = self.current_usage.fetch_add(bytes, Ordering::Relaxed);
        let new_usage = old_usage + bytes;

        // 更新峰值使用量
        let mut current_peak = self.peak_usage.load(Ordering::Relaxed);
        while new_usage > current_peak {
            match self.peak_usage.compare_exchange_weak(
                current_peak,
                new_usage,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_peak = actual,
            }
        }
    }

    fn register_deallocation(&self, bytes: u64) {
        self.current_usage.fetch_sub(bytes, Ordering::Relaxed);
    }

    fn get_current_usage(&self) -> u64 {
        self.current_usage.load(Ordering::Relaxed)
    }

    fn get_limit(&self) -> u64 {
        self.limit
    }

    fn get_peak_usage(&self) -> u64 {
        self.peak_usage.load(Ordering::Relaxed)
    }
}

/// 内存使用统计信息
#[derive(Debug, Clone)]
pub struct MemoryUsageInfo {
    pub current_usage: u64,
    pub peak_usage: u64,
    pub limit: u64,
    pub utilization_ratio: f64,
}

impl MemoryUsageInfo {
    pub fn new(current: u64, peak: u64, limit: u64) -> Self {
        let utilization_ratio = if limit > 0 {
            current as f64 / limit as f64
        } else {
            0.0
        };

        Self {
            current_usage: current,
            peak_usage: peak,
            limit,
            utilization_ratio,
        }
    }
}

/// 内存监控器
#[derive(Clone)]
pub struct MemoryMonitor {
    manager: Arc<dyn MemoryManager>,
}

impl std::fmt::Debug for MemoryMonitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryMonitor")
            .field("current_usage", &self.manager.get_current_usage())
            .field("peak_usage", &self.manager.get_peak_usage())
            .field("limit", &self.manager.get_limit())
            .finish()
    }
}

impl MemoryMonitor {
    pub fn new(manager: Arc<dyn MemoryManager>) -> Self {
        Self { manager }
    }

    /// 获取内存使用信息
    pub fn get_usage_info(&self) -> MemoryUsageInfo {
        MemoryUsageInfo::new(
            self.manager.get_current_usage(),
            self.manager.get_peak_usage(),
            self.manager.get_limit(),
        )
    }

    /// 检查是否接近内存限制
    pub fn is_near_limit(&self, threshold: f64) -> bool {
        let info = self.get_usage_info();
        info.utilization_ratio > threshold
    }

    /// 获取内存使用率
    pub fn utilization_ratio(&self) -> f64 {
        self.get_usage_info().utilization_ratio
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_memory_manager() {
        let manager = SimpleMemoryManager::new(1000);

        assert_eq!(manager.get_limit(), 1000);
        assert_eq!(manager.get_current_usage(), 0);
        assert_eq!(manager.get_peak_usage(), 0);

        // 检查内存
        assert!(manager
            .check_memory(500)
            .expect("Memory check should succeed when within limit"));

        // 注册分配
        manager.register_allocation(500);
        assert_eq!(manager.get_current_usage(), 500);
        assert_eq!(manager.get_peak_usage(), 500);

        // 检查剩余内存
        assert!(manager
            .check_memory(400)
            .expect("Memory check should succeed when within remaining limit"));

        // 超过限制
        assert!(manager.check_memory(600).is_err());

        // 注册释放
        manager.register_deallocation(200);
        assert_eq!(manager.get_current_usage(), 300);

        // 峰值应该保持不变
        assert_eq!(manager.get_peak_usage(), 500);
    }

    #[test]
    fn test_memory_usage_info() {
        let info = MemoryUsageInfo::new(500, 800, 1000);

        assert_eq!(info.current_usage, 500);
        assert_eq!(info.peak_usage, 800);
        assert_eq!(info.limit, 1000);
        assert_eq!(info.utilization_ratio, 0.5);
    }

    #[test]
    fn test_memory_monitor() {
        let manager = Arc::new(SimpleMemoryManager::new(1000));
        let monitor = MemoryMonitor::new(manager.clone());

        manager.register_allocation(300);

        let info = monitor.get_usage_info();
        assert_eq!(info.current_usage, 300);
        assert_eq!(info.utilization_ratio, 0.3);

        assert!(!monitor.is_near_limit(0.5));
        assert!(monitor.is_near_limit(0.2));

        assert_eq!(monitor.utilization_ratio(), 0.3);
    }

    #[test]
    fn test_default_limit() {
        let manager = SimpleMemoryManager::with_default_limit();
        assert_eq!(manager.get_limit(), 100 * 1024 * 1024);
    }

    #[test]
    fn test_peak_usage_tracking() {
        let manager = SimpleMemoryManager::new(1000);

        manager.register_allocation(200);
        assert_eq!(manager.get_peak_usage(), 200);

        manager.register_allocation(300);
        assert_eq!(manager.get_peak_usage(), 500);

        manager.register_deallocation(100);
        assert_eq!(manager.get_current_usage(), 400);
        assert_eq!(manager.get_peak_usage(), 500); // 峰值应该保持
    }
}

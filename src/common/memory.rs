use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};

/// 内存统计信息
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub current_usage: u64,
    pub peak_usage: u64,
    pub limit: u64,
    pub allocation_count: u64,
    pub deallocation_count: u64,
}

impl MemoryStats {
    pub fn new(current: u64, peak: u64, limit: u64) -> Self {
        Self {
            current_usage: current,
            peak_usage: peak,
            limit,
            allocation_count: 0,
            deallocation_count: 0,
        }
    }

    pub fn utilization_ratio(&self) -> f64 {
        if self.limit > 0 {
            self.current_usage as f64 / self.limit as f64
        } else {
            0.0
        }
    }
}

/// 内存管理配置
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    pub max_query_memory: u64,
    pub check_interval: usize,
    pub spill_enabled: bool,
    pub spill_threshold: u8,
    pub enable_system_monitor: bool,
    pub limit_ratio: f64,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_query_memory: 100 * 1024 * 1024,
            check_interval: 1000,
            spill_enabled: true,
            spill_threshold: 80,
            enable_system_monitor: true,
            limit_ratio: 0.8,
        }
    }
}

impl MemoryConfig {
    pub fn new(limit: u64) -> Self {
        Self {
            max_query_memory: limit,
            ..Default::default()
        }
    }

    pub fn with_system_monitor(mut self, enable: bool) -> Self {
        self.enable_system_monitor = enable;
        self
    }

    pub fn with_check_interval(mut self, interval: usize) -> Self {
        self.check_interval = interval;
        self
    }

    pub fn with_limit_ratio(mut self, ratio: f64) -> Self {
        self.limit_ratio = ratio;
        self
    }

    pub fn with_spill_threshold(mut self, threshold: u8) -> Self {
        self.spill_threshold = threshold.clamp(50, 100);
        self
    }

    pub fn with_spill_enabled(mut self, enabled: bool) -> Self {
        self.spill_enabled = enabled;
        self
    }
}

/// 全局内存管理器
pub struct GlobalMemoryManager {
    limit: AtomicU64,
    used: AtomicU64,
    peak: AtomicU64,
    allocation_count: AtomicU64,
    deallocation_count: AtomicU64,
}

impl GlobalMemoryManager {
    pub fn new(limit: u64) -> Self {
        Self {
            limit: AtomicU64::new(limit),
            used: AtomicU64::new(0),
            peak: AtomicU64::new(0),
            allocation_count: AtomicU64::new(0),
            deallocation_count: AtomicU64::new(0),
        }
    }

    pub fn with_config(config: &MemoryConfig) -> Self {
        let limit = if config.max_query_memory > 0 {
            config.max_query_memory
        } else {
            100 * 1024 * 1024
        };

        Self::new(limit)
    }

    pub fn adjust_limit_based_on_system(&self, available_memory: u64, ratio: f64) {
        let new_limit = (available_memory as f64 * ratio) as u64;
        self.set_limit(new_limit);
    }

    pub fn is_memory_pressure(&self, threshold: u8) -> bool {
        let limit_val = self.limit();
        let utilization = if limit_val > 0 {
            (self.current_usage() * 100) / limit_val
        } else {
            0
        };
        utilization >= threshold as u64
    }

    pub fn can_allocate(&self, size: u64) -> bool {
        self.current_usage() + size <= self.limit()
    }

    pub fn remaining_capacity(&self) -> u64 {
        self.limit().saturating_sub(self.current_usage())
    }

    pub fn alloc(&self, size: u64, _throw_if_exceeded: bool) -> Result<(), String> {
        let old_used = self.used.fetch_add(size, Ordering::Relaxed);
        let new_used = old_used + size;

        let limit = self.limit.load(Ordering::Relaxed);
        if new_used > limit {
            self.used.fetch_sub(size, Ordering::Relaxed);
            return Err(format!(
                "Memory limit exceeded: {} + {} > {}",
                old_used, size, limit
            ));
        }

        self.allocation_count.fetch_add(1, Ordering::Relaxed);

        let mut current_peak = self.peak.load(Ordering::Relaxed);
        while new_used > current_peak {
            match self.peak.compare_exchange_weak(
                current_peak,
                new_used,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_peak = actual,
            }
        }

        Ok(())
    }

    pub fn free(&self, size: u64) {
        self.used.fetch_sub(size, Ordering::Relaxed);
        self.deallocation_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn current_usage(&self) -> u64 {
        self.used.load(Ordering::Relaxed)
    }

    pub fn peak_usage(&self) -> u64 {
        self.peak.load(Ordering::Relaxed)
    }

    pub fn limit(&self) -> u64 {
        self.limit.load(Ordering::Relaxed)
    }

    pub fn set_limit(&self, limit: u64) {
        self.limit.store(limit, Ordering::Relaxed);
    }

    pub fn stats(&self) -> MemoryStats {
        MemoryStats {
            current_usage: self.current_usage(),
            peak_usage: self.peak_usage(),
            limit: self.limit(),
            allocation_count: self.allocation_count.load(Ordering::Relaxed),
            deallocation_count: self.deallocation_count.load(Ordering::Relaxed),
        }
    }
}

/// 全局内存管理器单例
static GLOBAL_MEMORY_MANAGER: OnceLock<Arc<GlobalMemoryManager>> = OnceLock::new();

/// 获取全局内存管理器
pub fn global_memory_manager() -> &'static Arc<GlobalMemoryManager> {
    GLOBAL_MEMORY_MANAGER.get_or_init(|| {
        Arc::new(GlobalMemoryManager::new(100 * 1024 * 1024))
    })
}

/// 内存跟踪器，用于监控内存分配
#[derive(Debug)]
pub struct MemoryTracker {
    limit: AtomicI64,
    used: AtomicI64,
    peak: AtomicI64,
    allocation_count: AtomicU64,
    deallocation_count: AtomicU64,
}

impl MemoryTracker {
    pub fn new(limit: i64) -> Self {
        Self {
            limit: AtomicI64::new(limit),
            used: AtomicI64::new(0),
            peak: AtomicI64::new(0),
            allocation_count: AtomicU64::new(0),
            deallocation_count: AtomicU64::new(0),
        }
    }

    pub fn record_allocation(&self, size: usize) -> Result<(), String> {
        let size = size as i64;
        let new_used = self.used.fetch_add(size, Ordering::Relaxed) + size;
        
        if new_used > self.limit.load(Ordering::Relaxed) {
            self.used.fetch_sub(size, Ordering::Relaxed);
            return Err(format!(
                "Memory limit exceeded: {} > {}",
                new_used,
                self.limit.load(Ordering::Relaxed)
            ));
        }
        
        self.allocation_count.fetch_add(1, Ordering::Relaxed);
        self.peak.fetch_max(new_used, Ordering::Relaxed);
        
        Ok(())
    }

    pub fn record_deallocation(&self, size: usize) {
        self.used.fetch_sub(size as i64, Ordering::Relaxed);
        self.deallocation_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn current_usage(&self) -> i64 {
        self.used.load(Ordering::Relaxed)
    }

    pub fn peak_usage(&self) -> i64 {
        self.peak.load(Ordering::Relaxed)
    }

    pub fn limit(&self) -> i64 {
        self.limit.load(Ordering::Relaxed)
    }

    pub fn set_limit(&self, limit: i64) {
        self.limit.store(limit, Ordering::Relaxed);
    }

    pub fn allocation_count(&self) -> u64 {
        self.allocation_count.load(Ordering::Relaxed)
    }

    pub fn deallocation_count(&self) -> u64 {
        self.deallocation_count.load(Ordering::Relaxed)
    }

    pub fn reset(&self) {
        self.allocation_count.store(0, Ordering::Relaxed);
        self.deallocation_count.store(0, Ordering::Relaxed);
        self.used.store(0, Ordering::Relaxed);
        self.peak.store(0, Ordering::Relaxed);
    }
}

/// 全局内存跟踪器
static MEMORY_TRACKER: once_cell::sync::Lazy<MemoryTracker> =
    once_cell::sync::Lazy::new(|| MemoryTracker::new(100 * 1024 * 1024));

/// 获取全局内存跟踪器的引用
pub fn memory_tracker() -> &'static MemoryTracker {
    &MEMORY_TRACKER
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tracker() {
        let tracker = MemoryTracker::new(1000);

        tracker.record_allocation(100).expect("Alloc failed");
        assert_eq!(tracker.allocation_count(), 1);
        assert_eq!(tracker.current_usage(), 100);

        tracker.record_allocation(50).expect("Alloc failed");
        assert_eq!(tracker.allocation_count(), 2);
        assert_eq!(tracker.current_usage(), 150);

        tracker.record_deallocation(30);
        assert_eq!(tracker.deallocation_count(), 1);
        assert_eq!(tracker.current_usage(), 120);

        tracker.reset();
        assert_eq!(tracker.allocation_count(), 0);
        assert_eq!(tracker.current_usage(), 0);
    }

    #[test]
    fn test_global_memory_manager() {
        let manager = GlobalMemoryManager::new(1000);

        assert_eq!(manager.limit(), 1000);
        assert_eq!(manager.current_usage(), 0);
        assert_eq!(manager.peak_usage(), 0);

        assert!(manager.alloc(500, false).is_ok());
        assert_eq!(manager.current_usage(), 500);
        assert_eq!(manager.peak_usage(), 500);

        assert!(manager.alloc(400, false).is_ok());
        assert_eq!(manager.current_usage(), 900);
        assert_eq!(manager.peak_usage(), 900);

        assert!(manager.alloc(600, false).is_err());

        manager.free(200);
        assert_eq!(manager.current_usage(), 700);

        assert_eq!(manager.peak_usage(), 900);
    }

    #[test]
    fn test_memory_tracker_limit() {
        let tracker = MemoryTracker::new(100);

        assert!(tracker.record_allocation(50).is_ok());
        assert!(tracker.record_allocation(60).is_err());
        
        assert_eq!(tracker.current_usage(), 50);
    }
}

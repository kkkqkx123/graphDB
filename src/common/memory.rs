use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};


/// 内存管理配置
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    pub max_query_memory: u64,
    pub spill_enabled: bool,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_query_memory: 100 * 1024 * 1024,
            spill_enabled: true,
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

    pub fn with_spill_enabled(mut self, enabled: bool) -> Self {
        self.spill_enabled = enabled;
        self
    }
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
    fn test_memory_tracker_limit() {
        let tracker = MemoryTracker::new(100);

        assert!(tracker.record_allocation(50).is_ok());
        assert!(tracker.record_allocation(60).is_err());
        
        assert_eq!(tracker.current_usage(), 50);
    }

}

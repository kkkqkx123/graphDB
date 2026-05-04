//! Memory Tracker
//!
//! Tracks memory usage across storage components with atomic operations.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use super::MemoryConfig;

/// Memory tracker for monitoring and limiting memory usage
#[derive(Debug)]
pub struct MemoryTracker {
    /// Current vertex memory usage
    vertex_memory: AtomicUsize,
    /// Current edge memory usage
    edge_memory: AtomicUsize,
    /// Current cache memory usage
    cache_memory: AtomicUsize,
    /// Configuration
    config: MemoryConfig,
    /// Whether stalling is active
    stalling: AtomicBool,
    /// Peak memory usage
    peak_memory: AtomicUsize,
}

impl MemoryTracker {
    /// Create a new memory tracker with the given configuration
    pub fn new(config: MemoryConfig) -> Self {
        Self {
            vertex_memory: AtomicUsize::new(0),
            edge_memory: AtomicUsize::new(0),
            cache_memory: AtomicUsize::new(0),
            config,
            stalling: AtomicBool::new(false),
            peak_memory: AtomicUsize::new(0),
        }
    }

    /// Create a new memory tracker with default configuration
    pub fn with_defaults() -> Self {
        Self::new(MemoryConfig::default())
    }

    /// Try to allocate memory for vertex data
    /// Returns true if allocation succeeded, false if limit exceeded
    pub fn try_allocate_vertex(&self, size: usize) -> bool {
        self.try_allocate_internal(
            &self.vertex_memory,
            size,
            self.config.max_vertex_memory(),
        )
    }

    /// Try to allocate memory for edge data
    /// Returns true if allocation succeeded, false if limit exceeded
    pub fn try_allocate_edge(&self, size: usize) -> bool {
        self.try_allocate_internal(
            &self.edge_memory,
            size,
            self.config.max_edge_memory(),
        )
    }

    /// Try to allocate memory for cache
    /// Returns true if allocation succeeded, false if limit exceeded
    pub fn try_allocate_cache(&self, size: usize) -> bool {
        self.try_allocate_internal(
            &self.cache_memory,
            size,
            self.config.max_cache_memory(),
        )
    }

    /// Internal allocation logic
    fn try_allocate_internal(
        &self,
        counter: &AtomicUsize,
        size: usize,
        max: usize,
    ) -> bool {
        loop {
            let current = counter.load(Ordering::Relaxed);
            let new_total = current.saturating_add(size);

            if new_total > max {
                if self.config.is_stall_enabled() && self.should_stall() {
                    self.stalling.store(true, Ordering::Relaxed);
                    self.wait_for_memory();
                    continue;
                }
                return false;
            }

            match counter.compare_exchange_weak(
                current,
                new_total,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    self.update_peak();
                    return true;
                }
                Err(_) => continue,
            }
        }
    }

    /// Force allocate memory (may exceed soft limit)
    /// Use with caution - primarily for critical allocations
    pub fn force_allocate_vertex(&self, size: usize) {
        self.vertex_memory.fetch_add(size, Ordering::SeqCst);
        self.update_peak();
    }

    /// Force allocate memory for edge data
    pub fn force_allocate_edge(&self, size: usize) {
        self.edge_memory.fetch_add(size, Ordering::SeqCst);
        self.update_peak();
    }

    /// Force allocate memory for cache
    pub fn force_allocate_cache(&self, size: usize) {
        self.cache_memory.fetch_add(size, Ordering::SeqCst);
        self.update_peak();
    }

    /// Release vertex memory
    pub fn release_vertex(&self, size: usize) {
        self.vertex_memory.fetch_sub(size, Ordering::SeqCst);
        self.check_stall_recovery();
    }

    /// Release edge memory
    pub fn release_edge(&self, size: usize) {
        self.edge_memory.fetch_sub(size, Ordering::SeqCst);
        self.check_stall_recovery();
    }

    /// Release cache memory
    pub fn release_cache(&self, size: usize) {
        self.cache_memory.fetch_sub(size, Ordering::SeqCst);
        self.check_stall_recovery();
    }

    /// Get current vertex memory usage
    pub fn vertex_memory_usage(&self) -> usize {
        self.vertex_memory.load(Ordering::Relaxed)
    }

    /// Get current edge memory usage
    pub fn edge_memory_usage(&self) -> usize {
        self.edge_memory.load(Ordering::Relaxed)
    }

    /// Get current cache memory usage
    pub fn cache_memory_usage(&self) -> usize {
        self.cache_memory.load(Ordering::Relaxed)
    }

    /// Get total memory usage
    pub fn total_memory_usage(&self) -> usize {
        self.vertex_memory_usage()
            .saturating_add(self.edge_memory_usage())
            .saturating_add(self.cache_memory_usage())
    }

    /// Get peak memory usage
    pub fn peak_memory_usage(&self) -> usize {
        self.peak_memory.load(Ordering::Relaxed)
    }

    /// Get memory usage statistics
    pub fn stats(&self) -> MemoryStats {
        MemoryStats {
            vertex_memory: self.vertex_memory_usage(),
            edge_memory: self.edge_memory_usage(),
            cache_memory: self.cache_memory_usage(),
            total_memory: self.total_memory_usage(),
            peak_memory: self.peak_memory_usage(),
            max_vertex_memory: self.config.max_vertex_memory(),
            max_edge_memory: self.config.max_edge_memory(),
            max_cache_memory: self.config.max_cache_memory(),
            max_total_memory: self.config.max_total_memory,
            is_stalling: self.stalling.load(Ordering::Relaxed),
        }
    }

    /// Check if we should start stalling
    fn should_stall(&self) -> bool {
        self.total_memory_usage() >= self.config.stall_threshold_bytes()
    }

    /// Wait for memory to become available
    fn wait_for_memory(&self) {
        let start = Instant::now();
        let max_wait = Duration::from_millis(100);

        while self.should_stall() && start.elapsed() < max_wait {
            thread::yield_now();
        }
    }

    /// Check if stalling can be recovered
    fn check_stall_recovery(&self) {
        if !self.should_stall() {
            self.stalling.store(false, Ordering::Relaxed);
        }
    }

    /// Update peak memory tracking
    fn update_peak(&self) {
        let current = self.total_memory_usage();
        loop {
            let peak = self.peak_memory.load(Ordering::Relaxed);
            if current <= peak {
                break;
            }
            match self.peak_memory.compare_exchange_weak(
                peak,
                current,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(_) => continue,
            }
        }
    }

    /// Reset all counters (use with caution)
    pub fn reset(&self) {
        self.vertex_memory.store(0, Ordering::SeqCst);
        self.edge_memory.store(0, Ordering::SeqCst);
        self.cache_memory.store(0, Ordering::SeqCst);
        self.peak_memory.store(0, Ordering::SeqCst);
        self.stalling.store(false, Ordering::SeqCst);
    }

    /// Check if memory usage is within limits
    pub fn is_within_limits(&self) -> bool {
        self.vertex_memory_usage() <= self.config.max_vertex_memory()
            && self.edge_memory_usage() <= self.config.max_edge_memory()
            && self.cache_memory_usage() <= self.config.max_cache_memory()
    }

    /// Get memory utilization percentage (0.0 - 1.0)
    pub fn utilization(&self) -> f32 {
        self.total_memory_usage() as f32 / self.config.max_total_memory as f32
    }
}

impl Clone for MemoryTracker {
    fn clone(&self) -> Self {
        Self {
            vertex_memory: AtomicUsize::new(self.vertex_memory_usage()),
            edge_memory: AtomicUsize::new(self.edge_memory_usage()),
            cache_memory: AtomicUsize::new(self.cache_memory_usage()),
            config: self.config.clone(),
            stalling: AtomicBool::new(self.stalling.load(Ordering::Relaxed)),
            peak_memory: AtomicUsize::new(self.peak_memory_usage()),
        }
    }
}

/// Memory usage statistics
#[derive(Debug, Clone, Copy)]
pub struct MemoryStats {
    /// Current vertex memory usage
    pub vertex_memory: usize,
    /// Current edge memory usage
    pub edge_memory: usize,
    /// Current cache memory usage
    pub cache_memory: usize,
    /// Total memory usage
    pub total_memory: usize,
    /// Peak memory usage
    pub peak_memory: usize,
    /// Maximum vertex memory
    pub max_vertex_memory: usize,
    /// Maximum edge memory
    pub max_edge_memory: usize,
    /// Maximum cache memory
    pub max_cache_memory: usize,
    /// Maximum total memory
    pub max_total_memory: usize,
    /// Whether stalling is active
    pub is_stalling: bool,
}

impl MemoryStats {
    /// Get vertex memory utilization (0.0 - 1.0)
    pub fn vertex_utilization(&self) -> f32 {
        if self.max_vertex_memory == 0 {
            return 0.0;
        }
        self.vertex_memory as f32 / self.max_vertex_memory as f32
    }

    /// Get edge memory utilization (0.0 - 1.0)
    pub fn edge_utilization(&self) -> f32 {
        if self.max_edge_memory == 0 {
            return 0.0;
        }
        self.edge_memory as f32 / self.max_edge_memory as f32
    }

    /// Get cache memory utilization (0.0 - 1.0)
    pub fn cache_utilization(&self) -> f32 {
        if self.max_cache_memory == 0 {
            return 0.0;
        }
        self.cache_memory as f32 / self.max_cache_memory as f32
    }

    /// Get total memory utilization (0.0 - 1.0)
    pub fn total_utilization(&self) -> f32 {
        if self.max_total_memory == 0 {
            return 0.0;
        }
        self.total_memory as f32 / self.max_total_memory as f32
    }

    /// Format bytes as human-readable string
    pub fn format_bytes(bytes: usize) -> String {
        const KB: usize = 1024;
        const MB: usize = KB * 1024;
        const GB: usize = MB * 1024;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }
}

impl std::fmt::Display for MemoryStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Memory Usage: {}/{} ({:.1}%)",
            Self::format_bytes(self.total_memory),
            Self::format_bytes(self.max_total_memory),
            self.total_utilization() * 100.0
        )?;
        writeln!(
            f,
            "  Vertex: {}/{} ({:.1}%)",
            Self::format_bytes(self.vertex_memory),
            Self::format_bytes(self.max_vertex_memory),
            self.vertex_utilization() * 100.0
        )?;
        writeln!(
            f,
            "  Edge: {}/{} ({:.1}%)",
            Self::format_bytes(self.edge_memory),
            Self::format_bytes(self.max_edge_memory),
            self.edge_utilization() * 100.0
        )?;
        writeln!(
            f,
            "  Cache: {}/{} ({:.1}%)",
            Self::format_bytes(self.cache_memory),
            Self::format_bytes(self.max_cache_memory),
            self.cache_utilization() * 100.0
        )?;
        writeln!(
            f,
            "  Peak: {}",
            Self::format_bytes(self.peak_memory)
        )?;
        if self.is_stalling {
            write!(f, "  Status: STALLING")?;
        }
        Ok(())
    }
}

/// Shared memory tracker wrapper
pub type SharedMemoryTracker = Arc<MemoryTracker>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryConfigBuilder;

    #[test]
    fn test_basic_allocation() {
        let config = MemoryConfig::with_total_memory(1000);
        let tracker = MemoryTracker::new(config);

        assert!(tracker.try_allocate_vertex(100));
        assert_eq!(tracker.vertex_memory_usage(), 100);

        assert!(tracker.try_allocate_edge(200));
        assert_eq!(tracker.edge_memory_usage(), 200);

        assert!(tracker.try_allocate_cache(50));
        assert_eq!(tracker.cache_memory_usage(), 50);
    }

    #[test]
    fn test_memory_limit() {
        let config = MemoryConfigBuilder::default()
            .total_memory(100)
            .vertex_ratio(0.4)
            .edge_ratio(0.4)
            .cache_ratio(0.2)
            .enable_stall(false)
            .build()
            .unwrap();

        let tracker = MemoryTracker::new(config);

        // Should succeed (40 bytes limit for vertex)
        assert!(tracker.try_allocate_vertex(30));
        // Should fail (exceeds limit)
        assert!(!tracker.try_allocate_vertex(20));
    }

    #[test]
    fn test_release_memory() {
        let config = MemoryConfig::with_total_memory(1000);
        let tracker = MemoryTracker::new(config);

        assert!(tracker.try_allocate_vertex(100));
        assert_eq!(tracker.vertex_memory_usage(), 100);

        tracker.release_vertex(50);
        assert_eq!(tracker.vertex_memory_usage(), 50);
    }

    #[test]
    fn test_peak_tracking() {
        let config = MemoryConfig::with_total_memory(1000);
        let tracker = MemoryTracker::new(config);

        tracker.try_allocate_vertex(100);
        tracker.try_allocate_edge(200);
        assert_eq!(tracker.peak_memory_usage(), 300);

        tracker.release_vertex(50);
        tracker.release_edge(100);
        assert_eq!(tracker.peak_memory_usage(), 300); // Peak should not decrease
    }

    #[test]
    fn test_stats() {
        let config = MemoryConfig::with_total_memory(1000);
        let tracker = MemoryTracker::new(config);

        tracker.try_allocate_vertex(100);
        tracker.try_allocate_edge(200);
        tracker.try_allocate_cache(50);

        let stats = tracker.stats();
        assert_eq!(stats.vertex_memory, 100);
        assert_eq!(stats.edge_memory, 200);
        assert_eq!(stats.cache_memory, 50);
        assert_eq!(stats.total_memory, 350);
    }

    #[test]
    fn test_utilization() {
        let config = MemoryConfig::with_total_memory(1000);
        let tracker = MemoryTracker::new(config);

        tracker.try_allocate_vertex(200); // 200/400 = 50%
        tracker.try_allocate_edge(200); // 200/400 = 50%
        tracker.try_allocate_cache(100); // 100/200 = 50%

        assert!((tracker.utilization() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let config = MemoryConfig::with_total_memory(10000);
        let tracker = Arc::new(MemoryTracker::new(config));

        let mut handles = vec![];

        for _ in 0..10 {
            let tracker_clone = tracker.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    if tracker_clone.try_allocate_vertex(10) {
                        tracker_clone.release_vertex(10);
                    }
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert!(tracker.is_within_limits());
    }
}

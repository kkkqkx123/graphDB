use std::alloc::Layout;
use std::collections::{BTreeMap, HashMap};
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock, RwLock};
use std::thread_local;

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

thread_local! {
    /// 线程本地预留
    static LOCAL_RESERVED: std::cell::RefCell<u64> = const { std::cell::RefCell::new(0) };
}

const LOCAL_LIMIT: u64 = 1024 * 1024;

/// 线程本地分配
pub fn alloc_local(size: u64, throw_if_exceeded: bool) -> Result<(), String> {
    LOCAL_RESERVED.with(|reserved| {
        let current = *reserved.borrow();
        if current + size > LOCAL_LIMIT {
            if throw_if_exceeded {
                return Err("Local reservation exceeded".to_string());
            }
        } else {
            let needed = size - (LOCAL_LIMIT - current);
            global_memory_manager().alloc(needed, throw_if_exceeded)?;
            *reserved.borrow_mut() += size;
        }
        Ok(())
    })
}

/// 线程本地释放
pub fn free_local(size: u64) {
    LOCAL_RESERVED.with(|reserved| {
        *reserved.borrow_mut() = reserved.borrow().saturating_sub(size);

        let current = *reserved.borrow();
        if current > LOCAL_LIMIT {
            let excess = current - LOCAL_LIMIT;
            global_memory_manager().free(excess);
            *reserved.borrow_mut() = LOCAL_LIMIT;
        }
    });
}

/// 内存检查 Guard
pub struct MemoryCheckGuard {
    throw_on_exceeded: bool,
}

impl MemoryCheckGuard {
    pub fn new() -> Self {
        Self {
            throw_on_exceeded: true,
        }
    }

    pub fn without_check() -> Self {
        Self {
            throw_on_exceeded: false,
        }
    }

    pub fn throw_on_exceeded(&self) -> bool {
        self.throw_on_exceeded
    }
}

impl Default for MemoryCheckGuard {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII 风格的内存检查控制
pub struct ScopedMemoryCheck {
    previous: bool,
}

impl ScopedMemoryCheck {
    pub fn new(enable: bool) -> Self {
        let previous = is_memory_check_enabled();
        set_memory_check_enabled(enable);
        Self { previous }
    }
}

impl Drop for ScopedMemoryCheck {
    fn drop(&mut self) {
        set_memory_check_enabled(self.previous);
    }
}

thread_local! {
    static MEMORY_CHECK_ENABLED: std::cell::RefCell<bool> = const { std::cell::RefCell::new(false) };
}

/// 设置内存检查是否启用
pub fn set_memory_check_enabled(enabled: bool) {
    MEMORY_CHECK_ENABLED.with(|flag| {
        *flag.borrow_mut() = enabled;
    });
}

/// 检查内存检查是否启用
pub fn is_memory_check_enabled() -> bool {
    MEMORY_CHECK_ENABLED.with(|flag| {
        *flag.borrow()
    })
}

/// 内存跟踪器，用于监控内存分配
#[derive(Debug)]
pub struct MemoryTracker {
    total_allocated: AtomicUsize,
    total_deallocated: AtomicUsize,
    current_allocated: AtomicUsize,
    peak_usage: AtomicUsize,
}

impl MemoryTracker {
    pub fn new() -> Self {
        Self {
            total_allocated: AtomicUsize::new(0),
            total_deallocated: AtomicUsize::new(0),
            current_allocated: AtomicUsize::new(0),
            peak_usage: AtomicUsize::new(0),
        }
    }

    pub fn record_allocation(&self, size: usize) {
        let old_current = self.current_allocated.fetch_add(size, Ordering::SeqCst);
        self.total_allocated.fetch_add(size, Ordering::SeqCst);

        let mut current_peak = self.peak_usage.load(Ordering::SeqCst);
        loop {
            if old_current + size > current_peak {
                match self.peak_usage.compare_exchange(
                    current_peak,
                    old_current + size,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                ) {
                    Ok(_) => break,
                    Err(current) => current_peak = current,
                }
            } else {
                break;
            }
        }
    }

    pub fn record_deallocation(&self, size: usize) {
        self.current_allocated.fetch_sub(size, Ordering::SeqCst);
        self.total_deallocated.fetch_add(size, Ordering::SeqCst);
    }

    pub fn total_allocated(&self) -> usize {
        self.total_allocated.load(Ordering::SeqCst)
    }

    pub fn total_deallocated(&self) -> usize {
        self.total_deallocated.load(Ordering::SeqCst)
    }

    pub fn current_allocated(&self) -> usize {
        self.current_allocated.load(Ordering::SeqCst)
    }

    pub fn peak_usage(&self) -> usize {
        self.peak_usage.load(Ordering::SeqCst)
    }

    pub fn reset(&self) {
        self.total_allocated.store(0, Ordering::SeqCst);
        self.total_deallocated.store(0, Ordering::SeqCst);
        self.current_allocated.store(0, Ordering::SeqCst);
        self.peak_usage.store(0, Ordering::SeqCst);
    }
}

/// 全局内存跟踪器
static MEMORY_TRACKER: once_cell::sync::Lazy<MemoryTracker> =
    once_cell::sync::Lazy::new(MemoryTracker::new);

/// 获取全局内存跟踪器的引用
pub fn memory_tracker() -> &'static MemoryTracker {
    &MEMORY_TRACKER
}

/// 使用 Arena 模式的安全内存池实现
pub struct MemoryPool {
    pool: Arc<RwLock<Vec<u8>>>,
    available_chunks: Arc<RwLock<BTreeMap<usize, Vec<usize>>>>,
    chunk_size_map: Arc<RwLock<BTreeMap<usize, usize>>>,
    total_size: usize,
    used_size: Arc<AtomicUsize>,
}

/// 从 MemoryPool 分配的内存的安全句柄
pub struct MemoryChunk {
    pool: Arc<RwLock<Vec<u8>>>,
    available_chunks: Arc<RwLock<BTreeMap<usize, Vec<usize>>>>,
    chunk_size_map: Arc<RwLock<BTreeMap<usize, usize>>>,
    used_size: Arc<AtomicUsize>,
    start_idx: usize,
    size: usize,
}

impl MemoryChunk {
    pub fn with_slice<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        let pool = self.pool.read().ok();
        match pool {
            Some(p) => f(&p[self.start_idx..self.start_idx + self.size]),
            None => f(&[]),
        }
    }

    pub fn with_mut_slice<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut pool = self.pool.write().ok();
        match pool {
            Some(ref mut p) => f(&mut p[self.start_idx..self.start_idx + self.size]),
            None => f(&mut []),
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

impl Drop for MemoryChunk {
    fn drop(&mut self) {
        if let (Ok(mut available_chunks), Ok(mut chunk_size_map)) = (
            self.available_chunks.write(),
            self.chunk_size_map.write(),
        ) {
            available_chunks
                .entry(self.size)
                .or_insert_with(Vec::new)
                .push(self.start_idx);

            chunk_size_map.remove(&self.start_idx);

            self.used_size.fetch_sub(self.size, Ordering::SeqCst);
        }
    }
}

impl MemoryPool {
    pub fn new(size: usize) -> Result<Self, String> {
        if size == 0 {
            return Err("内存池大小不能为零".to_string());
        }

        let mut pool = Vec::with_capacity(size);
        pool.resize(size, 0);

        let mut available_chunks = BTreeMap::new();
        available_chunks.insert(size, vec![0]);

        Ok(Self {
            pool: Arc::new(RwLock::new(pool)),
            available_chunks: Arc::new(RwLock::new(available_chunks)),
            chunk_size_map: Arc::new(RwLock::new(BTreeMap::new())),
            total_size: size,
            used_size: Arc::new(AtomicUsize::new(0)),
        })
    }

    pub fn allocate(&self, size: usize) -> Result<MemoryChunk, String> {
        if size == 0 {
            return Err("分配大小不能为零".to_string());
        }

        if size > self.total_size {
            return Err(format!("请求大小 {} 超过内存池总大小 {}", size, self.total_size));
        }

        let mut available_chunks = self
            .available_chunks
            .write()
            .map_err(|e| format!("获取可用块锁失败: {}", e))?;

        if let Some(indices) = available_chunks.get(&size) {
            if let Some(&start_idx) = indices.first() {
                available_chunks
                    .get_mut(&size)
                    .map(|v| v.remove(0));

                if available_chunks.get(&size).map_or(true, |v| v.is_empty()) {
                    available_chunks.remove(&size);
                }

                if let Ok(mut chunk_map) = self.chunk_size_map.write() {
                    chunk_map.insert(start_idx, size);
                }

                self.used_size.fetch_add(size, Ordering::SeqCst);

                return Ok(MemoryChunk {
                    pool: Arc::clone(&self.pool),
                    available_chunks: Arc::clone(&self.available_chunks),
                    chunk_size_map: Arc::clone(&self.chunk_size_map),
                    used_size: Arc::clone(&self.used_size),
                    start_idx,
                    size,
                });
            }
        }

        let mut suitable_chunk = None;
        for (&chunk_size, indices) in available_chunks.iter() {
            if chunk_size >= size && !indices.is_empty() {
                suitable_chunk = Some((chunk_size, indices[0]));
                break;
            }
        }

        if let Some((original_chunk_size, start_idx)) = suitable_chunk {
            available_chunks
                .get_mut(&original_chunk_size)
                .map(|v| v.retain(|&x| x != start_idx));

            if available_chunks
                .get(&original_chunk_size)
                .map_or(true, |v| v.is_empty())
            {
                available_chunks.remove(&original_chunk_size);
            }

            if original_chunk_size > size {
                let remaining_size = original_chunk_size - size;
                let remaining_start_idx = start_idx + size;
                available_chunks
                    .entry(remaining_size)
                    .or_insert_with(Vec::new)
                    .push(remaining_start_idx);
            }

            if let Ok(mut chunk_map) = self.chunk_size_map.write() {
                chunk_map.insert(start_idx, size);
            }

            self.used_size.fetch_add(size, Ordering::SeqCst);

            Ok(MemoryChunk {
                pool: Arc::clone(&self.pool),
                available_chunks: Arc::clone(&self.available_chunks),
                chunk_size_map: Arc::clone(&self.chunk_size_map),
                used_size: Arc::clone(&self.used_size),
                start_idx,
                size,
            })
        } else {
            Err(format!("无法分配 {} 字节的内存，内存池已满或碎片化", size))
        }
    }

    pub fn total_size(&self) -> usize {
        self.total_size
    }

    pub fn used_size(&self) -> usize {
        self.used_size.load(Ordering::SeqCst)
    }

    pub fn available_size(&self) -> usize {
        self.total_size - self.used_size.load(Ordering::SeqCst)
    }
}

/// 内存统计信息（兼容旧版本）
#[derive(Debug, Clone)]
pub struct LegacyMemoryStats {
    pub total_allocated: usize,
    pub total_deallocated: usize,
    pub current_allocated: usize,
    pub peak_usage: usize,
    pub pool_total_size: usize,
    pub pool_used_size: usize,
    pub pool_available_size: usize,
}

/// 获取内存统计信息（兼容旧版本）
pub fn get_memory_stats(pool: Option<&MemoryPool>) -> LegacyMemoryStats {
    LegacyMemoryStats {
        total_allocated: MEMORY_TRACKER.total_allocated(),
        total_deallocated: MEMORY_TRACKER.total_deallocated(),
        current_allocated: MEMORY_TRACKER.current_allocated(),
        peak_usage: MEMORY_TRACKER.peak_usage(),
        pool_total_size: match pool {
            Some(p) => p.total_size(),
            None => 0,
        },
        pool_used_size: match pool {
            Some(p) => p.used_size(),
            None => 0,
        },
        pool_available_size: match pool {
            Some(p) => p.available_size(),
            None => 0,
        },
    }
}

/// 内存工具函数
pub mod memory_utils {
    use super::*;
    use std::mem;

    pub fn size_of<T>() -> usize {
        mem::size_of::<T>()
    }

    pub fn align_of<T>() -> usize {
        mem::align_of::<T>()
    }

    pub fn fill_buffer(buffer: &mut [u8], value: u8) {
        for b in buffer.iter_mut() {
            *b = value;
        }
    }

    pub fn compare_buffers(buf1: &[u8], buf2: &[u8]) -> bool {
        buf1 == buf2
    }

    pub unsafe fn copy_memory(src: *const u8, dest: *mut u8, size: usize) {
        ptr::copy_nonoverlapping(src, dest, size);
    }

    pub unsafe fn set_memory(ptr: *mut u8, value: u8, size: usize) {
        ptr::write_bytes(ptr, value, size);
    }

    pub fn get_memory_address<T: ?Sized>(r: &T) -> usize {
        r as *const T as *const () as usize
    }
}

/// 分配类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AllocationType {
    Malloc,
    Arena,
    Pool,
    Vec,
    String,
    Box,
    Other(String),
}

impl From<&str> for AllocationType {
    fn from(s: &str) -> Self {
        match s {
            "malloc" => AllocationType::Malloc,
            "arena" => AllocationType::Arena,
            "pool" => AllocationType::Pool,
            "vec" => AllocationType::Vec,
            "string" => AllocationType::String,
            "box" => AllocationType::Box,
            _ => AllocationType::Other(s.to_string()),
        }
    }
}

/// 分配信息结构体
#[derive(Debug, Clone)]
pub struct AllocationInfo {
    pub ptr: usize,
    pub size: usize,
    pub align: usize,
    pub location: String,
    pub allocation_type: AllocationType,
    pub timestamp: std::time::Instant,
}

impl AllocationInfo {
    pub fn new(ptr: usize, layout: Layout, location: String, allocation_type: AllocationType) -> Self {
        Self {
            ptr,
            size: layout.size(),
            align: layout.align(),
            location,
            allocation_type,
            timestamp: std::time::Instant::now(),
        }
    }

    pub fn layout(&self) -> Result<Layout, String> {
        Layout::from_size_align(self.size, self.align)
            .map_err(|e| format!("Invalid size or alignment for layout: {}", e))
    }
}

/// 泄漏统计信息
#[derive(Debug, Default, Clone)]
pub struct LeakStats {
    pub total_leaks: usize,
    pub total_leaked_bytes: usize,
    pub leaks_by_type: HashMap<AllocationType, usize>,
    pub leaks_by_size: BTreeMap<usize, usize>,
}

impl LeakStats {
    pub fn from_allocations(allocations: &HashMap<usize, AllocationInfo>) -> Self {
        let mut stats = Self::default();
        stats.total_leaks = allocations.len();

        for info in allocations.values() {
            stats.total_leaked_bytes += info.size;

            *stats.leaks_by_type.entry(info.allocation_type.clone()).or_insert(0) += 1;

            let size_bucket = if info.size < 64 {
                64
            } else if info.size < 256 {
                256
            } else if info.size < 1024 {
                1024
            } else if info.size < 4096 {
                4096
            } else if info.size < 16384 {
                16384
            } else {
                usize::MAX
            };
            *stats.leaks_by_size.entry(size_bucket).or_insert(0) += 1;
        }

        stats
    }

    pub fn format_report(&self) -> String {
        let mut lines = Vec::new();
        lines.push("=== 内存泄漏统计 ===".to_string());
        lines.push(format!("泄漏总数: {} 处", self.total_leaks));
        lines.push(format!("泄漏总大小: {} 字节", self.total_leaked_bytes));
        lines.push("按类型分布:".to_string());
        for (alloc_type, count) in &self.leaks_by_type {
            lines.push(format!("  {:?}: {} 处", alloc_type, count));
        }
        lines.push("按大小分布:".to_string());
        for (size, count) in &self.leaks_by_size {
            let size_str = if *size == usize::MAX {
                "> 16KB".to_string()
            } else {
                format!("<= {}B", size)
            };
            lines.push(format!("  {}: {} 处", size_str, count));
        }
        lines.join("\n")
    }
}

/// 内存泄漏检测器（增强版）
pub struct MemoryLeakDetector {
    enabled: AtomicBool,
    allocations: Arc<RwLock<HashMap<usize, AllocationInfo>>>,
    allocation_count: AtomicUsize,
    deallocation_count: AtomicUsize,
    leaked_bytes: AtomicUsize,
}

impl MemoryLeakDetector {
    pub fn new() -> Self {
        Self {
            enabled: AtomicBool::new(true),
            allocations: Arc::new(RwLock::new(HashMap::new())),
            allocation_count: AtomicUsize::new(0),
            deallocation_count: AtomicUsize::new(0),
            leaked_bytes: AtomicUsize::new(0),
        }
    }

    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Relaxed);
    }

    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn record_allocation(&self, ptr: usize, layout: Layout, location: String) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }

        if let Ok(mut allocations) = self.allocations.write() {
            let allocation_type = self.infer_allocation_type(&location);
            allocations.insert(
                ptr,
                AllocationInfo::new(ptr, layout, location, allocation_type),
            );
            self.allocation_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn record_allocation_with_type(
        &self,
        ptr: usize,
        layout: Layout,
        location: String,
        allocation_type: AllocationType,
    ) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }

        if let Ok(mut allocations) = self.allocations.write() {
            allocations.insert(
                ptr,
                AllocationInfo::new(ptr, layout, location, allocation_type),
            );
            self.allocation_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn record_deallocation(&self, ptr: usize) -> Result<(), String> {
        if !self.enabled.load(Ordering::Relaxed) {
            return Ok(());
        }

        if let Ok(mut allocations) = self.allocations.write() {
            if allocations.remove(&ptr).is_some() {
                self.deallocation_count.fetch_add(1, Ordering::Relaxed);
                return Ok(());
            }
            return Err(format!("尝试释放未记录的内存地址: 0x{:x}", ptr));
        }
        Ok(())
    }

    pub fn report_leaks(&self) -> Vec<AllocationInfo> {
        if let Ok(allocations) = self.allocations.read() {
            allocations.values().cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub fn stats(&self) -> LeakStats {
        if let Ok(allocations) = self.allocations.read() {
            LeakStats::from_allocations(&allocations)
        } else {
            LeakStats::default()
        }
    }

    pub fn detailed_report(&self) -> String {
        let leaks = self.report_leaks();
        let mut lines = Vec::new();
        lines.push("=== 详细内存泄漏报告".to_string());
        lines.push(format!("泄漏数量: {}", leaks.len()));
        lines.push(String::new());

        for (i, info) in leaks.iter().enumerate() {
            lines.push(format!(
                "泄漏 #{}: 地址=0x{:x}, 大小={} 字节, 对齐={}",
                i + 1, info.ptr, info.size, info.align
            ));
            lines.push(format!("  类型: {:?}", info.allocation_type));
            lines.push(format!("  位置: {}", info.location));
            lines.push(format!(
                "  分配时间: {} 秒前",
                info.timestamp.elapsed().as_secs()
            ));
        }

        lines.push(String::new());
        lines.push("=== 统计摘要 ===".to_string());
        lines.push(self.stats().format_report());

        lines.join("\n")
    }

    pub fn has_leaks(&self) -> bool {
        self.allocations
            .read()
            .map(|allocations| !allocations.is_empty())
            .unwrap_or(false)
    }

    pub fn allocation_count(&self) -> usize {
        self.allocation_count.load(Ordering::Relaxed)
    }

    pub fn deallocation_count(&self) -> usize {
        self.deallocation_count.load(Ordering::Relaxed)
    }

    pub fn leaked_bytes(&self) -> usize {
        self.leaked_bytes.load(Ordering::Relaxed)
    }

    pub fn clear(&self) {
        if let Ok(mut allocations) = self.allocations.write() {
            self.leaked_bytes.fetch_add(
                allocations.values().map(|a| a.size).sum(),
                Ordering::Relaxed,
            );
            allocations.clear();
        }
    }

    fn infer_allocation_type(&self, location: &str) -> AllocationType {
        if location.contains("Vec::with_capacity") || location.contains("vec!") {
            AllocationType::Vec
        } else if location.contains("String::from") || location.contains("to_string") {
            AllocationType::String
        } else if location.contains("Box::new") {
            AllocationType::Box
        } else if location.contains("arena") || location.contains("Arena") {
            AllocationType::Arena
        } else if location.contains("pool") || location.contains("Pool") {
            AllocationType::Pool
        } else if location.contains("malloc") || location.contains("alloc") {
            AllocationType::Malloc
        } else {
            AllocationType::Other(location.to_string())
        }
    }
}

static LEAK_DETECTOR: once_cell::sync::Lazy<MemoryLeakDetector> =
    once_cell::sync::Lazy::new(MemoryLeakDetector::new);

pub fn leak_detector() -> &'static MemoryLeakDetector {
    &LEAK_DETECTOR
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tracker() {
        let tracker = MemoryTracker::new();

        tracker.record_allocation(100);
        assert_eq!(tracker.total_allocated(), 100);
        assert_eq!(tracker.current_allocated(), 100);

        tracker.record_allocation(50);
        assert_eq!(tracker.total_allocated(), 150);
        assert_eq!(tracker.current_allocated(), 150);

        tracker.record_deallocation(30);
        assert_eq!(tracker.total_deallocated(), 30);
        assert_eq!(tracker.current_allocated(), 120);

        tracker.reset();
        assert_eq!(tracker.total_allocated(), 0);
        assert_eq!(tracker.current_allocated(), 0);
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
    fn test_memory_pool() {
        let pool = MemoryPool::new(1024).expect("内存池创建失败");

        assert_eq!(pool.total_size(), 1024);
        assert_eq!(pool.available_size(), 1024);
        assert_eq!(pool.used_size(), 0);

        let mut chunk1 = pool.allocate(100).expect("分配失败");
        assert_eq!(pool.available_size(), 924);
        assert_eq!(pool.used_size(), 100);

        let _chunk2 = pool.allocate(50).expect("分配失败");
        assert_eq!(pool.available_size(), 874);
        assert_eq!(pool.used_size(), 150);

        let value = chunk1.with_mut_slice(|slice| {
            slice[0] = 42;
            slice[0]
        });
        assert_eq!(value, 42);

        let read_value = chunk1.with_slice(|slice| slice[0]);
        assert_eq!(read_value, 42);

        drop(chunk1);
        assert_eq!(pool.available_size(), 974);
        assert_eq!(pool.used_size(), 50);
    }

    #[test]
    fn test_memory_pool_zero_size() {
        let pool = MemoryPool::new(0);
        assert!(pool.is_err());
    }

    #[test]
    fn test_memory_pool_allocation_too_large() {
        let pool = MemoryPool::new(1024).expect("内存池创建失败");
        let result = pool.allocate(2048);
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_utils() {
        assert_eq!(memory_utils::size_of::<i32>(), 4);
        assert_eq!(memory_utils::size_of::<u64>(), 8);

        assert_eq!(memory_utils::align_of::<i32>(), std::mem::align_of::<i32>());

        let mut buffer = [0u8; 10];
        memory_utils::fill_buffer(&mut buffer, 5);
        assert_eq!(buffer, [5u8; 10]);

        assert!(memory_utils::compare_buffers(&[1, 2, 3], &[1, 2, 3]));
        assert!(!memory_utils::compare_buffers(&[1, 2, 3], &[1, 2, 4]));
    }

    #[test]
    fn test_leak_detector() {
        let detector = MemoryLeakDetector::new();

        assert!(!detector.has_leaks());
        assert!(detector.is_enabled());

        let layout = Layout::from_size_align(100, 8).expect("布局创建失败");
        detector.record_allocation(0x1000, layout, "test_location".to_string());

        assert!(detector.has_leaks());
        assert_eq!(detector.allocation_count(), 1);

        let result = detector.record_deallocation(0x1000);
        assert!(result.is_ok());

        assert!(!detector.has_leaks());
    }

    #[test]
    fn test_leak_detector_enable_disable() {
        let detector = MemoryLeakDetector::new();

        let layout = Layout::from_size_align(100, 8).expect("布局创建失败");
        detector.record_allocation(0x1000, layout, "test_location".to_string());
        assert!(detector.has_leaks());

        detector.disable();
        assert!(!detector.is_enabled());

        let layout2 = Layout::from_size_align(200, 8).expect("布局创建失败");
        detector.record_allocation(0x2000, layout2, "test_location2".to_string());

        let leaks = detector.report_leaks();
        assert_eq!(leaks.len(), 1);

        detector.enable();
        assert!(detector.is_enabled());
    }

    #[test]
    fn test_leak_detector_stats() {
        let detector = MemoryLeakDetector::new();

        let layout1 = Layout::from_size_align(100, 8).expect("布局创建失败");
        detector.record_allocation(0x1000, layout1, "vec![1, 2, 3]".to_string());

        let layout2 = Layout::from_size_align(50, 8).expect("布局创建失败");
        detector.record_allocation(0x2000, layout2, "String::from".to_string());

        let stats = detector.stats();
        assert_eq!(stats.total_leaks, 2);
        assert_eq!(stats.total_leaked_bytes, 150);

        let report = stats.format_report();
        assert!(report.contains("泄漏总数: 2"));
        assert!(report.contains("泄漏总大小: 150"));

        detector.record_deallocation(0x1000).expect("Failed to record deallocation");
        detector.record_deallocation(0x2000).expect("Failed to record deallocation");

        let stats2 = detector.stats();
        assert_eq!(stats2.total_leaks, 0);
    }

    #[test]
    fn test_leak_detector_detailed_report() {
        let detector = MemoryLeakDetector::new();

        let layout = Layout::from_size_align(100, 8).expect("布局创建失败");
        detector.record_allocation(0x1000, layout, "Box::new".to_string());

        let report = detector.detailed_report();
        assert!(report.contains("详细内存泄漏报告"));
        assert!(report.contains("Box"));

        detector.record_deallocation(0x1000).expect("Failed to record deallocation");
    }

    #[test]
    fn test_leak_detector_double_free() {
        let detector = MemoryLeakDetector::new();

        let layout = Layout::from_size_align(100, 8).expect("布局创建失败");
        detector.record_allocation(0x1000, layout, "test".to_string());

        detector.record_deallocation(0x1000).expect("Failed to record deallocation");

        let result = detector.record_deallocation(0x1000);
        assert!(result.is_err());
    }

    #[test]
    fn test_leak_detector_allocation_type_inference() {
        let detector = MemoryLeakDetector::new();

        let layout = Layout::from_size_align(100, 8).expect("布局创建失败");
        detector.record_allocation(0x1000, layout, "vec![1, 2, 3]".to_string());
        let leaks = detector.report_leaks();
        assert_eq!(leaks[0].allocation_type, AllocationType::Vec);

        detector.record_deallocation(0x1000).expect("Failed to record deallocation");

        let layout2 = Layout::from_size_align(100, 8).expect("布局创建失败");
        detector.record_allocation(0x2000, layout2, "String::from".to_string());
        let leaks2 = detector.report_leaks();
        assert_eq!(leaks2[0].allocation_type, AllocationType::String);

        detector.record_deallocation(0x2000).expect("Failed to record deallocation");
    }

    #[test]
    fn test_leak_detector_clear() {
        let detector = MemoryLeakDetector::new();

        let layout = Layout::from_size_align(100, 8).expect("布局创建失败");
        detector.record_allocation(0x1000, layout, "test".to_string());

        assert!(detector.has_leaks());

        detector.clear();

        assert!(!detector.has_leaks());
    }

    #[test]
    fn test_memory_check_guard() {
        let guard = MemoryCheckGuard::new();
        assert!(guard.throw_on_exceeded());

        let guard2 = MemoryCheckGuard::without_check();
        assert!(!guard2.throw_on_exceeded());
    }

    #[test]
    fn test_scoped_memory_check() {
        set_memory_check_enabled(false);
        assert!(!is_memory_check_enabled());

        {
            let _guard = ScopedMemoryCheck::new(true);
            assert!(is_memory_check_enabled());
        }

        assert!(!is_memory_check_enabled());
    }
}

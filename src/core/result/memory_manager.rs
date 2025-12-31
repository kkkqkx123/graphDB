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
    fn set_limit(&self, limit: u64);
}

/// 内存配置
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    pub limit_bytes: u64,
    pub check_interval: usize,
    pub enable_system_monitor: bool,
    pub limit_ratio: f64,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            limit_bytes: 0,
            check_interval: 1000,
            enable_system_monitor: true,
            limit_ratio: 0.8,
        }
    }
}

impl MemoryConfig {
    pub fn new(limit_bytes: u64) -> Self {
        Self {
            limit_bytes,
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
}

/// 简单的内存管理器实现
#[derive(Debug, Clone)]
pub struct SimpleMemoryManager {
    current_usage: Arc<AtomicU64>,
    peak_usage: Arc<AtomicU64>,
    limit: Arc<AtomicU64>,
    config: MemoryConfig,
}

impl SimpleMemoryManager {
    pub fn new(limit: u64) -> Self {
        Self {
            current_usage: Arc::new(AtomicU64::new(0)),
            peak_usage: Arc::new(AtomicU64::new(0)),
            limit: Arc::new(AtomicU64::new(limit)),
            config: MemoryConfig::new(limit),
        }
    }

    pub fn with_config(config: MemoryConfig) -> Self {
        let limit = if config.limit_bytes > 0 {
            config.limit_bytes
        } else {
            Self::default_limit()
        };

        Self {
            current_usage: Arc::new(AtomicU64::new(0)),
            peak_usage: Arc::new(AtomicU64::new(0)),
            limit: Arc::new(AtomicU64::new(limit)),
            config,
        }
    }

    pub fn with_default_limit() -> Self {
        Self::new(Self::default_limit())
    }

    fn default_limit() -> u64 {
        100 * 1024 * 1024 // 默认100MB
    }

    pub fn adjust_limit_from_system(&self) {
        if !self.config.enable_system_monitor {
            return;
        }

        if let Ok(available) = get_system_available_memory() {
            let new_limit = (available as f64 * self.config.limit_ratio) as u64;
            self.set_limit(new_limit);
        }
    }

    pub fn get_config(&self) -> &MemoryConfig {
        &self.config
    }
}

impl MemoryManager for SimpleMemoryManager {
    fn check_memory(&self, bytes: u64) -> Result<bool, String> {
        let current = self.current_usage.load(Ordering::Relaxed);
        let limit = self.limit.load(Ordering::Relaxed);
        if current + bytes > limit {
            Err(format!(
                "Memory limit exceeded: {} + {} > {}",
                current, bytes, limit
            ))
        } else {
            Ok(true)
        }
    }

    fn register_allocation(&self, bytes: u64) {
        let old_usage = self.current_usage.fetch_add(bytes, Ordering::Relaxed);
        let new_usage = old_usage + bytes;

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
        self.limit.load(Ordering::Relaxed)
    }

    fn get_peak_usage(&self) -> u64 {
        self.peak_usage.load(Ordering::Relaxed)
    }

    fn set_limit(&self, limit: u64) {
        self.limit.store(limit, Ordering::Relaxed);
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

    pub fn get_usage_info(&self) -> MemoryUsageInfo {
        MemoryUsageInfo::new(
            self.manager.get_current_usage(),
            self.manager.get_peak_usage(),
            self.manager.get_limit(),
        )
    }

    pub fn is_near_limit(&self, threshold: f64) -> bool {
        let info = self.get_usage_info();
        info.utilization_ratio > threshold
    }

    pub fn utilization_ratio(&self) -> f64 {
        self.get_usage_info().utilization_ratio
    }
}

/// RAII风格的内存检查控制
pub struct MemoryCheckGuard {
    manager: Option<Arc<dyn MemoryManager>>,
    bytes_reserved: u64,
}

impl MemoryCheckGuard {
    pub fn new(manager: Arc<dyn MemoryManager>, bytes: u64) -> Result<Self, String> {
        manager.check_memory(bytes)?;
        manager.register_allocation(bytes);

        Ok(Self {
            manager: Some(manager),
            bytes_reserved: bytes,
        })
    }

    pub fn without_check(manager: Arc<dyn MemoryManager>, bytes: u64) -> Self {
        manager.register_allocation(bytes);

        Self {
            manager: Some(manager),
            bytes_reserved: bytes,
        }
    }

    pub fn release(&mut self) {
        if let Some(manager) = &self.manager {
            manager.register_deallocation(self.bytes_reserved);
            self.bytes_reserved = 0;
        }
    }

    pub fn bytes_reserved(&self) -> u64 {
        self.bytes_reserved
    }
}

impl Drop for MemoryCheckGuard {
    fn drop(&mut self) {
        if self.bytes_reserved > 0 {
            if let Some(manager) = &self.manager {
                manager.register_deallocation(self.bytes_reserved);
            }
        }
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
        assert_eq!(manager.get_peak_usage(), 500);
    }

    #[test]
    fn test_memory_config() {
        let config = MemoryConfig::new(1000)
            .with_system_monitor(false)
            .with_check_interval(500)
            .with_limit_ratio(0.9);

        assert_eq!(config.limit_bytes, 1000);
        assert!(!config.enable_system_monitor);
        assert_eq!(config.check_interval, 500);
        assert_eq!(config.limit_ratio, 0.9);
    }

    #[test]
    fn test_set_limit() {
        let manager = SimpleMemoryManager::new(1000);
        assert_eq!(manager.get_limit(), 1000);

        manager.set_limit(2000);
        assert_eq!(manager.get_limit(), 2000);
    }

    #[test]
    fn test_memory_check_guard() {
        let manager = Arc::new(SimpleMemoryManager::new(1000));

        {
            let guard = MemoryCheckGuard::new(manager.clone(), 500).unwrap();
            assert_eq!(manager.get_current_usage(), 500);
            assert_eq!(guard.bytes_reserved(), 500);
        }

        assert_eq!(manager.get_current_usage(), 0);
    }

    #[test]
    fn test_memory_check_guard_without_check() {
        let manager = Arc::new(SimpleMemoryManager::new(1000));

        {
            let guard = MemoryCheckGuard::without_check(manager.clone(), 500);
            assert_eq!(manager.get_current_usage(), 500);
        }

        assert_eq!(manager.get_current_usage(), 0);
    }

    #[test]
    fn test_memory_check_guard_release() {
        let manager = Arc::new(SimpleMemoryManager::new(1000));

        let mut guard = MemoryCheckGuard::new(manager.clone(), 500).unwrap();
        assert_eq!(manager.get_current_usage(), 500);

        guard.release();
        assert_eq!(manager.get_current_usage(), 0);
        assert_eq!(guard.bytes_reserved(), 0);
    }

    #[test]
    fn test_memory_check_guard_limit_exceeded() {
        let manager = Arc::new(SimpleMemoryManager::new(1000));

        let guard = MemoryCheckGuard::new(manager.clone(), 500).unwrap();
        assert!(MemoryCheckGuard::new(manager.clone(), 600).is_err());

        drop(guard);
        assert!(MemoryCheckGuard::new(manager.clone(), 600).is_ok());
    }
}

#[cfg(feature = "system_monitor")]
fn get_system_available_memory() -> Result<u64, String> {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

        let mut status: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
        status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;

        unsafe {
            if GlobalMemoryStatusEx(&mut status) != 0 {
                Ok(status.ullAvailPhys)
            } else {
                Err("Failed to get system memory status".to_string())
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open("/proc/meminfo")
            .map_err(|e| format!("Failed to open /proc/meminfo: {}", e))?;

        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| format!("Failed to read /proc/meminfo: {}", e))?;

        for line in content.lines() {
            if line.starts_with("MemAvailable:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let kb: u64 = parts[1]
                        .parse()
                        .map_err(|e| format!("Failed to parse memory value: {}", e))?;
                    return Ok(kb * 1024);
                }
            }
        }

        Err("MemAvailable not found in /proc/meminfo".to_string())
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        let output = Command::new("vm_stat")
            .output()
            .map_err(|e| format!("Failed to execute vm_stat: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        let page_size: u64 = 4096;
        let mut free_pages: u64 = 0;

        for line in stdout.lines() {
            if line.contains("Pages free:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    free_pages = parts[2].trim_end_matches('.').parse().unwrap_or(0);
                }
            }
        }

        Ok(free_pages * page_size)
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Err("System memory monitoring not supported on this platform".to_string())
    }
}

#[cfg(not(feature = "system_monitor"))]
fn get_system_available_memory() -> Result<u64, String> {
    Err("System memory monitoring is disabled".to_string())
}

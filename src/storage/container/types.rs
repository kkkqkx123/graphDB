//! Container Types
//!
//! Type definitions for storage containers

use std::fmt;

/// Container error type
#[derive(Debug, Clone, thiserror::Error)]
pub enum ContainerError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Out of memory")]
    OutOfMemory,

    #[error("Container not initialized")]
    NotInitialized,

    #[error("Invalid size: {0}")]
    InvalidSize(String),

    #[error("File does not exist: {0}")]
    FileNotFound(String),

    #[error("Mapping failed: {0}")]
    MappingFailed(String),
}

impl From<std::io::Error> for ContainerError {
    fn from(e: std::io::Error) -> Self {
        ContainerError::IoError(e.to_string())
    }
}

/// Container result type
pub type ContainerResult<T> = Result<T, ContainerError>;

/// Container configuration
#[derive(Debug, Clone)]
pub struct ContainerConfig {
    /// Initial capacity in bytes
    pub initial_capacity: usize,
    /// Maximum capacity in bytes (0 means unlimited)
    pub max_capacity: usize,
    /// Growth factor when resizing
    pub growth_factor: f64,
    /// Enable huge pages on supported platforms
    pub enable_huge_pages: bool,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            initial_capacity: 4 * 1024 * 1024, // 4MB
            max_capacity: 0,
            growth_factor: 2.0,
            enable_huge_pages: false,
        }
    }
}

impl ContainerConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_initial_capacity(mut self, capacity: usize) -> Self {
        self.initial_capacity = capacity;
        self
    }

    pub fn with_max_capacity(mut self, capacity: usize) -> Self {
        self.max_capacity = capacity;
        self
    }

    pub fn with_growth_factor(mut self, factor: f64) -> Self {
        self.growth_factor = factor;
        self
    }

    pub fn with_huge_pages(mut self, enable: bool) -> Self {
        self.enable_huge_pages = enable;
        self
    }
}

/// File header for persistent containers
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FileHeader {
    /// Magic number for validation
    pub magic: u32,
    /// Version number
    pub version: u32,
    /// Data size
    pub data_size: u64,
    /// Reserved for future use
    pub reserved: [u8; 48],
}

impl FileHeader {
    pub const MAGIC: u32 = 0x47444243; // "GDBC" - GraphDB Container
    pub const VERSION: u32 = 1;
    pub const SIZE: usize = 64;

    pub fn new(data_size: u64) -> Self {
        Self {
            magic: Self::MAGIC,
            version: Self::VERSION,
            data_size,
            reserved: [0u8; 48],
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const FileHeader as *const u8,
                std::mem::size_of::<FileHeader>(),
            )
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::SIZE {
            return None;
        }
        let header: FileHeader = unsafe { std::ptr::read(bytes.as_ptr() as *const FileHeader) };
        if header.magic != Self::MAGIC {
            return None;
        }
        Some(header)
    }
}

impl Default for FileHeader {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Container statistics
#[derive(Debug, Clone, Default)]
pub struct ContainerStats {
    /// Total capacity in bytes
    pub capacity: usize,
    /// Used bytes
    pub used: usize,
    /// Number of allocations
    pub allocation_count: u64,
    /// Number of deallocations
    pub deallocation_count: u64,
}

impl ContainerStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn utilization(&self) -> f64 {
        if self.capacity == 0 {
            0.0
        } else {
            self.used as f64 / self.capacity as f64
        }
    }
}

impl fmt::Display for ContainerStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ContainerStats(capacity={}, used={}, utilization={:.2}%)",
            self.capacity,
            self.used,
            self.utilization() * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_config() {
        let config = ContainerConfig::new()
            .with_initial_capacity(1024)
            .with_max_capacity(4096);

        assert_eq!(config.initial_capacity, 1024);
        assert_eq!(config.max_capacity, 4096);
    }

    #[test]
    fn test_file_header() {
        let header = FileHeader::new(1024);
        assert_eq!(header.magic, FileHeader::MAGIC);
        assert_eq!(header.version, FileHeader::VERSION);
        assert_eq!(header.data_size, 1024);
    }

    #[test]
    fn test_container_stats() {
        let mut stats = ContainerStats::new();
        stats.capacity = 1000;
        stats.used = 500;
        assert!((stats.utilization() - 0.5).abs() < 0.001);
    }
}

//! Container Types
//!
//! Unified type definitions for storage containers.

use std::fmt;

/// Default huge page size (2MB)
pub const DEFAULT_HUGE_PAGE_SIZE: usize = 2 * 1024 * 1024;

/// Memory level for storage operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MemoryLevel {
    /// Pure in-memory storage using anonymous mmap
    InMemory,
    /// In-memory with sync to file (MAP_SHARED)
    #[default]
    SyncToFile,
    /// Prefer huge pages for large allocations
    HugePagePreferred,
}

impl MemoryLevel {
    pub fn prefers_huge_pages(&self) -> bool {
        matches!(self, MemoryLevel::HugePagePreferred)
    }

    pub fn requires_persistence(&self) -> bool {
        matches!(self, MemoryLevel::SyncToFile)
    }

    pub fn is_in_memory(&self) -> bool {
        matches!(self, MemoryLevel::InMemory)
    }
}

/// Container configuration
#[derive(Debug, Clone)]
pub struct ContainerConfig {
    /// Initial capacity in bytes
    pub initial_capacity: usize,
    /// Maximum capacity (0 = unlimited)
    pub max_capacity: usize,
    /// Growth factor for resizing
    pub growth_factor: f64,
    /// Memory level
    pub memory_level: MemoryLevel,
    /// Huge page size (for HugePagePreferred)
    pub huge_page_size: usize,
    /// Fallback to regular pages if huge pages unavailable
    pub huge_page_fallback: bool,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            initial_capacity: 4 * 1024 * 1024,
            max_capacity: 0,
            growth_factor: 2.0,
            memory_level: MemoryLevel::default(),
            huge_page_size: DEFAULT_HUGE_PAGE_SIZE,
            huge_page_fallback: true,
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

    pub fn with_memory_level(mut self, level: MemoryLevel) -> Self {
        self.memory_level = level;
        self
    }

    pub fn with_huge_page_size(mut self, size: usize) -> Self {
        self.huge_page_size = size;
        self
    }

    pub fn with_huge_page_fallback(mut self, fallback: bool) -> Self {
        self.huge_page_fallback = fallback;
        self
    }

    pub fn align_to_huge_page(&self, size: usize) -> usize {
        let mask = self.huge_page_size - 1;
        (size + mask) & !mask
    }
}

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

    #[error("Huge pages not available")]
    HugePagesNotAvailable,
}

impl From<std::io::Error> for ContainerError {
    fn from(e: std::io::Error) -> Self {
        ContainerError::IoError(e.to_string())
    }
}

/// Container result type
pub type ContainerResult<T> = Result<T, ContainerError>;

/// Container statistics
#[derive(Debug, Clone, Default)]
pub struct ContainerStats {
    /// Total capacity in bytes
    pub capacity: usize,
    /// Used bytes
    pub used: usize,
    /// Whether using huge pages
    pub is_huge_page: bool,
    /// Number of allocations
    pub allocation_count: u64,
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
            "ContainerStats(capacity={}, used={}, utilization={:.2}%, huge_page={})",
            self.capacity,
            self.used,
            self.utilization() * 100.0,
            self.is_huge_page
        )
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
    /// Checksum (MD5)
    pub checksum: [u8; 16],
    /// Reserved for future use
    pub reserved: [u8; 32],
}

impl FileHeader {
    pub const MAGIC: u32 = 0x47444243;
    pub const VERSION: u32 = 1;
    pub const SIZE: usize = 64;

    pub fn new(data_size: u64) -> Self {
        Self {
            magic: Self::MAGIC,
            version: Self::VERSION,
            data_size,
            checksum: [0u8; 16],
            reserved: [0u8; 32],
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
        let header: FileHeader = unsafe { std::ptr::read_unaligned(bytes.as_ptr() as *const FileHeader) };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_level() {
        assert!(MemoryLevel::HugePagePreferred.prefers_huge_pages());
        assert!(!MemoryLevel::InMemory.prefers_huge_pages());
        assert!(MemoryLevel::SyncToFile.requires_persistence());
        assert!(MemoryLevel::InMemory.is_in_memory());
    }

    #[test]
    fn test_container_config() {
        let config = ContainerConfig::new()
            .with_initial_capacity(1024)
            .with_memory_level(MemoryLevel::HugePagePreferred);

        assert_eq!(config.initial_capacity, 1024);
        assert_eq!(config.memory_level, MemoryLevel::HugePagePreferred);
    }

    #[test]
    fn test_file_header() {
        let header = FileHeader::new(1024);
        assert_eq!(header.magic, FileHeader::MAGIC);
        assert_eq!(header.version, FileHeader::VERSION);
        assert_eq!(header.data_size, 1024);

        let bytes = header.as_bytes();
        assert_eq!(bytes.len(), FileHeader::SIZE);

        let parsed = FileHeader::from_bytes(bytes);
        assert!(parsed.is_some());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.data_size, 1024);
    }

    #[test]
    fn test_huge_page_alignment() {
        let config = ContainerConfig::default();
        assert_eq!(config.align_to_huge_page(1), DEFAULT_HUGE_PAGE_SIZE);
        assert_eq!(
            config.align_to_huge_page(DEFAULT_HUGE_PAGE_SIZE),
            DEFAULT_HUGE_PAGE_SIZE
        );
        assert_eq!(
            config.align_to_huge_page(DEFAULT_HUGE_PAGE_SIZE + 1),
            DEFAULT_HUGE_PAGE_SIZE * 2
        );
    }
}

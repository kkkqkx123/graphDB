//! Container Types
//!
//! Unified type definitions for storage containers.

use std::fmt;

use md5::{Digest, Md5};

/// Default huge page size (2MB)
pub const DEFAULT_HUGE_PAGE_SIZE: usize = 2 * 1024 * 1024;

/// Storage backend strategy
///
/// For database systems, persistence is mandatory by default.
/// Volatile storage is only for special cases like temporary data, caches, or testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StorageBackend {
    /// Persistent storage (default for database)
    /// Data is synced to disk via mmap
    #[default]
    Persistent,
    
    /// Volatile in-memory storage
    /// Used for: temp tables, caches, testing
    Volatile {
        /// Use huge pages if available (Linux only)
        prefer_huge_pages: bool,
    },
}

impl StorageBackend {
    pub fn is_persistent(&self) -> bool {
        matches!(self, StorageBackend::Persistent)
    }

    pub fn is_volatile(&self) -> bool {
        matches!(self, StorageBackend::Volatile { .. })
    }

    pub fn prefers_huge_pages(&self) -> bool {
        matches!(self, StorageBackend::Volatile { prefer_huge_pages: true })
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
    /// Storage backend
    pub storage_backend: StorageBackend,
    /// Huge page size (for Volatile with huge pages)
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
            storage_backend: StorageBackend::default(),
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

    pub fn with_storage_backend(mut self, backend: StorageBackend) -> Self {
        self.storage_backend = backend;
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
#[derive(Debug, thiserror::Error)]
pub enum ContainerError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

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

    #[error("Checksum verification failed")]
    ChecksumMismatch,

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Disk full")]
    DiskFull,

    #[error("Invalid file header: {0}")]
    InvalidHeader(String),
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

const _: () = {
    assert!(std::mem::size_of::<FileHeader>() == 64);
    assert!(std::mem::align_of::<FileHeader>() == 8);
};

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

    /// Create a new header with checksum calculated from data
    pub fn with_checksum(data_size: u64, data: &[u8]) -> Self {
        let mut header = Self::new(data_size);
        header.checksum = Self::compute_checksum(data);
        header
    }

    /// Compute MD5 checksum for data
    pub fn compute_checksum(data: &[u8]) -> [u8; 16] {
        let mut hasher = Md5::new();
        hasher.update(data);
        hasher.finalize().into()
    }

    /// Verify the data against the stored checksum
    pub fn verify_checksum(&self, data: &[u8]) -> bool {
        self.checksum == Self::compute_checksum(data)
    }

    /// Check if the header has a valid checksum
    pub fn has_valid_checksum(&self) -> bool {
        // A zero checksum means no checksum was computed
        self.checksum != [0u8; 16]
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
    fn test_storage_backend() {
        assert!(StorageBackend::Persistent.is_persistent());
        assert!(!StorageBackend::Persistent.is_volatile());
        
        let volatile = StorageBackend::Volatile { prefer_huge_pages: true };
        assert!(volatile.is_volatile());
        assert!(volatile.prefers_huge_pages());
        
        let volatile_no_hp = StorageBackend::Volatile { prefer_huge_pages: false };
        assert!(!volatile_no_hp.prefers_huge_pages());
    }

    #[test]
    fn test_container_config() {
        let config = ContainerConfig::new()
            .with_initial_capacity(1024)
            .with_storage_backend(StorageBackend::Volatile { prefer_huge_pages: true });

        assert_eq!(config.initial_capacity, 1024);
        assert!(config.storage_backend.is_volatile());
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
    fn test_file_header_checksum() {
        let data = b"test data for checksum verification";
        let header = FileHeader::with_checksum(data.len() as u64, data);
        
        assert!(header.has_valid_checksum());
        assert!(header.verify_checksum(data));
        
        // Verify that different data fails checksum
        let different_data = b"different data";
        assert!(!header.verify_checksum(different_data));
        
        // Verify compute_checksum is consistent
        let checksum1 = FileHeader::compute_checksum(data);
        let checksum2 = FileHeader::compute_checksum(data);
        assert_eq!(checksum1, checksum2);
    }

    #[test]
    fn test_file_header_zero_checksum() {
        // Header without checksum (zero checksum) should report as invalid
        let header = FileHeader::new(100);
        assert!(!header.has_valid_checksum());
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

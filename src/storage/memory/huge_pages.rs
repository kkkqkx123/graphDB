//! Huge Page Allocator
//!
//! Provides support for huge page allocation on Linux systems.
//! Huge pages (typically 2MB or 1GB) can significantly improve performance
//! for large memory allocations by reducing TLB misses.

use std::alloc::{alloc, dealloc, Layout};
use std::io;
use std::ptr::NonNull;

/// Default huge page size on Linux (2MB)
pub const DEFAULT_HUGE_PAGE_SIZE: usize = 2 * 1024 * 1024;

/// Huge page allocator configuration
#[derive(Debug, Clone)]
pub struct HugePageConfig {
    /// Huge page size in bytes
    pub page_size: usize,
    /// Whether to fall back to regular pages if huge pages are unavailable
    pub fallback_enabled: bool,
}

impl Default for HugePageConfig {
    fn default() -> Self {
        Self {
            page_size: DEFAULT_HUGE_PAGE_SIZE,
            fallback_enabled: true,
        }
    }
}

impl HugePageConfig {
    /// Create a new configuration with the specified page size
    pub fn with_page_size(page_size: usize) -> Self {
        Self {
            page_size,
            ..Default::default()
        }
    }

    /// Set whether to fall back to regular pages
    pub fn fallback(mut self, enabled: bool) -> Self {
        self.fallback_enabled = enabled;
        self
    }
}

/// Result of a huge page allocation attempt
#[derive(Debug)]
pub enum AllocationResult {
    /// Successfully allocated with huge pages
    HugePage(NonNull<u8>),
    /// Allocated with regular pages (fallback)
    RegularPage(NonNull<u8>),
}

impl AllocationResult {
    /// Get the pointer regardless of allocation type
    pub fn ptr(&self) -> NonNull<u8> {
        match self {
            AllocationResult::HugePage(ptr) => *ptr,
            AllocationResult::RegularPage(ptr) => *ptr,
        }
    }

    /// Check if this is a huge page allocation
    pub fn is_huge_page(&self) -> bool {
        matches!(self, AllocationResult::HugePage(_))
    }
}

/// Error type for huge page operations
#[derive(Debug)]
pub enum HugePageError {
    /// Huge pages not supported on this platform
    NotSupported,
    /// Failed to allocate huge pages
    AllocationFailed(String),
    /// Invalid size (not aligned to huge page boundary)
    InvalidSize { size: usize, page_size: usize },
    /// System error
    SystemError(io::Error),
}

impl std::fmt::Display for HugePageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HugePageError::NotSupported => write!(f, "Huge pages not supported on this platform"),
            HugePageError::AllocationFailed(msg) => write!(f, "Allocation failed: {}", msg),
            HugePageError::InvalidSize { size, page_size } => {
                write!(f, "Size {} is not aligned to huge page size {}", size, page_size)
            }
            HugePageError::SystemError(e) => write!(f, "System error: {}", e),
        }
    }
}

impl std::error::Error for HugePageError {}

impl From<io::Error> for HugePageError {
    fn from(e: io::Error) -> Self {
        HugePageError::SystemError(e)
    }
}

/// Huge page allocator for Linux systems
///
/// On non-Linux systems, this falls back to regular memory allocation.
#[derive(Debug, Clone)]
pub struct HugePageAllocator {
    config: HugePageConfig,
    /// Whether huge pages are available on this system
    huge_pages_available: bool,
}

impl HugePageAllocator {
    /// Create a new huge page allocator with default configuration
    pub fn new() -> Self {
        Self::with_config(HugePageConfig::default())
    }

    /// Create a new huge page allocator with custom configuration
    pub fn with_config(config: HugePageConfig) -> Self {
        let huge_pages_available = Self::check_huge_pages_available();
        Self {
            config,
            huge_pages_available,
        }
    }

    /// Check if huge pages are available on this system
    pub fn is_huge_pages_available(&self) -> bool {
        self.huge_pages_available
    }

    /// Get the configured huge page size
    pub fn page_size(&self) -> usize {
        self.config.page_size
    }

    /// Allocate memory with huge pages if available
    ///
    /// # Safety
    ///
    /// The caller must ensure the returned memory is properly initialized
    /// before use and deallocated with [`deallocate`].
    pub fn allocate(&self, size: usize) -> Result<AllocationResult, HugePageError> {
        let aligned_size = self.align_to_page(size);

        if self.huge_pages_available {
            match self.allocate_huge_page(aligned_size) {
                Ok(ptr) => return Ok(AllocationResult::HugePage(ptr)),
                Err(e) => {
                    if !self.config.fallback_enabled {
                        return Err(e);
                    }
                }
            }
        }

        if self.config.fallback_enabled {
            let ptr = self.allocate_regular(aligned_size)?;
            return Ok(AllocationResult::RegularPage(ptr));
        }

        Err(HugePageError::NotSupported)
    }

    /// Deallocate memory previously allocated with this allocator
    ///
    /// # Safety
    ///
    /// - `ptr` must have been allocated by this allocator
    /// - `size` must be the same size passed to `allocate`
    /// - The memory must not be used after deallocation
    pub unsafe fn deallocate(&self, ptr: NonNull<u8>, size: usize, is_huge_page: bool) {
        let aligned_size = self.align_to_page(size);

        if is_huge_page {
            self.deallocate_huge_page(ptr, aligned_size);
        } else {
            self.deallocate_regular(ptr, aligned_size);
        }
    }

    /// Align a size to the page boundary
    pub fn align_to_page(&self, size: usize) -> usize {
        let mask = self.config.page_size - 1;
        (size + mask) & !mask
    }

    /// Check if huge pages are available on the current system
    #[cfg(target_os = "linux")]
    fn check_huge_pages_available() -> bool {
        use std::fs;

        if let Ok(content) = fs::read_to_string("/proc/sys/vm/nr_hugepages") {
            if let Ok(count) = content.trim().parse::<usize>() {
                return count > 0;
            }
        }
        false
    }

    #[cfg(not(target_os = "linux"))]
    fn check_huge_pages_available() -> bool {
        false
    }

    /// Allocate huge pages on Linux
    #[cfg(target_os = "linux")]
    fn allocate_huge_page(&self, size: usize) -> Result<NonNull<u8>, HugePageError> {
        use libc::{mmap, MAP_ANONYMOUS, MAP_HUGETLB, PROT_READ, PROT_WRITE};
        use std::ptr::null_mut;

        if size % self.config.page_size != 0 {
            return Err(HugePageError::InvalidSize {
                size,
                page_size: self.config.page_size,
            });
        }

        unsafe {
            let ptr = mmap(
                null_mut(),
                size,
                PROT_READ | PROT_WRITE,
                MAP_ANONYMOUS | MAP_HUGETLB,
                -1,
                0,
            );

            if ptr == libc::MAP_FAILED {
                return Err(HugePageError::AllocationFailed(
                    io::Error::last_os_error().to_string(),
                ));
            }

            Ok(NonNull::new_unchecked(ptr as *mut u8))
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn allocate_huge_page(&self, _size: usize) -> Result<NonNull<u8>, HugePageError> {
        Err(HugePageError::NotSupported)
    }

    /// Deallocate huge pages on Linux
    #[cfg(target_os = "linux")]
    unsafe fn deallocate_huge_page(&self, ptr: NonNull<u8>, size: usize) {
        use libc::munmap;

        let result = munmap(ptr.as_ptr() as *mut libc::c_void, size);
        debug_assert_eq!(result, 0, "munmap failed");
    }

    #[cfg(not(target_os = "linux"))]
    unsafe fn deallocate_huge_page(&self, _ptr: NonNull<u8>, _size: usize) {}

    /// Allocate regular memory as fallback
    fn allocate_regular(&self, size: usize) -> Result<NonNull<u8>, HugePageError> {
        let layout = Layout::from_size_align(size, self.config.page_size)
            .map_err(|e| HugePageError::AllocationFailed(e.to_string()))?;

        unsafe {
            let ptr = alloc(layout);
            NonNull::new(ptr).ok_or_else(|| HugePageError::AllocationFailed("alloc returned null".to_string()))
        }
    }

    /// Deallocate regular memory
    unsafe fn deallocate_regular(&self, ptr: NonNull<u8>, size: usize) {
        let layout = Layout::from_size_align_unchecked(size, self.config.page_size);
        dealloc(ptr.as_ptr(), layout);
    }
}

impl Default for HugePageAllocator {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the system's huge page size
#[cfg(target_os = "linux")]
pub fn get_system_huge_page_size() -> Option<usize> {
    use std::fs;

    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if line.starts_with("Hugepagesize:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<usize>() {
                        return Some(kb * 1024);
                    }
                }
            }
        }
    }
    None
}

#[cfg(not(target_os = "linux"))]
pub fn get_system_huge_page_size() -> Option<usize> {
    None
}

/// A buffer that uses huge pages when available
pub struct HugePageBuffer {
    ptr: NonNull<u8>,
    size: usize,
    capacity: usize,
    is_huge_page: bool,
    allocator: HugePageAllocator,
}

impl HugePageBuffer {
    /// Create a new huge page buffer with the specified capacity
    pub fn new(capacity: usize) -> Result<Self, HugePageError> {
        Self::with_config(capacity, HugePageConfig::default())
    }

    /// Create a new huge page buffer with custom configuration
    pub fn with_config(capacity: usize, config: HugePageConfig) -> Result<Self, HugePageError> {
        let allocator = HugePageAllocator::with_config(config);
        let result = allocator.allocate(capacity)?;

        Ok(Self {
            ptr: result.ptr(),
            size: 0,
            capacity,
            is_huge_page: result.is_huge_page(),
            allocator,
        })
    }

    /// Get the buffer as a slice
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.size) }
    }

    /// Get the buffer as a mutable slice
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.size) }
    }

    /// Get the capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the current size
    pub fn size(&self) -> usize {
        self.size
    }

    /// Check if this buffer uses huge pages
    pub fn is_huge_page(&self) -> bool {
        self.is_huge_page
    }

    /// Write data to the buffer
    pub fn write(&mut self, data: &[u8]) -> usize {
        let available = self.capacity.saturating_sub(self.size);
        let to_write = data.len().min(available);

        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr(),
                self.ptr.as_ptr().add(self.size),
                to_write,
            );
        }

        self.size += to_write;
        to_write
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.size = 0;
    }

    /// Resize the buffer
    pub fn resize(&mut self, new_size: usize) -> Result<(), HugePageError> {
        if new_size > self.capacity {
            let new_capacity = self.allocator.align_to_page(new_size);
            let result = self.allocator.allocate(new_capacity)?;

            unsafe {
                std::ptr::copy_nonoverlapping(
                    self.ptr.as_ptr(),
                    result.ptr().as_ptr(),
                    self.size,
                );
                self.allocator.deallocate(self.ptr, self.capacity, self.is_huge_page);
            }

            self.ptr = result.ptr();
            self.capacity = new_capacity;
            self.is_huge_page = result.is_huge_page();
        }

        self.size = new_size;
        Ok(())
    }
}

impl Drop for HugePageBuffer {
    fn drop(&mut self) {
        unsafe {
            self.allocator
                .deallocate(self.ptr, self.capacity, self.is_huge_page);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocator_creation() {
        let allocator = HugePageAllocator::new();
        assert!(allocator.config.fallback_enabled);
    }

    #[test]
    fn test_alignment() {
        let allocator = HugePageAllocator::new();
        let page_size = allocator.page_size();

        assert_eq!(allocator.align_to_page(1), page_size);
        assert_eq!(allocator.align_to_page(page_size), page_size);
        assert_eq!(allocator.align_to_page(page_size + 1), page_size * 2);
    }

    #[test]
    fn test_fallback_allocation() {
        let config = HugePageConfig::default().fallback(true);
        let allocator = HugePageAllocator::with_config(config);

        let result = allocator.allocate(4096);
        assert!(result.is_ok());
    }

    #[test]
    fn test_huge_page_buffer() {
        let buffer = HugePageBuffer::new(DEFAULT_HUGE_PAGE_SIZE);
        assert!(buffer.is_ok());

        let mut buffer = buffer.unwrap();
        assert_eq!(buffer.size(), 0);

        let written = buffer.write(b"hello");
        assert_eq!(written, 5);
        assert_eq!(buffer.size(), 5);
        assert_eq!(buffer.as_slice(), b"hello");
    }

    #[test]
    fn test_buffer_resize() {
        let mut buffer = HugePageBuffer::new(1024).unwrap();
        buffer.write(b"test data");
        assert_eq!(buffer.size(), 9);

        buffer.resize(2048).unwrap();
        assert!(buffer.capacity() >= 2048);
        assert_eq!(buffer.size(), 2048);

        let slice = &buffer.as_slice()[..9];
        assert_eq!(slice, b"test data");
    }

    #[test]
    fn test_config_builder() {
        let config = HugePageConfig::with_page_size(1024 * 1024).fallback(false);

        assert_eq!(config.page_size, 1024 * 1024);
        assert!(!config.fallback_enabled);
    }
}

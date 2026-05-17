//! Base MMap Container
//!
//! Core memory-mapped container functionality and trait definitions.

use std::path::PathBuf;

use super::types::{ContainerError, ContainerResult, ContainerStats, MemoryLevel};

pub use super::types::FileHeader;

/// Trait for data containers
pub trait IDataContainer: Send + Sync {
    /// Get the data pointer
    fn data(&self) -> *const u8;

    /// Get the mutable data pointer
    fn data_mut(&mut self) -> *mut u8;

    /// Get the size of the data
    fn size(&self) -> usize;

    /// Get the capacity
    fn capacity(&self) -> usize;

    /// Check if the container is open
    fn is_open(&self) -> bool;

    /// Sync data to disk
    fn sync(&self) -> crate::storage::container::ContainerResult<()>;

    /// Resize the container
    fn resize(&mut self, new_size: usize) -> crate::storage::container::ContainerResult<()>;

    /// Close the container
    fn close(&mut self);

    /// Get container statistics
    fn stats(&self) -> ContainerStats;

    /// Get memory level
    fn memory_level(&self) -> MemoryLevel;

    /// Get the file path (if file-backed)
    fn path(&self) -> Option<&std::path::Path> {
        None
    }

    /// Check if using huge pages
    fn is_huge_page(&self) -> bool {
        self.stats().is_huge_page
    }
}

// --- Common container operations (shared by AnonMmap and HugePageMmap) ---

pub(crate) fn checked_read_at(
    data: *const u8,
    size: usize,
    offset: usize,
    len: usize,
) -> ContainerResult<Vec<u8>> {
    if data.is_null() {
        return Err(ContainerError::NotInitialized);
    }
    if offset + len > size {
        return Err(ContainerError::InvalidSize(format!(
            "Read of {} bytes at offset {} exceeds size {}",
            len, offset, size
        )));
    }
    let mut result = vec![0u8; len];
    unsafe {
        std::ptr::copy_nonoverlapping(data.add(offset), result.as_mut_ptr(), len);
    }
    Ok(result)
}

pub(crate) fn checked_write_at(
    data: *mut u8,
    capacity: usize,
    offset: usize,
    buf: &[u8],
) -> ContainerResult<()> {
    if data.is_null() {
        return Err(ContainerError::NotInitialized);
    }
    let end = offset + buf.len();
    if end > capacity {
        return Err(ContainerError::InvalidSize(format!(
            "Write of {} bytes at offset {} exceeds capacity {}",
            buf.len(),
            offset,
            capacity
        )));
    }
    unsafe {
        std::ptr::copy_nonoverlapping(buf.as_ptr(), data.add(offset), buf.len());
    }
    Ok(())
}

/// Base mmap container (internal implementation)
pub(crate) struct MmapBase {
    pub(crate) path: Option<PathBuf>,
    pub(crate) data: *mut u8,
    pub(crate) size: usize,
    pub(crate) capacity: usize,
    pub(crate) is_huge_page: bool,
}

impl MmapBase {
    pub(crate) fn new() -> Self {
        Self {
            path: None,
            data: std::ptr::null_mut(),
            size: 0,
            capacity: 0,
            is_huge_page: false,
        }
    }

    pub(crate) fn align_to_huge_page(size: usize, huge_page_size: usize) -> usize {
        let mask = huge_page_size - 1;
        (size + mask) & !mask
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn allocate_huge_pages(size: usize) -> crate::storage::container::ContainerResult<*mut u8> {
        use std::ptr::null_mut;
        const MAP_HUGETLB: i32 = 0x40000;

        let ptr = unsafe {
            libc::mmap(
                null_mut(),
                size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | MAP_HUGETLB,
                -1,
                0,
            )
        };

        if ptr == libc::MAP_FAILED {
            return Err(crate::storage::container::ContainerError::HugePagesNotAvailable);
        }
        Ok(ptr as *mut u8)
    }

    #[cfg(not(target_os = "linux"))]
    pub(crate) fn allocate_huge_pages(_size: usize) -> crate::storage::container::ContainerResult<*mut u8> {
        Err(crate::storage::container::ContainerError::HugePagesNotAvailable)
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn deallocate_huge_pages(ptr: *mut u8, size: usize) {
        unsafe {
            libc::munmap(ptr as *mut _, size);
        }
    }

    #[cfg(not(target_os = "linux"))]
    pub(crate) fn deallocate_huge_pages(_ptr: *mut u8, _size: usize) {}
}

impl Default for MmapBase {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Send for MmapBase {}
unsafe impl Sync for MmapBase {}

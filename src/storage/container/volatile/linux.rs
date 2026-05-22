//! Linux-specific huge page implementation
//!
//! Uses `mmap(2)` with `MAP_HUGETLB` flag for huge page allocation.
//!
//! # Requirements
//!
//! - System must have huge pages configured
//! - Check `/proc/sys/vm/nr_hugepages` for available huge pages
//! - Default huge page size is typically 2MB
//!
//! # Implementation
//!
//! Uses `libc::mmap` with:
//! - `MAP_HUGETLB`: Allocate from huge page pool
//! - `MAP_PRIVATE | MAP_ANONYMOUS`: Private anonymous mapping

use crate::storage::container::types::{ContainerError, ContainerResult};

/// Huge page memory region for Linux
///
/// Safely wraps a huge page allocation, ensuring proper cleanup via Drop.
pub struct LargePageRegion {
    ptr: *mut u8,
    size: usize,
}

impl LargePageRegion {
    /// Allocate a new huge page region
    ///
    /// # Arguments
    ///
    /// * `size` - Minimum size in bytes (will be aligned to huge page size)
    /// * `huge_page_size` - The huge page size to use (typically 2MB)
    pub fn new(size: usize, huge_page_size: usize) -> ContainerResult<Self> {
        let aligned = align_to_huge_page(size, huge_page_size);
        let ptr = allocate_huge_pages(aligned)?;
        Ok(Self { ptr, size: aligned })
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.ptr as *const u8
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr
    }

    pub fn len(&self) -> usize {
        self.size
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

impl Drop for LargePageRegion {
    fn drop(&mut self) {
        if !self.ptr.is_null() && self.size > 0 {
            deallocate_huge_pages(self.ptr, self.size);
        }
    }
}

fn align_to_huge_page(size: usize, huge_page_size: usize) -> usize {
    let mask = huge_page_size - 1;
    (size + mask) & !mask
}

/// Allocate huge pages using mmap with MAP_HUGETLB
fn allocate_huge_pages(size: usize) -> ContainerResult<*mut u8> {
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
        return Err(ContainerError::HugePagesNotAvailable);
    }
    Ok(ptr as *mut u8)
}

/// Deallocate huge pages using munmap
fn deallocate_huge_pages(ptr: *mut u8, size: usize) {
    unsafe {
        libc::munmap(ptr as *mut _, size);
    }
}

unsafe impl Send for LargePageRegion {}

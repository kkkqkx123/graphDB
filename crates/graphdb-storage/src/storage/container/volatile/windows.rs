//! Windows-specific large page implementation
//!
//! Windows Large Pages are not currently implemented.
//! This module provides a stub that returns an error.
//!
//! # Future Implementation
//!
//! Windows supports Large Pages via `VirtualAlloc` with `MEM_LARGE_PAGES`:
//!
//! ```ignore
//! use winapi::um::memoryapi::VirtualAlloc;
//! use winapi::um::winnt::{MEM_LARGE_PAGES, MEM_RESERVE, MEM_COMMIT, PAGE_READWRITE};
//!
//! let ptr = VirtualAlloc(
//!     null_mut(),
//!     size,
//!     MEM_LARGE_PAGES | MEM_RESERVE | MEM_COMMIT,
//!     PAGE_READWRITE,
//! );
//! ```
//!
//! # Requirements for Large Pages on Windows
//!
//! - `SeLockMemoryPrivilege` must be enabled for the process
//! - Typically requires Administrator privileges or Group Policy configuration
//! - Large page size is typically 2MB on x64 systems
//!
//! # Alternative: AWE (Address Windowing Extensions)
//!
//! For very large allocations, consider using AWE:
//! - `AllocateUserPhysicalPages`
//! - `VirtualAlloc` with `MEM_PHYSICAL` and `MEM_RESERVE`
//! - `MapUserPhysicalPages`

use crate::storage::container::types::{ContainerError, ContainerResult};

/// Large page memory region for Windows (stub implementation)
///
/// Currently returns an error as Large Pages are not implemented.
pub struct LargePageRegion {
    _ptr: *mut u8,
    _size: usize,
}

impl LargePageRegion {
    /// Attempt to allocate a large page region
    ///
    /// Currently always returns `HugePagesNotAvailable` error.
    /// Future implementation will use `VirtualAlloc` with `MEM_LARGE_PAGES`.
    pub fn new(_size: usize, _huge_page_size: usize) -> ContainerResult<Self> {
        Err(ContainerError::HugePagesNotAvailable)
    }

    pub fn as_ptr(&self) -> *const u8 {
        std::ptr::null()
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        std::ptr::null_mut()
    }

    pub fn len(&self) -> usize {
        0
    }
}

impl Drop for LargePageRegion {
    fn drop(&mut self) {}
}

unsafe impl Send for LargePageRegion {}

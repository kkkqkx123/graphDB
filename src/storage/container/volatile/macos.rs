//! macOS-specific super page implementation
//!
//! macOS Super Pages are not currently implemented.
//! This module provides a stub that returns an error.
//!
//! # Future Implementation
//!
//! macOS supports Super Pages via `mmap` with special flags:
//!
//! ```ignore
//! const VM_FLAGS_SUPERPAGE_SIZE_2MB: i32 = 0x00080000;
//!
//! let ptr = mmap(
//!     null_mut(),
//!     size,
//!     PROT_READ | PROT_WRITE,
//!     MAP_PRIVATE | MAP_ANONYMOUS | VM_FLAGS_SUPERPAGE_SIZE_2MB,
//!     -1,
//!     0,
//! );
//! ```
//!
//! # Limitations on macOS
//!
//! - Super Pages have limited support compared to Linux HugeTLB
//! - May require specific system configurations
//! - Performance benefits are less pronounced than on Linux
//!
//! # Alternative: vm_remap
//!
//! macOS provides `vm_remap` which could potentially be used for
//! efficient memory operations, but it's not exposed by libc.

use crate::storage::container::types::{ContainerError, ContainerResult};

/// Super page memory region for macOS (stub implementation)
///
/// Currently returns an error as Super Pages are not implemented.
pub struct LargePageRegion {
    _ptr: *mut u8,
    _size: usize,
}

impl LargePageRegion {
    /// Attempt to allocate a super page region
    ///
    /// Currently always returns `HugePagesNotAvailable` error.
    /// Future implementation will use `mmap` with `VM_FLAGS_SUPERPAGE_SIZE_2MB`.
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

    pub fn is_empty(&self) -> bool {
        true
    }
}

impl Drop for LargePageRegion {
    fn drop(&mut self) {}
}

unsafe impl Send for LargePageRegion {}

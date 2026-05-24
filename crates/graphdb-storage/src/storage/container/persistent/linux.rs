//! Linux-specific memory map resize implementation
//!
//! Uses `mremap(2)` system call for efficient in-place expansion.
//!
//! # Advantages
//!
//! - Better performance compared to recreating the mapping
//! - Avoids pointer invalidation issues
//! - Can expand the mapping in-place when possible
//!
//! # Implementation
//!
//! Uses `memmap2::MmapMut::remap()` which wraps the `mremap(2)` system call.

use std::fs::File;

use crate::storage::container::types::ContainerError;
use crate::storage::container::types::ContainerResult;

/// Resize memory map using Linux's mremap(2) system call
///
/// # Arguments
///
/// * `mmap` - The memory map to resize
/// * `_file` - The backing file (unused on Linux, kept for API consistency)
/// * `new_size` - The new size in bytes
///
/// # Safety
///
/// This function is unsafe because it deals with raw memory mappings.
/// The caller must ensure that:
/// - The file has been resized to at least `new_size` bytes
/// - No other code holds pointers to the old mapping
pub fn resize_mmap(
    mmap: &mut memmap2::MmapMut,
    _file: &File,
    new_size: usize,
) -> ContainerResult<()> {
    unsafe {
        mmap.remap(new_size, memmap2::RemapOptions::new().may_move(true))
            .map_err(|e| ContainerError::MappingFailed(e.to_string()))?;
    }
    Ok(())
}

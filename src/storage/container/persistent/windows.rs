//! Windows-specific memory map resize implementation
//!
//! Windows does not have an equivalent to Linux's `mremap(2)`.
//! This implementation recreates the entire memory mapping.
//!
//! # Performance Considerations
//!
//! - Moderate overhead due to recreating the mapping
//! - Uses `VirtualAlloc` underneath (via memmap2)
//! - The OS may optimize by reusing the same virtual address
//!
//! # Future Optimizations
//!
//! Potential improvements for Windows:
//! - Use `VirtualAlloc` with `MEM_LARGE_PAGES` for large allocations
//! - Consider Address Windowing Extensions (AWE) for very large files
//! - Use `VirtualFree` and `VirtualAlloc` sequence with `MEM_RESERVE` first

use std::fs::File;

use crate::storage::container::types::ContainerError;
use crate::storage::container::types::ContainerResult;

/// Resize memory map by recreating the mapping
///
/// # Arguments
///
/// * `mmap` - The memory map to resize (will be replaced)
/// * `file` - The backing file
/// * `new_size` - The new size in bytes (unused, file size is used)
///
/// # Safety
///
/// This function is unsafe because it deals with raw memory mappings.
/// The caller must ensure that:
/// - The file has been resized to the desired size
/// - No other code holds pointers to the old mapping
pub fn resize_mmap(
    mmap: &mut memmap2::MmapMut,
    file: &File,
    _new_size: usize,
) -> ContainerResult<()> {
    // On Windows, we must recreate the entire mapping
    // This is less efficient than Linux's mremap but is the only option
    *mmap = unsafe {
        memmap2::MmapMut::map_mut(file).map_err(|e| ContainerError::MappingFailed(e.to_string()))?
    };
    Ok(())
}

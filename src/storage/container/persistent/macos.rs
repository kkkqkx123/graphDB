//! macOS-specific memory map resize implementation
//!
//! macOS does not have an equivalent to Linux's `mremap(2)`.
//! This implementation recreates the entire memory mapping.
//!
//! # Performance Considerations
//!
//! - Moderate overhead due to recreating the mapping
//! - Uses `mmap(2)` with `MAP_FIXED` option (via memmap2)
//! - macOS has copy-on-write semantics for some operations
//!
//! # macOS Specifics
//!
//! - `mmap(2)` on macOS has some differences from Linux
//! - Super Pages feature exists but has limited support
//! - `vm_remap` could potentially be used but is not exposed by memmap2
//!
//! # Future Optimizations
//!
//! Potential improvements for macOS:
//! - Use `mmap` with `VM_FLAGS_SUPERPAGE_SIZE_2MB` for large allocations
//! - Consider using `vm_remap` via direct syscall for better performance

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
    // On macOS, we must recreate the entire mapping
    // This is less efficient than Linux's mremap but is the only option
    // Note: macOS has copy-on-write semantics for some mmap operations
    *mmap = unsafe {
        memmap2::MmapMut::map_mut(file)
            .map_err(|e| ContainerError::MappingFailed(e.to_string()))?
    };
    Ok(())
}

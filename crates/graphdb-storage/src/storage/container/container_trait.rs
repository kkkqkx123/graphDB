//! Data Container Trait
//!
//! Core trait definition for storage containers.

use std::path::Path;

use super::types::{ContainerError, ContainerResult, ContainerStats, StorageBackend};

pub use super::types::FileHeader;

/// Trait for data containers
///
/// Provides a unified interface for both persistent and volatile storage.
/// NOTE: This trait only requires Send, not Sync. Containers are not intended
/// to be shared across threads directly; thread safety is managed by upper layers.
pub trait IDataContainer: Send {
    // === Core methods (must implement) ===

    /// Get the data pointer
    fn data(&self) -> *const u8;

    /// Get the mutable data pointer
    fn data_mut(&mut self) -> *mut u8;

    /// Get the size of the data
    fn size(&self) -> usize;

    /// Get the capacity
    fn capacity(&self) -> usize;

    /// Resize the container
    fn resize(&mut self, new_size: usize) -> ContainerResult<()>;

    /// Close the container
    fn close(&mut self);

    // === Persistence methods ===

    /// Sync data to disk (no-op for volatile containers)
    fn sync(&mut self) -> ContainerResult<()>;

    /// Get storage backend type
    fn storage_backend(&self) -> StorageBackend;

    // === Integrity methods ===

    /// Verify data integrity using checksum (default: no-op)
    fn verify_integrity(&self) -> ContainerResult<()> {
        Ok(())
    }

    // === Default implementations ===

    /// Check if the container is open
    fn is_open(&self) -> bool {
        !self.data().is_null()
    }

    /// Get container statistics
    fn stats(&self) -> ContainerStats {
        ContainerStats {
            capacity: self.capacity(),
            used: self.size(),
            is_huge_page: false,
            allocation_count: 0,
        }
    }

    /// Get the file path (if file-backed)
    fn file_path(&self) -> Option<&Path> {
        None
    }

    /// Check if using huge pages
    fn is_huge_page(&self) -> bool {
        self.stats().is_huge_page
    }

    // === Batch operations for improved performance ===

    /// Write multiple data chunks at different offsets in a single operation
    ///
    /// This is more efficient than multiple `write_at` calls as it minimizes
    /// internal overhead and can be optimized by the implementation.
    ///
    /// # Arguments
    ///
    /// * `operations` - Slice of (offset, data) tuples
    ///
    /// # Returns
    ///
    /// Total bytes written, or error if any operation fails
    fn write_batch(&mut self, operations: &[(usize, &[u8])]) -> ContainerResult<usize> {
        let mut total_written = 0;
        for (offset, data) in operations {
            self.write_at(*offset, data)?;
            total_written += data.len();
        }
        Ok(total_written)
    }

    /// Read multiple data chunks at different offsets in a single operation
    ///
    /// Returns a vector of results, one for each operation in order.
    fn read_batch(&self, operations: &[(usize, usize)]) -> ContainerResult<Vec<Vec<u8>>> {
        let mut results = Vec::with_capacity(operations.len());
        for (offset, len) in operations {
            results.push(self.read_at(*offset, *len)?);
        }
        Ok(results)
    }

    /// Read data at offset
    fn read_at(&self, offset: usize, len: usize) -> ContainerResult<Vec<u8>> {
        let data = self.data();
        let size = self.size();

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

    /// Write data at offset
    fn write_at(&mut self, offset: usize, buf: &[u8]) -> ContainerResult<()> {
        if buf.is_empty() {
            return Ok(());
        }

        let end = offset + buf.len();
        if end > self.size() {
            self.resize(end)?;
        }

        let data = self.data_mut();
        let capacity = self.capacity();

        if data.is_null() {
            return Err(ContainerError::NotInitialized);
        }
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

    /// Get data as slice
    fn as_slice(&self) -> &[u8] {
        if self.data().is_null() || self.size() == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.data(), self.size()) }
        }
    }

    /// Get data as mutable slice
    fn as_mut_slice(&mut self) -> &mut [u8] {
        if self.data().is_null() || self.size() == 0 {
            &mut []
        } else {
            unsafe { std::slice::from_raw_parts_mut(self.data_mut(), self.size()) }
        }
    }
}

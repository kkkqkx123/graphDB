//! Volatile Container
//!
//! In-memory container without file backing.
//! Supports both regular memory and huge pages (platform-dependent).
//!
//! Used for: temporary data, caches, testing.
//!
//! # Platform-Specific Features
//!
//! ## Huge Pages Support
//!
//! - **Linux**: Full support via mmap(2) with MAP_HUGETLB flag
//!   - Default huge page size: 2MB
//!   - Requires proper system configuration (e.g., `/proc/sys/vm/nr_hugepages`)
//!
//! - **Windows**: Not currently supported
//!   - Windows has Large Pages feature but requires SeLockMemoryPrivilege
//!   - Future implementation could use VirtualAlloc with MEM_LARGE_PAGES
//!
//! - **macOS**: Not currently supported
//!   - macOS has Super Pages feature but with limited support
//!   - Future implementation could use mmap with VM_FLAGS_SUPERPAGE_SIZE_2MB
//!
//! When huge pages are requested on non-Linux platforms, the container
//! will fall back to regular memory allocation if `huge_page_fallback` is true,
//! or return an error if `huge_page_fallback` is false.

use super::types::{
    ContainerConfig, ContainerError, ContainerResult, ContainerStats, StorageBackend,
};
use crate::storage::container::mmap::IDataContainer;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
use linux::LargePageRegion;
#[cfg(target_os = "windows")]
use windows::LargePageRegion;
#[cfg(target_os = "macos")]
use macos::LargePageRegion;

/// Volatile in-memory container
///
/// Used for temporary data, caches, or testing.
/// Uses Vec<u8> for regular memory, with optional huge page support.
pub struct VolatileContainer {
    /// Regular memory backing (used when not using huge pages)
    data: Vec<u8>,
    /// Huge page backing (platform-specific)
    huge_page: Option<LargePageRegion>,
    /// Configuration
    config: ContainerConfig,
    /// Whether currently using huge pages
    is_huge_page: bool,
    /// Number of allocation events
    allocation_count: u64,
}

impl VolatileContainer {
    pub fn new(capacity: usize) -> ContainerResult<Self> {
        Self::with_config(ContainerConfig {
            initial_capacity: capacity,
            storage_backend: StorageBackend::Volatile {
                prefer_huge_pages: false,
            },
            ..Default::default()
        })
    }

    pub fn with_huge_pages(capacity: usize) -> ContainerResult<Self> {
        Self::with_config(ContainerConfig {
            initial_capacity: capacity,
            storage_backend: StorageBackend::Volatile {
                prefer_huge_pages: true,
            },
            ..Default::default()
        })
    }

    pub fn with_config(config: ContainerConfig) -> ContainerResult<Self> {
        let prefer_huge_pages = config.storage_backend.prefers_huge_pages();
        let capacity = if prefer_huge_pages {
            align_to_huge_page(config.initial_capacity, config.huge_page_size)
        } else {
            config.initial_capacity
        };

        let mut allocation_count = 0;

        let (huge_page, is_huge_page) = if prefer_huge_pages && capacity > 0 {
            match LargePageRegion::new(capacity, config.huge_page_size) {
                Ok(region) => {
                    allocation_count = 1;
                    (Some(region), true)
                }
                Err(_) if config.huge_page_fallback => (None, false),
                Err(e) => return Err(e),
            }
        } else {
            (None, false)
        };

        let data = if is_huge_page {
            Vec::new()
        } else if capacity > 0 {
            allocation_count = 1;
            vec![0u8; capacity]
        } else {
            Vec::new()
        };

        Ok(Self {
            data,
            huge_page,
            config,
            is_huge_page,
            allocation_count,
        })
    }

    pub fn is_huge_page(&self) -> bool {
        self.is_huge_page
    }

    fn do_resize(&mut self, new_size: usize) -> ContainerResult<()> {
        if new_size <= self.size() {
            return Ok(());
        }

        if self.config.max_capacity > 0 && new_size > self.config.max_capacity {
            return Err(ContainerError::InvalidSize(
                "Exceeds maximum capacity".to_string(),
            ));
        }

        if self.huge_page.is_some() {
            return self.do_resize_huge_page(new_size);
        }

        self.do_resize_regular(new_size)
    }

    fn do_resize_regular(&mut self, new_size: usize) -> ContainerResult<()> {
        let growth_capacity =
            ((self.data.capacity() as f64 * self.config.growth_factor) as usize).max(new_size);

        if growth_capacity > self.data.capacity() {
            self.data.reserve(growth_capacity - self.data.len());
        }

        self.data.resize(new_size, 0);
        self.allocation_count += 1;
        Ok(())
    }

    fn do_resize_huge_page(&mut self, new_size: usize) -> ContainerResult<()> {
        let hp = match &mut self.huge_page {
            Some(hp) => hp,
            None => return Ok(()),
        };

        if new_size <= hp.len() {
            return Ok(());
        }

        let growth_size = ((hp.len() as f64 * self.config.growth_factor) as usize).max(new_size);
        let aligned = align_to_huge_page(growth_size, self.config.huge_page_size);

        let mut new_region = match LargePageRegion::new(aligned, self.config.huge_page_size) {
            Ok(region) => region,
            Err(_) if self.config.huge_page_fallback => {
                let mut v = Vec::with_capacity(growth_size);
                unsafe {
                    std::ptr::copy_nonoverlapping(hp.as_ptr(), v.as_mut_ptr(), self.data.len());
                    v.set_len(self.data.len());
                }
                self.data = v;
                self.is_huge_page = false;
                self.huge_page = None;
                self.allocation_count += 1;
                return self.do_resize_regular(new_size);
            }
            Err(e) => return Err(e),
        };

        unsafe {
            std::ptr::copy_nonoverlapping(hp.as_ptr(), new_region.as_mut_ptr(), self.data.len());
        }

        self.huge_page = Some(new_region);
        self.allocation_count += 1;
        Ok(())
    }

    fn do_close(&mut self) {
        self.data.clear();
        self.huge_page = None;
        self.is_huge_page = false;
        self.allocation_count = 0;
    }
}

impl super::IDataContainer for VolatileContainer {
    fn data(&self) -> *const u8 {
        if let Some(ref hp) = self.huge_page {
            return hp.as_ptr();
        }
        self.data.as_ptr()
    }

    fn data_mut(&mut self) -> *mut u8 {
        if let Some(ref mut hp) = self.huge_page {
            return hp.as_mut_ptr();
        }
        self.data.as_mut_ptr()
    }

    fn size(&self) -> usize {
        if self.huge_page.is_some() {
            return self.data.len();
        }
        self.data.len()
    }

    fn capacity(&self) -> usize {
        if let Some(ref hp) = self.huge_page {
            return hp.len();
        }
        self.data.capacity()
    }

    fn is_open(&self) -> bool {
        if self.huge_page.is_some() {
            return true;
        }
        !self.data.is_empty() || self.data.capacity() > 0
    }

    fn sync(&self) -> ContainerResult<()> {
        Ok(())
    }

    fn resize(&mut self, new_size: usize) -> ContainerResult<()> {
        self.do_resize(new_size)
    }

    fn close(&mut self) {
        self.do_close();
    }

    fn stats(&self) -> ContainerStats {
        ContainerStats {
            capacity: self.capacity(),
            used: self.size(),
            is_huge_page: self.is_huge_page,
            allocation_count: self.allocation_count,
        }
    }

    fn storage_backend(&self) -> StorageBackend {
        StorageBackend::Volatile {
            prefer_huge_pages: self.is_huge_page,
        }
    }

    fn write_batch(&mut self, operations: &[(usize, &[u8])]) -> ContainerResult<usize> {
        if operations.is_empty() {
            return Ok(0);
        }

        // Find the maximum offset
        let max_end = operations
            .iter()
            .map(|(offset, data)| offset + data.len())
            .max()
            .unwrap_or(0);

        // Resize if needed
        if max_end > self.size() {
            self.do_resize(max_end)?;
        }

        // Get the data pointer
        let ptr = self.data_mut();
        if ptr.is_null() {
            return Err(ContainerError::NotInitialized);
        }

        // Perform all writes
        let mut total_written = 0;
        for (offset, data) in operations {
            if !data.is_empty() {
                unsafe {
                    std::ptr::copy_nonoverlapping(data.as_ptr(), ptr.add(*offset), data.len());
                }
                total_written += data.len();
            }
        }

        Ok(total_written)
    }

    fn read_batch(&self, operations: &[(usize, usize)]) -> ContainerResult<Vec<Vec<u8>>> {
        let ptr = self.data();
        if ptr.is_null() {
            return Err(ContainerError::NotInitialized);
        }

        let size = self.size();
        let mut results = Vec::with_capacity(operations.len());

        for (offset, len) in operations {
            if offset + len > size {
                return Err(ContainerError::InvalidSize(format!(
                    "Read at offset {} with len {} exceeds size {}",
                    offset,
                    len,
                    size
                )));
            }

            let mut result = vec![0u8; *len];
            unsafe {
                std::ptr::copy_nonoverlapping(ptr.add(*offset), result.as_mut_ptr(), *len);
            }
            results.push(result);
        }

        Ok(results)
    }
}

unsafe impl Send for VolatileContainer {}

impl Drop for VolatileContainer {
    fn drop(&mut self) {
        self.do_close();
    }
}

fn align_to_huge_page(size: usize, huge_page_size: usize) -> usize {
    let mask = huge_page_size - 1;
    (size + mask) & !mask
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::container::mmap::IDataContainer;

    #[test]
    fn test_volatile_container_basic() {
        let mut container = VolatileContainer::new(1024).expect("Failed to create container");
        assert!(container.is_open());
        assert!(container.capacity() >= 1024);

        container.write_at(0, b"hello").expect("Failed to write");
        let data = container.read_at(0, 5).expect("Failed to read");
        assert_eq!(&data, b"hello");
    }

    #[test]
    fn test_volatile_container_resize() {
        let mut container = VolatileContainer::new(100).expect("Failed to create container");
        assert!(container.resize(1000).is_ok());
        assert!(container.capacity() >= 1000);
    }

    #[test]
    fn test_volatile_container_stats() {
        let mut container = VolatileContainer::new(1024).expect("Failed to create container");
        let stats = container.stats();
        assert_eq!(stats.allocation_count, 1);
        assert!(!stats.is_huge_page);

        container.resize(4096).expect("Failed to resize");
        let stats = container.stats();
        assert_eq!(stats.allocation_count, 2);
    }

    #[test]
    fn test_volatile_container_storage_backend() {
        let container = VolatileContainer::new(1024).expect("Failed to create container");
        assert!(container.storage_backend().is_volatile());
    }

    #[test]
    fn test_volatile_container_empty_write() {
        let mut container = VolatileContainer::new(0).expect("Failed to create container");
        container.write_at(0, b"").expect("Empty write should succeed");
        assert_eq!(container.size(), 0);
    }

    #[test]
    fn test_volatile_container_boundary_write() {
        let mut container = VolatileContainer::new(0).expect("Failed to create container");
        container.write_at(10, b"boundary").expect("Failed to write at offset 10");
        assert_eq!(container.size(), 18);

        let data = container.read_at(10, 8).expect("Failed to read");
        assert_eq!(&data, b"boundary");
    }

    #[test]
    fn test_volatile_container_send_sync() {
        fn assert_send<T: Send>() {}
        assert_send::<VolatileContainer>();
    }

    #[test]
    fn test_volatile_container_batch_write() {
        let mut container = VolatileContainer::new(1024).expect("Failed to create container");

        let operations = vec![
            (0, b"first".as_slice()),
            (10, b"second".as_slice()),
            (20, b"third".as_slice()),
        ];

        let written = container.write_batch(&operations).expect("Batch write failed");
        assert_eq!(written, 15); // 5 + 6 + 4

        let results = container
            .read_batch(&[(0, 5), (10, 6), (20, 5)])
            .expect("Batch read failed");
        assert_eq!(&results[0], b"first");
        assert_eq!(&results[1], b"second");
        assert_eq!(&results[2], b"third");
    }
}

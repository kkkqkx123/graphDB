//! Volatile Container
//!
//! In-memory container without file backing.
//! Supports both regular memory and huge pages (Linux only).
//!
//! Used for: temporary data, caches, testing.

use std::alloc::{alloc, dealloc, Layout};
use std::ptr::NonNull;

use super::types::{ContainerConfig, ContainerError, ContainerResult, ContainerStats, StorageBackend};

/// Volatile in-memory container
///
/// Used for temporary data, caches, or testing.
/// Supports optional huge pages on Linux for better TLB performance.
pub struct VolatileContainer {
    data: *mut u8,
    size: usize,
    capacity: usize,
    config: ContainerConfig,
    is_huge_page: bool,
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
        let mut is_huge_page = false;
        let mut data = std::ptr::null_mut();

        if capacity > 0 {
            if prefer_huge_pages {
                match allocate_huge_pages(capacity) {
                    Ok(ptr) => {
                        data = ptr;
                        is_huge_page = true;
                    }
                    Err(_) if config.huge_page_fallback => {
                        data = allocate_regular(capacity)?;
                    }
                    Err(e) => return Err(e),
                }
            } else {
                data = allocate_regular(capacity)?;
            }
            allocation_count = 1;
        }

        Ok(Self {
            data,
            size: 0,
            capacity,
            config,
            is_huge_page,
            allocation_count,
        })
    }

    pub fn as_slice(&self) -> &[u8] {
        if self.data.is_null() || self.size == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.data, self.size) }
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        if self.data.is_null() || self.size == 0 {
            &mut []
        } else {
            unsafe { std::slice::from_raw_parts_mut(self.data, self.size) }
        }
    }

    pub fn write_at(&mut self, offset: usize, data: &[u8]) -> ContainerResult<()> {
        if data.is_empty() {
            return Ok(());
        }
        let end = offset + data.len();
        if end > self.size {
            self.do_resize(end)?;
        }
        checked_write_at(self.data, self.capacity, offset, data)
    }

    pub fn read_at(&self, offset: usize, len: usize) -> ContainerResult<Vec<u8>> {
        checked_read_at(self.data, self.size, offset, len)
    }

    pub fn is_huge_page(&self) -> bool {
        self.is_huge_page
    }

    fn do_resize(&mut self, new_size: usize) -> ContainerResult<()> {
        if new_size <= self.capacity {
            self.size = new_size;
            return Ok(());
        }

        if self.config.max_capacity > 0 && new_size > self.config.max_capacity {
            return Err(ContainerError::InvalidSize(
                "Exceeds maximum capacity".to_string(),
            ));
        }

        let prefer_huge_pages = self.config.storage_backend.prefers_huge_pages();
        let new_capacity = if prefer_huge_pages {
            let growth_size =
                ((self.capacity as f64 * self.config.growth_factor) as usize).max(new_size);
            align_to_huge_page(growth_size, self.config.huge_page_size)
        } else {
            ((self.capacity as f64 * self.config.growth_factor) as usize).max(new_size)
        };

        let (new_data, new_is_huge_page) = if prefer_huge_pages {
            match allocate_huge_pages(new_capacity) {
                Ok(ptr) => (ptr, true),
                Err(_) if self.config.huge_page_fallback => {
                    (allocate_regular(new_capacity)?, false)
                }
                Err(e) => return Err(e),
            }
        } else {
            (allocate_regular(new_capacity)?, false)
        };

        if !self.data.is_null() && self.size > 0 {
            unsafe {
                std::ptr::copy_nonoverlapping(self.data, new_data, self.size);
            }
        }

        self.deallocate();

        self.data = new_data;
        self.capacity = new_capacity;
        self.size = new_size;
        self.is_huge_page = new_is_huge_page;
        self.allocation_count += 1;
        Ok(())
    }

    fn do_close(&mut self) {
        self.deallocate();
        self.size = 0;
        self.capacity = 0;
    }

    fn deallocate(&mut self) {
        if !self.data.is_null() && self.capacity > 0 {
            if self.is_huge_page {
                deallocate_huge_pages(self.data, self.capacity);
            } else if let Ok(layout) = Layout::from_size_align(self.capacity, 8) {
                unsafe {
                    dealloc(self.data, layout);
                }
            }
            self.data = std::ptr::null_mut();
        }
    }
}

impl super::IDataContainer for VolatileContainer {
    fn data(&self) -> *const u8 {
        self.data
    }

    fn data_mut(&mut self) -> *mut u8 {
        self.data
    }

    fn size(&self) -> usize {
        self.size
    }

    fn capacity(&self) -> usize {
        self.capacity
    }

    fn is_open(&self) -> bool {
        !self.data.is_null()
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
            capacity: self.capacity,
            used: self.size,
            is_huge_page: self.is_huge_page,
            allocation_count: self.allocation_count,
        }
    }

    fn storage_backend(&self) -> StorageBackend {
        StorageBackend::Volatile {
            prefer_huge_pages: self.is_huge_page,
        }
    }
}

impl Default for VolatileContainer {
    fn default() -> Self {
        Self::new(0).expect("Failed to create default VolatileContainer")
    }
}

unsafe impl Send for VolatileContainer {}
unsafe impl Sync for VolatileContainer {}

impl Drop for VolatileContainer {
    fn drop(&mut self) {
        self.do_close();
    }
}

#[deprecated(since = "0.2.0", note = "Use VolatileContainer instead")]
pub type AnonMmap = VolatileContainer;

#[deprecated(since = "0.2.0", note = "Use VolatileContainer::with_huge_pages instead")]
pub type HugePageMmap = VolatileContainer;

fn allocate_regular(capacity: usize) -> ContainerResult<*mut u8> {
    let layout = Layout::from_size_align(capacity, 8)
        .map_err(|e| ContainerError::InvalidSize(e.to_string()))?;
    let ptr = unsafe { alloc(layout) };
    NonNull::new(ptr)
        .map(|nn| nn.as_ptr())
        .ok_or(ContainerError::OutOfMemory)
}

fn align_to_huge_page(size: usize, huge_page_size: usize) -> usize {
    let mask = huge_page_size - 1;
    (size + mask) & !mask
}

#[cfg(target_os = "linux")]
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

#[cfg(not(target_os = "linux"))]
fn allocate_huge_pages(_size: usize) -> ContainerResult<*mut u8> {
    Err(ContainerError::HugePagesNotAvailable)
}

#[cfg(target_os = "linux")]
fn deallocate_huge_pages(ptr: *mut u8, size: usize) {
    unsafe {
        libc::munmap(ptr as *mut _, size);
    }
}

#[cfg(not(target_os = "linux"))]
fn deallocate_huge_pages(_ptr: *mut u8, _size: usize) {}

fn checked_read_at(data: *const u8, size: usize, offset: usize, len: usize) -> ContainerResult<Vec<u8>> {
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

fn checked_write_at(data: *mut u8, capacity: usize, offset: usize, buf: &[u8]) -> ContainerResult<()> {
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
        let mut container = VolatileContainer::new(1024).expect("Failed to create container");
        container.write_at(0, b"").expect("Empty write should succeed");
        assert_eq!(container.size(), 0);
    }

    #[test]
    fn test_volatile_container_boundary_write() {
        let mut container = VolatileContainer::new(20).expect("Failed to create container");
        container.write_at(10, b"boundary").expect("Failed to write at offset 10");
        assert_eq!(container.size(), 18);

        let data = container.read_at(10, 8).expect("Failed to read");
        assert_eq!(&data, b"boundary");
    }

    #[test]
    fn test_volatile_container_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<VolatileContainer>();
        assert_sync::<VolatileContainer>();
    }
}

//! Anonymous MMap Container
//!
//! In-memory containers without file backing.
//! Supports both regular memory and huge pages.

use std::alloc::{alloc, dealloc, Layout};
use std::ptr::NonNull;

use super::mmap::{IDataContainer, MmapBase};
use super::types::{
    ContainerConfig, ContainerError, ContainerResult, ContainerStats, MemoryLevel,
};

/// Anonymous mmap container (pure in-memory)
pub struct AnonMmap {
    base: MmapBase,
    config: ContainerConfig,
}

impl AnonMmap {
    pub fn new(capacity: usize) -> ContainerResult<Self> {
        Self::with_config(ContainerConfig {
            initial_capacity: capacity,
            memory_level: MemoryLevel::InMemory,
            ..Default::default()
        })
    }

    pub fn with_config(config: ContainerConfig) -> ContainerResult<Self> {
        let mut base = MmapBase::new();
        let capacity = config.initial_capacity;

        if capacity > 0 {
            let layout = Layout::from_size_align(capacity, 8)
                .map_err(|e| ContainerError::InvalidSize(e.to_string()))?;

            let ptr = unsafe { alloc(layout) };
            let data = NonNull::new(ptr).ok_or(ContainerError::OutOfMemory)?;

            base.data = data.as_ptr();
            base.capacity = capacity;
        }

        Ok(Self { base, config })
    }

    pub fn as_slice(&self) -> &[u8] {
        if self.base.data.is_null() || self.base.size == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.base.data, self.base.size) }
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        if self.base.data.is_null() || self.base.size == 0 {
            &mut []
        } else {
            unsafe { std::slice::from_raw_parts_mut(self.base.data, self.base.size) }
        }
    }

    pub fn write_at(&mut self, offset: usize, data: &[u8]) -> ContainerResult<()> {
        let end = offset + data.len();
        if end > self.base.size {
            self.resize(end)?;
        }

        if !self.base.data.is_null() && end <= self.base.capacity {
            unsafe {
                std::ptr::copy_nonoverlapping(data.as_ptr(), self.base.data.add(offset), data.len());
            }
        }
        Ok(())
    }

    pub fn read_at(&self, offset: usize, len: usize) -> ContainerResult<Vec<u8>> {
        if offset + len > self.base.size {
            return Err(ContainerError::InvalidOperation("Read out of bounds".to_string()));
        }

        if self.base.data.is_null() {
            return Ok(vec![0u8; len]);
        }

        let mut result = vec![0u8; len];
        unsafe {
            std::ptr::copy_nonoverlapping(self.base.data.add(offset), result.as_mut_ptr(), len);
        }
        Ok(result)
    }
}

impl IDataContainer for AnonMmap {
    fn data(&self) -> *const u8 {
        self.base.data
    }

    fn data_mut(&mut self) -> *mut u8 {
        self.base.data
    }

    fn size(&self) -> usize {
        self.base.size
    }

    fn capacity(&self) -> usize {
        self.base.capacity
    }

    fn is_open(&self) -> bool {
        !self.base.data.is_null()
    }

    fn sync(&self) -> ContainerResult<()> {
        Ok(())
    }

    fn resize(&mut self, new_size: usize) -> ContainerResult<()> {
        if new_size <= self.base.capacity {
            self.base.size = new_size;
            return Ok(());
        }

        if self.config.max_capacity > 0 && new_size > self.config.max_capacity {
            return Err(ContainerError::InvalidSize(
                "Exceeds maximum capacity".to_string(),
            ));
        }

        let new_capacity = ((self.base.capacity as f64 * self.config.growth_factor) as usize)
            .max(new_size);

        let layout = Layout::from_size_align(new_capacity, 8)
            .map_err(|e| ContainerError::InvalidSize(e.to_string()))?;

        let new_ptr = unsafe { alloc(layout) };
        let new_data = NonNull::new(new_ptr).ok_or(ContainerError::OutOfMemory)?;

        if !self.base.data.is_null() && self.base.size > 0 {
            unsafe {
                std::ptr::copy_nonoverlapping(self.base.data, new_data.as_ptr(), self.base.size);
                let old_layout = Layout::from_size_align_unchecked(self.base.capacity, 8);
                dealloc(self.base.data, old_layout);
            }
        }

        self.base.data = new_data.as_ptr();
        self.base.capacity = new_capacity;
        self.base.size = new_size;
        Ok(())
    }

    fn close(&mut self) {
        if !self.base.data.is_null() && self.base.capacity > 0 {
            unsafe {
                let layout = Layout::from_size_align_unchecked(self.base.capacity, 8);
                dealloc(self.base.data, layout);
            }
            self.base.data = std::ptr::null_mut();
        }
        self.base.size = 0;
        self.base.capacity = 0;
    }

    fn stats(&self) -> ContainerStats {
        ContainerStats {
            capacity: self.base.capacity,
            used: self.base.size,
            is_huge_page: self.base.is_huge_page,
            allocation_count: 0,
        }
    }

    fn memory_level(&self) -> MemoryLevel {
        MemoryLevel::InMemory
    }
}

impl Default for AnonMmap {
    fn default() -> Self {
        Self::new(0).expect("Failed to create default AnonMmap")
    }
}

impl Drop for AnonMmap {
    fn drop(&mut self) {
        self.close();
    }
}

/// Huge page mmap container
pub struct HugePageMmap {
    base: MmapBase,
    config: ContainerConfig,
}

impl HugePageMmap {
    pub fn new(capacity: usize) -> ContainerResult<Self> {
        Self::with_config(ContainerConfig {
            initial_capacity: capacity,
            memory_level: MemoryLevel::HugePagePreferred,
            ..Default::default()
        })
    }

    pub fn with_config(config: ContainerConfig) -> ContainerResult<Self> {
        let mut base = MmapBase::new();
        let capacity = MmapBase::align_to_huge_page(config.initial_capacity, config.huge_page_size);

        if capacity > 0 {
            let ptr = match MmapBase::allocate_huge_pages(capacity) {
                Ok(p) => {
                    base.is_huge_page = true;
                    p
                }
                Err(_) if config.huge_page_fallback => {
                    let layout = Layout::from_size_align(capacity, 8)
                        .map_err(|e| ContainerError::InvalidSize(e.to_string()))?;
                    let p = unsafe { alloc(layout) };
                    NonNull::new(p).ok_or(ContainerError::OutOfMemory)?.as_ptr()
                }
                Err(e) => return Err(e),
            };

            base.data = ptr;
            base.capacity = capacity;
        }

        Ok(Self { base, config })
    }

    pub fn as_slice(&self) -> &[u8] {
        if self.base.data.is_null() || self.base.size == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.base.data, self.base.size) }
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        if self.base.data.is_null() || self.base.size == 0 {
            &mut []
        } else {
            unsafe { std::slice::from_raw_parts_mut(self.base.data, self.base.size) }
        }
    }

    pub fn write_at(&mut self, offset: usize, data: &[u8]) -> ContainerResult<()> {
        let end = offset + data.len();
        if end > self.base.size {
            self.resize(end)?;
        }

        if !self.base.data.is_null() && end <= self.base.capacity {
            unsafe {
                std::ptr::copy_nonoverlapping(data.as_ptr(), self.base.data.add(offset), data.len());
            }
        }
        Ok(())
    }

    pub fn read_at(&self, offset: usize, len: usize) -> ContainerResult<Vec<u8>> {
        if offset + len > self.base.size {
            return Err(ContainerError::InvalidOperation("Read out of bounds".to_string()));
        }

        if self.base.data.is_null() {
            return Ok(vec![0u8; len]);
        }

        let mut result = vec![0u8; len];
        unsafe {
            std::ptr::copy_nonoverlapping(self.base.data.add(offset), result.as_mut_ptr(), len);
        }
        Ok(result)
    }
}

impl IDataContainer for HugePageMmap {
    fn data(&self) -> *const u8 {
        self.base.data
    }

    fn data_mut(&mut self) -> *mut u8 {
        self.base.data
    }

    fn size(&self) -> usize {
        self.base.size
    }

    fn capacity(&self) -> usize {
        self.base.capacity
    }

    fn is_open(&self) -> bool {
        !self.base.data.is_null()
    }

    fn sync(&self) -> ContainerResult<()> {
        Ok(())
    }

    fn resize(&mut self, new_size: usize) -> ContainerResult<()> {
        if new_size <= self.base.capacity {
            self.base.size = new_size;
            return Ok(());
        }

        if self.config.max_capacity > 0 && new_size > self.config.max_capacity {
            return Err(ContainerError::InvalidSize(
                "Exceeds maximum capacity".to_string(),
            ));
        }

        let new_capacity = MmapBase::align_to_huge_page(new_size, self.config.huge_page_size);
        let mut new_is_huge_page = true;

        let new_ptr = match MmapBase::allocate_huge_pages(new_capacity) {
            Ok(p) => p,
            Err(_) if self.config.huge_page_fallback => {
                new_is_huge_page = false;
                let layout = Layout::from_size_align(new_capacity, 8)
                    .map_err(|e| ContainerError::InvalidSize(e.to_string()))?;
                let p = unsafe { alloc(layout) };
                NonNull::new(p).ok_or(ContainerError::OutOfMemory)?.as_ptr()
            }
            Err(e) => return Err(e),
        };

        if !self.base.data.is_null() && self.base.size > 0 {
            unsafe {
                std::ptr::copy_nonoverlapping(self.base.data, new_ptr, self.base.size);
            }

            if self.base.is_huge_page {
                MmapBase::deallocate_huge_pages(self.base.data, self.base.capacity);
            } else {
                unsafe {
                    let old_layout = Layout::from_size_align_unchecked(self.base.capacity, 8);
                    dealloc(self.base.data, old_layout);
                }
            }
        }

        self.base.data = new_ptr;
        self.base.capacity = new_capacity;
        self.base.size = new_size;
        self.base.is_huge_page = new_is_huge_page;
        Ok(())
    }

    fn close(&mut self) {
        if !self.base.data.is_null() && self.base.capacity > 0 {
            if self.base.is_huge_page {
                MmapBase::deallocate_huge_pages(self.base.data, self.base.capacity);
            } else {
                unsafe {
                    let layout = Layout::from_size_align_unchecked(self.base.capacity, 8);
                    dealloc(self.base.data, layout);
                }
            }
            self.base.data = std::ptr::null_mut();
        }
        self.base.size = 0;
        self.base.capacity = 0;
        self.base.is_huge_page = false;
    }

    fn stats(&self) -> ContainerStats {
        ContainerStats {
            capacity: self.base.capacity,
            used: self.base.size,
            is_huge_page: self.base.is_huge_page,
            allocation_count: 0,
        }
    }

    fn memory_level(&self) -> MemoryLevel {
        MemoryLevel::HugePagePreferred
    }
}

impl Default for HugePageMmap {
    fn default() -> Self {
        Self::new(0).expect("Failed to create default HugePageMmap")
    }
}

impl Drop for HugePageMmap {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anon_mmap_basic() {
        let mut container = AnonMmap::new(1024).expect("Failed to create container");
        assert!(container.is_open());
        assert!(container.capacity() >= 1024);

        container.write_at(0, b"hello").expect("Failed to write");
        let data = container.read_at(0, 5).expect("Failed to read");
        assert_eq!(&data, b"hello");
    }

    #[test]
    fn test_anon_mmap_resize() {
        let mut container = AnonMmap::new(100).expect("Failed to create container");
        assert!(container.resize(1000).is_ok());
        assert!(container.capacity() >= 1000);
    }

    #[test]
    fn test_huge_page_mmap() {
        let config = ContainerConfig {
            initial_capacity: 1024,
            memory_level: MemoryLevel::HugePagePreferred,
            huge_page_fallback: true,
            ..Default::default()
        };

        let mut container = HugePageMmap::with_config(config).expect("Failed to create container");
        assert!(container.is_open());

        container.write_at(0, b"test").expect("Failed to write");
        let data = container.read_at(0, 4).expect("Failed to read");
        assert_eq!(&data, b"test");
    }
}

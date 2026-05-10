//! Storage Container Module
//!
//! Provides memory-mapped file and arena-based memory allocation for the storage layer.
//!
//! ## Components
//!
//! - `AnonMmap`: Anonymous memory-mapped container for temporary storage
//! - `HugePageMmap`: Huge page memory container for large allocations
//! - `FileMmap`: Memory-mapped file container for persistent storage
//! - `ArenaAllocator`: High-performance arena allocator for batch allocations
//!
//! ## Usage
//!
//! ```rust,ignore
//! use graphdb::storage::container::{AnonMmap, FileMmap, MemoryLevel, open_container};
//!
//! // Create an anonymous mmap container
//! let mut container = AnonMmap::new(1024)?;
//! container.write_at(0, b"hello")?;
//!
//! // Create a file-backed container
//! let mut container = FileMmap::create("data.bin", 1024)?;
//! container.write_at(0, b"world")?;
//!
//! // Open container based on memory level
//! let container = open_container(MemoryLevel::InMemory, None, 1024)?;
//! ```

mod anon_mmap;
mod arena_allocator;
mod file_mmap;
mod mmap;
mod types;

pub use anon_mmap::{AnonMmap, HugePageMmap};
pub use arena_allocator::{ArenaAllocator, ArenaPool};
pub use file_mmap::FileMmap;
pub use mmap::{FileHeader, IDataContainer};
pub use types::{
    ContainerConfig, ContainerError, ContainerResult, ContainerStats, MemoryLevel,
    DEFAULT_HUGE_PAGE_SIZE,
};

use std::path::Path;

/// Open a data container based on memory level
pub fn open_container<P: AsRef<Path>>(
    memory_level: MemoryLevel,
    path: Option<P>,
    capacity: usize,
) -> ContainerResult<Box<dyn IDataContainer>> {
    match memory_level {
        MemoryLevel::InMemory => Ok(Box::new(AnonMmap::new(capacity)?)),
        MemoryLevel::HugePagePreferred => Ok(Box::new(HugePageMmap::new(capacity)?)),
        MemoryLevel::SyncToFile => {
            let path = path.ok_or_else(|| {
                ContainerError::InvalidOperation(
                    "File path required for SyncToFile mode".to_string(),
                )
            })?;
            Ok(Box::new(FileMmap::create(path, capacity)?))
        }
    }
}

/// Open container from existing file (for recovery)
pub fn open_container_from_file<P: AsRef<Path>>(
    memory_level: MemoryLevel,
    path: P,
) -> ContainerResult<Box<dyn IDataContainer>> {
    let file_mmap = FileMmap::open(&path)?;

    match memory_level {
        MemoryLevel::InMemory | MemoryLevel::HugePagePreferred => {
            let mut container = AnonMmap::new(file_mmap.size())?;
            unsafe {
                std::ptr::copy_nonoverlapping(
                    file_mmap.data(),
                    container.data_mut(),
                    file_mmap.size(),
                );
            }
            container.resize(file_mmap.size())?;
            Ok(Box::new(container))
        }
        MemoryLevel::SyncToFile => Ok(Box::new(file_mmap)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_anon_mmap_workflow() {
        let mut container = AnonMmap::new(1024).expect("Failed to create container");

        container
            .write_at(0, b"hello world")
            .expect("Failed to write");

        let data = container.read_at(0, 11).expect("Failed to read");
        assert_eq!(&data, b"hello world");
    }

    #[test]
    fn test_file_mmap_workflow() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mmap");

        {
            let mut container = FileMmap::create(&path, 1024).expect("Failed to create container");
            container.write_at(0, b"hello").expect("Failed to write");
            container.sync().expect("Failed to sync");
        }

        let container = FileMmap::open(&path).expect("Failed to open container");
        let data = container.read_at(0, 5).expect("Failed to read");
        assert_eq!(&data, b"hello");
    }

    #[test]
    fn test_open_container() {
        let container = open_container(MemoryLevel::InMemory, None::<&str>, 1024)
            .expect("Failed to open container");
        assert!(container.is_open());
    }

    #[test]
    fn test_arena_workflow() {
        let arena = ArenaAllocator::new().expect("Failed to create arena");

        let ptr = arena
            .allocate_bytes(b"test data")
            .expect("Failed to allocate");

        unsafe {
            let slice = std::slice::from_raw_parts(ptr.as_ptr(), 9);
            assert_eq!(slice, b"test data");
        }
    }
}

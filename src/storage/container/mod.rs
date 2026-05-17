//! Storage Container Module
//!
//! Provides memory-mapped file containers for the storage layer.
//!
//! ## Components
//!
//! - `AnonMmap`: Anonymous memory-mapped container for temporary storage
//! - `HugePageMmap`: Huge page memory container for large allocations
//! - `FileMmap`: Memory-mapped file container for persistent storage
//!
//! For arena allocation, use `graphdb::utils::Arena` instead.
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
mod file_mmap;
mod mmap;
mod types;

pub use anon_mmap::{AnonMmap, HugePageMmap};
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
    let file_size = file_mmap.size();

    match memory_level {
        MemoryLevel::InMemory | MemoryLevel::HugePagePreferred => {
            if file_size == 0 {
                return Ok(Box::new(AnonMmap::new(0)?));
            }

            let src_ptr = file_mmap.data();
            if src_ptr.is_null() {
                return Err(ContainerError::InvalidOperation(
                    "Source file mmap has null data pointer".to_string(),
                ));
            }

            let mut container = AnonMmap::new(file_size)?;
            let dst_ptr = container.data_mut();
            if dst_ptr.is_null() {
                return Err(ContainerError::NotInitialized);
            }

            unsafe {
                std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, file_size);
            }
            container.resize(file_size)?;
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
    fn test_open_container_from_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mmap");

        {
            let mut container = FileMmap::create(&path, 1024).expect("Failed to create container");
            container.write_at(0, b"recover").expect("Failed to write");
            container.sync().expect("Failed to sync");
        }

        let container = open_container_from_file(MemoryLevel::InMemory, &path)
            .expect("Failed to open from file");
        assert!(container.is_open());
        assert!(container.size() >= 7);
        let ptr = container.data();
        assert!(!ptr.is_null());
        let slice = unsafe { std::slice::from_raw_parts(ptr, 7) };
        assert_eq!(slice, b"recover");
    }

    #[test]
    fn test_open_container_from_file_sync() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mmap");

        {
            let mut container = FileMmap::create(&path, 1024).expect("Failed to create container");
            container.write_at(0, b"sync_recover").expect("Failed to write");
            container.sync().expect("Failed to sync");
        }

        let container = open_container_from_file(MemoryLevel::SyncToFile, &path)
            .expect("Failed to open from file");
        assert!(container.is_open());
        assert!(container.size() >= 12);
        let ptr = container.data();
        assert!(!ptr.is_null());
        let slice = unsafe { std::slice::from_raw_parts(ptr, 12) };
        assert_eq!(slice, b"sync_recover");
    }

    #[test]
    fn test_open_container_memory_levels() {
        let container = open_container(MemoryLevel::InMemory, None::<&str>, 1024)
            .expect("Failed to create InMemory container");
        assert!(container.is_open());
        assert_eq!(container.memory_level(), MemoryLevel::InMemory);

        let container = open_container(MemoryLevel::HugePagePreferred, None::<&str>, 1024)
            .expect("Failed to create HugePagePreferred container");
        assert!(container.is_open());
        assert_eq!(container.memory_level(), MemoryLevel::HugePagePreferred);
    }

    #[test]
    fn test_open_container_sync_to_file_requires_path() {
        let result = open_container::<&str>(MemoryLevel::SyncToFile, None, 1024);
        assert!(result.is_err());
        match result {
            Err(ContainerError::InvalidOperation(_)) => {}
            _ => panic!("Expected InvalidOperation error"),
        }
    }

    #[test]
    fn test_container_trait_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<Box<dyn IDataContainer>>();
        assert_sync::<Box<dyn IDataContainer>>();
    }
}

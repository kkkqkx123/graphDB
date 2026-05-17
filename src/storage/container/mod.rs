//! Storage Container Module
//!
//! Provides memory-mapped storage containers for the storage layer.
//!
//! ## Architecture
//!
//! The container module provides two storage backends:
//!
//! - `PersistentContainer`: File-backed mmap for persistent storage (default)
//! - `VolatileContainer`: In-memory storage for temporary data, caches, testing
//!
//! ## Design Philosophy
//!
//! For database systems, **persistence is mandatory by default**.
//! Volatile storage is only for special cases like temporary data, caches, or testing.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use graphdb::storage::container::{PersistentContainer, VolatileContainer, StorageBackend, open_container};
//!
//! // Create a persistent container (default for database)
//! let mut container = PersistentContainer::create("data.bin", 1024)?;
//! container.write_at(0, b"hello")?;
//!
//! // Create a volatile container (for temporary data)
//! let mut container = VolatileContainer::new(1024)?;
//! container.write_at(0, b"world")?;
//!
//! // Create a volatile container with huge pages (Linux only)
//! let mut container = VolatileContainer::with_huge_pages(1024)?;
//!
//! // Open container based on storage backend
//! let container = open_container(StorageBackend::Persistent, Some("data.bin"), 1024)?;
//! ```

mod mmap;
mod persistent;
mod types;
mod volatile;

pub use mmap::{FileHeader, IDataContainer};
pub use persistent::PersistentContainer;
pub use volatile::VolatileContainer;
pub use types::{
    ContainerConfig, ContainerError, ContainerResult, ContainerStats, StorageBackend,
    DEFAULT_HUGE_PAGE_SIZE,
};

#[allow(deprecated)]
pub use persistent::FileMmap;

#[allow(deprecated)]
pub use volatile::{AnonMmap, HugePageMmap};

use std::path::Path;

/// Open a data container based on storage backend
///
/// # Arguments
///
/// * `backend` - Storage backend strategy
/// * `path` - File path (required for Persistent backend)
/// * `capacity` - Initial capacity in bytes
pub fn open_container<P: AsRef<Path>>(
    backend: StorageBackend,
    path: Option<P>,
    capacity: usize,
) -> ContainerResult<Box<dyn IDataContainer>> {
    match backend {
        StorageBackend::Persistent => {
            let path = path.ok_or_else(|| {
                ContainerError::InvalidOperation(
                    "File path required for Persistent storage backend".to_string(),
                )
            })?;
            Ok(Box::new(PersistentContainer::create(path, capacity)?))
        }
        StorageBackend::Volatile { prefer_huge_pages } => {
            if prefer_huge_pages {
                Ok(Box::new(VolatileContainer::with_huge_pages(capacity)?))
            } else {
                Ok(Box::new(VolatileContainer::new(capacity)?))
            }
        }
    }
}

/// Open container from existing file (for recovery)
///
/// Loads data from an existing file. For Persistent backend, opens the file directly.
/// For Volatile backend, loads file content into memory.
pub fn open_container_from_file<P: AsRef<Path>>(
    backend: StorageBackend,
    path: P,
) -> ContainerResult<Box<dyn IDataContainer>> {
    let persistent = PersistentContainer::open(&path)?;
    let file_size = persistent.size();

    match backend {
        StorageBackend::Persistent => Ok(Box::new(persistent)),
        StorageBackend::Volatile { .. } => {
            if file_size == 0 {
                return Ok(Box::new(VolatileContainer::new(0)?));
            }

            let src_ptr = persistent.data();
            if src_ptr.is_null() {
                return Err(ContainerError::InvalidOperation(
                    "Source file has null data pointer".to_string(),
                ));
            }

            let mut container = VolatileContainer::new(file_size)?;
            container.resize(file_size)?;

            let dst_ptr = container.data_mut();
            if dst_ptr.is_null() {
                return Err(ContainerError::NotInitialized);
            }

            unsafe {
                std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, file_size);
            }
            Ok(Box::new(container))
        }
    }
}

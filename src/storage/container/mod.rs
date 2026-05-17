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
//! ## Features
//!
//! - **Data Integrity**: MD5 checksum verification for persistent containers
//! - **Pre-allocation**: Persistent containers pre-allocate space using growth factor
//! - **Batch Operations**: Optimized `write_batch` and `read_batch` methods
//! - **Cross-platform**: Platform-specific optimizations for resize operations
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
//!
//! // Batch operations for better performance
//! let mut container = PersistentContainer::create("data.bin", 4096)?;
//! container.write_batch(&vec![(0, b"data1"), (100, b"data2")])?;
//! let results = container.read_batch(&vec![(0, 5), (100, 5)])?;
//!
//! // Verify data integrity
//! container.verify_integrity()?;
//! ```

mod container_trait;
mod persistent;
mod types;
mod volatile;

pub use container_trait::{FileHeader, IDataContainer};
pub use persistent::PersistentContainer;
pub use volatile::VolatileContainer;
pub use types::{
    ContainerConfig, ContainerError, ContainerResult, ContainerStats, StorageBackend,
    DEFAULT_HUGE_PAGE_SIZE,
};

use std::io::Read;
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
/// For Volatile backend, reads file content into memory without intermediate mmap.
pub fn open_container_from_file<P: AsRef<Path>>(
    backend: StorageBackend,
    path: P,
) -> ContainerResult<Box<dyn IDataContainer>> {
    match backend {
        StorageBackend::Persistent => {
            let persistent = PersistentContainer::open(&path)?;
            Ok(Box::new(persistent))
        }
        StorageBackend::Volatile { .. } => {
            let path = path.as_ref();
            let mut file = std::fs::File::open(path)?;
            let metadata = file.metadata()?;
            let file_size = metadata.len() as usize;

            if file_size < FileHeader::SIZE {
                return Ok(Box::new(VolatileContainer::new(0)?));
            }

            // Read header to determine data size
            let mut header_buf = vec![0u8; FileHeader::SIZE];
            file.read_exact(&mut header_buf)?;

            let header = FileHeader::from_bytes(&header_buf).ok_or_else(|| {
                ContainerError::InvalidHeader("Failed to parse header".to_string())
            })?;

            let data_size = header.data_size as usize;
            let mut container = VolatileContainer::new(data_size)?;

            if data_size > 0 {
                container.resize(data_size)?;
                let ptr = container.data_mut();
                if ptr.is_null() {
                    return Err(ContainerError::NotInitialized);
                }
                let data_slice = unsafe { std::slice::from_raw_parts_mut(ptr, data_size) };
                file.read_exact(data_slice)?;
            }

            log::info!(
                "Loaded volatile container from {} ({} bytes data)",
                path.display(),
                data_size
            );

            Ok(Box::new(container))
        }
    }
}

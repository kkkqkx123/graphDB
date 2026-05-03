//! Storage Container Module
//!
//! Provides memory-mapped file and arena-based memory allocation for the storage layer.
//!
//! ## Components
//!
//! - `MmapContainer`: Memory-mapped file container for persistent storage
//! - `AnonMmap`: Anonymous memory-mapped container for temporary storage
//! - `ArenaAllocator`: High-performance arena allocator for batch allocations
//!
//! ## Usage
//!
//! ```rust,ignore
//! use graphdb::storage::container::{MmapContainer, ArenaAllocator};
//!
//! // Create a memory-mapped container
//! let mut container = MmapContainer::create("data.bin", 1024)?;
//! container.write_at(0, b"hello")?;
//!
//! // Create an arena allocator
//! let arena = ArenaAllocator::new()?;
//! let ptr = arena.allocate(100, 8)?;
//! ```

pub mod arena_allocator;
pub mod mmap_container;
pub mod types;

pub use arena_allocator::{ArenaAllocator, ArenaPool, ThreadLocalArena};
pub use mmap_container::{AnonMmap, FileSharedMmap, IDataContainer, MmapContainer};
pub use types::{
    ContainerConfig, ContainerError, ContainerResult, ContainerStats, FileHeader,
};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_container_workflow() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test.mmap");

        let mut container =
            MmapContainer::create(&path, 1024).expect("Failed to create container");

        container
            .write_at(0, b"hello world")
            .expect("Failed to write");

        let data = container.read_at(0, 11).expect("Failed to read");
        assert_eq!(&data, b"hello world");

        container.sync().expect("Failed to sync");
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

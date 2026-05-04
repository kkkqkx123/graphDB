//! Memory Management Module
//!
//! Provides memory configuration, tracking, and optimization for the storage engine.

mod huge_pages;
mod memory_config;
mod memory_tracker;
mod null_bitmap;

pub use huge_pages::{
    AllocationResult, HugePageAllocator, HugePageBuffer, HugePageConfig, HugePageError,
    DEFAULT_HUGE_PAGE_SIZE,
};
pub use memory_config::{MemoryConfig, MemoryConfigBuilder, MemoryLevel};
pub use memory_tracker::{MemoryStats, MemoryTracker, SharedMemoryTracker};
pub use null_bitmap::NullBitmap;

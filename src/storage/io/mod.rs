//! I/O Module
//!
//! Provides I/O utilities for the storage engine including memory-mapped file support.

mod mmap;

pub use mmap::{MmapFile, MmapFileError, MmapOptions};

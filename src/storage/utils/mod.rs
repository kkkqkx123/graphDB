//! Storage Utilities Module
//!
//! Provides shared utilities and abstractions used across the storage layer.

pub mod convert;
pub mod encoding;
pub mod name_indexer;

pub use convert::props_to_map;
pub use encoding::{
    read_header, read_u32_le, read_u64_le, write_header, write_header_to, HEADER_SIZE,
};
pub use name_indexer::NameIndexer;

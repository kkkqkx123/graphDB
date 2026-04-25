//! Storage Common Module

pub mod r#trait;
pub mod types;

pub use r#trait::{Bm25Stats, StorageInterface};
pub use types::{Bm25Stats as Stats, StorageInfo};

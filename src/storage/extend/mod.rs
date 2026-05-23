//! Extended Storage Module
//!
//! Contains storage implementations that rely on external crates.
//! This includes full-text search and vector search functionality.

pub mod fulltext_storage;

#[allow(deprecated)]
pub use fulltext_storage::FulltextStorage;

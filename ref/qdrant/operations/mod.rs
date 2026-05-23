//! Operations module
//!
//! This module contains all database operations for Qdrant, organized by functional area.

pub mod collection;
pub mod points;
pub mod search;
pub mod summary;

pub use collection::CollectionOperations;
pub use points::PointOperations;
pub use search::SearchOperations;
pub use summary::{SummaryOperations, SummarySearchResult};

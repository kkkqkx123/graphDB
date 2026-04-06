//! Vector Search Module
//!
//! Provides vector search capabilities for GraphDB using Qdrant as the backend.

pub mod config;
pub mod coordinator;
pub mod manager;

pub use config::*;
pub use coordinator::{VectorChangeType, VectorCoordinator};
pub use manager::VectorIndexManager;

pub use vector_client::types::{SearchQuery, SearchResult, VectorFilter, VectorPoint};

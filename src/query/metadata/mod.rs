//! Metadata Provider Module
//!
//! This module provides metadata resolution capabilities for the query planner.
//! It allows the planner to pre-resolve index, tag, and edge type metadata before
//! generating execution plans.

pub mod provider;
pub mod context;
pub mod types;
pub mod vector_provider;
pub mod schema_provider;

pub use provider::MetadataProvider;
pub use context::MetadataContext;
pub use types::*;
pub use vector_provider::{VectorIndexMetadataProvider, CachedMetadataProvider};
pub use schema_provider::SchemaMetadataProvider;

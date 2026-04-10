//! Metadata Provider Module
//!
//! This module provides metadata resolution capabilities for the query planner.
//! It allows the planner to pre-resolve index, tag, and edge type metadata before
//! generating execution plans.

pub mod context;
pub mod provider;
pub mod schema_provider;
pub mod types;
pub mod vector_provider;

pub use context::MetadataContext;
pub use provider::MetadataProvider;
pub use schema_provider::SchemaMetadataProvider;
pub use types::*;
pub use vector_provider::{CachedMetadataProvider, VectorIndexMetadataProvider};

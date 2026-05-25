//! Metadata Provider Module
//!
//! This module provides metadata resolution capabilities for the query planner.
//! It allows the planner to pre-resolve index, tag, and edge type metadata before
//! generating execution plans.

pub mod cache_provider;
pub mod context;
pub mod provider;
pub mod schema_provider;
pub mod types;
#[cfg(feature = "qdrant")]
pub mod vector_provider;

pub use cache_provider::CachedMetadataProvider;
pub use context::MetadataContext;
pub use provider::{CompositeMetadataProvider, MetadataProvider};
pub use schema_provider::SchemaMetadataProvider;
pub use types::*;
#[cfg(feature = "qdrant")]
pub use vector_provider::VectorIndexMetadataProvider;

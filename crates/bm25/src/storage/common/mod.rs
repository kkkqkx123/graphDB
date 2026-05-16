//! Storage Common Module
//!
//! Provides shared types, traits, and utilities for BM25 storage implementations.

pub mod r#trait;
pub mod types;
pub mod metrics;

pub use r#trait::{Bm25Stats, StorageInterface};
pub use types::{Bm25Stats as Stats, StorageInfo};
pub use metrics::{ErrorType, StorageMetrics, StorageMetricsCollector, OperationTimer};

//! Compact Operation Interface
//!
//! Defines the interface for storage compaction operations.
//! This module re-exports the compact types from core for cross-module access.

pub use crate::core::types::{CompactConfig, CompactError, CompactResult, CompactStats, CompactTarget};
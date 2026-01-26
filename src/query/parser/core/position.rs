//! Position and Span types for the query parser
//!
//! This module re-exports the Position and Span types from the core module.
//! Using core types ensures consistency across the codebase.

pub use crate::core::types::Position;
pub use crate::core::types::Span;
pub use crate::core::types::ToSpan;

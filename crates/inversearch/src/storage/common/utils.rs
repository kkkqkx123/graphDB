//! Storage Module Tool Functions
//!
//! Provide helper functions shared by each storage implementation

use crate::r#type::{DocId, SearchResults};

/// Auxiliary functions for applying limits and offsets
pub fn apply_limit_offset(results: &[DocId], limit: usize, offset: usize) -> SearchResults {
    if results.is_empty() {
        return Vec::new();
    }

    let start = offset.min(results.len());
    let end = if limit > 0 {
        (start + limit).min(results.len())
    } else {
        results.len()
    };

    results[start..end].to_vec()
}

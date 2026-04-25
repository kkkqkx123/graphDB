//! Unified export of intersection modules
//!
//! Provide a unified interface for all intersection-related functions

pub mod compat;
pub mod core;
pub mod scoring;
pub mod suggestion;

// Re-exporting core functions
pub use core::{intersect, intersect_simple, intersect_union, union, union_simple};

// Re-exporting the scoring function
pub use scoring::{Bm25Scorer, ScoreConfig, ScoredId, TfIdfScorer};

// Re-exporting the proposed function
pub use suggestion::{SuggestionConfig, SuggestionEngine};

// Re-exporting compatible functions
pub use compat::{
    convert_new_to_old, convert_old_to_new, flatten_intermediate, intersect_compatible,
    intersect_union_compatible, rebuild_intermediate, union_compatible,
};

/// Compatible intersection functions (old interface)
pub fn intersect_old(
    arrays: &crate::r#type::IntermediateSearchResults,
    resolution: usize,
    limit: usize,
    offset: usize,
    suggest: bool,
    boost: i32,
    resolve: bool,
) -> crate::r#type::IntermediateSearchResults {
    compat::intersect_compatible(arrays, resolution, limit, offset, suggest, boost, resolve)
}

/// Compatible concatenation functions (old interface)
pub fn union_old(
    arrays: &crate::r#type::IntermediateSearchResults,
    limit: usize,
    offset: usize,
    sort_by_score: bool,
    boost: i32,
) -> crate::r#type::IntermediateSearchResults {
    compat::union_compatible(arrays, limit, offset, sort_by_score, boost)
}

/// Compatible intersection and union functions (old interface)
pub fn intersect_union_old(
    arrays: &crate::r#type::IntermediateSearchResults,
    mandatory: &crate::r#type::IntermediateSearchResults,
    limit: usize,
    offset: usize,
    sort_by_score: bool,
    boost: i32,
) -> crate::r#type::SearchResults {
    compat::intersect_union_compatible(arrays, mandatory, limit, offset, sort_by_score, boost)
}

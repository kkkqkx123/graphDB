//! Compatibility layer module
//!
//! Provides compatibility between new and old interfaces

use crate::intersect::core;
use crate::r#type::{IntermediateSearchResults, SearchResults};

/// Compatible intersection function
pub fn intersect_compatible(
    arrays: &IntermediateSearchResults,
    resolution: usize,
    limit: usize,
    offset: usize,
    suggest: bool,
    boost: i32,
    resolve: bool,
) -> IntermediateSearchResults {
    // Use the new new core functions directly
    core::intersect(arrays, resolution, limit, offset, suggest, boost, resolve)
}

/// Compatible union function
pub fn union_compatible(
    arrays: &IntermediateSearchResults,
    _limit: usize,
    _offset: usize,
    _sort_by_score: bool,
    _boost: i32,
) -> IntermediateSearchResults {
    // Use the new core function directly
    core::union(arrays)
}

/// Compatible intersection and union functions
pub fn intersect_union_compatible(
    arrays: &IntermediateSearchResults,
    mandatory: &IntermediateSearchResults,
    _limit: usize,
    _offset: usize,
    _sort_by_score: bool,
    _boost: i32,
) -> SearchResults {
    // Use new core functions
    core::intersect_union(arrays, mandatory, true)
}

/// Type conversion functions: from old format to new format
pub fn convert_old_to_new(old_format: Vec<Vec<u64>>) -> IntermediateSearchResults {
    old_format
}

/// Type Conversion Functions: From New Format to Old Format
pub fn convert_new_to_old(new_format: &IntermediateSearchResults) -> Vec<Vec<u64>> {
    new_format.clone()
}

/// Spreading function: Spreading a multi-story structure into a single story.
pub fn flatten_intermediate(results: &IntermediateSearchResults) -> SearchResults {
    let mut flattened = Vec::new();
    for array in results {
        flattened.extend_from_slice(array);
    }
    flattened
}

/// Reconstruction Functions: Reconstructing Single-Layer Structures into Multiple Layers
pub fn rebuild_intermediate(
    flattened: &SearchResults,
    chunk_size: usize,
) -> IntermediateSearchResults {
    let mut result = Vec::new();
    for chunk in flattened.chunks(chunk_size) {
        result.push(chunk.to_vec());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_old_to_new() {
        let old = vec![vec![1, 2, 3], vec![4, 5, 6]];
        let new = convert_old_to_new(old.clone());
        assert_eq!(new, old);
    }

    #[test]
    fn test_convert_new_to_old() {
        let new = vec![vec![1, 2, 3], vec![4, 5, 6]];
        let old = convert_new_to_old(&new);
        assert_eq!(old, new);
    }

    #[test]
    fn test_flatten_intermediate() {
        let intermediate = vec![vec![1, 2, 3], vec![4, 5, 6]];
        let flattened = flatten_intermediate(&intermediate);
        assert_eq!(flattened, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_rebuild_intermediate() {
        let flattened = vec![1, 2, 3, 4, 5, 6];
        let rebuilt = rebuild_intermediate(&flattened, 2);
        assert_eq!(rebuilt, vec![vec![1, 2], vec![3, 4], vec![5, 6]]);
    }
}

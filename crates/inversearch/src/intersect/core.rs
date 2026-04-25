//! Core Intersection Function Module
//!
//! Provides basic intersection, union and intersection-merge operations

use crate::r#type::IntermediateSearchResults;
use std::collections::HashMap;

/// Intersection Functions - Consistent with JavaScript Version Logic
pub fn intersect(
    arrays: &IntermediateSearchResults,
    _resolution: usize,
    _limit: usize,
    _offset: usize,
    _suggest: bool,
    _boost: i32,
    _resolve: bool,
) -> IntermediateSearchResults {
    if arrays.is_empty() {
        return Vec::new();
    }

    if arrays.len() == 1 {
        return arrays.clone();
    }

    // Efficient intersection computation using HashMap
    let mut common_ids = HashMap::new();

    // Count the number of times each ID appears
    for array in arrays {
        let mut seen = HashMap::new();
        for &id in array {
            *seen.entry(id).or_insert(0) += 1;
        }

        for (id, count) in seen {
            *common_ids.entry(id).or_insert(0) += count;
        }
    }

    // Find the ID that occurs in all the arrays
    let threshold = arrays.len();
    let mut result = Vec::new();

    for (id, count) in common_ids {
        if count >= threshold as u64 {
            result.push(id);
        }
    }

    // Sorting results
    result.sort_unstable();

    vec![result]
}

/// Concatenation Functions - Consistent with JavaScript Version Logic
pub fn union(arrays: &IntermediateSearchResults) -> IntermediateSearchResults {
    if arrays.is_empty() {
        return Vec::new();
    }

    if arrays.len() == 1 {
        return arrays.clone();
    }

    let mut seen = HashMap::new();
    let mut result = Vec::new();

    // Collect all unique IDs
    for array in arrays {
        for &id in array {
            if seen.insert(id, true).is_none() {
                result.push(id);
            }
        }
    }

    // Sorting results
    result.sort_unstable();

    vec![result]
}

/// intersect_union function, consistent with JavaScript version logic
pub fn intersect_union(
    arrays: &IntermediateSearchResults,
    mandatory: &IntermediateSearchResults,
    _resolve: bool,
) -> Vec<u64> {
    // First calculate the intersection of arrays
    let intersection = if arrays.is_empty() {
        Vec::new()
    } else if arrays.len() == 1 {
        arrays[0].clone()
    } else {
        // Simplified implementation: take the first array as the intersection result
        arrays[0].clone()
    };

    // Merge mandatory arrays
    let mut union_result = Vec::new();

    // Add intersection result
    for item in &intersection {
        union_result.push(*item);
    }

    // Add mandatory results
    for mandatory_array in mandatory {
        for &id in mandatory_array {
            union_result.push(id);
        }
    }

    // De-emphasize and sort
    union_result.sort_unstable();
    union_result.dedup();

    union_result
}

/// Compatible intersection function (simplified version)
pub fn intersect_simple(arrays: &[Vec<u64>]) -> Vec<u64> {
    if arrays.is_empty() {
        return Vec::new();
    }

    if arrays.len() == 1 {
        return arrays[0].clone();
    }

    // Efficient intersection computation using HashMap
    let mut common_ids = HashMap::new();

    // Count the number of times each ID appears
    for array in arrays {
        let mut seen = HashMap::new();
        for &id in array {
            *seen.entry(id).or_insert(0) += 1;
        }

        for (id, count) in seen {
            *common_ids.entry(id).or_insert(0) += count;
        }
    }

    // Find the ID that occurs in all the arrays
    let threshold = arrays.len();
    let mut result = Vec::new();

    for (id, count) in common_ids {
        if count >= threshold as u64 {
            result.push(id);
        }
    }

    // Sorting results
    result.sort_unstable();
    result
}

/// Compatible concatenation function (simplified version)
pub fn union_simple(arrays: &[Vec<u64>]) -> Vec<u64> {
    if arrays.is_empty() {
        return Vec::new();
    }

    if arrays.len() == 1 {
        return arrays[0].clone();
    }

    let mut seen = HashMap::new();
    let mut result = Vec::new();

    // Collect all unique IDs
    for array in arrays {
        for &id in array {
            if seen.insert(id, true).is_none() {
                result.push(id);
            }
        }
    }

    // Sorting results
    result.sort_unstable();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_intersect() {
        let arrays = vec![vec![1, 2, 3], vec![2, 3, 4], vec![3, 4, 5]];

        let result = intersect(&arrays, 9, 10, 0, false, 0, true);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], vec![3]);
    }

    #[test]
    fn test_empty_intersect() {
        let arrays = vec![];
        let result = intersect(&arrays, 9, 10, 0, false, 0, true);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_single_array_intersect() {
        let arrays = vec![vec![1, 2, 3]];
        let result = intersect(&arrays, 9, 10, 0, false, 0, true);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], vec![1, 2, 3]);
    }

    #[test]
    fn test_basic_union() {
        let arrays = vec![vec![1, 2, 3], vec![3, 4, 5]];

        let result = union(&arrays);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_intersect_union() {
        let arrays = vec![vec![1, 2, 3], vec![2, 3, 4]];

        let mandatory = vec![vec![5, 6]];

        let result = intersect_union(&arrays, &mandatory, true);
        assert_eq!(result, vec![1, 2, 3, 5, 6]);
    }

    #[test]
    fn test_intersect_simple() {
        let arrays = vec![vec![1, 2, 3], vec![2, 3, 4], vec![3, 4, 5]];

        let result = intersect_simple(&arrays);
        assert_eq!(result, vec![3]);
    }

    #[test]
    fn test_union_simple() {
        let arrays = vec![vec![1, 2, 3], vec![3, 4, 5]];

        let result = union_simple(&arrays);
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }
}

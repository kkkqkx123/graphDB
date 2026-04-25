//! 集合操作测试
//!
//! 测试范围：
//! - AND 交集操作
//! - OR 并集操作
//! - NOT 差集操作
//! - XOR 异或操作

use inversearch_service::r#type::IntermediateSearchResults;
use inversearch_service::resolver::{exclusion, intersect_and, union_op, xor_op};

type SearchResults = Vec<u64>;

// ============================================================================
// AND 操作测试
// ============================================================================

/// 测试空数组 AND 操作
#[test]
fn test_and_empty_arrays() {
    let arrays: Vec<IntermediateSearchResults> = vec![];
    let result = intersect_and(arrays, 100);
    assert!(result.is_empty());
}

/// 测试单个数组 AND 操作
#[test]
fn test_and_single_array() {
    let arrays: Vec<IntermediateSearchResults> = vec![vec![vec![1, 2, 3]]];
    let result = intersect_and(arrays, 100);
    assert_eq!(result, vec![vec![1, 2, 3]]);
}

/// 测试两个数组 AND 操作
#[test]
fn test_and_two_arrays() {
    let arrays: Vec<IntermediateSearchResults> =
        vec![vec![vec![1, 2, 3, 4]], vec![vec![2, 3, 4, 5]]];
    let result = intersect_and(arrays, 100);
    assert_eq!(result, vec![vec![2, 3, 4]]);
}

/// 测试无重叠 AND 操作
#[test]
fn test_and_no_overlap() {
    let arrays: Vec<IntermediateSearchResults> = vec![vec![vec![1, 2, 3]], vec![vec![4, 5, 6]]];
    let result = intersect_and(arrays, 100);
    assert!(result[0].is_empty());
}

/// 测试 AND 操作带限制
#[test]
fn test_and_with_limit() {
    let arrays: Vec<IntermediateSearchResults> =
        vec![vec![vec![1, 2, 3, 4, 5]], vec![vec![1, 2, 3, 4, 5]]];
    let result = intersect_and(arrays, 2);
    assert_eq!(result[0].len(), 2);
}

/// 测试多个数组 AND 操作
#[test]
fn test_and_multiple_arrays() {
    let arrays: Vec<IntermediateSearchResults> = vec![
        vec![vec![1, 2, 3, 4, 5]],
        vec![vec![2, 3, 4, 5, 6]],
        vec![vec![3, 4, 5, 6, 7]],
    ];
    let result = intersect_and(arrays, 100);
    assert_eq!(result, vec![vec![3, 4, 5]]);
}

// ============================================================================
// OR 操作测试
// ============================================================================

/// 测试空数组 OR 操作
#[test]
fn test_or_empty_arrays() {
    let arrays: Vec<IntermediateSearchResults> = vec![];
    let result = union_op(arrays, 0);
    assert!(result.is_empty());
}

/// 测试单个数组 OR 操作
#[test]
fn test_or_single_array() {
    let arrays: Vec<IntermediateSearchResults> = vec![vec![vec![1, 2, 3]]];
    let result = union_op(arrays, 0);
    assert_eq!(result, vec![vec![1, 2, 3]]);
}

/// 测试无重叠 OR 操作
#[test]
fn test_or_no_overlap() {
    let arrays: Vec<IntermediateSearchResults> = vec![vec![vec![1, 2, 3]], vec![vec![4, 5, 6]]];
    let result = union_op(arrays, 0);
    assert_eq!(result.len(), 2);
}

/// 测试有重叠 OR 操作
#[test]
fn test_or_with_overlap() {
    let arrays: Vec<IntermediateSearchResults> = vec![vec![vec![1, 2, 3]], vec![vec![3, 4, 5]]];
    let result = union_op(arrays, 0);
    assert_eq!(result, vec![vec![1, 2, 3, 4, 5]]);
}

/// 测试多个数组 OR 操作
#[test]
fn test_or_multiple_arrays() {
    let arrays: Vec<IntermediateSearchResults> =
        vec![vec![vec![1, 2]], vec![vec![2, 3]], vec![vec![3, 4]]];
    let result = union_op(arrays, 0);
    assert_eq!(result, vec![vec![1, 2, 3, 4]]);
}

// ============================================================================
// NOT 操作测试
// ============================================================================

/// 测试基本 NOT 操作
#[test]
fn test_not_basic() {
    let arrays: IntermediateSearchResults = vec![vec![1, 2, 3, 4, 5]];
    let exclude: SearchResults = vec![2, 3];
    let result = exclusion(arrays, &exclude, 100);
    assert_eq!(result, vec![vec![1, 4, 5]]);
}

/// 测试 NOT 操作带限制
#[test]
fn test_not_with_limit() {
    let arrays: IntermediateSearchResults = vec![vec![1, 2, 3, 4, 5]];
    let exclude: SearchResults = vec![];
    let result = exclusion(arrays, &exclude, 3);
    assert_eq!(result, vec![vec![1, 2, 3]]);
}

/// 测试全部排除 NOT 操作
#[test]
fn test_not_all_excluded() {
    let arrays: IntermediateSearchResults = vec![vec![1, 2, 3]];
    let exclude: SearchResults = vec![1, 2, 3];
    let result = exclusion(arrays, &exclude, 100);
    assert!(result.is_empty() || result[0].is_empty());
}

/// 测试空排除 NOT 操作
#[test]
fn test_not_empty_exclude() {
    let arrays: IntermediateSearchResults = vec![vec![1, 2, 3]];
    let exclude: SearchResults = vec![];
    let result = exclusion(arrays, &exclude, 100);
    assert_eq!(result, vec![vec![1, 2, 3]]);
}

/// 测试多数组 NOT 操作
#[test]
fn test_not_multiple_arrays() {
    let arrays: IntermediateSearchResults = vec![vec![1, 2], vec![3, 4, 5]];
    let exclude: SearchResults = vec![2, 4];
    let result = exclusion(arrays, &exclude, 100);
    assert_eq!(result, vec![vec![1], vec![3, 5]]);
}

// ============================================================================
// XOR 操作测试
// ============================================================================

/// 测试空数组 XOR 操作
#[test]
fn test_xor_empty_arrays() {
    let arrays: Vec<IntermediateSearchResults> = vec![];
    let result = xor_op(arrays, 0);
    assert!(result.is_empty());
}

/// 测试单个数组 XOR 操作
#[test]
fn test_xor_single_array() {
    let arrays: Vec<IntermediateSearchResults> = vec![vec![vec![1, 2, 3]]];
    let result = xor_op(arrays, 0);
    assert_eq!(result, vec![vec![1, 2, 3]]);
}

/// 测试无重叠 XOR 操作
#[test]
fn test_xor_no_overlap() {
    let arrays: Vec<IntermediateSearchResults> = vec![vec![vec![1, 2]], vec![vec![3, 4]]];
    let result = xor_op(arrays, 0);
    assert_eq!(result, vec![vec![1, 2], vec![3, 4]]);
}

/// 测试有重叠 XOR 操作
#[test]
fn test_xor_with_overlap() {
    let arrays: Vec<IntermediateSearchResults> = vec![vec![vec![1, 2, 3]], vec![vec![2, 3, 4]]];
    let result = xor_op(arrays, 0);
    assert_eq!(result, vec![vec![1], vec![4]]);
}

/// 测试完全重叠 XOR 操作
#[test]
fn test_xor_all_overlap() {
    let arrays: Vec<IntermediateSearchResults> = vec![vec![vec![1, 2]], vec![vec![1, 2]]];
    let result = xor_op(arrays, 0);
    assert!(result.iter().all(|arr| arr.is_empty()));
}

/// 测试多个数组 XOR 操作
#[test]
fn test_xor_multiple_arrays() {
    let arrays: Vec<IntermediateSearchResults> = vec![
        vec![vec![1, 2, 3]],
        vec![vec![2, 3, 4]],
        vec![vec![3, 4, 5]],
    ];
    let result = xor_op(arrays, 0);
    assert!(!result.is_empty());
}

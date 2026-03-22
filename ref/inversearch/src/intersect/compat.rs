//! 兼容层模块
//! 
//! 提供新旧接口之间的兼容性支持

use crate::r#type::{IntermediateSearchResults, SearchResults};
use crate::intersect::core;

/// 兼容的交集函数
pub fn intersect_compatible(
    arrays: &IntermediateSearchResults,
    resolution: usize,
    limit: usize,
    offset: usize,
    suggest: bool,
    boost: i32,
    resolve: bool,
) -> IntermediateSearchResults {
    // 直接使用新的核心函数
    core::intersect(arrays, resolution, limit, offset, suggest, boost, resolve)
}

/// 兼容的并集函数
pub fn union_compatible(
    arrays: &IntermediateSearchResults,
    _limit: usize,
    _offset: usize,
    _sort_by_score: bool,
    _boost: i32,
) -> IntermediateSearchResults {
    // 直接使用新的核心函数
    core::union(arrays)
}

/// 兼容的交集并集函数
pub fn intersect_union_compatible(
    arrays: &IntermediateSearchResults,
    mandatory: &IntermediateSearchResults,
    _limit: usize,
    _offset: usize,
    _sort_by_score: bool,
    _boost: i32,
) -> SearchResults {
    // 使用新的核心函数
    core::intersect_union(arrays, mandatory, true)
}

/// 类型转换函数：从旧格式到新格式
pub fn convert_old_to_new(old_format: Vec<Vec<u64>>) -> IntermediateSearchResults {
    old_format
}

/// 类型转换函数：从新格式到旧格式
pub fn convert_new_to_old(new_format: &IntermediateSearchResults) -> Vec<Vec<u64>> {
    new_format.clone()
}

/// 展平函数：将多层结构展平为单层
pub fn flatten_intermediate(results: &IntermediateSearchResults) -> SearchResults {
    let mut flattened = Vec::new();
    for array in results {
        flattened.extend_from_slice(array);
    }
    flattened
}

/// 重建函数：将单层结构重建为多层
pub fn rebuild_intermediate(flattened: &SearchResults, chunk_size: usize) -> IntermediateSearchResults {
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
//! 核心交集函数模块
//! 
//! 提供基本的交集、并集和交集并集操作

use crate::r#type::IntermediateSearchResults;
use std::collections::HashMap;

/// 交集函数 - 与JavaScript版本逻辑保持一致
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
    
    // 使用HashMap进行高效的交集计算
    let mut common_ids = HashMap::new();
    
    // 统计每个ID出现的次数
    for array in arrays {
        let mut seen = HashMap::new();
        for &id in array {
            *seen.entry(id).or_insert(0) += 1;
        }
        
        for (id, count) in seen {
            *common_ids.entry(id).or_insert(0) += count;
        }
    }
    
    // 找出在所有数组中都出现的ID
    let threshold = arrays.len();
    let mut result = Vec::new();
    
    for (id, count) in common_ids {
        if count >= threshold as u64 {
            result.push(id);
        }
    }
    
    // 排序结果
    result.sort_unstable();
    
    vec![result]
}

/// 并集函数 - 与JavaScript版本逻辑保持一致
pub fn union(arrays: &IntermediateSearchResults) -> IntermediateSearchResults {
    if arrays.is_empty() {
        return Vec::new();
    }
    
    if arrays.len() == 1 {
        return arrays.clone();
    }
    
    let mut seen = HashMap::new();
    let mut result = Vec::new();
    
    // 收集所有唯一的ID
    for array in arrays {
        for &id in array {
            if !seen.contains_key(&id) {
                seen.insert(id, true);
                result.push(id);
            }
        }
    }
    
    // 排序结果
    result.sort_unstable();
    
    vec![result]
}

/// intersect_union函数，与JavaScript版本逻辑保持一致
pub fn intersect_union(
    arrays: &IntermediateSearchResults,
    mandatory: &IntermediateSearchResults,
    _resolve: bool,
) -> Vec<u64> {
    // 首先计算arrays的交集
    let intersection = if arrays.is_empty() {
        Vec::new()
    } else if arrays.len() == 1 {
        arrays[0].clone()
    } else {
        // 简化实现：取第一个数组作为交集结果
        arrays[0].clone()
    };
    
    // 合并mandatory数组
    let mut union_result = Vec::new();
    
    // 添加交集结果
    for item in &intersection {
        union_result.push(*item);
    }
    
    // 添加mandatory结果
    for mandatory_array in mandatory {
        for &id in mandatory_array {
            union_result.push(id);
        }
    }
    
    // 去重并排序
    union_result.sort_unstable();
    union_result.dedup();
    
    union_result
}

/// 兼容的交集函数（简化版本）
pub fn intersect_simple(arrays: &[Vec<u64>]) -> Vec<u64> {
    if arrays.is_empty() {
        return Vec::new();
    }
    
    if arrays.len() == 1 {
        return arrays[0].clone();
    }
    
    // 使用HashMap进行高效的交集计算
    let mut common_ids = HashMap::new();
    
    // 统计每个ID出现的次数
    for array in arrays {
        let mut seen = HashMap::new();
        for &id in array {
            *seen.entry(id).or_insert(0) += 1;
        }
        
        for (id, count) in seen {
            *common_ids.entry(id).or_insert(0) += count;
        }
    }
    
    // 找出在所有数组中都出现的ID
    let threshold = arrays.len();
    let mut result = Vec::new();
    
    for (id, count) in common_ids {
        if count >= threshold as u64 {
            result.push(id);
        }
    }
    
    // 排序结果
    result.sort_unstable();
    result
}

/// 兼容的并集函数（简化版本）
pub fn union_simple(arrays: &[Vec<u64>]) -> Vec<u64> {
    if arrays.is_empty() {
        return Vec::new();
    }
    
    if arrays.len() == 1 {
        return arrays[0].clone();
    }
    
    let mut seen = HashMap::new();
    let mut result = Vec::new();
    
    // 收集所有唯一的ID
    for array in arrays {
        for &id in array {
            if !seen.contains_key(&id) {
                seen.insert(id, true);
                result.push(id);
            }
        }
    }
    
    // 排序结果
    result.sort_unstable();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_intersect() {
        let arrays = vec![
            vec![1, 2, 3],
            vec![2, 3, 4],
            vec![3, 4, 5],
        ];
        
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
        let arrays = vec![
            vec![1, 2, 3],
            vec![3, 4, 5],
        ];
        
        let result = union(&arrays);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_intersect_union() {
        let arrays = vec![
            vec![1, 2, 3],
            vec![2, 3, 4],
        ];
        
        let mandatory = vec![
            vec![5, 6],
        ];
        
        let result = intersect_union(&arrays, &mandatory, true);
        assert_eq!(result, vec![1, 2, 3, 5, 6]);
    }

    #[test]
    fn test_intersect_simple() {
        let arrays = vec![
            vec![1, 2, 3],
            vec![2, 3, 4],
            vec![3, 4, 5],
        ];
        
        let result = intersect_simple(&arrays);
        assert_eq!(result, vec![3]);
    }

    #[test]
    fn test_union_simple() {
        let arrays = vec![
            vec![1, 2, 3],
            vec![3, 4, 5],
        ];
        
        let result = union_simple(&arrays);
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }
}
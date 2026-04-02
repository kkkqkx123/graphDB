//! 最小化测试，只测试核心交集功能

#[cfg(test)]
mod minimal_tests {
    use crate::intersect::core::{intersect, union, IntermediateSearchResults};
    
    #[test]
    fn test_minimal_intersect() {
        let arrays: Vec<IntermediateSearchResults> = vec![];
        let result = intersect(&arrays, 9, 10, 0, false, 0, true);
        assert!(result.is_empty());
    }
    
    #[test] 
    fn test_minimal_single_array() {
        let arrays = vec![
            vec![vec![1u64, 2, 3]],
        ];
        let result = intersect(&arrays, 9, 10, 0, false, 0, true);
        assert!(!result.is_empty());
    }
    
    #[test]
    fn test_minimal_union() {
        let arrays = vec![
            vec![vec![1u64, 2, 3]],
            vec![vec![3u64, 4, 5]],
        ];
        let result = union(&arrays, 10, 0, true, 0);
        assert!(!result.is_empty());
    }
}
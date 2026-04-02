//! 简单测试
//! 
//! 测试交集模块的基本功能

#[cfg(test)]
mod simple_tests {
    use crate::intersect::core::{intersect, union, IntermediateSearchResults};
    
    #[test]
    fn test_basic_intersect() {
        let arrays = vec![
            vec![vec![1, 2, 3]],
            vec![vec![2, 3, 4]],
        ];
        let result = intersect(&arrays, 9, 10, 0, false, 0, true);
        assert!(!result.is_empty());
        
        // 验证交集结果
        let flat_result: Vec<u64> = result.iter().flatten().cloned().collect();
        println!("Intersection result: {:?}", flat_result);
    }

    #[test]
    fn test_basic_union() {
        let arrays = vec![
            vec![vec![1, 2, 3]],
            vec![vec![3, 4, 5]],
        ];
        let result = union(&arrays, 10, 0, false, 0);
        assert!(!result.is_empty());
        
        let flat_result: Vec<u64> = result.iter().flatten().cloned().collect();
        println!("Union result: {:?}", flat_result);
    }
}
//! 集成测试
//! 
//! 测试交集模块的完整功能

#[cfg(test)]
mod integration_tests {
    use crate::intersect::core::{intersect, union, ScoredId};
    use crate::intersect::scoring::{ScoreManager, ScoreConfig};
    use crate::intersect::suggestion::{SuggestionEngine, SuggestionConfig};
    use crate::r#type::IntermediateSearchResults;

    #[test]
    fn test_full_intersection_workflow() {
        // 创建测试数据
        let arrays = vec![
            vec![vec![1, 2, 3, 4, 5]],
            vec![vec![2, 3, 4, 6, 7]],
            vec![vec![3, 4, 8, 9, 10]],
        ];
        
        // 执行交集
        let result = intersect(&arrays, 9, 10, 0, false, 0, true);
        
        // 验证结果
        assert!(!result.is_empty());
        
        let flat_result: Vec<u64> = result.iter().flatten().cloned().collect();
        assert!(flat_result.contains(&3));
        assert!(flat_result.contains(&4));
    }

    #[test]
    fn test_scoring_system() {
        // 测试评分系统
        let score_manager = ScoreManager::new();
        let config = ScoreConfig::default();
        
        let scored_id = ScoredId {
            id: 123,
            score: 5.0,
            count: 2,
            positions: vec![0, 2],
        };
        
        let score = score_manager.score_document(&scored_id, &["test".to_string()], None, &config);
        assert!(score >= 0.0);
    }

    #[test]
    fn test_suggestion_system() {
        // 测试建议系统
        let mut suggestion_engine = SuggestionEngine::default();
        
        // 更新一些数据
        suggestion_engine.update_term_frequency("test", 10);
        suggestion_engine.update_co_occurrence("test", "testing", 3);
        suggestion_engine.update_co_occurrence("test", "tests", 2);
        
        let arrays = vec![
            vec![vec![1, 2, 3]],
            vec![vec![4, 5, 6]],
        ];
        
        let result = suggestion_engine.generate_suggestions_from_intermediate(
            &arrays,
            &["test".to_string()],
            5,
        );
        
        assert!(!result.is_empty());
    }

    #[test]
    fn test_empty_arrays() {
        // 测试空数组
        let empty_arrays: Vec<IntermediateSearchResults> = vec![];
        let result = intersect(&empty_arrays, 9, 10, 0, false, 0, true);
        assert!(result.is_empty());
        
        let single_array = vec![vec![vec![1, 2, 3]]];
        let result = intersect(&single_array, 9, 10, 0, false, 0, true);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_limit_and_offset() {
        // 测试限制和偏移
        let arrays = vec![
            vec![vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]],
        ];
        
        // 测试限制
        let result = intersect(&arrays, 9, 5, 0, false, 0, true);
        let flat_result: Vec<u64> = result.iter().flatten().cloned().collect();
        assert!(flat_result.len() <= 5);
        
        // 测试偏移
        let result = intersect(&arrays, 9, 10, 3, false, 0, true);
        let flat_result: Vec<u64> = result.iter().flatten().cloned().collect();
        assert!(!flat_result.contains(&1));
        assert!(!flat_result.contains(&2));
        assert!(!flat_result.contains(&3));
    }
}
//! Integration Testing
//! 
//! Test the full functionality of the intersection module

#[cfg(test)]
mod integration_tests {
    use crate::intersect::core::{intersect, union, ScoredId};
    use crate::intersect::scoring::{ScoreManager, ScoreConfig};
    use crate::intersect::suggestion::{SuggestionEngine, SuggestionConfig};
    use crate::r#type::IntermediateSearchResults;

    #[test]
    fn test_full_intersection_workflow() {
        // Creating Test Data
        let arrays = vec![
            vec![vec![1, 2, 3, 4, 5]],
            vec![vec![2, 3, 4, 6, 7]],
            vec![vec![3, 4, 8, 9, 10]],
        ];
        
        // implementation intersection
        let result = intersect(&arrays, 9, 10, 0, false, 0, true);
        
        // Verification results
        assert!(!result.is_empty());
        
        let flat_result: Vec<u64> = result.iter().flatten().cloned().collect();
        assert!(flat_result.contains(&3));
        assert!(flat_result.contains(&4));
    }

    #[test]
    fn test_scoring_system() {
        // Test scoring system
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
        // Test recommendation system
        let mut suggestion_engine = SuggestionEngine::default();
        
        // Update some data
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
        // Testing empty arrays
        let empty_arrays: Vec<IntermediateSearchResults> = vec![];
        let result = intersect(&empty_arrays, 9, 10, 0, false, 0, true);
        assert!(result.is_empty());
        
        let single_array = vec![vec![vec![1, 2, 3]]];
        let result = intersect(&single_array, 9, 10, 0, false, 0, true);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_limit_and_offset() {
        // Test limits and offsets
        let arrays = vec![
            vec![vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]],
        ];
        
        // Test Limitations
        let result = intersect(&arrays, 9, 5, 0, false, 0, true);
        let flat_result: Vec<u64> = result.iter().flatten().cloned().collect();
        assert!(flat_result.len() <= 5);
        
        // Test Offset
        let result = intersect(&arrays, 9, 10, 3, false, 0, true);
        let flat_result: Vec<u64> = result.iter().flatten().cloned().collect();
        assert!(!flat_result.contains(&1));
        assert!(!flat_result.contains(&2));
        assert!(!flat_result.contains(&3));
    }
}
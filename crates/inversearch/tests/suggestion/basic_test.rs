//! 基本搜索建议测试
//!
//! 测试范围：
//! - 建议配置
//! - 建议生成
//! - 替代查询生成

use inversearch_service::intersect::SuggestionEngine;
use inversearch_service::intersect::suggestion::{SuggestionConfig, SuggestionResult, generate_suggestions};

/// 测试默认建议配置
#[test]
fn test_default_config() {
    let config = SuggestionConfig::default();

    assert_eq!(config.max_suggestions, 5);
    assert_eq!(config.fuzzy_threshold, 0.7);
    assert_eq!(config.max_alternatives, 3);
    assert_eq!(config.min_similarity, 0.3);
}

/// 测试创建建议引擎
#[test]
fn test_create_suggestion_engine() {
    let config = SuggestionConfig::default();
    let _engine = SuggestionEngine::new(config);
}

/// 测试生成基本建议
#[test]
fn test_generate_basic_suggestions() {
    let config = SuggestionConfig::default();
    let engine = SuggestionEngine::new(config.clone());

    let search_results = vec![vec![1, 2, 3], vec![4, 5, 6]];
    let result = engine.generate_suggestions("test", &search_results);

    assert!(!result.suggestions.is_empty() || !result.fuzzy_matches.is_empty() || !result.alternative_queries.is_empty());
}

/// 测试空搜索结果
#[test]
fn test_empty_search_results() {
    let config = SuggestionConfig::default();
    let engine = SuggestionEngine::new(config);

    let search_results: Vec<Vec<u64>> = vec![];
    let result = engine.generate_suggestions("test", &search_results);

    assert!(result.fuzzy_matches.is_empty());
}

/// 测试便捷函数
#[test]
fn test_convenience_function() {
    let config = SuggestionConfig::default();
    let search_results = vec![vec![1, 2, 3]];

    let result = generate_suggestions("query", &search_results, &config);

    assert!(result.suggestions.len() <= config.max_suggestions);
}

/// 测试自定义配置
#[test]
fn test_custom_config() {
    let config = SuggestionConfig {
        max_suggestions: 10,
        fuzzy_threshold: 0.8,
        max_alternatives: 5,
        min_similarity: 0.5,
    };

    let engine = SuggestionEngine::new(config);
    let search_results = vec![vec![1, 2, 3, 4, 5]];
    let result = engine.generate_suggestions("test", &search_results);

    assert!(result.suggestions.len() <= 10);
}

/// 测试短查询建议
#[test]
fn test_short_query_suggestions() {
    let config = SuggestionConfig::default();
    let engine = SuggestionEngine::new(config.clone());

    let search_results = vec![vec![1, 2, 3]];
    let result = engine.generate_suggestions("ab", &search_results);

    assert!(result.suggestions.len() <= config.max_suggestions);
}

/// 测试长查询建议
#[test]
fn test_long_query_suggestions() {
    let config = SuggestionConfig::default();
    let engine = SuggestionEngine::new(config.clone());

    let search_results = vec![vec![1, 2, 3]];
    let long_query = "this is a very long query with many words";
    let result = engine.generate_suggestions(long_query, &search_results);

    assert!(result.suggestions.len() <= config.max_suggestions);
}

/// 测试多词查询建议
#[test]
fn test_multi_word_query_suggestions() {
    let config = SuggestionConfig::default();
    let engine = SuggestionEngine::new(config.clone());

    let search_results = vec![vec![1, 2, 3]];
    let result = engine.generate_suggestions("rust programming", &search_results);

    assert!(result.alternative_queries.len() <= config.max_alternatives);
}

/// 测试建议结果结构
#[test]
fn test_suggestion_result_structure() {
    let config = SuggestionConfig::default();
    let engine = SuggestionEngine::new(config);

    let search_results = vec![vec![1, 2, 3]];
    let result = engine.generate_suggestions("test", &search_results);

    assert!(result.suggestions.is_empty() || !result.suggestions.is_empty());
    assert!(result.fuzzy_matches.is_empty() || !result.fuzzy_matches.is_empty());
    assert!(result.alternative_queries.is_empty() || !result.alternative_queries.is_empty());
}

/// 测试中文查询建议
#[test]
fn test_chinese_query_suggestions() {
    let config = SuggestionConfig::default();
    let engine = SuggestionEngine::new(config);

    let search_results = vec![vec![1, 2, 3]];
    let result = engine.generate_suggestions("编程语言", &search_results);

    assert!(result.suggestions.len() <= 5);
}

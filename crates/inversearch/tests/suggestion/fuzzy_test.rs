//! 模糊匹配测试
//!
//! 测试范围：
//! - 相似度计算
//! - Levenshtein距离
//! - 模糊匹配结果

use inversearch_service::intersect::SuggestionEngine;
use inversearch_service::intersect::suggestion::{SuggestionConfig, SuggestionScorer};

/// 测试相似度计算 - 相同字符串
#[test]
fn test_similarity_identical() {
    let config = SuggestionConfig::default();
    let scorer = SuggestionScorer::new(config);

    let score = scorer.score_suggestion("test", "test");
    assert!((score - 1.0).abs() < 0.01, "相同字符串相似度应该接近1.0");
}

/// 测试相似度计算 - 完全不同
#[test]
fn test_similarity_completely_different() {
    let config = SuggestionConfig::default();
    let scorer = SuggestionScorer::new(config);

    let score = scorer.score_suggestion("abc", "xyz");
    assert!(score < 0.5, "完全不同的字符串相似度应该很低");
}

/// 测试相似度计算 - 相似字符串
#[test]
fn test_similarity_similar() {
    let config = SuggestionConfig::default();
    let scorer = SuggestionScorer::new(config);

    let score = scorer.score_suggestion("test", "tset");
    assert!(score > 0.3, "相似字符串应该有较高相似度");
}

/// 测试相似度计算 - 单字符差异
#[test]
fn test_similarity_single_char_diff() {
    let config = SuggestionConfig::default();
    let scorer = SuggestionScorer::new(config);

    let score = scorer.score_suggestion("test", "tast");
    assert!(score > 0.7, "单字符差异应该有较高相似度");
}

/// 测试相似度计算 - 空字符串
#[test]
fn test_similarity_empty() {
    let config = SuggestionConfig::default();
    let scorer = SuggestionScorer::new(config);

    let score = scorer.score_suggestion("", "");
    assert!((score - 1.0).abs() < 0.01, "两个空字符串相似度应该为1.0");

    let score2 = scorer.score_suggestion("test", "");
    assert!(score2 < 0.5, "非空与空字符串相似度应该很低");
}

/// 测试模糊匹配结果过滤
#[test]
fn test_fuzzy_match_filtering() {
    let config = SuggestionConfig {
        max_suggestions: 3,
        fuzzy_threshold: 0.5,
        max_alternatives: 2,
        min_similarity: 0.3,
    };
    let engine = SuggestionEngine::new(config);

    let search_results = vec![vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]];
    let result = engine.generate_suggestions("test", &search_results);

    assert!(result.fuzzy_matches.len() <= 3, "模糊匹配结果应该受max_suggestions限制");
}

/// 测试高阈值过滤
#[test]
fn test_high_threshold_filtering() {
    let config = SuggestionConfig {
        max_suggestions: 5,
        fuzzy_threshold: 0.9,
        max_alternatives: 3,
        min_similarity: 0.8,
    };
    let engine = SuggestionEngine::new(config);

    let search_results = vec![vec![1, 2, 3]];
    let result = engine.generate_suggestions("xyz", &search_results);

    assert!(result.fuzzy_matches.len() <= 5);
}

/// 测试低阈值过滤
#[test]
fn test_low_threshold_filtering() {
    let config = SuggestionConfig {
        max_suggestions: 5,
        fuzzy_threshold: 0.1,
        max_alternatives: 3,
        min_similarity: 0.1,
    };
    let engine = SuggestionEngine::new(config);

    let search_results = vec![vec![1, 2, 3]];
    let result = engine.generate_suggestions("test", &search_results);

    assert!(result.fuzzy_matches.len() <= 5);
}

/// 测试拼写变体生成
#[test]
fn test_spelling_variants() {
    let config = SuggestionConfig::default();
    let engine = SuggestionEngine::new(config);

    let search_results = vec![vec![1, 2, 3]];
    let result = engine.generate_suggestions("test", &search_results);

    assert!(result.suggestions.len() <= 5);
}

/// 测试中文模糊匹配
#[test]
fn test_chinese_fuzzy_match() {
    let config = SuggestionConfig::default();
    let scorer = SuggestionScorer::new(config);

    let score = scorer.score_suggestion("编程", "编成");
    assert!(score > 0.5, "相似中文应该有较高相似度");
}

/// 测试混合字符模糊匹配
#[test]
fn test_mixed_char_fuzzy_match() {
    let config = SuggestionConfig::default();
    let scorer = SuggestionScorer::new(config);

    let score = scorer.score_suggestion("Rust编程", "rust编成");
    assert!(score > 0.5, "相似混合字符串应该有较高相似度");
}

/// 测试数字模糊匹配
#[test]
fn test_numeric_fuzzy_match() {
    let config = SuggestionConfig::default();
    let scorer = SuggestionScorer::new(config);

    let score = scorer.score_suggestion("test123", "test124");
    assert!(score > 0.8, "相似数字字符串应该有较高相似度");
}

/// 测试特殊字符模糊匹配
#[test]
fn test_special_char_fuzzy_match() {
    let config = SuggestionConfig::default();
    let scorer = SuggestionScorer::new(config);

    let score = scorer.score_suggestion("test@email.com", "test@email.con");
    assert!(score > 0.8, "相似特殊字符字符串应该有较高相似度");
}

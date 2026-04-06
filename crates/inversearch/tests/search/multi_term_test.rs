//! 多词搜索测试
//!
//! 测试范围：
//! - AND 逻辑
//! - OR 逻辑
//! - 词组搜索

use inversearch_service::search::search;

use crate::common::{
    create_english_index, create_empty_index,
    basic_search_options,
};

/// 测试多词搜索返回更多结果
#[test]
fn test_multi_term_returns_more_results() {
    let index = create_english_index();

    // 单词搜索
    let options = basic_search_options("Rust");
    let single_result = search(&index, &options).unwrap();

    // 多词搜索
    let options = basic_search_options("Rust Python JavaScript");
    let multi_result = search(&index, &options).unwrap();

    // 多词搜索应该返回更多或相同数量的结果
    assert!(
        multi_result.total >= single_result.total,
        "多词搜索应该返回更多结果"
    );
}

/// 测试搜索多个不同词
#[test]
fn test_search_multiple_different_terms() {
    let mut index = create_empty_index();

    index.add(1, "apple banana cherry", false).unwrap();
    index.add(2, "apple banana", false).unwrap();
    index.add(3, "banana cherry", false).unwrap();
    index.add(4, "apple cherry", false).unwrap();

    // 搜索包含所有三个词的文档
    let options = basic_search_options("apple banana cherry");
    let result = search(&index, &options).unwrap();

    // 应该找到文档 1
    assert!(result.results.contains(&1), "Expected results to contain document 1");
}

/// 测试词组搜索
#[test]
fn test_phrase_search() {
    let mut index = create_empty_index();

    index.add(1, "quick brown fox", false).unwrap();
    index.add(2, "brown quick fox", false).unwrap();
    index.add(3, "the quick brown fox jumps", false).unwrap();

    // 搜索词组
    let options = basic_search_options("quick brown");
    let result = search(&index, &options).unwrap();

    // 应该找到包含这个词组的文档
    assert!(!result.results.is_empty(), "Expected non-empty results");
    assert!(result.results.contains(&1), "Expected results to contain document 1");
}

/// 测试停用词处理
#[test]
fn test_stop_words_handling() {
    let mut index = create_empty_index();

    index.add(1, "the quick brown fox", false).unwrap();
    index.add(2, "a quick brown dog", false).unwrap();

    // 搜索包含停用词的查询
    let options = basic_search_options("the quick");
    let result = search(&index, &options).unwrap();

    // 应该返回结果（停用词被忽略）
    assert!(!result.results.is_empty(), "Expected non-empty results");
}

/// 测试大小写不敏感搜索
#[test]
fn test_case_insensitive_search() {
    let mut index = create_empty_index();

    index.add(1, "Rust Programming Language", false).unwrap();
    index.add(2, "rust programming language", false).unwrap();
    index.add(3, "RUST PROGRAMMING LANGUAGE", false).unwrap();

    // 小写搜索
    let options = basic_search_options("rust");
    let result = search(&index, &options).unwrap();

    // 应该找到所有三个文档
    assert_eq!(result.results.len(), 3, "大小写不敏感搜索应该找到所有文档");
}

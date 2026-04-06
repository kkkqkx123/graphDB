//! 拉丁字符集测试
//!
//! 测试范围：
//! - 基本拉丁字符
//! - 大小写不敏感
//! - 重音符号处理
//! - 特殊拉丁字符

use inversearch_service::search::search;

use crate::common::{
    create_empty_index, basic_search_options,
};

/// 测试基本拉丁字符
#[test]
fn test_basic_latin() {
    let mut index = create_empty_index();

    index.add(1, "Hello World", false).unwrap();
    index.add(2, "Rust Programming", false).unwrap();

    let options = basic_search_options("Hello");
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "Expected non-empty results");
    assert!(result.results.contains(&1), "Expected results to contain document 1");
}

/// 测试大小写不敏感
#[test]
fn test_case_insensitive() {
    let mut index = create_empty_index();

    index.add(1, "Hello World", false).unwrap();

    // 搜索小写
    let options = basic_search_options("hello");
    let result_lower = search(&index, &options).unwrap();

    // 搜索大写
    let options = basic_search_options("HELLO");
    let result_upper = search(&index, &options).unwrap();

    // 搜索混合大小写
    let options = basic_search_options("HeLLo");
    let result_mixed = search(&index, &options).unwrap();

    // 结果应该相同
    assert!(!result_lower.results.is_empty(), "小写搜索应该有结果");
    assert!(!result_upper.results.is_empty(), "大写搜索应该有结果");
    assert!(!result_mixed.results.is_empty(), "混合大小写搜索应该有结果");
    
    // 所有搜索都应该找到文档 1
    assert!(result_lower.results.contains(&1));
    assert!(result_upper.results.contains(&1));
    assert!(result_mixed.results.contains(&1));
}

/// 测试带重音符号的字符
#[test]
fn test_accented_characters() {
    let mut index = create_empty_index();

    // 添加带重音符号的文本
    index.add(1, "café résumé naïve", false).unwrap();
    index.add(2, "Café is a place", false).unwrap();

    // 搜索带重音符号的词
    let options = basic_search_options("café");
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "Expected non-empty results");
    
    // 搜索不带重音符号的版本（取决于规范化实现）
    let options = basic_search_options("cafe");
    let _result = search(&index, &options);
}

/// 测试德语字符
#[test]
fn test_german_characters() {
    let mut index = create_empty_index();

    index.add(1, "Übergröße Straße", false).unwrap();

    let options = basic_search_options("Übergröße");
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "Expected non-empty results");
}

/// 测试法语字符
#[test]
fn test_french_characters() {
    let mut index = create_empty_index();

    index.add(1, "Français œuf", false).unwrap();

    let options = basic_search_options("Français");
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "Expected non-empty results");
}

/// 测试西班牙语字符
#[test]
fn test_spanish_characters() {
    let mut index = create_empty_index();

    index.add(1, "El niño español", false).unwrap();

    let options = basic_search_options("español");
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "Expected non-empty results");
}

/// 测试数字和拉丁字符混合
#[test]
fn test_latin_with_numbers() {
    let mut index = create_empty_index();

    index.add(1, "Version 2.0 released", false).unwrap();
    index.add(2, "Chapter 1: Introduction", false).unwrap();

    let options = basic_search_options("Version 2.0");
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "Expected non-empty results");
}

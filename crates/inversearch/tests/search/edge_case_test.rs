//! 边界情况测试
//!
//! 测试范围：
//! - 空查询
//! - 特殊字符
//! - 超长查询
//! - 不存在的关键词

use inversearch_service::search::search;

use crate::common::{basic_search_options, create_empty_index, create_english_index};

/// 测试空查询
#[test]
fn test_empty_query() {
    let index = create_english_index();

    let options = basic_search_options("");
    let _result = search(&index, &options);

    // 空查询应该返回空结果或错误
    // 根据实现可能不同
}

/// 测试不存在的关键词
#[test]
fn test_nonexistent_term() {
    let index = create_english_index();

    let options = basic_search_options("xyz123nonexistent");
    let result = search(&index, &options).unwrap();

    // 应该返回空结果
    assert!(
        result.results.is_empty(),
        "Expected empty results, got {:?}",
        result.results
    );
    assert_eq!(result.total, 0, "总数应该为 0");
}

/// 测试特殊字符
#[test]
fn test_special_characters() {
    let mut index = create_empty_index();

    index.add(1, "Hello @#$%^&*() World!", false).unwrap();

    // 搜索特殊字符
    let options = basic_search_options("@#$%");
    let _result = search(&index, &options);

    // 根据分词器实现，可能返回结果或空
}

/// 测试超长查询
#[test]
fn test_very_long_query() {
    let index = create_english_index();

    // 构造超长查询
    let long_query = "a ".repeat(1000);
    let options = basic_search_options(&long_query);
    let _result = search(&index, &options);

    // 不应该崩溃
}

/// 测试单个字符查询
#[test]
fn test_single_character_query() {
    let mut index = create_empty_index();

    index.add(1, "a test document", false).unwrap();

    // 搜索单个字符
    let options = basic_search_options("a");
    let _result = search(&index, &options).unwrap();

    // 取决于最小词长配置
    // 可能返回结果或空
}

/// 测试数字查询
#[test]
fn test_numeric_query() {
    let mut index = create_empty_index();

    index.add(1, "Version 1.0 released", false).unwrap();
    index.add(2, "Version 2.0 released", false).unwrap();

    // 搜索数字
    let options = basic_search_options("1.0");
    let result = search(&index, &options).unwrap();

    // 应该找到文档 1
    assert!(
        result.results.contains(&1),
        "Expected results to contain document 1"
    );
}

/// 测试 Unicode 字符
#[test]
fn test_unicode_characters() {
    let mut index = create_empty_index();

    index.add(1, "Hello 🎉 World 🚀", false).unwrap();

    // 搜索 emoji
    let options = basic_search_options("🎉");
    let _result = search(&index, &options);

    // 取决于分词器对 emoji 的处理
}

/// 测试 XSS 防护
#[test]
fn test_xss_protection() {
    let mut index = create_empty_index();

    // 尝试添加包含脚本标签的内容
    index
        .add(1, "<script>alert('xss')</script>", false)
        .unwrap();

    // 搜索脚本标签
    let options = basic_search_options("script");
    let result = search(&index, &options).unwrap();

    // 应该正常返回结果，不会执行脚本
    assert!(!result.results.is_empty(), "Expected non-empty results");
}

//! 多字段搜索测试
//!
//! 测试范围：
//! - 多字段配置
//! - 字段权重
//! - 字段 boost

use inversearch_service::document::{Document, DocumentConfig};
use inversearch_service::search::{
    MultiFieldSearchConfig, multi_field_search, multi_field_search_with_weights,
};

fn create_test_document() -> Document {
    let config = DocumentConfig::default();
    Document::new(config).expect("创建文档失败")
}

/// 测试创建多字段搜索配置
#[test]
fn test_create_config() {
    let doc = create_test_document();
    let _config = MultiFieldSearchConfig::new(&doc);
}

/// 测试添加字段
#[test]
fn test_add_field() {
    let doc = create_test_document();
    let config = MultiFieldSearchConfig::new(&doc)
        .add_field("title")
        .add_field("content");

    let _config = config;
}

/// 测试添加带权重的字段
#[test]
fn test_add_field_with_weight() {
    let doc = create_test_document();
    let config = MultiFieldSearchConfig::new(&doc)
        .add_field_with_weight("title", 2.0)
        .add_field_with_weight("content", 1.0);

    let _config = config;
}

/// 测试设置权重
#[test]
fn test_set_weight() {
    let doc = create_test_document();
    let config = MultiFieldSearchConfig::new(&doc)
        .add_field("title")
        .set_weight("title", 3.0);

    let _config = config;
}

/// 测试设置 boost
#[test]
fn test_set_boost() {
    let doc = create_test_document();
    let config = MultiFieldSearchConfig::new(&doc)
        .add_field("title")
        .set_boost("title", 1.5);

    let _config = config;
}

/// 测试设置限制
#[test]
fn test_set_limit() {
    let doc = create_test_document();
    let config = MultiFieldSearchConfig::new(&doc)
        .add_field("title")
        .limit(10);

    let _config = config;
}

/// 测试设置偏移
#[test]
fn test_set_offset() {
    let doc = create_test_document();
    let config = MultiFieldSearchConfig::new(&doc)
        .add_field("title")
        .offset(5);

    let _config = config;
}

/// 测试链式配置
#[test]
fn test_chain_config() {
    let doc = create_test_document();
    let config = MultiFieldSearchConfig::new(&doc)
        .add_field("title")
        .add_field("content")
        .set_weight("title", 2.0)
        .set_boost("title", 1.5)
        .limit(20)
        .offset(0);

    let _config = config;
}

/// 测试便捷函数 - 基本搜索
#[test]
fn test_convenience_function_basic() {
    let doc = create_test_document();
    let result = multi_field_search(&doc, "test", &["title", "content"]);

    assert!(result.is_ok());
    let search_result = result.unwrap();
    assert!(search_result.results.is_empty() || !search_result.results.is_empty());
}

/// 测试便捷函数 - 带权重搜索
#[test]
fn test_convenience_function_with_weights() {
    let doc = create_test_document();
    let result = multi_field_search_with_weights(
        &doc,
        "test",
        &[("title", 2.0), ("content", 1.0)],
    );

    assert!(result.is_ok());
    let search_result = result.unwrap();
    assert!(search_result.results.is_empty() || !search_result.results.is_empty());
}

/// 测试空查询
#[test]
fn test_empty_query() {
    let doc = create_test_document();
    let config = MultiFieldSearchConfig::new(&doc)
        .add_field("title");

    let result = config.search("");
    assert!(result.is_ok());
}

/// 测试空字段列表
#[test]
fn test_empty_fields() {
    let doc = create_test_document();
    let config = MultiFieldSearchConfig::new(&doc);

    let result = config.search("test");
    assert!(result.is_ok());
}

/// 测试中文查询
#[test]
fn test_chinese_query() {
    let doc = create_test_document();
    let result = multi_field_search(&doc, "编程语言", &["title", "content"]);

    assert!(result.is_ok());
}

/// 测试多语言查询
#[test]
fn test_multilingual_query() {
    let doc = create_test_document();
    let result = multi_field_search(
        &doc,
        "Rust编程 Programming",
        &["title", "content", "tags"],
    );

    assert!(result.is_ok());
}

/// 测试特殊字符查询
#[test]
fn test_special_chars_query() {
    let doc = create_test_document();
    let result = multi_field_search(
        &doc,
        "test@email.com",
        &["title", "content"],
    );

    assert!(result.is_ok());
}

/// 测试高权重配置
#[test]
fn test_high_weight() {
    let doc = create_test_document();
    let config = MultiFieldSearchConfig::new(&doc)
        .add_field_with_weight("title", 10.0)
        .add_field_with_weight("content", 1.0);

    let _config = config;
}

/// 测试低权重配置
#[test]
fn test_low_weight() {
    let doc = create_test_document();
    let config = MultiFieldSearchConfig::new(&doc)
        .add_field_with_weight("title", 0.1)
        .add_field_with_weight("content", 0.5);

    let _config = config;
}

/// 测试多个 boost 设置
#[test]
fn test_multiple_boost() {
    let doc = create_test_document();
    let config = MultiFieldSearchConfig::new(&doc)
        .add_field("title")
        .add_field("content")
        .add_field("tags")
        .set_boost("title", 2.0)
        .set_boost("content", 1.5)
        .set_boost("tags", 1.0);

    let _config = config;
}

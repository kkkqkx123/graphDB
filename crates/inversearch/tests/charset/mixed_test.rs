//! 混合字符集测试
//!
//! 测试范围：
//! - 中英文混合
//! - 多语言混合
//! - Emoji 和特殊符号
//! - URL 和代码片段

use inversearch_service::search::search;

use crate::common::{
    create_empty_index, basic_search_options,
};

/// 测试中英文混合
#[test]
fn test_chinese_english_mix() {
    let mut index = create_empty_index();

    index.add(1, "Rust 编程语言", false).unwrap();
    index.add(2, "Python is 简单易学", false).unwrap();
    index.add(3, "JavaScript Web开发", false).unwrap();

    // 搜索中文
    let options = basic_search_options("编程");
    let result = search(&index, &options).unwrap();
    assert!(!result.results.is_empty(), "中文搜索应该有结果");

    // 搜索英文
    let options = basic_search_options("Rust");
    let result = search(&index, &options).unwrap();
    assert!(!result.results.is_empty(), "英文搜索应该有结果");
}

/// 测试多语言混合
#[test]
fn test_multi_language_mix() {
    let mut index = create_empty_index();

    index.add(1, "Hello 世界 Bonjour мир", false).unwrap();
    index.add(2, "Rust ラスト Rust", false).unwrap();

    let options = basic_search_options("世界");
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "Expected non-empty results");
}

/// 测试 Emoji
#[test]
fn test_emoji() {
    let mut index = create_empty_index();

    index.add(1, "Hello 🎉 World 🚀", false).unwrap();
    index.add(2, "Rust 🦀 is awesome", false).unwrap();

    // 搜索 emoji（取决于分词器实现）
    let options = basic_search_options("🎉");
    let _result = search(&index, &options);

    // 搜索普通文本
    let options = basic_search_options("Hello");
    let result = search(&index, &options).unwrap();
    assert!(!result.results.is_empty(), "Expected non-empty results");
}

/// 测试代码片段
#[test]
fn test_code_snippets() {
    let mut index = create_empty_index();

    index.add(1, "fn main() { println!(\"Hello\"); }", false).unwrap();
    index.add(2, "const x = 42; // comment", false).unwrap();
    index.add(3, "class MyClass { constructor() {} }", false).unwrap();

    let options = basic_search_options("println");
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "Expected non-empty results");
}

/// 测试 URL
#[test]
fn test_urls() {
    let mut index = create_empty_index();

    index.add(1, "Visit https://www.rust-lang.org for more info", false).unwrap();
    index.add(2, "Check out http://example.com/path?query=1", false).unwrap();

    let options = basic_search_options("rust-lang");
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "Expected non-empty results");
}

/// 测试电子邮件地址
#[test]
fn test_email_addresses() {
    let mut index = create_empty_index();

    index.add(1, "Contact us at support@example.com", false).unwrap();
    index.add(2, "Email: user.name@company.co.uk", false).unwrap();

    let options = basic_search_options("support");
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "Expected non-empty results");
}

/// 测试特殊符号
#[test]
fn test_special_symbols() {
    let mut index = create_empty_index();

    index.add(1, "Price: $100.00 (50% off)", false).unwrap();
    index.add(2, "Temperature: -5°C to +30°C", false).unwrap();
    index.add(3, "Equation: E = mc²", false).unwrap();

    let options = basic_search_options("Price");
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "Expected non-empty results");
}

/// 测试 HTML 标签
#[test]
fn test_html_tags() {
    let mut index = create_empty_index();

    index.add(1, "<div>Hello World</div>", false).unwrap();
    index.add(2, "<p>This is a paragraph</p>", false).unwrap();

    let options = basic_search_options("Hello");
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "Expected non-empty results");
}

/// 测试 Markdown
#[test]
fn test_markdown() {
    let mut index = create_empty_index();

    index.add(1, "# Heading\n\nThis is **bold** and *italic*", false).unwrap();
    index.add(2, "- List item 1\n- List item 2", false).unwrap();

    let options = basic_search_options("Heading");
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "Expected non-empty results");
}

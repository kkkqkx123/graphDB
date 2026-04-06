//! 基本高亮功能测试
//!
//! 测试范围：
//! - 单文档高亮
//! - 多关键词高亮
//! - 自定义模板
//! - 大小写处理

use inversearch_service::highlight::highlight_single_document;
use inversearch_service::highlight::types::{
    HighlightConfig, HighlightOptions, HighlightBoundaryOptions,
};
use inversearch_service::encoder::Encoder;
use inversearch_service::r#type::EncoderOptions;

fn create_test_encoder() -> Encoder {
    Encoder::new(EncoderOptions::default()).expect("创建编码器失败")
}

fn create_default_highlight_config() -> HighlightConfig {
    HighlightConfig {
        template: "<mark>$1</mark>".to_string(),
        markup_open: "<mark>".to_string(),
        markup_close: "</mark>".to_string(),
        boundary: None,
        clip: true,
        merge: None,
        ellipsis: "...".to_string(),
        ellipsis_markup_length: 0,
    }
}

/// 测试基本高亮功能
#[test]
fn test_basic_highlight() {
    let encoder = create_test_encoder();
    let config = create_default_highlight_config();

    let content = "Rust is a systems programming language";
    let result = highlight_single_document("Rust", content, &encoder, &config).unwrap();

    assert!(result.contains("<mark>"), "结果应该包含高亮标记");
    assert!(result.contains("Rust"), "结果应该包含关键词");
}

/// 测试多关键词高亮
#[test]
fn test_multi_term_highlight() {
    let encoder = create_test_encoder();
    let config = create_default_highlight_config();

    let content = "Rust is a systems programming language focused on safety";
    let result = highlight_single_document("Rust programming", content, &encoder, &config).unwrap();

    assert!(result.contains("<mark>"), "结果应该包含高亮标记");
}

/// 测试自定义高亮模板
#[test]
fn test_custom_template() {
    let encoder = create_test_encoder();
    let config = HighlightConfig {
        template: "**$1**".to_string(),
        markup_open: "**".to_string(),
        markup_close: "**".to_string(),
        boundary: None,
        clip: true,
        merge: None,
        ellipsis: "...".to_string(),
        ellipsis_markup_length: 0,
    };

    let content = "Hello World";
    let result = highlight_single_document("Hello", content, &encoder, &config).unwrap();

    assert!(result.contains("**"), "结果应该包含自定义标记");
}

/// 测试大小写不敏感高亮
#[test]
fn test_case_insensitive_highlight() {
    let encoder = create_test_encoder();
    let config = create_default_highlight_config();

    let content = "RUST is great. rust is fast. Rust is safe.";
    let result = highlight_single_document("rust", content, &encoder, &config).unwrap();

    assert!(result.contains("<mark>"), "结果应该包含高亮标记");
}

/// 测试无匹配时的高亮
#[test]
fn test_no_match_highlight() {
    let encoder = create_test_encoder();
    let config = create_default_highlight_config();

    let content = "Hello World";
    let result = highlight_single_document("xyz", content, &encoder, &config).unwrap();

    assert!(!result.contains("<mark>"), "无匹配时不应包含高亮标记");
    assert!(result.contains("Hello"), "结果应该包含原始内容");
}

/// 测试空内容高亮
#[test]
fn test_empty_content_highlight() {
    let encoder = create_test_encoder();
    let config = create_default_highlight_config();

    let result = highlight_single_document("test", "", &encoder, &config).unwrap();
    assert!(result.is_empty(), "空内容应该返回空结果");
}

/// 测试中文高亮
#[test]
fn test_chinese_highlight() {
    let encoder = create_test_encoder();
    let config = create_default_highlight_config();

    let content = "Rust是一种系统编程语言";
    let result = highlight_single_document("Rust", content, &encoder, &config).unwrap();

    assert!(result.contains("<mark>"), "中文内容高亮应该工作");
}

/// 测试长文本高亮
#[test]
fn test_long_text_highlight() {
    let encoder = create_test_encoder();
    let config = create_default_highlight_config();

    let content = "This is a very long text that contains the keyword Rust somewhere in the middle of the content. We want to make sure that highlighting works correctly even for long texts.";
    let result = highlight_single_document("Rust", content, &encoder, &config).unwrap();

    assert!(result.contains("<mark>"), "长文本高亮应该工作");
    assert!(result.contains("Rust"), "结果应该包含关键词");
}

/// 测试特殊字符处理
#[test]
fn test_special_characters() {
    let encoder = create_test_encoder();
    let config = create_default_highlight_config();

    let content = "Price: $100 (50% off)";
    let result = highlight_single_document("Price", content, &encoder, &config).unwrap();

    assert!(result.contains("<mark>"), "特殊字符内容高亮应该工作");
}

/// 测试从 HighlightOptions 创建配置
#[test]
fn test_config_from_options() {
    let options = HighlightOptions {
        template: "<em>$1</em>".to_string(),
        boundary: None,
        clip: Some(true),
        merge: None,
        ellipsis: None,
    };

    let config = HighlightConfig::from_options(&options).unwrap();

    assert_eq!(config.markup_open, "<em>");
    assert_eq!(config.markup_close, "</em>");
}

/// 测试带边界配置的高亮
#[test]
fn test_highlight_with_boundary_config() {
    let encoder = create_test_encoder();
    let config = HighlightConfig {
        template: "<mark>$1</mark>".to_string(),
        markup_open: "<mark>".to_string(),
        markup_close: "</mark>".to_string(),
        boundary: Some(HighlightBoundaryOptions {
            before: Some(10),
            after: Some(10),
            total: Some(100),
        }),
        clip: true,
        merge: None,
        ellipsis: "...".to_string(),
        ellipsis_markup_length: 0,
    };

    let content = "This is a long text with Rust programming language in the middle";
    let result = highlight_single_document("Rust", content, &encoder, &config).unwrap();

    assert!(result.contains("<mark>"), "带边界配置的高亮应该工作");
}

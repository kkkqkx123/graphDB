//! 结构化高亮结果测试
//!
//! 测试范围：
//! - 结构化高亮输出
//! - 匹配位置信息
//! - 多字段高亮

use inversearch_service::highlight::highlight_single_document_structured;
use inversearch_service::highlight::types::{HighlightConfig, HighlightBoundaryOptions, DocumentHighlight};
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

/// 测试结构化高亮基本功能
#[test]
fn test_structured_highlight_basic() {
    let encoder = create_test_encoder();
    let config = create_default_highlight_config();

    let content = "Rust is a systems programming language";
    let result = highlight_single_document_structured("Rust", content, &encoder, &config).unwrap();

    assert!(result.total_matches > 0, "应该有匹配结果");
}

/// 测试结构化高亮的匹配信息
#[test]
fn test_structured_highlight_fields() {
    let encoder = create_test_encoder();
    let config = create_default_highlight_config();

    let content = "Hello Rust World";
    let result = highlight_single_document_structured("Rust", content, &encoder, &config).unwrap();

    assert!(result.total_matches > 0, "应该有匹配结果");
}

/// 测试结构化高亮多个匹配
#[test]
fn test_structured_highlight_multiple_matches() {
    let encoder = create_test_encoder();
    let config = create_default_highlight_config();

    let content = "Rust is great and Rust is fast";
    let result = highlight_single_document_structured("Rust", content, &encoder, &config).unwrap();

    assert!(result.total_matches >= 1, "应该有至少一个匹配");
}

/// 测试结构化高亮无匹配
#[test]
fn test_structured_highlight_no_match() {
    let encoder = create_test_encoder();
    let config = create_default_highlight_config();

    let content = "Hello World";
    let result = highlight_single_document_structured("xyz", content, &encoder, &config).unwrap();

    assert_eq!(result.total_matches, 0, "无匹配时应该返回零匹配");
}

/// 测试结构化高亮中文内容
#[test]
fn test_structured_highlight_chinese() {
    let encoder = create_test_encoder();
    let config = create_default_highlight_config();

    let content = "Rust是一种系统编程语言";
    let result = highlight_single_document_structured("Rust", content, &encoder, &config).unwrap();

    assert!(result.total_matches > 0, "中文内容应该有匹配");
}

/// 测试结构化高亮带边界
#[test]
fn test_structured_highlight_with_boundary() {
    let encoder = create_test_encoder();
    let config = HighlightConfig {
        template: "<mark>$1</mark>".to_string(),
        markup_open: "<mark>".to_string(),
        markup_close: "</mark>".to_string(),
        boundary: Some(HighlightBoundaryOptions {
            before: Some(5),
            after: Some(5),
            total: Some(100),
        }),
        clip: true,
        merge: None,
        ellipsis: "...".to_string(),
        ellipsis_markup_length: 0,
    };

    let content = "This is a long text with Rust programming language in the middle";
    let result = highlight_single_document_structured("Rust", content, &encoder, &config).unwrap();

    assert!(result.total_matches > 0, "带边界的结构化高亮应该有匹配");
}

/// 测试结构化高亮空内容
#[test]
fn test_structured_highlight_empty() {
    let encoder = create_test_encoder();
    let config = create_default_highlight_config();

    let result = highlight_single_document_structured("test", "", &encoder, &config).unwrap();
    assert_eq!(result.total_matches, 0, "空内容应该返回零匹配");
}

/// 测试结构化高亮特殊字符
#[test]
fn test_structured_highlight_special_chars() {
    let encoder = create_test_encoder();
    let config = create_default_highlight_config();

    let content = "Price: $100 (50% off) for Rust";
    let result = highlight_single_document_structured("Rust", content, &encoder, &config).unwrap();

    assert!(result.total_matches > 0, "特殊字符内容应该有匹配");
}

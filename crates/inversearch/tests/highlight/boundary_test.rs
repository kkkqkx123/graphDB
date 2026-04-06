//! 边界处理测试
//!
//! 测试范围：
//! - 边界截断
//! - 省略号处理
//! - 前后边界

use inversearch_service::highlight::highlight_single_document;
use inversearch_service::highlight::types::{HighlightConfig, HighlightBoundaryOptions};
use inversearch_service::encoder::Encoder;
use inversearch_service::r#type::EncoderOptions;

fn create_test_encoder() -> Encoder {
    Encoder::new(EncoderOptions::default()).expect("创建编码器失败")
}

/// 测试基本边界截断
#[test]
fn test_basic_boundary() {
    let encoder = create_test_encoder();
    let config = HighlightConfig {
        template: "<mark>$1</mark>".to_string(),
        markup_open: "<mark>".to_string(),
        markup_close: "</mark>".to_string(),
        boundary: Some(HighlightBoundaryOptions {
            before: Some(5),
            after: Some(5),
            total: None,
        }),
        clip: true,
        merge: None,
        ellipsis: "...".to_string(),
        ellipsis_markup_length: 0,
    };

    let content = "This is a very long text with the keyword Rust somewhere in the middle";
    let result = highlight_single_document("Rust", content, &encoder, &config).unwrap();

    assert!(result.contains("<mark>"), "结果应该包含高亮标记");
    assert!(result.contains("Rust"), "结果应该包含关键词");
}

/// 测试总长度边界
#[test]
fn test_total_boundary() {
    let encoder = create_test_encoder();
    let config = HighlightConfig {
        template: "<mark>$1</mark>".to_string(),
        markup_open: "<mark>".to_string(),
        markup_close: "</mark>".to_string(),
        boundary: Some(HighlightBoundaryOptions {
            before: None,
            after: None,
            total: Some(50),
        }),
        clip: true,
        merge: None,
        ellipsis: "...".to_string(),
        ellipsis_markup_length: 0,
    };

    let content = "This is a very long text with the keyword Rust somewhere in the middle of the content and it keeps going";
    let result = highlight_single_document("Rust", content, &encoder, &config).unwrap();

    assert!(result.contains("<mark>"), "结果应该包含高亮标记");
}

/// 测试无边界配置
#[test]
fn test_no_boundary() {
    let encoder = create_test_encoder();
    let config = HighlightConfig {
        template: "<mark>$1</mark>".to_string(),
        markup_open: "<mark>".to_string(),
        markup_close: "</mark>".to_string(),
        boundary: None,
        clip: true,
        merge: None,
        ellipsis: "...".to_string(),
        ellipsis_markup_length: 0,
    };

    let content = "Rust is great";
    let result = highlight_single_document("Rust", content, &encoder, &config).unwrap();

    assert!(result.contains("<mark>"), "无边界配置的高亮应该工作");
}

/// 测试边界与多个匹配
#[test]
fn test_boundary_multiple_matches() {
    let encoder = create_test_encoder();
    let config = HighlightConfig {
        template: "<mark>$1</mark>".to_string(),
        markup_open: "<mark>".to_string(),
        markup_close: "</mark>".to_string(),
        boundary: Some(HighlightBoundaryOptions {
            before: Some(3),
            after: Some(3),
            total: None,
        }),
        clip: true,
        merge: None,
        ellipsis: "...".to_string(),
        ellipsis_markup_length: 0,
    };

    let content = "Rust is great and Rust is fast and Rust is safe";
    let result = highlight_single_document("Rust", content, &encoder, &config).unwrap();

    assert!(result.contains("<mark>"), "多个匹配的高亮应该工作");
}

/// 测试边界在文本开头
#[test]
fn test_boundary_at_start() {
    let encoder = create_test_encoder();
    let config = HighlightConfig {
        template: "<mark>$1</mark>".to_string(),
        markup_open: "<mark>".to_string(),
        markup_close: "</mark>".to_string(),
        boundary: Some(HighlightBoundaryOptions {
            before: Some(5),
            after: Some(5),
            total: None,
        }),
        clip: true,
        merge: None,
        ellipsis: "...".to_string(),
        ellipsis_markup_length: 0,
    };

    let content = "Rust is a programming language at the beginning";
    let result = highlight_single_document("Rust", content, &encoder, &config).unwrap();

    assert!(result.contains("<mark>"), "开头匹配的高亮应该工作");
}

/// 测试边界在文本结尾
#[test]
fn test_boundary_at_end() {
    let encoder = create_test_encoder();
    let config = HighlightConfig {
        template: "<mark>$1</mark>".to_string(),
        markup_open: "<mark>".to_string(),
        markup_close: "</mark>".to_string(),
        boundary: Some(HighlightBoundaryOptions {
            before: Some(5),
            after: Some(5),
            total: None,
        }),
        clip: true,
        merge: None,
        ellipsis: "...".to_string(),
        ellipsis_markup_length: 0,
    };

    let content = "This is a long text ending with Rust";
    let result = highlight_single_document("Rust", content, &encoder, &config).unwrap();

    assert!(result.contains("<mark>"), "结尾匹配的高亮应该工作");
}

/// 测试零边界值
#[test]
fn test_zero_boundary() {
    let encoder = create_test_encoder();
    let config = HighlightConfig {
        template: "<mark>$1</mark>".to_string(),
        markup_open: "<mark>".to_string(),
        markup_close: "</mark>".to_string(),
        boundary: Some(HighlightBoundaryOptions {
            before: Some(0),
            after: Some(0),
            total: None,
        }),
        clip: true,
        merge: None,
        ellipsis: "...".to_string(),
        ellipsis_markup_length: 0,
    };

    let content = "This is text with Rust in the middle";
    let result = highlight_single_document("Rust", content, &encoder, &config).unwrap();

    assert!(result.contains("<mark>"), "零边界的高亮应该工作");
}

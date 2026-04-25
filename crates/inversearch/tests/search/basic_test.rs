//! 基本搜索功能测试
//!
//! 测试范围：
//! - 单关键词搜索
//! - 多关键词搜索
//! - 搜索结果验证

use inversearch_service::search::search;

use crate::common::{
    basic_search_options, create_empty_index, create_english_index, create_full_index,
    PROGRAMMING_DOCS,
};

/// 测试基本搜索功能
/// 验证：添加文档后，可以正确搜索到
#[test]
fn test_basic_search() {
    let mut index = create_empty_index();

    // 添加测试文档
    index
        .add(1, "Rust is a systems programming language", false)
        .unwrap();
    index
        .add(2, "Python is great for data science", false)
        .unwrap();
    index
        .add(3, "JavaScript runs in the browser", false)
        .unwrap();

    // 执行搜索
    let options = basic_search_options("Rust");
    let result = search(&index, &options).unwrap();

    // 验证结果
    assert!(!result.results.is_empty(), "Expected non-empty results");
    assert!(
        result.results.contains(&1),
        "Expected results to contain document 1"
    );
    assert!(!result.results.contains(&2));
    assert!(!result.results.contains(&3));
}

/// 测试多词搜索
/// 验证：多个关键词的搜索
#[test]
fn test_multi_term_search() {
    let index = create_english_index();

    // 搜索 "programming language"
    let options = basic_search_options("programming language");
    let result = search(&index, &options).unwrap();

    // 验证结果包含多个文档
    assert!(!result.results.is_empty(), "Expected non-empty results");
    assert!(result.total >= 2, "多词搜索应该返回多个结果");
}

/// 测试搜索结果包含正确的文档
#[test]
fn test_search_results_accuracy() {
    let index = create_english_index();

    // 搜索 "Rust"
    let options = basic_search_options("Rust");
    let result = search(&index, &options).unwrap();

    // 验证只有包含 Rust 的文档被返回
    assert!(
        result.results.contains(&1),
        "Expected results to contain document 1"
    );

    // 验证其他文档不在结果中
    for doc in PROGRAMMING_DOCS.iter().filter(|d| d.id != 1) {
        if !doc.content.contains("Rust") {
            assert!(
                !result.results.contains(&doc.id),
                "文档 {} 不应该出现在 Rust 搜索结果中",
                doc.id
            );
        }
    }
}

/// 测试中文搜索
#[test]
fn test_chinese_search() {
    let index = create_full_index();

    // 搜索中文关键词
    let options = basic_search_options("编程");
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "Expected non-empty results");
    // 应该找到中文文档
    assert!(result.results.contains(&100) || result.results.contains(&101));
}

/// 测试日文搜索
#[test]
fn test_japanese_search() {
    let index = create_full_index();

    let options = basic_search_options("プログラミング");
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "Expected non-empty results");
}

/// 测试韩文搜索
#[test]
fn test_korean_search() {
    let index = create_full_index();

    let options = basic_search_options("프로그래밍");
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "Expected non-empty results");
}

/// 测试混合语言搜索
#[test]
fn test_mixed_language_search() {
    let index = create_full_index();

    // 搜索英文
    let options = basic_search_options("Rust");
    let result = search(&index, &options).unwrap();
    assert!(!result.results.is_empty(), "Expected non-empty results");

    // 搜索中文
    let options = basic_search_options("语言");
    let result = search(&index, &options).unwrap();
    assert!(!result.results.is_empty(), "Expected non-empty results");
}

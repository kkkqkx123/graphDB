//! 清空索引测试
//!
//! 测试范围：
//! - 清空索引
//! - 清空后重新添加

use inversearch_service::search::search;

use crate::common::{basic_search_options, create_empty_index};

/// 测试清空索引
#[test]
fn test_clear_index() {
    let mut index = create_empty_index();

    // 添加文档
    for i in 1..=10 {
        index.add(i, &format!("Document {}", i), false).unwrap();
    }

    // 验证文档存在
    let options = basic_search_options("Document");
    let result = search(&index, &options).unwrap();
    assert!(!result.results.is_empty(), "Expected non-empty results");

    // 清空索引
    index.clear();

    // 验证所有文档都被删除
    for i in 1..=10 {
        assert!(!index.contains(i), "文档 {} 应该已被删除", i);
    }

    // 验证搜索不到
    let options = basic_search_options("Document");
    let result = search(&index, &options).unwrap();
    assert!(
        result.results.is_empty(),
        "Expected empty results, got {:?}",
        result.results
    );
}

/// 测试清空后重新添加文档
#[test]
fn test_clear_and_readd() {
    let mut index = create_empty_index();

    // 添加初始文档
    for i in 1..=5 {
        index.add(i, &format!("Original {}", i), false).unwrap();
    }

    // 清空
    index.clear();

    // 重新添加文档
    for i in 1..=5 {
        index.add(i + 100, &format!("New {}", i), false).unwrap();
    }

    // 验证新文档可以搜索到
    let options = basic_search_options("New");
    let result = search(&index, &options).unwrap();
    assert!(!result.results.is_empty(), "Expected non-empty results");

    // 验证旧文档搜索不到
    let options = basic_search_options("Original");
    let result = search(&index, &options).unwrap();
    assert!(
        result.results.is_empty(),
        "Expected empty results, got {:?}",
        result.results
    );
}

/// 测试空索引清空
#[test]
fn test_clear_empty_index() {
    let mut index = create_empty_index();

    // 清空空索引不应该出错
    index.clear();

    // 验证索引仍然为空
    let options = basic_search_options("test");
    let result = search(&index, &options).unwrap();
    assert!(
        result.results.is_empty(),
        "Expected empty results, got {:?}",
        result.results
    );
}

/// 测试多次清空
#[test]
fn test_multiple_clears() {
    let mut index = create_empty_index();

    // 第一次添加和清空
    index.add(1, "First", false).unwrap();
    index.clear();

    // 第二次添加和清空
    index.add(2, "Second", false).unwrap();
    index.clear();

    // 第三次添加和清空
    index.add(3, "Third", false).unwrap();
    index.clear();

    // 验证索引为空
    assert!(!index.contains(1));
    assert!(!index.contains(2));
    assert!(!index.contains(3));
}

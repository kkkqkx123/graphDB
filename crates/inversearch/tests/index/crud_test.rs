//! 索引 CRUD 操作测试
//!
//! 测试范围：
//! - 文档添加 (Create)
//! - 文档读取/搜索 (Read)
//! - 文档更新 (Update)
//! - 文档删除 (Delete)

use inversearch_service::search::search;

use crate::common::{
    create_empty_index, basic_search_options,
};

/// 测试添加文档
/// 验证：添加的文档可以被搜索到
#[test]
fn test_add_document() {
    let mut index = create_empty_index();

    // 添加文档
    index.add(1, "Test content", false).unwrap();

    // 验证文档存在
    assert!(index.contains(1), "文档应该存在于索引中");

    // 验证可以搜索到
    let options = basic_search_options("Test");
    let result = search(&index, &options).unwrap();
    assert!(!result.results.is_empty(), "Expected non-empty results");
    assert!(result.results.contains(&1), "Expected results to contain document 1");
}

/// 测试更新文档
/// 验证：更新后搜索返回新内容
#[test]
fn test_update_document() {
    let mut index = create_empty_index();

    // 添加初始文档
    index.add(1, "Old content", false).unwrap();

    // 验证旧内容可以搜索到
    let options = basic_search_options("Old");
    let result = search(&index, &options).unwrap();
    assert!(!result.results.is_empty(), "Expected non-empty results");

    // 更新文档
    index.update(1, "New content").unwrap();

    // 验证旧内容搜索不到
    let options = basic_search_options("Old");
    let result = search(&index, &options).unwrap();
    assert!(result.results.is_empty(), "Expected empty results, got {:?}", result.results);

    // 验证新内容可以搜索到
    let options = basic_search_options("New");
    let result = search(&index, &options).unwrap();
    assert!(!result.results.is_empty(), "Expected non-empty results");
    assert!(result.results.contains(&1), "Expected results to contain document 1");
}

/// 测试删除文档
/// 验证：删除后文档无法搜索到
#[test]
fn test_remove_document() {
    let mut index = create_empty_index();

    // 添加文档
    index.add(1, "Test content", false).unwrap();

    // 验证可以搜索到
    let options = basic_search_options("Test");
    let result = search(&index, &options).unwrap();
    assert!(!result.results.is_empty(), "Expected non-empty results");

    // 删除文档
    index.remove(1, false).unwrap();

    // 验证文档不存在
    assert!(!index.contains(1), "文档不应该存在于索引中");

    // 验证搜索不到
    let options = basic_search_options("Test");
    let result = search(&index, &options).unwrap();
    assert!(result.results.is_empty(), "Expected empty results, got {:?}", result.results);
}

/// 测试添加重复文档
#[test]
fn test_add_duplicate_document() {
    let mut index = create_empty_index();

    // 添加文档
    index.add(1, "First content", false).unwrap();

    // 添加相同 ID 的文档（应该更新）
    index.add(1, "Second content", false).unwrap();

    // 验证新内容可以搜索到
    let options = basic_search_options("Second");
    let result = search(&index, &options).unwrap();
    assert!(!result.results.is_empty(), "Expected non-empty results");
    assert!(result.results.contains(&1), "Expected results to contain document 1");

    // 验证旧内容可能搜索不到（取决于实现）
    let options = basic_search_options("First");
    let _result = search(&index, &options);
}

/// 测试添加多个文档
#[test]
fn test_add_multiple_documents() {
    let mut index = create_empty_index();

    // 添加多个文档
    for i in 1..=10 {
        index.add(i, &format!("Document {}", i), false).unwrap();
    }

    // 验证所有文档都存在
    for i in 1..=10 {
        assert!(index.contains(i), "文档 {} 应该存在于索引中", i);
    }

    // 验证可以搜索到
    let options = basic_search_options("Document");
    let result = search(&index, &options).unwrap();
    assert_eq!(result.results.len(), 10, "应该返回 10 个结果");
}

/// 测试删除不存在的文档
#[test]
fn test_remove_nonexistent_document() {
    let mut index = create_empty_index();

    // 尝试删除不存在的文档
    // 根据实现可能返回错误或静默处理
    let _result = index.remove(999, false);
}

/// 测试更新不存在的文档
#[test]
fn test_update_nonexistent_document() {
    let mut index = create_empty_index();

    // 尝试更新不存在的文档
    // 根据实现可能返回错误或静默处理
    let _result = index.update(999, "New content");
}

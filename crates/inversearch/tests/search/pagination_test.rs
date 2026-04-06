//! 分页功能测试
//!
//! 测试范围：
//! - 基本分页
//! - 边界情况
//! - 大偏移量

use inversearch_service::search::search;

use crate::common::{
    create_english_index, create_empty_index,
    paginated_search_options,
};

/// 测试基本分页功能
#[test]
fn test_basic_pagination() {
    let index = create_english_index();

    // 第一页，每页 3 条
    let options = paginated_search_options("programming", 3, 0);
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "Expected non-empty results");
    assert_eq!(result.results.len(), 3, "第一页应该返回 3 条结果");

    // 第二页
    let options = paginated_search_options("programming", 3, 3);
    let result = search(&index, &options).unwrap();

    assert!(!result.results.is_empty(), "第二页也应该有结果");
}

/// 测试偏移量超过总数的情况
#[test]
fn test_offset_beyond_total() {
    let index = create_english_index();

    // 偏移量设置得很大
    let options = paginated_search_options("programming", 10, 1000);
    let result = search(&index, &options).unwrap();

    // 应该返回空结果，但不报错
    assert!(result.results.is_empty(), "Expected empty results, got {:?}", result.results);
}

/// 测试 limit 为 0 的情况
#[test]
fn test_zero_limit() {
    let index = create_english_index();

    let options = paginated_search_options("programming", 0, 0);
    let result = search(&index, &options).unwrap();

    // limit 为 0 应该返回空结果
    assert!(result.results.is_empty(), "Expected empty results, got {:?}", result.results);
}

/// 测试分页结果总数
#[test]
fn test_pagination_total() {
    let mut index = create_empty_index();

    // 添加 10 个文档
    for i in 1..=10 {
        index.add(i, &format!("Document number {} about programming", i), false).unwrap();
    }

    // 搜索所有文档
    let options = paginated_search_options("programming", 100, 0);
    let result = search(&index, &options).unwrap();

    // 验证总数
    assert_eq!(result.total, 10, "总数应该为 10");
}

/// 测试分页结果不重复
#[test]
fn test_pagination_no_duplicates() {
    let index = create_english_index();

    // 获取第一页
    let options = paginated_search_options("programming", 2, 0);
    let result1 = search(&index, &options).unwrap();

    // 获取第二页
    let options = paginated_search_options("programming", 2, 2);
    let result2 = search(&index, &options).unwrap();

    // 验证两页结果没有重复
    for doc_id in &result1.results {
        assert!(!result2.results.contains(doc_id), 
            "文档 {} 不应该同时出现在两页中", doc_id);
    }
}

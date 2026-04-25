//! 测试辅助函数
//!
//! 提供测试所需的辅助函数和宏

#![allow(dead_code)]

use inversearch_service::index::IndexOptions;
use inversearch_service::{Index, SearchOptions};

/// 创建测试索引并填充数据
pub fn create_index_with_docs(docs: &[super::documents::TestDocument]) -> Index {
    let mut index = Index::new(IndexOptions::default()).unwrap();

    for doc in docs {
        index.add(doc.id, doc.content, false).unwrap();
    }

    index
}

/// 创建空的测试索引
pub fn create_empty_index() -> Index {
    Index::new(IndexOptions::default()).unwrap()
}

/// 创建英文测试索引
pub fn create_english_index() -> Index {
    create_index_with_docs(super::documents::PROGRAMMING_DOCS)
}

/// 创建完整测试索引（包含所有文档）
pub fn create_full_index() -> Index {
    let all_docs: Vec<_> = super::documents::PROGRAMMING_DOCS
        .iter()
        .chain(super::documents::CHINESE_DOCS)
        .chain(super::documents::JAPANESE_DOCS)
        .chain(super::documents::KOREAN_DOCS)
        .chain(super::documents::MIXED_LANG_DOCS)
        .copied()
        .collect();
    create_index_with_docs(&all_docs)
}

/// 创建基本搜索选项
pub fn basic_search_options(query: &str) -> SearchOptions {
    SearchOptions {
        query: Some(query.to_string()),
        limit: Some(100),
        offset: Some(0),
        resolution: None,
        context: Some(false),
        suggest: Some(false),
        resolve: Some(true),
        enrich: Some(false),
        cache: Some(false),
        tag: None,
        field: None,
        pluck: None,
        merge: Some(false),
        boost: None,
    }
}

/// 创建分页搜索选项
pub fn paginated_search_options(query: &str, limit: usize, offset: usize) -> SearchOptions {
    SearchOptions {
        query: Some(query.to_string()),
        limit: Some(limit),
        offset: Some(offset),
        resolution: None,
        context: Some(false),
        suggest: Some(false),
        resolve: Some(true),
        enrich: Some(false),
        cache: Some(false),
        tag: None,
        field: None,
        pluck: None,
        merge: Some(false),
        boost: None,
    }
}

// ============================================================================
// 断言宏
// ============================================================================

/// 断言搜索结果包含指定文档
#[macro_export]
macro_rules! assert_contains_doc {
    ($results:expr, $doc_id:expr) => {
        assert!(
            $results.contains(&$doc_id),
            "Expected results to contain document {}",
            $doc_id
        );
    };
}

/// 断言搜索结果不包含指定文档
#[macro_export]
macro_rules! assert_not_contains_doc {
    ($results:expr, $doc_id:expr) => {
        assert!(
            !$results.contains(&$doc_id),
            "Expected results NOT to contain document {}",
            $doc_id
        );
    };
}

/// 断言搜索结果包含所有指定文档
#[macro_export]
macro_rules! assert_contains_all {
    ($results:expr, $doc_ids:expr) => {
        for doc_id in $doc_ids {
            assert_contains_doc!($results, doc_id);
        }
    };
}

/// 断言搜索结果数量
#[macro_export]
macro_rules! assert_result_count {
    ($results:expr, $expected:expr) => {
        assert_eq!(
            $results.len(),
            $expected,
            "Expected {} results, got {}",
            $expected,
            $results.len()
        );
    };
}

/// 断言搜索结果为空
#[macro_export]
macro_rules! assert_empty_results {
    ($results:expr) => {
        assert!(
            $results.is_empty(),
            "Expected empty results, got {:?}",
            $results
        );
    };
}

/// 断言搜索结果非空
#[macro_export]
macro_rules! assert_not_empty_results {
    ($results:expr) => {
        assert!(!$results.is_empty(), "Expected non-empty results");
    };
}

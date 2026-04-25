//! 结果丰富化测试
//!
//! 测试范围：
//! - Enricher 功能
//! - 结果合并
//! - 元数据处理

use inversearch_service::r#type::IntermediateSearchResults;
use inversearch_service::resolver::{
    combine_search_results, resolve_default, Resolver, ResolverOptions,
};

/// 测试 Resolver 创建
#[test]
fn test_resolver_new() {
    let result: IntermediateSearchResults = vec![vec![1, 2, 3]];
    let resolver = Resolver::new(result, None);

    assert_eq!(resolver.result.len(), 1);
}

/// 测试 Resolver 链式操作
#[test]
fn test_resolver_chain() {
    let result: IntermediateSearchResults = vec![vec![1, 2, 3, 4, 5]];
    let mut resolver = Resolver::new(result, None);
    resolver.limit(3).offset(1).boost(5);

    assert_eq!(resolver.boostval, 5);
}

/// 测试 resolve_default 函数
#[test]
fn test_resolve_default_basic() {
    let result: IntermediateSearchResults = vec![vec![1, 2, 3, 4, 5]];
    let resolved = resolve_default(&result, 3, 0, false);

    assert_eq!(resolved, vec![1, 2, 3]);
}

/// 测试 resolve_default 带偏移
#[test]
fn test_resolve_default_with_offset() {
    let result: IntermediateSearchResults = vec![vec![1, 2, 3, 4, 5]];
    let resolved = resolve_default(&result, 2, 2, false);

    assert_eq!(resolved, vec![3, 4]);
}

/// 测试 resolve_default 空结果
#[test]
fn test_resolve_default_empty() {
    let result: IntermediateSearchResults = vec![];
    let resolved = resolve_default(&result, 10, 0, false);

    assert!(resolved.is_empty());
}

/// 测试 resolve_default 偏移超出范围
#[test]
fn test_resolve_default_offset_beyond() {
    let result: IntermediateSearchResults = vec![vec![1, 2, 3]];
    let resolved = resolve_default(&result, 10, 10, false);

    assert!(resolved.is_empty());
}

/// 测试 combine_search_results 基本功能
#[test]
fn test_combine_search_results_basic() {
    let results1: IntermediateSearchResults = vec![vec![1, 2, 3]];
    let results2: IntermediateSearchResults = vec![vec![3, 4, 5]];

    let combined = combine_search_results(vec![results1, results2]);

    assert!(!combined.is_empty());
}

/// 测试 combine_search_results 空输入
#[test]
fn test_combine_search_results_empty() {
    let combined = combine_search_results(vec![]);
    assert!(combined.is_empty());
}

/// 测试 Resolver AND 操作
#[test]
fn test_resolver_and() {
    let result: IntermediateSearchResults = vec![vec![1, 2, 3]];
    let other: IntermediateSearchResults = vec![vec![2, 3, 4]];

    let resolver = Resolver::new(result, None);
    let mut resolver = resolver;
    resolver.and(other);
    let resolved = resolver.get();

    assert!(!resolved.is_empty());
}

/// 测试 Resolver OR 操作
#[test]
fn test_resolver_or() {
    let result: IntermediateSearchResults = vec![vec![1, 2]];
    let other: IntermediateSearchResults = vec![vec![3, 4]];

    let resolver = Resolver::new(result, None);
    let mut resolver = resolver;
    resolver.or(other);
    let resolved = resolver.get();

    assert!(!resolved.is_empty());
}

/// 测试 Resolver NOT 操作
#[test]
fn test_resolver_not() {
    let result: IntermediateSearchResults = vec![vec![1, 2, 3, 4, 5]];
    let exclude: IntermediateSearchResults = vec![vec![2, 3]];

    let resolver = Resolver::new(result, None);
    let mut resolver = resolver;
    resolver.not(exclude);
    let resolved = resolver.get();

    assert!(!resolved.contains(&2));
    assert!(!resolved.contains(&3));
}

/// 测试 Resolver 限制
#[test]
fn test_resolver_limit() {
    let result: IntermediateSearchResults = vec![vec![1, 2, 3, 4, 5]];
    let mut resolver = Resolver::new(result, None);
    resolver.limit(3);

    let resolved = resolver.get();
    assert_eq!(resolved.len(), 3);
}

/// 测试 Resolver 偏移
#[test]
fn test_resolver_offset() {
    let result: IntermediateSearchResults = vec![vec![1, 2, 3, 4, 5]];
    let mut resolver = Resolver::new(result, None);
    resolver.offset(2);

    let resolved = resolver.get();
    assert_eq!(resolved, vec![3, 4, 5]);
}

/// 测试 Resolver boost
#[test]
fn test_resolver_boost() {
    let result: IntermediateSearchResults = vec![vec![1, 2, 3]];
    let mut resolver = Resolver::new(result, None);
    resolver.boost(10);

    assert_eq!(resolver.boostval, 10);
}

/// 测试 ResolverOptions 默认值
#[test]
fn test_resolver_options_default() {
    let options = ResolverOptions::default();

    assert!(options.limit().is_none() || options.limit() > Some(0));
}

/// 测试多层嵌套结果处理
#[test]
fn test_nested_results() {
    let result: IntermediateSearchResults = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
    let resolved = resolve_default(&result, 5, 0, false);

    assert_eq!(resolved.len(), 5);
}

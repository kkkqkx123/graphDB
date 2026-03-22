//! 搜索模块
//!
//! 提供搜索功能，包括单术语搜索、多术语搜索和多字段搜索协调

mod single_term;
mod cache;
mod coordinator;
mod multi_field;

use crate::r#type::{IntermediateSearchResults, SearchResults, SearchOptions};
use crate::error::Result;
use crate::Index;
pub use single_term::{single_term_query, multi_term_search, SingleTermResult};
pub use cache::{SearchCache, CachedSearch, CacheStats, CacheKeyGenerator};
pub use coordinator::{
    SearchCoordinator,
    MultiFieldSearchOptions,
    CombineStrategy,
};
pub use multi_field::multi_field_search;

/// 搜索结果结构体
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub results: SearchResults,
    pub total: usize,
    pub query: String,
}

/// 执行搜索 - 完整实现
pub fn search(index: &Index, options: &SearchOptions) -> Result<SearchResult> {
    let query = options.query.as_deref().unwrap_or("");
    if query.is_empty() {
        return Ok(SearchResult {
            results: Vec::new(),
            total: 0,
            query: String::new(),
        });
    }

    // 解析查询词
    let encoded_terms = index.encoder.encode(query)?;
    if encoded_terms.is_empty() {
        return Ok(SearchResult {
            results: Vec::new(),
            total: 0,
            query: query.to_string(),
        });
    }

    let limit = options.limit.unwrap_or(100);
    let offset = options.offset.unwrap_or(0);
    let context = options.context;

    // 根据术语数量选择不同的搜索策略
    let results = if encoded_terms.len() == 1 {
        // 单术语快速路径
        let result = single_term_query(
            index,
            &encoded_terms[0],
            None,
            limit,
            offset,
            options.resolve.unwrap_or(true),
            options.context.unwrap_or(false),
            None,
        )?;
        result.results
    } else {
        // 多术语搜索
        let terms: Vec<&str> = encoded_terms.iter().map(|s| s.as_str()).collect();
        multi_term_search(index, terms, options)?
    };

    let total = results.len();

    Ok(SearchResult {
        results,
        total,
        query: query.to_string(),
    })
}

/// 默认解析函数（兼容函数）
pub fn resolve_default_search(
    results: &IntermediateSearchResults,
    limit: usize,
    offset: usize,
) -> Vec<u64> {
    if results.is_empty() {
        return Vec::new();
    }
    
    // 展平结果
    let mut flattened = Vec::new();
    for array in results {
        flattened.extend_from_slice(array);
    }
    
    // 应用限制和偏移
    if offset > 0 {
        if offset >= flattened.len() {
            return Vec::new();
        }
        flattened.drain(0..offset);
    }
    
    if limit > 0 && limit < flattened.len() {
        flattened.truncate(limit);
    }
    
    flattened
}
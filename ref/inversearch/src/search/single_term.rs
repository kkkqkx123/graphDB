//! 单术语搜索模块
//! 
//! 提供单术语和简单查询的搜索功能

use crate::r#type::{SearchResults, SearchOptions};
use crate::error::Result;
use crate::Index;
use std::collections::HashMap;

/// 单术语查询结果
#[derive(Debug, Clone)]
pub struct SingleTermResult {
    pub results: SearchResults,
    pub term: String,
    pub context: Option<String>,
    pub total: usize,
}

/// 执行单术语查询
pub fn single_term_query(
    index: &Index,
    term: &str,
    context: Option<&str>,
    limit: usize,
    offset: usize,
    _resolve: bool,
    _enrich: bool,
    _tag: Option<&str>,
) -> Result<SingleTermResult> {
    if term.is_empty() {
        return Ok(SingleTermResult {
            results: Vec::new(),
            term: term.to_string(),
            context: context.map(|s| s.to_string()),
            total: 0,
        });
    }

    // 获取编码后的术语
    let encoded_term = index.encoder.encode(term)?;
    if encoded_term.is_empty() {
        return Ok(SingleTermResult {
            results: Vec::new(),
            term: term.to_string(),
            context: context.map(|s| s.to_string()),
            total: 0,
        });
    }

    let first_term = &encoded_term[0];
    
    // 根据是否有上下文选择不同的查询方式
    let results = if let Some(ctx) = context {
        // 上下文搜索
        single_context_query(index, first_term, ctx, limit, offset)?
    } else {
        // 普通术语搜索
        single_plain_query(index, first_term, limit, offset)?
    };

    let total = results.len();
    
    Ok(SingleTermResult {
        results,
        term: term.to_string(),
        context: context.map(|s| s.to_string()),
        total,
    })
}

/// 普通术语搜索
fn single_plain_query(
    index: &Index,
    term: &str,
    limit: usize,
    offset: usize,
) -> Result<SearchResults> {
    // 直接从主索引map中获取文档ID列表
    let term_str = term.to_string();
    let doc_ids = if let Some(entries) = index.map.get(&term_str) {
        entries.clone()
    } else {
        Vec::new()
    };

    // 应用限制和偏移
    Ok(apply_limit_offset(&doc_ids, limit, offset))
}

/// 上下文搜索
fn single_context_query(
    index: &Index,
    term: &str,
    context: &str,
    limit: usize,
    offset: usize,
) -> Result<SearchResults> {
    // 首先检查上下文索引
    let context_str = context.to_string();
    if let Some(doc_ids) = index.ctx.get(&context_str) {
        let term_str = term.to_string();
        // 检查term是否在文档ID列表中
        if doc_ids.iter().any(|id| *id == term_str.parse::<u64>().unwrap_or(0)) {
            return Ok(doc_ids.clone());
        }
    }

    // 如果没有找到上下文特定结果，回退到普通搜索
    single_plain_query(index, term, limit, offset)
}

/// 应用限制和偏移
fn apply_limit_offset(results: &[u64], limit: usize, offset: usize) -> SearchResults {
    if results.is_empty() {
        return Vec::new();
    }

    let start = offset.min(results.len());
    let end = if limit > 0 {
        (start + limit).min(results.len())
    } else {
        results.len()
    };

    results[start..end].to_vec()
}

/// 多术语搜索
pub fn multi_term_search(
    index: &Index,
    terms: Vec<&str>,
    options: &SearchOptions,
) -> Result<SearchResults> {
    if terms.is_empty() {
        return Ok(Vec::new());
    }

    let limit = options.limit.unwrap_or(100);
    let offset = options.offset.unwrap_or(0);
    let context = options.context;

    // 收集每个术语的搜索结果
    let mut intermediate_results = Vec::new();
    
    for term in terms {
        let result = single_term_query(
            index,
            term,
            None,
            0, 0, true, false, None
        )?;
        
        if !result.results.is_empty() {
            intermediate_results.push(result.results);
        }
    }

    if intermediate_results.is_empty() {
        return Ok(Vec::new());
    }

    // 执行交集操作
    let intersected = if intermediate_results.len() == 1 {
        intermediate_results.into_iter().next().unwrap()
    } else {
        perform_intersection(&intermediate_results)
    };

    // 应用限制和偏移
    Ok(apply_limit_offset(&intersected, limit, offset))
}

/// 执行交集操作
fn perform_intersection(results: &[SearchResults]) -> SearchResults {
    if results.is_empty() {
        return Vec::new();
    }
    
    if results.len() == 1 {
        return results[0].clone();
    }

    // 找到最小的结果集作为基础
    let mut min_idx = 0;
    let mut min_size = results[0].len();
    
    for (i, result) in results.iter().enumerate().skip(1) {
        if result.len() < min_size {
            min_size = result.len();
            min_idx = i;
        }
    }

    let base = &results[min_idx];
    let mut intersection = Vec::new();

    // 检查基础集中的每个ID是否存在于所有其他结果集中
    'outer: for &doc_id in base {
        for (i, result) in results.iter().enumerate() {
            if i == min_idx {
                continue;
            }
            if !result.contains(&doc_id) {
                continue 'outer;
            }
        }
        intersection.push(doc_id);
    }

    intersection
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Index;

    #[test]
    fn test_single_plain_query() {
        let mut index = Index::default();
        
        // 添加测试数据
        index.add(1, "hello world", false).unwrap();
        index.add(2, "hello rust", false).unwrap();
        index.add(3, "goodbye world", false).unwrap();
        
        // 搜索存在的术语
        let results = single_plain_query(&index, "hello", 10, 0).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.contains(&1));
        assert!(results.contains(&2));
        
        // 搜索不存在的术语
        let results = single_plain_query(&index, "nonexistent", 10, 0).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_apply_limit_offset() {
        let results = vec![1, 2, 3, 4, 5];
        
        // 测试限制
        let limited = apply_limit_offset(&results, 3, 0);
        assert_eq!(limited, vec![1, 2, 3]);
        
        // 测试偏移
        let offset = apply_limit_offset(&results, 10, 2);
        assert_eq!(offset, vec![3, 4, 5]);
        
        // 测试限制和偏移
        let both = apply_limit_offset(&results, 2, 1);
        assert_eq!(both, vec![2, 3]);
        
        // 测试边界条件
        let empty = apply_limit_offset(&results, 0, 10);
        assert_eq!(empty, Vec::<u64>::new());
    }

    #[test]
    fn test_perform_intersection() {
        let results1 = vec![1, 2, 3, 4];
        let results2 = vec![2, 3, 5, 6];
        let results3 = vec![2, 4, 6, 7];
        
        let intersection = perform_intersection(&[results1, results2, results3]);
        assert_eq!(intersection, vec![2]);
        
        // 测试空结果
        let empty = perform_intersection(&[]);
        assert_eq!(empty, Vec::<u64>::new());
        
        // 测试单结果
        let single = perform_intersection(&[vec![1, 2, 3]]);
        assert_eq!(single, vec![1, 2, 3]);
    }

    #[test]
    fn test_multi_term_search() {
        let mut index = Index::default();
        
        // 添加测试数据
        index.add(1, "hello world", false).unwrap();
        index.add(2, "rust programming", false).unwrap();
        index.add(3, "rust programming", false).unwrap();
        index.add(4, "hello rust world", false).unwrap();
        
        let options = SearchOptions::default();
        
        // 多术语搜索（交集）
        let results = multi_term_search(&index, vec!["hello", "rust"], &options).unwrap();
        
        // 文档1: "hello world" - 只有hello
        // 文档2: "rust programming" - 只有rust  
        // 文档3: "rust programming" - 只有rust
        // 文档4: "hello rust world" - 有hello和rust
        // 所以交集应该只返回文档4
        assert_eq!(results.len(), 1);
        assert!(results.contains(&4));
        
        // 单术语搜索（退化情况）
        let results = multi_term_search(&index, vec!["hello"], &options).unwrap();
        assert_eq!(results.len(), 2); // 文档1和4包含"hello"
    }
}
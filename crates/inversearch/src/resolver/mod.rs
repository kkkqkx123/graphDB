//! Resolver模块
//!
//! 提供搜索结果的解析和处理功能
//!
//! # 模块结构
//!
//! - `resolver.rs`: 主Resolver结构体和核心方法
//! - `handler.rs`: 集合操作处理器(and/or/xor/not)
//! - `and.rs`: 交集操作
//! - `or.rs`: 并集操作
//! - `not.rs`: 差集操作
//! - `xor.rs`: 异或操作
//! - `combine.rs`: 结果合并工具
//! - `core.rs`: 包含同步和异步Resolver方法
//! - `enrich.rs`: 结果丰富化功能

mod core;
mod handler;
mod and;
mod or;
mod not;
mod xor;
mod combine;
mod enrich;

pub use core::{
    Resolver,
    resolve_default,
    ResolverOptions,
    ResolverError,
    ResolverResult,
};
pub use handler::Handler;
pub use and::intersect_and;
pub use or::union_op;
pub use not::exclusion;
pub use xor::xor_op;
pub use combine::combine_search_results;
pub use enrich::{Enricher, FieldSelector, TagIntegrationConfig, HighlightConfig, MetadataSource};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::r#type::{IntermediateSearchResults, SearchOptions};

    #[test]
    fn test_resolver_chain_operations() {
        let result: IntermediateSearchResults = vec![vec![1, 2, 3, 4, 5]];
        let mut resolver = Resolver::new(result, None);
        resolver.limit(3).offset(1).boost(5);

        assert_eq!(resolver.boostval, 5);
    }

    #[test]
    fn test_operations_basic() {
        let result: IntermediateSearchResults = vec![vec![1, 2, 3]];
        let other: IntermediateSearchResults = vec![vec![2, 3, 4]];

        let resolver = Resolver::new(result, None);
        let mut resolver = resolver;
        resolver.and(other);
        let resolved = resolver.get();

        assert!(!resolved.is_empty());
    }

    #[test]
    fn test_resolve_default() {
        let result: IntermediateSearchResults = vec![vec![1, 2, 3, 4, 5]];
        let resolved = resolve_default(&result, 3, 0, false);
        assert_eq!(resolved, vec![1, 2, 3]);
    }

    #[test]
    fn test_search_options_builder() {
        let options = SearchOptions {
            query: Some("test".to_string()),
            limit: Some(10),
            offset: Some(5),
            boost: Some(3),
            ..Default::default()
        };

        assert_eq!(options.query, Some("test".to_string()));
        assert_eq!(options.limit, Some(10));
        assert_eq!(options.offset, Some(5));
        assert_eq!(options.boost, Some(3));
    }

    #[test]
    fn test_resolver_async_methods() {
        let result: IntermediateSearchResults = vec![vec![1, 2, 3]];
        let resolver = Resolver::new(result, None);

        // 测试异步方法存在且可调用
        assert_eq!(resolver.result.len(), 1);
    }

    #[test]
    fn test_enricher_basic() {
        use serde_json::json;

        let ids = vec![0, 1, 2];
        let documents = vec![
            Some(json!({"id": 1, "name": "test1"})),
            Some(json!({"id": 2, "name": "test2"})),
            Some(json!({"id": 3, "name": "test3"})),
        ];

        let enriched = Enricher::apply_enrich(&ids, &documents);

        assert_eq!(enriched.len(), 3);
        assert_eq!(enriched[0].id, 0);
        assert_eq!(enriched[1].id, 1);
        assert_eq!(enriched[2].id, 2);
        if let Some(ref doc) = enriched[0].doc {
            assert_eq!(doc["name"], "test1");
        }
        if let Some(ref doc) = enriched[1].doc {
            assert_eq!(doc["name"], "test2");
        }
        if let Some(ref doc) = enriched[2].doc {
            assert_eq!(doc["name"], "test3");
        }
    }
}

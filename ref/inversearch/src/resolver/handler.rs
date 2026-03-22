use crate::resolver::{Resolver, ResolverOptions};
use crate::r#type::{IntermediateSearchResults, SearchResults};

pub struct Handler;

impl Handler {
    pub fn handle_and(
        resolver: &mut Resolver,
        mut other_results: Vec<IntermediateSearchResults>,
        limit: usize,
        offset: usize,
        _enrich: bool,
        _resolve: bool,
        _suggest: bool,
    ) -> &mut Resolver {
        if other_results.is_empty() {
            return resolver;
        }

        if !resolver.result.is_empty() {
            other_results.insert(0, resolver.result.clone());
        }

        if other_results.len() < 2 {
            if let Some(first) = other_results.first() {
                resolver.result = first.clone();
            }
        } else {
            let mut all_results: Vec<IntermediateSearchResults> = Vec::new();
            for result in other_results {
                if !result.is_empty() {
                    all_results.push(result);
                }
            }

            if all_results.len() >= 2 {
                let mut resolution = 0;
                for arr in &all_results {
                    let len: usize = arr.iter().map(|a| a.len()).sum();
                    if resolution < len {
                        resolution = len;
                    }
                }

                if resolution > 0 {
                    resolver.result = crate::resolver::and::intersect_and(all_results, limit);
                } else {
                    resolver.result = vec![];
                }
            } else if let Some(first) = all_results.first() {
                resolver.result = first.clone();
            }
        }

        resolver
    }

    pub fn handle_or(
        resolver: &mut Resolver,
        mut other_results: Vec<IntermediateSearchResults>,
        _limit: usize,
        _offset: usize,
        _enrich: bool,
        _resolve: bool,
        _suggest: bool,
    ) -> &mut Resolver {
        if other_results.is_empty() {
            return resolver;
        }

        if !resolver.result.is_empty() {
            other_results.push(resolver.result.clone());
        }

        if other_results.len() < 2 {
            if let Some(first) = other_results.first() {
                resolver.result = first.clone();
            }
        } else {
            resolver.result = crate::resolver::or::union_op(other_results, resolver.boostval);
        }

        resolver
    }

    pub fn handle_not(
        resolver: &mut Resolver,
        mut other_results: Vec<IntermediateSearchResults>,
        limit: usize,
        _offset: usize,
        _enrich: bool,
        _resolve: bool,
        _suggest: bool,
    ) -> &mut Resolver {
        if other_results.is_empty() || resolver.result.is_empty() {
            return resolver;
        }

        let exclude_flat: SearchResults = other_results.into_iter().flatten().flatten().collect();
        resolver.result = crate::resolver::not::exclusion(resolver.result.clone(), &exclude_flat, limit);

        resolver
    }

    pub fn handle_xor(
        resolver: &mut Resolver,
        mut other_results: Vec<IntermediateSearchResults>,
        _limit: usize,
        _offset: usize,
        _enrich: bool,
        _resolve: bool,
        _suggest: bool,
    ) -> &mut Resolver {
        if other_results.is_empty() {
            return resolver;
        }

        if !resolver.result.is_empty() {
            other_results.insert(0, resolver.result.clone());
        }

        if other_results.len() < 2 {
            if let Some(first) = other_results.first() {
                resolver.result = first.clone();
            }
        } else {
            resolver.result = crate::resolver::xor::xor_op(other_results, resolver.boostval);
        }

        resolver
    }

    pub fn handle_with_options(
        resolver: &mut Resolver,
        options: &ResolverOptions,
    ) {
        let index = match resolver.index.as_ref() {
            Some(idx) => idx,
            None => return,
        };

        let limit = options.limit().unwrap_or(0);
        let offset = options.offset().unwrap_or(0);
        let enrich = options.enrich().unwrap_or(false);
        let resolve = options.resolve().unwrap_or(false);
        let suggest = options.suggest().unwrap_or(false);

        if let Some(query_str) = options.query().cloned() {
            let search_options = crate::r#type::SearchOptions {
                query: Some(query_str),
                limit: Some(limit),
                offset: Some(offset),
                enrich: Some(enrich),
                resolve: Some(resolve),
                suggest: Some(suggest),
                boost: options.boost(),
                ..Default::default()
            };

            if let Ok(search_result) = index.search(&search_options) {
                let result_ids = search_result.results;
                resolver.result = vec![result_ids];
            }
        }
    }

    pub fn handle_nested(
        resolver: &mut Resolver,
        options_list: Vec<ResolverOptions>,
    ) {
        if options_list.is_empty() {
            return;
        }

        let mut results: Vec<SearchResults> = Vec::new();
        let index = match resolver.index.as_ref() {
            Some(idx) => idx,
            None => return,
        };

        for options in &options_list {
            if let Some(query_str) = options.query().cloned() {
                let search_options = crate::r#type::SearchOptions {
                    query: Some(query_str),
                    limit: options.limit(),
                    offset: options.offset(),
                    enrich: options.enrich(),
                    resolve: Some(false),
                    suggest: options.suggest(),
                    boost: options.boost(),
                    ..Default::default()
                };

                if let Ok(search_result) = index.search(&search_options) {
                    let result_ids = search_result.results;
                    if !result_ids.is_empty() {
                        results.push(result_ids);
                    }
                }
            }
        }

        if !results.is_empty() {
            resolver.result = results;
        }
    }

    pub fn execute_chain(
        resolver: &mut Resolver,
        operations: Vec<(String, ResolverOptions)>,
    ) {
        let mut current = resolver.clone();

        for (method, options) in operations {
            let index = match current.index.as_ref() {
                Some(idx) => idx,
                None => break,
            };

            let limit = options.limit().unwrap_or(0);
            let offset = options.offset().unwrap_or(0);
            let enrich = options.enrich().unwrap_or(false);
            let resolve = options.resolve().unwrap_or(false);
            let suggest = options.suggest().unwrap_or(false);

            if let Some(query_str) = options.query().cloned() {
                let search_options = crate::r#type::SearchOptions {
                    query: Some(query_str),
                    limit: Some(limit),
                    offset: Some(offset),
                    enrich: Some(enrich),
                    resolve: Some(resolve),
                    suggest: Some(suggest),
                    boost: options.boost(),
                    ..Default::default()
                };

                if let Ok(search_result) = index.search(&search_options) {
                    let result_ids = search_result.results;

                    match method.as_str() {
                        "and" => {
                            current.result = vec![result_ids.clone()];
                            let op_results: Vec<IntermediateSearchResults> = vec![current.result.clone()];
                            Handler::handle_and(&mut current, op_results, limit, offset, enrich, resolve, suggest);
                        }
                        "or" => {
                            let op_results: Vec<IntermediateSearchResults> = vec![vec![result_ids.clone()]];
                            Handler::handle_or(&mut current, op_results, limit, offset, enrich, resolve, suggest);
                        }
                        "not" => {
                            let op_results: Vec<IntermediateSearchResults> = vec![vec![result_ids.clone()]];
                            Handler::handle_not(&mut current, op_results, limit, offset, enrich, resolve, suggest);
                        }
                        "xor" => {
                            let op_results: Vec<IntermediateSearchResults> = vec![vec![result_ids.clone()]];
                            Handler::handle_xor(&mut current, op_results, limit, offset, enrich, resolve, suggest);
                        }
                        _ => {
                            current.result = vec![result_ids];
                        }
                    }
                }
            }
        }

        resolver.result = current.result;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::r#type::SearchOptions;

    #[test]
    fn test_handler_and() {
        let mut resolver = Resolver::new(vec![vec![1, 2, 3]], None);
        let inner: IntermediateSearchResults = vec![vec![2, 3, 4]];
        let other: Vec<IntermediateSearchResults> = vec![inner];
        Handler::handle_and(&mut resolver, other, 100, 0, false, false, false);
        
        let result = resolver.get();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_handler_or() {
        let mut resolver = Resolver::new(vec![vec![1, 2]], None);
        let inner: IntermediateSearchResults = vec![vec![3, 4, 5]];
        let other: Vec<IntermediateSearchResults> = vec![inner];
        Handler::handle_or(&mut resolver, other, 100, 0, false, false, false);
        
        let result = resolver.get();
        assert!(result.contains(&1) && result.contains(&2) && result.contains(&3) && result.contains(&4) && result.contains(&5));
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_handler_not() {
        let mut resolver = Resolver::new(vec![vec![1, 2, 3, 4, 5]], None);
        let inner: IntermediateSearchResults = vec![vec![2, 3]];
        let other: Vec<IntermediateSearchResults> = vec![inner];
        Handler::handle_not(&mut resolver, other, 100, 0, false, false, false);
        
        let result = resolver.get();
        assert_eq!(result, vec![1, 4, 5]);
    }

    #[test]
    fn test_handler_xor() {
        let mut resolver = Resolver::new(vec![vec![1, 2, 3]], None);
        let inner: IntermediateSearchResults = vec![vec![2, 3, 4]];
        let other: Vec<IntermediateSearchResults> = vec![inner];
        Handler::handle_xor(&mut resolver, other, 100, 0, false, false, false);
        
        let result = resolver.get();
        assert_eq!(result, vec![1, 4]);
    }

    #[test]
    fn test_search_options_builder() {
        let mut options = SearchOptions::default();
        options.query = Some("test".to_string());
        options.limit = Some(10);
        options.offset = Some(5);
        options.boost = Some(3);

        assert_eq!(options.query, Some("test".to_string()));
        assert_eq!(options.limit, Some(10));
        assert_eq!(options.offset, Some(5));
        assert_eq!(options.boost, Some(3));
    }
}

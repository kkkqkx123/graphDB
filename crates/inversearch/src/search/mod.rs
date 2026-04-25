//! Search Module
//!
//! Provides search functionality, including single term search, multi-term search and multi-field search coordination

mod cache;
mod coordinator;
mod multi_field;
mod single_term;

use crate::error::Result;
use crate::r#type::{IntermediateSearchResults, SearchOptions, SearchResults};
use crate::Index;
pub use cache::{CacheKeyGenerator, CacheStats, CachedSearch, SearchCache};
pub use coordinator::{
    BoostStrategy, CombineStrategy, FieldBoostConfig, FieldSearch, MultiFieldSearchOptions,
    SearchCoordinator,
};
pub use multi_field::{
    multi_field_search, multi_field_search_with_weights, MultiFieldSearchConfig,
};
pub use single_term::{multi_term_search, single_term_query, SingleTermResult};

/// Search Result Structures
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub results: SearchResults,
    pub total: usize,
    pub query: String,
}

/// Execute Search - Full Implementation
pub fn search(index: &Index, options: &SearchOptions) -> Result<SearchResult> {
    let query = options.query.as_deref().unwrap_or("");
    if query.is_empty() {
        return Ok(SearchResult {
            results: Vec::new(),
            total: 0,
            query: String::new(),
        });
    }

    // Parse search terms
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
    let context = options.context.unwrap_or(false);

    // Choose different search strategies based on the number of terms
    let results = if encoded_terms.len() == 1 {
        // Single Term Fast Path
        let result = single_term_query(
            index,
            &encoded_terms[0],
            None,
            limit,
            offset,
            options.resolve.unwrap_or(true),
            context,
            None,
        )?;
        result.results
    } else {
        // Multi-term search
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

/// Default parsing function (compatible function)
pub fn resolve_default_search(
    results: &IntermediateSearchResults,
    limit: usize,
    offset: usize,
) -> Vec<u64> {
    if results.is_empty() {
        return Vec::new();
    }

    // Flattening results
    let mut flattened = Vec::new();
    for array in results {
        flattened.extend_from_slice(array);
    }

    // Applying Limits and Offsets
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

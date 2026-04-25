//! Single Term Search Module
//!
//! Provides search functionality for single terms and simple queries

use crate::error::Result;
use crate::r#type::{SearchOptions, SearchResults};
use crate::Index;

/// Single Term Search Results
#[derive(Debug, Clone)]
pub struct SingleTermResult {
    pub results: SearchResults,
    pub term: String,
    pub context: Option<String>,
    pub total: usize,
}

/// Execution order terminology queries
#[allow(clippy::too_many_arguments)]
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

    // Getting the encoded terms
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

    // Select different queries based on the presence or absence of a context
    let results = if let Some(ctx) = context {
        // context search
        single_context_query(index, first_term, ctx, limit, offset)?
    } else {
        // General Terms Search
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

/// General Terms Search
fn single_plain_query(
    index: &Index,
    term: &str,
    limit: usize,
    offset: usize,
) -> Result<SearchResults> {
    // Get the list of document IDs directly from the main index map
    let term_str = term.to_string();
    let doc_ids = if let Some(entries) = index.map.get(&term_str) {
        entries.clone()
    } else {
        Vec::new()
    };

    // Applying Limits and Offsets
    Ok(apply_limit_offset(&doc_ids, limit, offset))
}

/// context search
fn single_context_query(
    index: &Index,
    term: &str,
    context: &str,
    limit: usize,
    offset: usize,
) -> Result<SearchResults> {
    // First check the context index
    let context_str = context.to_string();
    if let Some(doc_ids) = index.ctx.get(&context_str) {
        let term_str = term.to_string();
        // Check if the term is in the document ID list
        if doc_ids
            .iter()
            .any(|id| *id == term_str.parse::<u64>().unwrap_or(0))
        {
            return Ok(doc_ids.clone());
        }
    }

    // If no context-specific results are found, fall back to the normal search
    single_plain_query(index, term, limit, offset)
}

/// Applying Limits and Offsets
fn apply_limit_offset(results: &[u64], limit: usize, offset: usize) -> SearchResults {
    if results.is_empty() {
        return Vec::new();
    }

    // If limit is 0, return empty results
    if limit == 0 {
        return Vec::new();
    }

    let start = offset.min(results.len());
    let end = (start + limit).min(results.len());

    results[start..end].to_vec()
}

/// Multi-term search
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

    // Collect search results for each term
    let mut intermediate_results = Vec::new();

    for term in terms {
        // Use usize::MAX to get all results for union
        let result = single_term_query(index, term, None, usize::MAX, 0, true, false, None)?;

        if !result.results.is_empty() {
            intermediate_results.push(result.results);
        }
    }

    if intermediate_results.is_empty() {
        return Ok(Vec::new());
    }

    // Performs an OR logic - returns a result that contains any one of the words
    let unioned = if intermediate_results.len() == 1 {
        intermediate_results.into_iter().next().unwrap_or_default()
    } else {
        perform_union(&intermediate_results)
    };

    // Applying Limits and Offsets
    Ok(apply_limit_offset(&unioned, limit, offset))
}

/// Perform intersection operations
#[allow(dead_code)]
fn perform_intersection(results: &[SearchResults]) -> SearchResults {
    if results.is_empty() {
        return Vec::new();
    }

    if results.len() == 1 {
        return results[0].clone();
    }

    // Find the smallest result set to use as a base
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

    // Check that each ID in the base set exists in all other result sets
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

/// perform a union operation
fn perform_union(results: &[SearchResults]) -> SearchResults {
    if results.is_empty() {
        return Vec::new();
    }

    if results.len() == 1 {
        return results[0].clone();
    }

    let mut seen = std::collections::HashSet::new();
    let mut union = Vec::new();

    // Merge all result sets, de-duplicate
    for result in results {
        for &doc_id in result {
            if seen.insert(doc_id) {
                union.push(doc_id);
            }
        }
    }

    // Sorting to maintain stable output
    union.sort();
    union
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Index;

    #[test]
    fn test_single_plain_query() {
        let mut index = Index::default();

        // Add test data
        index.add(1, "hello world", false).unwrap();
        index.add(2, "hello rust", false).unwrap();
        index.add(3, "goodbye world", false).unwrap();

        // Searching for Presence Terms
        let results = single_plain_query(&index, "hello", 10, 0).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.contains(&1));
        assert!(results.contains(&2));

        // Search for non-existent terms
        let results = single_plain_query(&index, "nonexistent", 10, 0).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_apply_limit_offset() {
        let results = vec![1, 2, 3, 4, 5];

        // Test Limitations
        let limited = apply_limit_offset(&results, 3, 0);
        assert_eq!(limited, vec![1, 2, 3]);

        // Test Offset
        let offset = apply_limit_offset(&results, 10, 2);
        assert_eq!(offset, vec![3, 4, 5]);

        // Test limits and offsets
        let both = apply_limit_offset(&results, 2, 1);
        assert_eq!(both, vec![2, 3]);

        // Test boundary conditions
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

        // Test Empty Results
        let empty = perform_intersection(&[]);
        assert_eq!(empty, Vec::<u64>::new());

        // Test Order Results
        let single = perform_intersection(&[vec![1, 2, 3]]);
        assert_eq!(single, vec![1, 2, 3]);
    }

    #[test]
    fn test_multi_term_search() {
        let mut index = Index::default();

        // Add test data
        index.add(1, "hello world", false).unwrap();
        index.add(2, "rust programming", false).unwrap();
        index.add(3, "rust programming", false).unwrap();
        index.add(4, "hello rust world", false).unwrap();

        let options = SearchOptions::default();

        // Multi-term search (concatenation/OR logic)
        let results = multi_term_search(&index, vec!["hello", "rust"], &options).unwrap();

        // Document 1: "hello world" - only hello
        // Document 2: "rust programming" - only rust
        // Document 3: "rust programming" - only rust
        // Document 4: "hello rust world" - with hello and rust
        // So the concatenation should return documents 1, 2, 3, 4 (all documents containing hello or rust)
        assert_eq!(results.len(), 4);
        assert!(results.contains(&1));
        assert!(results.contains(&2));
        assert!(results.contains(&3));
        assert!(results.contains(&4));

        // Single term search (degradation)
        let results = multi_term_search(&index, vec!["hello"], &options).unwrap();
        assert_eq!(results.len(), 2); // Documents 1 and 4 contain "hello".
    }
}

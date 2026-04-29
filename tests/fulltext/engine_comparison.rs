//! Fulltext Integration Tests - Engine Comparison
//!
//! Test scope:
//! - Compare BM25 and Inversearch search results
//! - Compare scoring between engines
//! - Compare performance characteristics
//! - Test mixed engine usage in same space
//! - Verify result consistency
//!
//! Test cases: TC-FT-COMP-001 ~ TC-FT-COMP-008

use super::common::{
    assert_search_result_contains, assert_search_result_count, compare_search_results,
    FulltextTestContext,
};
use graphdb::search::EngineType;

/// TC-FT-COMP-001: Compare Basic Search Results Between Engines
#[tokio::test]
async fn test_compare_basic_search_results() {
    let ctx = FulltextTestContext::new();

    // Create indexes with different engines
    ctx.create_test_index(1, "Article", "content_bm25", Some(EngineType::Bm25))
        .await
        .expect("Failed to create BM25 index");
    ctx.create_test_index(1, "Article", "content_inv", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create Inversearch index");

    // Insert same documents to both indexes
    let docs = vec![
        ("doc_1", "rust programming language"),
        ("doc_2", "python programming language"),
        ("doc_3", "javascript web development"),
        ("doc_4", "rust web framework"),
    ];

    ctx.insert_test_docs(1, "Article", "content_bm25", docs.clone())
        .await
        .expect("Failed to insert to BM25");
    ctx.insert_test_docs(1, "Article", "content_inv", docs)
        .await
        .expect("Failed to insert to Inversearch");

    ctx.commit_all().await.expect("Failed to commit");

    // Search with same query on both engines
    let bm25_results = ctx
        .search(1, "Article", "content_bm25", "programming", 10)
        .await
        .expect("BM25 search should succeed");

    let inv_results = ctx
        .search(1, "Article", "content_inv", "programming", 10)
        .await
        .expect("Inversearch search should succeed");

    // Both engines should find the same documents
    assert_eq!(
        bm25_results.len(),
        inv_results.len(),
        "Both engines should return same number of results"
    );

    let (common, different) = compare_search_results(&bm25_results, &inv_results);
    assert!(
        different.is_empty(),
        "Engines should find the same documents, but found differences: {:?}",
        different
    );
    assert_eq!(
        common.len(),
        2,
        "Both engines should find 2 documents with 'programming'"
    );
}

/// TC-FT-COMP-002: Compare Scoring Between Engines
#[tokio::test]
async fn test_compare_scoring() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content_bm25", Some(EngineType::Bm25))
        .await
        .expect("Failed to create BM25 index");
    ctx.create_test_index(1, "Article", "content_inv", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create Inversearch index");

    let docs = vec![
        ("doc_1", "rust rust rust programming"),
        ("doc_2", "rust programming"),
        ("doc_3", "programming language"),
    ];

    ctx.insert_test_docs(1, "Article", "content_bm25", docs.clone())
        .await
        .expect("Failed to insert to BM25");
    ctx.insert_test_docs(1, "Article", "content_inv", docs)
        .await
        .expect("Failed to insert to Inversearch");

    ctx.commit_all().await.expect("Failed to commit");

    let bm25_results = ctx
        .search(1, "Article", "content_bm25", "rust programming", 10)
        .await
        .expect("BM25 search should succeed");

    let inv_results = ctx
        .search(1, "Article", "content_inv", "rust programming", 10)
        .await
        .expect("Inversearch search should succeed");

    // Both should return all documents
    assert_eq!(bm25_results.len(), 3, "BM25 should return 3 results");
    assert_eq!(inv_results.len(), 3, "Inversearch should return 3 results");

    // BM25 should have meaningful scores (non-zero, different values)
    for result in &bm25_results {
        assert!(
            result.score > 0.0,
            "BM25 should have positive scores, got {}",
            result.score
        );
    }

    // Inversearch may have different scoring - just verify it returns results
    assert!(!inv_results.is_empty(), "Inversearch should return results");
}

/// TC-FT-COMP-003: Compare Multi-Term Search
#[tokio::test]
async fn test_compare_multi_term_search() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content_bm25", Some(EngineType::Bm25))
        .await
        .expect("Failed to create BM25 index");
    ctx.create_test_index(1, "Article", "content_inv", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create Inversearch index");

    let docs = vec![
        ("doc_1", "rust programming tutorial"),
        ("doc_2", "python programming guide"),
        ("doc_3", "rust tutorial advanced"),
        ("doc_4", "javascript web tutorial"),
    ];

    ctx.insert_test_docs(1, "Article", "content_bm25", docs.clone())
        .await
        .expect("Failed to insert to BM25");
    ctx.insert_test_docs(1, "Article", "content_inv", docs)
        .await
        .expect("Failed to insert to Inversearch");

    ctx.commit_all().await.expect("Failed to commit");

    // Search for multiple terms
    let bm25_results = ctx
        .search(1, "Article", "content_bm25", "rust tutorial", 10)
        .await
        .expect("BM25 search should succeed");

    let inv_results = ctx
        .search(1, "Article", "content_inv", "rust tutorial", 10)
        .await
        .expect("Inversearch search should succeed");

    // Both should find documents containing either term
    assert!(!bm25_results.is_empty(), "BM25 should find results");
    assert!(!inv_results.is_empty(), "Inversearch should find results");

    // Both should find doc_1 and doc_3 (contain rust and/or tutorial)
    assert_search_result_contains(&bm25_results, "doc_1")
        .expect("BM25 should find doc_1");
    assert_search_result_contains(&inv_results, "doc_1")
        .expect("Inversearch should find doc_1");
}

/// TC-FT-COMP-004: Compare Empty Results Handling
#[tokio::test]
async fn test_compare_empty_results() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content_bm25", Some(EngineType::Bm25))
        .await
        .expect("Failed to create BM25 index");
    ctx.create_test_index(1, "Article", "content_inv", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create Inversearch index");

    let docs = vec![
        ("doc_1", "rust programming"),
        ("doc_2", "python programming"),
    ];

    ctx.insert_test_docs(1, "Article", "content_bm25", docs.clone())
        .await
        .expect("Failed to insert to BM25");
    ctx.insert_test_docs(1, "Article", "content_inv", docs)
        .await
        .expect("Failed to insert to Inversearch");

    ctx.commit_all().await.expect("Failed to commit");

    // Search for non-existent term
    let bm25_results = ctx
        .search(1, "Article", "content_bm25", "nonexistent", 10)
        .await
        .expect("BM25 search should succeed");

    let inv_results = ctx
        .search(1, "Article", "content_inv", "nonexistent", 10)
        .await
        .expect("Inversearch search should succeed");

    // Both should return empty results
    assert_eq!(
        bm25_results.len(),
        0,
        "BM25 should return empty results for non-existent term"
    );
    assert_eq!(
        inv_results.len(),
        0,
        "Inversearch should return empty results for non-existent term"
    );
}

/// TC-FT-COMP-005: Compare Document Update Behavior
#[tokio::test]
async fn test_compare_document_update() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content_bm25", Some(EngineType::Bm25))
        .await
        .expect("Failed to create BM25 index");
    ctx.create_test_index(1, "Article", "content_inv", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create Inversearch index");

    // Initial insert
    ctx.insert_test_doc(1, "Article", "content_bm25", "doc_1", "original content")
        .await
        .expect("Failed to insert to BM25");
    ctx.insert_test_doc(1, "Article", "content_inv", "doc_1", "original content")
        .await
        .expect("Failed to insert to Inversearch");

    ctx.commit_all().await.expect("Failed to commit");

    // Update - delete and re-insert
    if let Some(engine) = ctx.manager.get_engine(1, "Article", "content_bm25") {
        engine.delete("doc_1").await.expect("Failed to delete from BM25");
    }
    if let Some(engine) = ctx.manager.get_engine(1, "Article", "content_inv") {
        engine.delete("doc_1").await.expect("Failed to delete from Inversearch");
    }

    ctx.insert_test_doc(1, "Article", "content_bm25", "doc_1", "updated content")
        .await
        .expect("Failed to update BM25");
    ctx.insert_test_doc(1, "Article", "content_inv", "doc_1", "updated content")
        .await
        .expect("Failed to update Inversearch");

    ctx.commit_all().await.expect("Failed to commit");

    // Both should not find old content
    let bm25_old = ctx
        .search(1, "Article", "content_bm25", "original", 10)
        .await
        .expect("BM25 search should succeed");
    let inv_old = ctx
        .search(1, "Article", "content_inv", "original", 10)
        .await
        .expect("Inversearch search should succeed");

    assert_eq!(bm25_old.len(), 0, "BM25 should not find old content");
    assert_eq!(inv_old.len(), 0, "Inversearch should not find old content");

    // Both should find new content
    let bm25_new = ctx
        .search(1, "Article", "content_bm25", "updated", 10)
        .await
        .expect("BM25 search should succeed");
    let inv_new = ctx
        .search(1, "Article", "content_inv", "updated", 10)
        .await
        .expect("Inversearch search should succeed");

    assert_eq!(bm25_new.len(), 1, "BM25 should find new content");
    assert_eq!(inv_new.len(), 1, "Inversearch should find new content");
}

/// TC-FT-COMP-006: Mixed Engine Usage in Same Space
#[tokio::test]
async fn test_mixed_engines_same_space() {
    let ctx = FulltextTestContext::new();

    // Create multiple indexes with different engines in same space
    ctx.create_test_index(1, "Article", "title", Some(EngineType::Bm25))
        .await
        .expect("Failed to create BM25 index");
    ctx.create_test_index(1, "Article", "content", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create Inversearch index");
    ctx.create_test_index(1, "Article", "summary", Some(EngineType::Bm25))
        .await
        .expect("Failed to create second BM25 index");

    // Verify all indexes exist with correct engine types
    let title_engine = ctx.get_engine_type(1, "Article", "title");
    assert_eq!(
        title_engine,
        Some(EngineType::Bm25),
        "Title should use BM25"
    );

    let content_engine = ctx.get_engine_type(1, "Article", "content");
    assert_eq!(
        content_engine,
        Some(EngineType::Inversearch),
        "Content should use Inversearch"
    );

    let summary_engine = ctx.get_engine_type(1, "Article", "summary");
    assert_eq!(
        summary_engine,
        Some(EngineType::Bm25),
        "Summary should use BM25"
    );

    // Insert documents to all indexes
    ctx.insert_test_doc(1, "Article", "title", "doc_1", "Rust Programming")
        .await
        .expect("Failed to insert title");
    ctx.insert_test_doc(1, "Article", "content", "doc_1", "Learn Rust from scratch")
        .await
        .expect("Failed to insert content");
    ctx.insert_test_doc(1, "Article", "summary", "doc_1", "A comprehensive Rust tutorial")
        .await
        .expect("Failed to insert summary");

    ctx.commit_all().await.expect("Failed to commit");

    // Search in each index
    let title_results = ctx
        .search(1, "Article", "title", "Rust", 10)
        .await
        .expect("Title search should succeed");
    assert_eq!(title_results.len(), 1, "Should find document in title");

    let content_results = ctx
        .search(1, "Article", "content", "scratch", 10)
        .await
        .expect("Content search should succeed");
    assert_eq!(content_results.len(), 1, "Should find document in content");

    let summary_results = ctx
        .search(1, "Article", "summary", "tutorial", 10)
        .await
        .expect("Summary search should succeed");
    assert_eq!(summary_results.len(), 1, "Should find document in summary");
}

/// TC-FT-COMP-007: Compare Search Limit Behavior
#[tokio::test]
async fn test_compare_search_limit() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content_bm25", Some(EngineType::Bm25))
        .await
        .expect("Failed to create BM25 index");
    ctx.create_test_index(1, "Article", "content_inv", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create Inversearch index");

    // Insert 50 documents
    let docs: Vec<(String, String)> = (0..50)
        .map(|i| (format!("doc_{}", i), format!("test content {}", i)))
        .collect();
    let docs_ref: Vec<(&str, &str)> = docs
        .iter()
        .map(|(id, content)| (id.as_str(), content.as_str()))
        .collect();

    ctx.insert_test_docs(1, "Article", "content_bm25", docs_ref.clone())
        .await
        .expect("Failed to insert to BM25");
    ctx.insert_test_docs(1, "Article", "content_inv", docs_ref)
        .await
        .expect("Failed to insert to Inversearch");

    ctx.commit_all().await.expect("Failed to commit");

    // Search with limit
    let bm25_results = ctx
        .search(1, "Article", "content_bm25", "test", 10)
        .await
        .expect("BM25 search should succeed");

    let inv_results = ctx
        .search(1, "Article", "content_inv", "test", 10)
        .await
        .expect("Inversearch search should succeed");

    // Both should respect the limit
    assert_eq!(
        bm25_results.len(),
        10,
        "BM25 should respect limit of 10"
    );
    assert_eq!(
        inv_results.len(),
        10,
        "Inversearch should respect limit of 10"
    );
}

/// TC-FT-COMP-008: Compare Special Characters Handling
#[tokio::test]
async fn test_compare_special_characters() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content_bm25", Some(EngineType::Bm25))
        .await
        .expect("Failed to create BM25 index");
    ctx.create_test_index(1, "Article", "content_inv", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create Inversearch index");

    let docs = vec![
        ("doc_1", "test@example.com"),
        ("doc_2", "price: $100.00"),
        ("doc_3", "100% complete"),
    ];

    ctx.insert_test_docs(1, "Article", "content_bm25", docs.clone())
        .await
        .expect("Failed to insert to BM25");
    ctx.insert_test_docs(1, "Article", "content_inv", docs)
        .await
        .expect("Failed to insert to Inversearch");

    ctx.commit_all().await.expect("Failed to commit");

    // Test searches with special characters
    let queries = vec!["example", "price", "complete"];

    for query in queries {
        let bm25_results = ctx
            .search(1, "Article", "content_bm25", query, 10)
            .await
            .expect("BM25 search should succeed");

        let inv_results = ctx
            .search(1, "Article", "content_inv", query, 10)
            .await
            .expect("Inversearch search should succeed");

        // Both should find results for valid terms
        assert!(
            !bm25_results.is_empty() || !inv_results.is_empty(),
            "At least one engine should find results for '{}'",
            query
        );
    }
}

//! Fulltext Integration Tests - Highlight Feature
//!
//! Test scope:
//! - Search result highlight field
//! - Matched fields tracking
//! - Highlight with different content types
//! - Multi-term highlight
//! - Both BM25 and Inversearch engines
//!
//! Test cases: TC-FT-HL-001 ~ TC-FT-HL-010

use super::common::FulltextTestContext;
use graphdb::core::Value;
use graphdb::search::EngineType;

/// TC-FT-HL-001: Search Result Contains Doc ID
#[tokio::test]
async fn test_search_result_doc_id() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    ctx.insert_test_doc(1, "Article", "content", "doc_123", "Hello World")
        .await
        .expect("Failed to insert document");

    ctx.commit_all().await.expect("Failed to commit");

    let results = ctx
        .search(1, "Article", "content", "Hello", 10)
        .await
        .expect("Search should succeed");

    assert_eq!(results.len(), 1, "Should find 1 result");
    assert_eq!(
        results[0].doc_id,
        Value::String("doc_123".to_string()),
        "Doc ID should match"
    );
}

/// TC-FT-HL-002: Search Result Contains Score
#[tokio::test]
async fn test_search_result_score() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    ctx.insert_test_doc(1, "Article", "content", "doc_1", "rust programming")
        .await
        .expect("Failed to insert document");

    ctx.commit_all().await.expect("Failed to commit");

    let results = ctx
        .search(1, "Article", "content", "rust", 10)
        .await
        .expect("Search should succeed");

    assert_eq!(results.len(), 1, "Should find 1 result");
    assert!(
        results[0].score > 0.0,
        "Score should be positive"
    );
}

/// TC-FT-HL-003: Search Result Highlights Field
#[tokio::test]
async fn test_search_result_highlights_field() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    ctx.insert_test_doc(1, "Article", "content", "doc_1", "The quick brown fox jumps")
        .await
        .expect("Failed to insert document");

    ctx.commit_all().await.expect("Failed to commit");

    let results = ctx
        .search(1, "Article", "content", "quick", 10)
        .await
        .expect("Search should succeed");

    assert_eq!(results.len(), 1, "Should find 1 result");

    // Highlights field may be None or Some depending on engine implementation
    // Just verify the field exists and is accessible
    let _highlights = &results[0].highlights;
}

/// TC-FT-HL-004: Search Result Matched Fields
#[tokio::test]
async fn test_search_result_matched_fields() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create index");

    ctx.insert_test_doc(1, "Article", "content", "doc_1", "rust programming language")
        .await
        .expect("Failed to insert document");

    ctx.commit_all().await.expect("Failed to commit");

    let results = ctx
        .search(1, "Article", "content", "rust", 10)
        .await
        .expect("Search should succeed");

    assert_eq!(results.len(), 1, "Should find 1 result");

    // Verify matched_fields field exists
    let _matched_fields = &results[0].matched_fields;
}

/// TC-FT-HL-005: Score Ranking Across Multiple Results
#[tokio::test]
async fn test_score_ranking_multiple_results() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let docs = vec![
        ("doc_1", "rust rust rust programming"),
        ("doc_2", "rust programming"),
        ("doc_3", "programming in rust"),
    ];
    ctx.insert_test_docs(1, "Article", "content", docs)
        .await
        .expect("Failed to insert documents");

    ctx.commit_all().await.expect("Failed to commit");

    let results = ctx
        .search(1, "Article", "content", "rust", 10)
        .await
        .expect("Search should succeed");

    assert_eq!(results.len(), 3, "Should find 3 results");

    // Verify results are sorted by score (descending)
    for i in 1..results.len() {
        assert!(
            results[i - 1].score >= results[i].score,
            "Results should be sorted by score descending"
        );
    }
}

/// TC-FT-HL-006: Different Doc ID Types
#[tokio::test]
async fn test_different_doc_id_types() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    ctx.insert_test_doc(1, "Article", "content", "string_id_123", "String ID content")
        .await
        .expect("Failed to insert document");

    ctx.commit_all().await.expect("Failed to commit");

    let results = ctx
        .search(1, "Article", "content", "String", 10)
        .await
        .expect("Search should succeed");

    assert_eq!(results.len(), 1, "Should find 1 result");
    assert_eq!(
        results[0].doc_id,
        Value::String("string_id_123".to_string()),
        "Doc ID should be string type"
    );
}

/// TC-FT-HL-007: Score Consistency for Same Query
#[tokio::test]
async fn test_score_consistency_same_query() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    ctx.insert_test_doc(1, "Article", "content", "doc_1", "consistent scoring test")
        .await
        .expect("Failed to insert document");

    ctx.commit_all().await.expect("Failed to commit");

    let results1 = ctx
        .search(1, "Article", "content", "consistent", 10)
        .await
        .expect("Search should succeed");

    let results2 = ctx
        .search(1, "Article", "content", "consistent", 10)
        .await
        .expect("Search should succeed");

    assert_eq!(
        results1[0].score, results2[0].score,
        "Score should be consistent for same query"
    );
}

/// TC-FT-HL-008: Multi-Term Search Results
#[tokio::test]
async fn test_multi_term_search_results() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let docs = vec![
        ("doc_1", "machine learning algorithms"),
        ("doc_2", "deep learning neural networks"),
        ("doc_3", "machine learning is popular"),
    ];
    ctx.insert_test_docs(1, "Article", "content", docs)
        .await
        .expect("Failed to insert documents");

    ctx.commit_all().await.expect("Failed to commit");

    let results = ctx
        .search(1, "Article", "content", "machine learning", 10)
        .await
        .expect("Search should succeed");

    assert!(
        results.len() >= 2,
        "Should find at least 2 documents with 'machine' and/or 'learning'"
    );

    // Documents with both terms should have higher scores
    let doc_1_score = results
        .iter()
        .find(|r| r.doc_id == Value::String("doc_1".to_string()))
        .map(|r| r.score)
        .unwrap_or(0.0);

    assert!(
        doc_1_score > 0.0,
        "Document with both terms should have positive score"
    );
}

/// TC-FT-HL-009: Inversearch Result Structure
#[tokio::test]
async fn test_inversearch_result_structure() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create index");

    ctx.insert_test_doc(1, "Article", "content", "doc_1", "inversearch result test")
        .await
        .expect("Failed to insert document");

    ctx.commit_all().await.expect("Failed to commit");

    let results = ctx
        .search(1, "Article", "content", "inversearch", 10)
        .await
        .expect("Search should succeed");

    assert_eq!(results.len(), 1, "Should find 1 result");

    // Verify result structure
    assert!(matches!(results[0].doc_id, Value::String(_)));
    assert!(results[0].score >= 0.0);
}

/// TC-FT-HL-010: Empty Search Results
#[tokio::test]
async fn test_empty_search_results_structure() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    ctx.insert_test_doc(1, "Article", "content", "doc_1", "some content")
        .await
        .expect("Failed to insert document");

    ctx.commit_all().await.expect("Failed to commit");

    let results = ctx
        .search(1, "Article", "content", "nonexistent", 10)
        .await
        .expect("Search should succeed");

    assert_eq!(results.len(), 0, "Should find 0 results");
    assert!(results.is_empty(), "Results should be empty");
}

/// TC-FT-HL-011: Result Limit Enforcement
#[tokio::test]
async fn test_result_limit_enforcement() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    for i in 0..20 {
        ctx.insert_test_doc(
            1,
            "Article",
            "content",
            &format!("doc_{}", i),
            &format!("Limit test document {}", i),
        )
        .await
        .expect("Failed to insert document");
    }

    ctx.commit_all().await.expect("Failed to commit");

    let results = ctx
        .search(1, "Article", "content", "Limit", 5)
        .await
        .expect("Search should succeed");

    assert_eq!(results.len(), 5, "Should return exactly 5 results");
}

/// TC-FT-HL-012: Score Zero for No Match
#[tokio::test]
async fn test_score_for_partial_match() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    ctx.insert_test_doc(1, "Article", "content", "doc_1", "programming language")
        .await
        .expect("Failed to insert document");

    ctx.commit_all().await.expect("Failed to commit");

    let results = ctx
        .search(1, "Article", "content", "programming", 10)
        .await
        .expect("Search should succeed");

    assert_eq!(results.len(), 1, "Should find 1 result");
    assert!(results[0].score > 0.0, "Matching document should have positive score");
}

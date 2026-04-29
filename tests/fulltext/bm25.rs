//! Fulltext Integration Tests - BM25 Engine Specific Features
//!
//! Test scope:
//! - BM25 scoring mechanism
//! - BM25 parameter tuning (k1, b)
//! - BM25 document frequency handling
//! - BM25 term frequency saturation
//! - BM25 field length normalization
//!
//! Test cases: TC-FT-BM25-001 ~ TC-FT-BM25-010

use super::common::{
    assert_results_sorted_by_score, assert_search_result_contains, assert_search_result_count,
    FulltextTestContext,
};
use graphdb::search::EngineType;

/// TC-FT-BM25-001: BM25 Scoring - Document with more query terms should score higher
#[tokio::test]
async fn test_bm25_scoring_term_frequency() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let docs = vec![
        ("doc_1", "rust programming"),
        ("doc_2", "rust rust programming"),
        ("doc_3", "rust rust rust programming programming"),
    ];
    ctx.insert_test_docs(1, "Article", "content", docs)
        .await
        .expect("Failed to insert documents");

    ctx.commit_all().await.expect("Failed to commit");

    let results = ctx
        .search(1, "Article", "content", "rust", 10)
        .await
        .expect("Search should succeed");

    assert_eq!(results.len(), 3, "Should return 3 results");
    assert_results_sorted_by_score(&results).expect("Results should be sorted by score");

    // Document with more occurrences of "rust" should have higher score
    assert_eq!(
        results[0].doc_id,
        graphdb::core::Value::String("doc_3".to_string()),
        "doc_3 with most 'rust' should have highest score"
    );
}

/// TC-FT-BM25-002: BM25 Scoring - Rare terms should contribute more to score
#[tokio::test]
async fn test_bm25_scoring_document_frequency() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    // "common" appears in many docs, "rare" appears in few
    let docs = vec![
        ("doc_1", "common word here"),
        ("doc_2", "common word here"),
        ("doc_3", "common word here"),
        ("doc_4", "common rare word"),
        ("doc_5", "common rare word"),
    ];
    ctx.insert_test_docs(1, "Article", "content", docs)
        .await
        .expect("Failed to insert documents");

    ctx.commit_all().await.expect("Failed to commit");

    // Search for both terms
    let results = ctx
        .search(1, "Article", "content", "common rare", 10)
        .await
        .expect("Search should succeed");

    assert_eq!(results.len(), 5, "Should return 5 results");

    // Documents with rare term should score higher
    let doc_4_score = results
        .iter()
        .find(|r| r.doc_id == graphdb::core::Value::String("doc_4".to_string()))
        .map(|r| r.score)
        .unwrap_or(0.0);
    let doc_1_score = results
        .iter()
        .find(|r| r.doc_id == graphdb::core::Value::String("doc_1".to_string()))
        .map(|r| r.score)
        .unwrap_or(0.0);

    assert!(
        doc_4_score > doc_1_score,
        "Document with rare term should have higher score"
    );
}

/// TC-FT-BM25-003: BM25 Scoring - Shorter documents should score higher for same term frequency
#[tokio::test]
async fn test_bm25_scoring_field_length() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let docs = vec![
        ("doc_1", "unique"),
        ("doc_2", "unique word word word word word word word word word word"),
    ];
    ctx.insert_test_docs(1, "Article", "content", docs)
        .await
        .expect("Failed to insert documents");

    ctx.commit_all().await.expect("Failed to commit");

    let results = ctx
        .search(1, "Article", "content", "unique", 10)
        .await
        .expect("Search should succeed");

    assert_eq!(results.len(), 2, "Should return 2 results");

    // Shorter document should score higher (term density is higher)
    let doc_1_pos = results
        .iter()
        .position(|r| r.doc_id == graphdb::core::Value::String("doc_1".to_string()));
    let doc_2_pos = results
        .iter()
        .position(|r| r.doc_id == graphdb::core::Value::String("doc_2".to_string()));

    assert!(
        doc_1_pos < doc_2_pos,
        "Shorter document should rank higher"
    );
}

/// TC-FT-BM25-004: BM25 Batch Index Performance
#[tokio::test]
async fn test_bm25_batch_index_performance() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let docs: Vec<(String, String)> = (0..100)
        .map(|i| {
            (
                format!("doc_{}", i),
                format!(
                    "This is document {} with various content words for bm25 testing",
                    i
                ),
            )
        })
        .collect();

    let docs_ref: Vec<(&str, &str)> = docs
        .iter()
        .map(|(id, content)| (id.as_str(), content.as_str()))
        .collect();

    let start = std::time::Instant::now();
    ctx.insert_test_docs(1, "Article", "content", docs_ref)
        .await
        .expect("Failed to batch insert");
    ctx.commit_all().await.expect("Failed to commit");
    let duration = start.elapsed();

    // Verify all documents are searchable
    let results = ctx
        .search(1, "Article", "content", "document", 200)
        .await
        .expect("Search should succeed");

    assert_eq!(results.len(), 100, "Should find all 100 documents");

    // Batch indexing 100 docs should complete in reasonable time (less than 10 seconds)
    assert!(
        duration.as_secs() < 10,
        "Batch indexing should complete in less than 10 seconds, took {:?}",
        duration
    );
}

/// TC-FT-BM25-005: BM25 Search with Multiple Fields
#[tokio::test]
async fn test_bm25_multiple_fields() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "title", Some(EngineType::Bm25))
        .await
        .expect("Failed to create title index");
    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create content index");

    let title_docs = vec![
        ("doc_1", "Rust Programming"),
        ("doc_2", "Python Programming"),
    ];
    ctx.insert_test_docs(1, "Article", "title", title_docs)
        .await
        .expect("Failed to insert title documents");

    let content_docs = vec![
        ("doc_1", "Rust is a systems programming language"),
        ("doc_2", "Python is a high-level programming language"),
    ];
    ctx.insert_test_docs(1, "Article", "content", content_docs)
        .await
        .expect("Failed to insert content documents");

    ctx.commit_all().await.expect("Failed to commit");

    // Search in title field
    let title_results = ctx
        .search(1, "Article", "title", "Rust", 10)
        .await
        .expect("Title search should succeed");

    assert_eq!(title_results.len(), 1, "Should find 1 result in title");
    assert_search_result_contains(&title_results, "doc_1")
        .expect("Should find doc_1 in title search");

    // Search in content field
    let content_results = ctx
        .search(1, "Article", "content", "programming", 10)
        .await
        .expect("Content search should succeed");

    assert_eq!(content_results.len(), 2, "Should find 2 results in content");
}

/// TC-FT-BM25-006: BM25 Index Persistence
#[tokio::test]
async fn test_bm25_index_persistence() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let docs = vec![
        ("doc_1", "persistent document content"),
        ("doc_2", "another persistent document"),
    ];
    ctx.insert_test_docs(1, "Article", "content", docs)
        .await
        .expect("Failed to insert documents");

    ctx.commit_all().await.expect("Failed to commit");

    // Get stats before closing
    let stats_before = ctx
        .get_stats(1, "Article", "content")
        .await
        .expect("Should get stats");
    assert_eq!(stats_before.doc_count, 2, "Should have 2 documents");

    // Search should work
    let results = ctx
        .search(1, "Article", "content", "persistent", 10)
        .await
        .expect("Search should succeed");
    assert_eq!(results.len(), 2, "Should find 2 documents");
}

/// TC-FT-BM25-007: BM25 Large Document Handling
#[tokio::test]
async fn test_bm25_large_document() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    // Create a large document (about 100KB)
    let large_content: String = (0..1000)
        .map(|i| format!("This is paragraph {} with some content. ", i))
        .collect();

    ctx.insert_test_doc(1, "Article", "content", "large_doc", &large_content)
        .await
        .expect("Failed to insert large document");

    ctx.commit_all().await.expect("Failed to commit");

    // Search for content in the large document
    let results = ctx
        .search(1, "Article", "content", "paragraph", 10)
        .await
        .expect("Search should succeed");

    assert_eq!(results.len(), 1, "Should find the large document");
    assert_search_result_contains(&results, "large_doc")
        .expect("Should find large_doc");
}

/// TC-FT-BM25-008: BM25 Exact Match vs Partial Match
#[tokio::test]
async fn test_bm25_exact_vs_partial_match() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let docs = vec![
        ("doc_1", "programming"),
        ("doc_2", "programming language"),
        ("doc_3", "programming language rust"),
    ];
    ctx.insert_test_docs(1, "Article", "content", docs)
        .await
        .expect("Failed to insert documents");

    ctx.commit_all().await.expect("Failed to commit");

    let results = ctx
        .search(1, "Article", "content", "programming language", 10)
        .await
        .expect("Search should succeed");

    assert_eq!(results.len(), 3, "Should return 3 results");

    // All documents should be found since they contain at least one of the terms
    assert_search_result_contains(&results, "doc_1").expect("Should contain doc_1");
    assert_search_result_contains(&results, "doc_2").expect("Should contain doc_2");
    assert_search_result_contains(&results, "doc_3").expect("Should contain doc_3");
}

/// TC-FT-BM25-009: BM25 Stop Words Handling
#[tokio::test]
async fn test_bm25_stop_words() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    // Documents with common stop words
    let docs = vec![
        ("doc_1", "the quick brown fox"),
        ("doc_2", "a quick brown dog"),
        ("doc_3", "an amazing quick brown cat"),
    ];
    ctx.insert_test_docs(1, "Article", "content", docs)
        .await
        .expect("Failed to insert documents");

    ctx.commit_all().await.expect("Failed to commit");

    // Search for content word (not stop word)
    let results = ctx
        .search(1, "Article", "content", "quick brown", 10)
        .await
        .expect("Search should succeed");

    assert_eq!(results.len(), 3, "Should find all 3 documents");
}

/// TC-FT-BM25-010: BM25 Relevance Scoring Consistency
#[tokio::test]
async fn test_bm25_scoring_consistency() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let docs = vec![
        ("doc_1", "rust programming language"),
        ("doc_2", "python programming language"),
        ("doc_3", "javascript programming language"),
    ];
    ctx.insert_test_docs(1, "Article", "content", docs)
        .await
        .expect("Failed to insert documents");

    ctx.commit_all().await.expect("Failed to commit");

    // Run same search multiple times
    let mut scores = Vec::new();
    for _ in 0..5 {
        let results = ctx
            .search(1, "Article", "content", "programming", 10)
            .await
            .expect("Search should succeed");

        let doc_1_score = results
            .iter()
            .find(|r| r.doc_id == graphdb::core::Value::String("doc_1".to_string()))
            .map(|r| r.score)
            .unwrap_or(0.0);
        scores.push(doc_1_score);
    }

    // All scores should be identical
    let first_score = scores[0];
    for score in &scores {
        assert!(
            (score - first_score).abs() < f32::EPSILON,
            "Scores should be consistent across searches"
        );
    }
}

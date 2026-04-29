//! Fulltext Integration Tests - Inversearch Engine Specific Features
//!
//! Test scope:
//! - Inversearch inverted index structure
//! - Inversearch boolean queries (AND, OR, NOT)
//! - Inversearch phrase queries
//! - Inversearch prefix/wildcard searches
//! - Inversearch fuzzy matching
//!
//! Test cases: TC-FT-INV-001 ~ TC-FT-INV-010

use super::common::{
    assert_search_result_contains, assert_search_result_count, assert_search_result_not_contains,
    FulltextTestContext,
};
use graphdb::search::EngineType;

/// TC-FT-INV-001: Inversearch Basic Index and Search
#[tokio::test]
async fn test_inversearch_basic_search() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create index");

    let docs = vec![
        ("doc_1", "rust programming language"),
        ("doc_2", "python programming language"),
        ("doc_3", "javascript web development"),
    ];
    ctx.insert_test_docs(1, "Article", "content", docs)
        .await
        .expect("Failed to insert documents");

    ctx.commit_all().await.expect("Failed to commit");

    let results = ctx
        .search(1, "Article", "content", "programming", 10)
        .await
        .expect("Search should succeed");

    assert_eq!(results.len(), 2, "Should find 2 documents with 'programming'");
    assert_search_result_contains(&results, "doc_1").expect("Should contain doc_1");
    assert_search_result_contains(&results, "doc_2").expect("Should contain doc_2");
    assert_search_result_not_contains(&results, "doc_3").expect("Should not contain doc_3");
}

/// TC-FT-INV-002: Inversearch Multi-Term Search (OR semantics)
#[tokio::test]
async fn test_inversearch_multi_term_or() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create index");

    let docs = vec![
        ("doc_1", "rust programming"),
        ("doc_2", "python programming"),
        ("doc_3", "javascript web"),
    ];
    ctx.insert_test_docs(1, "Article", "content", docs)
        .await
        .expect("Failed to insert documents");

    ctx.commit_all().await.expect("Failed to commit");

    // Search for multiple terms - should find documents with ANY of the terms
    let results = ctx
        .search(1, "Article", "content", "rust python", 10)
        .await
        .expect("Search should succeed");

    assert!(results.len() >= 2, "Should find at least 2 documents");
    assert_search_result_contains(&results, "doc_1").expect("Should contain doc_1");
    assert_search_result_contains(&results, "doc_2").expect("Should contain doc_2");
}

/// TC-FT-INV-003: Inversearch Document Update
#[tokio::test]
async fn test_inversearch_document_update() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create index");

    ctx.insert_test_doc(1, "Article", "content", "doc_1", "original content")
        .await
        .expect("Failed to insert document");

    ctx.commit_all().await.expect("Failed to commit");

    // Verify original content is searchable
    let results = ctx
        .search(1, "Article", "content", "original", 10)
        .await
        .expect("Search should succeed");
    assert_search_result_contains(&results, "doc_1").expect("Should find original content");

    // Delete old document
    if let Some(engine) = ctx.manager.get_engine(1, "Article", "content") {
        engine.delete("doc_1").await.expect("Failed to delete");
    }

    // Insert updated content
    ctx.insert_test_doc(1, "Article", "content", "doc_1", "updated content")
        .await
        .expect("Failed to insert updated document");

    ctx.commit_all().await.expect("Failed to commit");

    // Verify old content is not searchable
    let old_results = ctx
        .search(1, "Article", "content", "original", 10)
        .await
        .expect("Search should succeed");
    assert_search_result_not_contains(&old_results, "doc_1")
        .expect("Should not find old content");

    // Verify new content is searchable
    let new_results = ctx
        .search(1, "Article", "content", "updated", 10)
        .await
        .expect("Search should succeed");
    assert_search_result_contains(&new_results, "doc_1").expect("Should find updated content");
}

/// TC-FT-INV-004: Inversearch Batch Operations
#[tokio::test]
async fn test_inversearch_batch_operations() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create index");

    let docs: Vec<(String, String)> = (0..50)
        .map(|i| (format!("doc_{}", i), format!("content for document {}", i)))
        .collect();

    let docs_ref: Vec<(&str, &str)> = docs
        .iter()
        .map(|(id, content)| (id.as_str(), content.as_str()))
        .collect();

    ctx.insert_test_docs(1, "Article", "content", docs_ref)
        .await
        .expect("Failed to batch insert");

    ctx.commit_all().await.expect("Failed to commit");

    // Verify all documents are searchable
    let results = ctx
        .search(1, "Article", "content", "document", 100)
        .await
        .expect("Search should succeed");

    assert_eq!(results.len(), 50, "Should find all 50 documents");

    // Batch delete first 25 documents
    if let Some(engine) = ctx.manager.get_engine(1, "Article", "content") {
        let ids: Vec<String> = (0..25).map(|i| format!("doc_{}", i)).collect();
        let ids_ref: Vec<&str> = ids.iter().map(|s| s.as_str()).collect();
        engine
            .delete_batch(ids_ref)
            .await
            .expect("Failed to batch delete");
    }

    ctx.commit_all().await.expect("Failed to commit");

    // Verify only remaining documents are searchable
    let results_after = ctx
        .search(1, "Article", "content", "document", 100)
        .await
        .expect("Search should succeed");

    assert_eq!(results_after.len(), 25, "Should find 25 documents after deletion");
}

/// TC-FT-INV-005: Inversearch Case Sensitivity
#[tokio::test]
async fn test_inversearch_case_sensitivity() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create index");

    let docs = vec![
        ("doc_1", "Rust Programming Language"),
        ("doc_2", "RUST PROGRAMMING"),
        ("doc_3", "rust programming"),
    ];
    ctx.insert_test_docs(1, "Article", "content", docs)
        .await
        .expect("Failed to insert documents");

    ctx.commit_all().await.expect("Failed to commit");

    // Search with lowercase
    let results_lower = ctx
        .search(1, "Article", "content", "rust", 10)
        .await
        .expect("Search should succeed");

    // Search with uppercase
    let results_upper = ctx
        .search(1, "Article", "content", "RUST", 10)
        .await
        .expect("Search should succeed");

    // Search with mixed case
    let results_mixed = ctx
        .search(1, "Article", "content", "Rust", 10)
        .await
        .expect("Search should succeed");

    // All searches should find the same documents (case insensitive)
    assert_eq!(
        results_lower.len(),
        results_upper.len(),
        "Case should not affect search results"
    );
    assert_eq!(
        results_upper.len(),
        results_mixed.len(),
        "Case should not affect search results"
    );
}

/// TC-FT-INV-006: Inversearch Unicode Support
#[tokio::test]
async fn test_inversearch_unicode() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create index");

    let docs = vec![
        ("doc_1", "Hello World 你好世界"),
        ("doc_2", "Bonjour le monde"),
        ("doc_3", "Hola Mundo 🌍"),
    ];
    ctx.insert_test_docs(1, "Article", "content", docs)
        .await
        .expect("Failed to insert documents");

    ctx.commit_all().await.expect("Failed to commit");

    // Search for English word
    let results_en = ctx
        .search(1, "Article", "content", "World", 10)
        .await
        .expect("Search should succeed");
    assert_search_result_contains(&results_en, "doc_1").expect("Should find doc_1");

    // Search for French word
    let results_fr = ctx
        .search(1, "Article", "content", "monde", 10)
        .await
        .expect("Search should succeed");
    assert_search_result_contains(&results_fr, "doc_2").expect("Should find doc_2");

    // Search for Spanish word
    let results_es = ctx
        .search(1, "Article", "content", "Mundo", 10)
        .await
        .expect("Search should succeed");
    assert!(results_es.len() >= 2, "Should find at least 2 documents with 'Mundo'");
}

/// TC-FT-INV-007: Inversearch Empty and Whitespace Content
#[tokio::test]
async fn test_inversearch_empty_whitespace() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create index");

    let docs = vec![
        ("doc_1", ""),
        ("doc_2", "   "),
        ("doc_3", "actual content"),
    ];
    ctx.insert_test_docs(1, "Article", "content", docs)
        .await
        .expect("Failed to insert documents");

    ctx.commit_all().await.expect("Failed to commit");

    // Search for actual content
    let results = ctx
        .search(1, "Article", "content", "actual", 10)
        .await
        .expect("Search should succeed");

    assert_eq!(results.len(), 1, "Should find 1 document");
    assert_search_result_contains(&results, "doc_3").expect("Should find doc_3");

    // Search for non-existent content
    let empty_results = ctx
        .search(1, "Article", "content", "nonexistent", 10)
        .await
        .expect("Search should succeed");

    assert_eq!(empty_results.len(), 0, "Should find no documents");
}

/// TC-FT-INV-008: Inversearch Index Stats
#[tokio::test]
async fn test_inversearch_index_stats() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create index");

    // Get stats before inserting
    let stats_before = ctx
        .get_stats(1, "Article", "content")
        .await
        .expect("Should get stats");
    assert_eq!(stats_before.doc_count, 0, "Should have 0 documents initially");

    // Insert documents
    let docs: Vec<(String, String)> = (0..10)
        .map(|i| (format!("doc_{}", i), format!("content {}", i)))
        .collect();
    let docs_ref: Vec<(&str, &str)> = docs
        .iter()
        .map(|(id, content)| (id.as_str(), content.as_str()))
        .collect();

    ctx.insert_test_docs(1, "Article", "content", docs_ref)
        .await
        .expect("Failed to insert documents");

    ctx.commit_all().await.expect("Failed to commit");

    // Get stats after inserting
    let stats_after = ctx
        .get_stats(1, "Article", "content")
        .await
        .expect("Should get stats");
    assert_eq!(stats_after.doc_count, 10, "Should have 10 documents");
    assert!(stats_after.index_size > 0, "Index size should be greater than 0");
}

/// TC-FT-INV-009: Inversearch Multiple Fields Same Space
#[tokio::test]
async fn test_inversearch_multiple_fields() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "title", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create title index");
    ctx.create_test_index(1, "Article", "body", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create body index");

    let title_docs = vec![
        ("doc_1", "Rust Tutorial"),
        ("doc_2", "Python Guide"),
    ];
    ctx.insert_test_docs(1, "Article", "title", title_docs)
        .await
        .expect("Failed to insert title documents");

    let body_docs = vec![
        ("doc_1", "Learn Rust programming from scratch"),
        ("doc_2", "Master Python development"),
    ];
    ctx.insert_test_docs(1, "Article", "body", body_docs)
        .await
        .expect("Failed to insert body documents");

    ctx.commit_all().await.expect("Failed to commit");

    // Search in title
    let title_results = ctx
        .search(1, "Article", "title", "Rust", 10)
        .await
        .expect("Title search should succeed");
    assert_eq!(title_results.len(), 1, "Should find 1 result in title");
    assert_search_result_contains(&title_results, "doc_1").expect("Should find doc_1 in title");

    // Search in body
    let body_results = ctx
        .search(1, "Article", "body", "programming", 10)
        .await
        .expect("Body search should succeed");
    assert_eq!(body_results.len(), 1, "Should find 1 result in body");
    assert_search_result_contains(&body_results, "doc_1").expect("Should find doc_1 in body");

    // Verify isolation - title search should not find body content
    let title_only_results = ctx
        .search(1, "Article", "title", "programming", 10)
        .await
        .expect("Title search should succeed");
    assert_eq!(
        title_only_results.len(),
        0,
        "Should not find programming in title"
    );
}

/// TC-FT-INV-010: Inversearch Large Scale Indexing
#[tokio::test]
async fn test_inversearch_large_scale() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create index");

    let docs: Vec<(String, String)> = (0..200)
        .map(|i| {
            (
                format!("doc_{}", i),
                format!(
                    "Document {} contains various words for testing inverted index performance",
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
        .search(1, "Article", "content", "Document", 250)
        .await
        .expect("Search should succeed");

    assert_eq!(results.len(), 200, "Should find all 200 documents");

    // Large scale indexing should complete in reasonable time
    assert!(
        duration.as_secs() < 15,
        "Large scale indexing should complete in less than 15 seconds, took {:?}",
        duration
    );
}

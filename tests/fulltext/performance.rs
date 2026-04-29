//! Fulltext Integration Tests - Performance Tests
//!
//! Test scope:
//! - BM25 indexing performance
//! - Inversearch indexing performance
//! - Search performance comparison
//! - Large dataset handling
//! - Memory usage patterns
//!
//! Test cases: TC-FT-PERF-001 ~ TC-FT-PERF-008

use super::common::{
    assert_search_result_count, FulltextTestContext,
};
use graphdb::search::EngineType;
use std::time::Instant;

/// TC-FT-PERF-001: BM25 Indexing Performance - Small Batch
#[tokio::test]
async fn test_bm25_indexing_performance_small() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let num_docs = 100;
    let docs: Vec<(String, String)> = (0..num_docs)
        .map(|i| {
            (
                format!("doc_{}", i),
                format!("Performance test document number {} with sample content", i),
            )
        })
        .collect();

    let docs_ref: Vec<(&str, &str)> = docs
        .iter()
        .map(|(id, content)| (id.as_str(), content.as_str()))
        .collect();

    let start = Instant::now();
    ctx.insert_test_docs(1, "Article", "content", docs_ref)
        .await
        .expect("Failed to batch insert");
    ctx.commit_all().await.expect("Failed to commit");
    let duration = start.elapsed();

    // Verify all documents are searchable
    let results = ctx
        .search(1, "Article", "content", "Performance", 200)
        .await
        .expect("Search should succeed");

    assert_eq!(results.len(), num_docs, "Should find all documents");

    // Performance assertion: 100 docs should index in less than 5 seconds
    assert!(
        duration.as_secs() < 5,
        "Indexing 100 documents should take less than 5 seconds, took {:?}",
        duration
    );
}

/// TC-FT-PERF-002: Inversearch Indexing Performance - Small Batch
#[tokio::test]
async fn test_inversearch_indexing_performance_small() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create index");

    let num_docs = 100;
    let docs: Vec<(String, String)> = (0..num_docs)
        .map(|i| {
            (
                format!("doc_{}", i),
                format!("Performance test document number {} with sample content", i),
            )
        })
        .collect();

    let docs_ref: Vec<(&str, &str)> = docs
        .iter()
        .map(|(id, content)| (id.as_str(), content.as_str()))
        .collect();

    let start = Instant::now();
    ctx.insert_test_docs(1, "Article", "content", docs_ref)
        .await
        .expect("Failed to batch insert");
    ctx.commit_all().await.expect("Failed to commit");
    let duration = start.elapsed();

    // Verify all documents are searchable
    let results = ctx
        .search(1, "Article", "content", "Performance", 200)
        .await
        .expect("Search should succeed");

    assert_eq!(results.len(), num_docs, "Should find all documents");

    // Performance assertion: 100 docs should index in less than 5 seconds
    assert!(
        duration.as_secs() < 5,
        "Indexing 100 documents should take less than 5 seconds, took {:?}",
        duration
    );
}

/// TC-FT-PERF-003: BM25 Search Performance
#[tokio::test]
async fn test_bm25_search_performance() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    // Insert documents
    let num_docs = 500;
    let docs: Vec<(String, String)> = (0..num_docs)
        .map(|i| {
            (
                format!("doc_{}", i),
                format!("Search performance test document number {}", i),
            )
        })
        .collect();

    let docs_ref: Vec<(&str, &str)> = docs
        .iter()
        .map(|(id, content)| (id.as_str(), content.as_str()))
        .collect();

    ctx.insert_test_docs(1, "Article", "content", docs_ref)
        .await
        .expect("Failed to insert documents");
    ctx.commit_all().await.expect("Failed to commit");

    // Measure search performance
    let num_searches = 100;
    let start = Instant::now();

    for _ in 0..num_searches {
        let _results = ctx
            .search(1, "Article", "content", "performance", 50)
            .await
            .expect("Search should succeed");
    }

    let duration = start.elapsed();
    let avg_search_time = duration.as_millis() as f64 / num_searches as f64;

    // Performance assertion: average search should take less than 100ms
    assert!(
        avg_search_time < 100.0,
        "Average search time should be less than 100ms, got {:.2}ms",
        avg_search_time
    );
}

/// TC-FT-PERF-004: Inversearch Search Performance
#[tokio::test]
async fn test_inversearch_search_performance() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create index");

    // Insert documents
    let num_docs = 500;
    let docs: Vec<(String, String)> = (0..num_docs)
        .map(|i| {
            (
                format!("doc_{}", i),
                format!("Search performance test document number {}", i),
            )
        })
        .collect();

    let docs_ref: Vec<(&str, &str)> = docs
        .iter()
        .map(|(id, content)| (id.as_str(), content.as_str()))
        .collect();

    ctx.insert_test_docs(1, "Article", "content", docs_ref)
        .await
        .expect("Failed to insert documents");
    ctx.commit_all().await.expect("Failed to commit");

    // Measure search performance
    let num_searches = 100;
    let start = Instant::now();

    for _ in 0..num_searches {
        let _results = ctx
            .search(1, "Article", "content", "performance", 50)
            .await
            .expect("Search should succeed");
    }

    let duration = start.elapsed();
    let avg_search_time = duration.as_millis() as f64 / num_searches as f64;

    // Performance assertion: average search should take less than 100ms
    assert!(
        avg_search_time < 100.0,
        "Average search time should be less than 100ms, got {:.2}ms",
        avg_search_time
    );
}

/// TC-FT-PERF-005: Large Dataset Indexing - BM25
#[tokio::test]
async fn test_bm25_large_dataset_indexing() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let num_docs = 1000;
    let docs: Vec<(String, String)> = (0..num_docs)
        .map(|i| {
            (
                format!("doc_{}", i),
                format!(
                    "Large dataset test document number {} with various words for testing bm25 performance",
                    i
                ),
            )
        })
        .collect();

    let docs_ref: Vec<(&str, &str)> = docs
        .iter()
        .map(|(id, content)| (id.as_str(), content.as_str()))
        .collect();

    let start = Instant::now();
    ctx.insert_test_docs(1, "Article", "content", docs_ref)
        .await
        .expect("Failed to batch insert");
    ctx.commit_all().await.expect("Failed to commit");
    let duration = start.elapsed();

    // Verify all documents are searchable
    let results = ctx
        .search(1, "Article", "content", "dataset", 1500)
        .await
        .expect("Search should succeed");

    assert_eq!(results.len(), num_docs, "Should find all documents");

    // Performance assertion: 1000 docs should index in less than 30 seconds
    assert!(
        duration.as_secs() < 30,
        "Indexing 1000 documents should take less than 30 seconds, took {:?}",
        duration
    );
}

/// TC-FT-PERF-006: Large Dataset Indexing - Inversearch
#[tokio::test]
async fn test_inversearch_large_dataset_indexing() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create index");

    let num_docs = 1000;
    let docs: Vec<(String, String)> = (0..num_docs)
        .map(|i| {
            (
                format!("doc_{}", i),
                format!(
                    "Large dataset test document number {} with various words for testing inversearch performance",
                    i
                ),
            )
        })
        .collect();

    let docs_ref: Vec<(&str, &str)> = docs
        .iter()
        .map(|(id, content)| (id.as_str(), content.as_str()))
        .collect();

    let start = Instant::now();
    ctx.insert_test_docs(1, "Article", "content", docs_ref)
        .await
        .expect("Failed to batch insert");
    ctx.commit_all().await.expect("Failed to commit");
    let duration = start.elapsed();

    // Verify all documents are searchable
    let results = ctx
        .search(1, "Article", "content", "dataset", 1500)
        .await
        .expect("Search should succeed");

    assert_eq!(results.len(), num_docs, "Should find all documents");

    // Performance assertion: 1000 docs should index in less than 30 seconds
    assert!(
        duration.as_secs() < 30,
        "Indexing 1000 documents should take less than 30 seconds, took {:?}",
        duration
    );
}

/// TC-FT-PERF-007: Index Stats Performance
#[tokio::test]
async fn test_index_stats_performance() {
    let ctx = FulltextTestContext::new();

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    // Insert documents
    let num_docs = 500;
    let docs: Vec<(String, String)> = (0..num_docs)
        .map(|i| (format!("doc_{}", i), format!("Stats test document {}", i)))
        .collect();

    let docs_ref: Vec<(&str, &str)> = docs
        .iter()
        .map(|(id, content)| (id.as_str(), content.as_str()))
        .collect();

    ctx.insert_test_docs(1, "Article", "content", docs_ref)
        .await
        .expect("Failed to insert documents");
    ctx.commit_all().await.expect("Failed to commit");

    // Measure stats retrieval performance
    let num_stats_calls = 100;
    let start = Instant::now();

    for _ in 0..num_stats_calls {
        let _stats = ctx
            .get_stats(1, "Article", "content")
            .await
            .expect("Should get stats");
    }

    let duration = start.elapsed();
    let avg_stats_time = duration.as_millis() as f64 / num_stats_calls as f64;

    // Performance assertion: average stats call should take less than 50ms
    assert!(
        avg_stats_time < 50.0,
        "Average stats retrieval time should be less than 50ms, got {:.2}ms",
        avg_stats_time
    );
}

/// TC-FT-PERF-008: Multi-Field Search Performance
#[tokio::test]
async fn test_multi_field_search_performance() {
    let ctx = FulltextTestContext::new();

    // Create multiple indexes
    ctx.create_test_index(1, "Article", "title", Some(EngineType::Bm25))
        .await
        .expect("Failed to create title index");
    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create content index");
    ctx.create_test_index(1, "Article", "summary", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create summary index");

    // Insert documents to all fields
    let num_docs = 100;
    for i in 0..num_docs {
        ctx.insert_test_doc(1, "Article", "title", &format!("doc_{}", i), &format!("Title {}", i))
            .await
            .expect("Failed to insert title");
        ctx.insert_test_doc(1, "Article", "content", &format!("doc_{}", i), &format!("Content {}", i))
            .await
            .expect("Failed to insert content");
        ctx.insert_test_doc(1, "Article", "summary", &format!("doc_{}", i), &format!("Summary {}", i))
            .await
            .expect("Failed to insert summary");
    }

    ctx.commit_all().await.expect("Failed to commit");

    // Measure multi-field search performance
    let num_searches = 50;
    let start = Instant::now();

    for _ in 0..num_searches {
        let _ = ctx
            .search(1, "Article", "title", "Title", 100)
            .await
            .expect("Title search should succeed");
        let _ = ctx
            .search(1, "Article", "content", "Content", 100)
            .await
            .expect("Content search should succeed");
        let _ = ctx
            .search(1, "Article", "summary", "Summary", 100)
            .await
            .expect("Summary search should succeed");
    }

    let duration = start.elapsed();
    let avg_search_time = duration.as_millis() as f64 / (num_searches as f64 * 3.0);

    // Performance assertion: average search across any field should take less than 100ms
    assert!(
        avg_search_time < 100.0,
        "Average multi-field search time should be less than 100ms, got {:.2}ms",
        avg_search_time
    );
}

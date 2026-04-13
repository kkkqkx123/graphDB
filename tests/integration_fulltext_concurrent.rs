//! Fulltext Integration Tests - Concurrency
//!
//! Test scope:
//! - Concurrent inserts
//! - Concurrent searches
//! - Concurrent insert and search mix
//! - Concurrent updates to same document
//!
//! Test cases: TC-FT-022 ~ TC-FT-025

mod common;

use common::fulltext_helpers::{
    assert_search_result_contains, assert_search_result_count, FulltextTestContext,
};
use graphdb::search::EngineType;
use std::sync::Arc;
use tokio::sync::Barrier;

// ==================== Concurrency Tests ====================

/// TC-FT-022: Concurrent Inserts
#[tokio::test]
async fn test_concurrent_inserts() {
    let ctx = Arc::new(FulltextTestContext::new());
    let num_tasks = 100;
    let barrier = Arc::new(Barrier::new(num_tasks));

    // Create index
    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    // Spawn concurrent tasks
    let mut handles = vec![];
    for i in 0..num_tasks {
        let ctx_clone = Arc::clone(&ctx);
        let barrier_clone = Arc::clone(&barrier);

        let handle = tokio::spawn(async move {
            // Wait for all tasks to be ready
            barrier_clone.wait().await;

            // Insert document
            ctx_clone
                .insert_test_doc(
                    1,
                    "Article",
                    "content",
                    &format!("doc_{}", i),
                    &format!("Concurrent content {}", i),
                )
                .await
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        let result = handle.await.expect("Task panicked");
        assert!(result.is_ok(), "Document insertion should succeed");
    }

    // Commit all
    ctx.commit_all().await.expect("Failed to commit");

    // Verify all documents are searchable
    let results = ctx
        .search(1, "Article", "content", "Concurrent", 200)
        .await
        .expect("Search should succeed");

    assert_search_result_count(&results, num_tasks)
        .unwrap_or_else(|_| panic!("Should find all {} documents", num_tasks));

    // Verify each document is present
    for i in 0..num_tasks {
        assert_search_result_contains(&results, &format!("doc_{}", i))
            .unwrap_or_else(|_| panic!("Should contain doc_{}", i));
    }
}

/// TC-FT-023: Concurrent Searches
#[tokio::test]
async fn test_concurrent_searches() {
    let ctx = Arc::new(FulltextTestContext::new());
    let num_docs = 50;
    let num_searches = 100;
    let barrier = Arc::new(Barrier::new(num_searches));

    // Create index
    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    // Insert test documents
    for i in 0..num_docs {
        ctx.insert_test_doc(
            1,
            "Article",
            "content",
            &format!("doc_{}", i),
            &format!("Search test content {}", i),
        )
        .await
        .expect("Failed to insert document");
    }

    // Commit
    ctx.commit_all().await.expect("Failed to commit");

    // Spawn concurrent search tasks
    let mut handles = vec![];
    for _ in 0..num_searches {
        let ctx_clone = Arc::clone(&ctx);
        let barrier_clone = Arc::clone(&barrier);

        let handle = tokio::spawn(async move {
            // Wait for all tasks to be ready
            barrier_clone.wait().await;

            // Execute search
            ctx_clone
                .search(1, "Article", "content", "Search", 100)
                .await
        });

        handles.push(handle);
    }

    // Wait for all search tasks to complete
    for handle in handles {
        let result = handle.await.expect("Search task panicked");
        let results = result.expect("Search should succeed");

        // Verify search results
        assert_search_result_count(&results, num_docs)
            .unwrap_or_else(|_| panic!("Should find all {} documents", num_docs));
    }
}

/// TC-FT-024: Concurrent Insert and Search
#[tokio::test]
async fn test_concurrent_insert_and_search() {
    let ctx = Arc::new(FulltextTestContext::new());
    let num_inserts = 50;
    let num_searches = 50;

    // Create index
    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    // Insert initial documents
    for i in 0..10 {
        ctx.insert_test_doc(
            1,
            "Article",
            "content",
            &format!("initial_doc_{}", i),
            &format!("Initial content {}", i),
        )
        .await
        .expect("Failed to insert initial document");
    }

    // Commit initial documents
    ctx.commit_all().await.expect("Failed to commit");

    // Spawn concurrent insert tasks
    let mut insert_handles = vec![];
    for i in 0..num_inserts {
        let ctx_clone = Arc::clone(&ctx);

        let handle = tokio::spawn(async move {
            ctx_clone
                .insert_test_doc(
                    1,
                    "Article",
                    "content",
                    &format!("concurrent_doc_{}", i),
                    &format!("Concurrent insert {}", i),
                )
                .await
        });

        insert_handles.push(handle);
    }

    // Spawn concurrent search tasks
    let mut search_handles = vec![];
    for _ in 0..num_searches {
        let ctx_clone = Arc::clone(&ctx);

        let handle = tokio::spawn(async move {
            ctx_clone
                .search(1, "Article", "content", "content", 100)
                .await
        });

        search_handles.push(handle);
    }

    // Wait for all insert tasks to complete
    for handle in insert_handles {
        let result = handle.await.expect("Insert task panicked");
        assert!(result.is_ok(), "Insert should succeed");
    }

    // Wait for all search tasks to complete
    for handle in search_handles {
        let result = handle.await.expect("Search task panicked");
        assert!(result.is_ok(), "Search should succeed");
    }

    // Commit all
    ctx.commit_all().await.expect("Failed to commit");

    // Verify all documents are searchable
    let final_results = ctx
        .search(1, "Article", "content", "content", 200)
        .await
        .expect("Search should succeed");

    // Should have at least the initial documents
    assert!(
        final_results.len() >= 10,
        "Should have at least initial documents"
    );
}

/// TC-FT-025: Concurrent Updates to Same Document
#[tokio::test]
async fn test_concurrent_updates_same_document() {
    let ctx = Arc::new(FulltextTestContext::new());
    let num_updates = 20;

    // Create index
    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    // Insert initial document
    ctx.insert_test_doc(1, "Article", "content", "doc_1", "Initial content")
        .await
        .expect("Failed to insert document");

    // Commit
    ctx.commit_all().await.expect("Failed to commit");

    // Spawn concurrent update tasks
    let mut handles = vec![];
    for i in 0..num_updates {
        let ctx_clone = Arc::clone(&ctx);

        let handle = tokio::spawn(async move {
            // Delete old document first
            if let Some(engine) = ctx_clone.manager.get_engine(1, "Article", "content") {
                let _ = engine.delete("doc_1").await;
            }
            
            ctx_clone
                .insert_test_doc(
                    1,
                    "Article",
                    "content",
                    "doc_1",
                    &format!("Updated content {}", i),
                )
                .await
        });

        handles.push(handle);
    }

    // Wait for all updates to complete
    for handle in handles {
        let result = handle.await.expect("Update task panicked");
        assert!(result.is_ok(), "Update should succeed");
    }

    // Commit all
    ctx.commit_all().await.expect("Failed to commit");

    // Verify document is searchable (should have one of the updated versions)
    let results = ctx
        .search(1, "Article", "content", "Updated", 10)
        .await
        .expect("Search should succeed");

    // Should find the document with one of the updated contents
    assert_search_result_count(&results, 1).expect("Should find exactly 1 document");
    assert_search_result_contains(&results, "doc_1").expect("Should contain doc_1");
}

//! Fulltext Integration Tests - Concurrent Operations
//!
//! Test scope:
//! - Concurrent inserts to same index
//! - Concurrent searches
//! - Concurrent insert and search mix
//! - Concurrent updates to same document
//! - Concurrent operations on different indexes
//!
//! Test cases: TC-FT-CONC-001 ~ TC-FT-CONC-008

use super::common::{
    assert_search_result_contains, assert_search_result_count, FulltextTestContext,
};
use graphdb::search::EngineType;
use std::sync::Arc;
use tokio::sync::Barrier;

/// TC-FT-CONC-001: Concurrent Inserts to BM25 Index
#[tokio::test]
async fn test_concurrent_inserts_bm25() {
    let ctx = Arc::new(FulltextTestContext::new());
    let num_tasks = 50;
    let barrier = Arc::new(Barrier::new(num_tasks));

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    let mut handles = vec![];
    for i in 0..num_tasks {
        let ctx_clone = Arc::clone(&ctx);
        let barrier_clone = Arc::clone(&barrier);

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;

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

    for handle in handles {
        let result = handle.await.expect("Task panicked");
        assert!(result.is_ok(), "Document insertion should succeed");
    }

    ctx.commit_all().await.expect("Failed to commit");

    let results = ctx
        .search(1, "Article", "content", "Concurrent", 100)
        .await
        .expect("Search should succeed");

    assert_search_result_count(&results, num_tasks)
        .unwrap_or_else(|_| panic!("Should find all {} documents", num_tasks));

    for i in 0..num_tasks {
        assert_search_result_contains(&results, &format!("doc_{}", i))
            .unwrap_or_else(|_| panic!("Should contain doc_{}", i));
    }
}

/// TC-FT-CONC-002: Concurrent Inserts to Inversearch Index
#[tokio::test]
async fn test_concurrent_inserts_inversearch() {
    let ctx = Arc::new(FulltextTestContext::new());
    let num_tasks = 50;
    let barrier = Arc::new(Barrier::new(num_tasks));

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create index");

    let mut handles = vec![];
    for i in 0..num_tasks {
        let ctx_clone = Arc::clone(&ctx);
        let barrier_clone = Arc::clone(&barrier);

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;

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

    for handle in handles {
        let result = handle.await.expect("Task panicked");
        assert!(result.is_ok(), "Document insertion should succeed");
    }

    ctx.commit_all().await.expect("Failed to commit");

    let results = ctx
        .search(1, "Article", "content", "Concurrent", 100)
        .await
        .expect("Search should succeed");

    assert_search_result_count(&results, num_tasks)
        .unwrap_or_else(|_| panic!("Should find all {} documents", num_tasks));
}

/// TC-FT-CONC-003: Concurrent Searches
#[tokio::test]
async fn test_concurrent_searches() {
    let ctx = Arc::new(FulltextTestContext::new());
    let num_docs = 50;
    let num_searches = 50;
    let barrier = Arc::new(Barrier::new(num_searches));

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

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

    ctx.commit_all().await.expect("Failed to commit");

    let mut handles = vec![];
    for _ in 0..num_searches {
        let ctx_clone = Arc::clone(&ctx);
        let barrier_clone = Arc::clone(&barrier);

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;

            ctx_clone
                .search(1, "Article", "content", "Search", 100)
                .await
        });

        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.expect("Search task panicked");
        let results = result.expect("Search should succeed");

        assert_search_result_count(&results, num_docs)
            .unwrap_or_else(|_| panic!("Should find all {} documents", num_docs));
    }
}

/// TC-FT-CONC-004: Concurrent Insert and Search Mix
#[tokio::test]
async fn test_concurrent_insert_and_search() {
    let ctx = Arc::new(FulltextTestContext::new());
    let num_inserts = 30;
    let num_searches = 30;

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

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

    ctx.commit_all().await.expect("Failed to commit");

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

    for handle in insert_handles {
        let result = handle.await.expect("Insert task panicked");
        assert!(result.is_ok(), "Insert should succeed");
    }

    for handle in search_handles {
        let result = handle.await.expect("Search task panicked");
        assert!(result.is_ok(), "Search should succeed");
    }

    ctx.commit_all().await.expect("Failed to commit");

    let final_results = ctx
        .search(1, "Article", "content", "content", 200)
        .await
        .expect("Search should succeed");

    assert!(
        final_results.len() >= 10,
        "Should have at least initial documents"
    );
}

/// TC-FT-CONC-005: Concurrent Updates to Same Document
#[tokio::test]
async fn test_concurrent_updates_same_document() {
    let ctx = Arc::new(FulltextTestContext::new());
    let num_updates = 20;

    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    ctx.insert_test_doc(1, "Article", "content", "doc_1", "Initial content")
        .await
        .expect("Failed to insert document");

    ctx.commit_all().await.expect("Failed to commit");

    let mut handles = vec![];
    for i in 0..num_updates {
        let ctx_clone = Arc::clone(&ctx);

        let handle = tokio::spawn(async move {
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

    for handle in handles {
        let result = handle.await.expect("Update task panicked");
        assert!(result.is_ok(), "Update should succeed");
    }

    ctx.commit_all().await.expect("Failed to commit");

    let results = ctx
        .search(1, "Article", "content", "Updated", 10)
        .await
        .expect("Search should succeed");

    assert_search_result_count(&results, 1).expect("Should find exactly 1 document");
    assert_search_result_contains(&results, "doc_1").expect("Should contain doc_1");
}

/// TC-FT-CONC-006: Concurrent Operations on Different Indexes
#[tokio::test]
async fn test_concurrent_different_indexes() {
    let ctx = Arc::new(FulltextTestContext::new());
    let num_tasks = 20;
    let barrier = Arc::new(Barrier::new(num_tasks * 2));

    ctx.create_test_index(1, "Article", "content_bm25", Some(EngineType::Bm25))
        .await
        .expect("Failed to create BM25 index");
    ctx.create_test_index(1, "Article", "content_inv", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create Inversearch index");

    let mut handles = vec![];

    // Spawn tasks for BM25 index
    for i in 0..num_tasks {
        let ctx_clone = Arc::clone(&ctx);
        let barrier_clone = Arc::clone(&barrier);

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;

            ctx_clone
                .insert_test_doc(
                    1,
                    "Article",
                    "content_bm25",
                    &format!("bm25_doc_{}", i),
                    &format!("BM25 content {}", i),
                )
                .await
        });

        handles.push(handle);
    }

    // Spawn tasks for Inversearch index
    for i in 0..num_tasks {
        let ctx_clone = Arc::clone(&ctx);
        let barrier_clone = Arc::clone(&barrier);

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;

            ctx_clone
                .insert_test_doc(
                    1,
                    "Article",
                    "content_inv",
                    &format!("inv_doc_{}", i),
                    &format!("Inversearch content {}", i),
                )
                .await
        });

        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.expect("Task panicked");
        assert!(result.is_ok(), "Insertion should succeed");
    }

    ctx.commit_all().await.expect("Failed to commit");

    // Verify BM25 index
    let bm25_results = ctx
        .search(1, "Article", "content_bm25", "BM25", 50)
        .await
        .expect("BM25 search should succeed");
    assert_eq!(bm25_results.len(), num_tasks, "BM25 should have all documents");

    // Verify Inversearch index
    let inv_results = ctx
        .search(1, "Article", "content_inv", "Inversearch", 50)
        .await
        .expect("Inversearch search should succeed");
    assert_eq!(
        inv_results.len(),
        num_tasks,
        "Inversearch should have all documents"
    );
}

/// TC-FT-CONC-007: Concurrent Index Creation
#[tokio::test]
async fn test_concurrent_index_creation() {
    let ctx = Arc::new(FulltextTestContext::new());
    let num_indexes = 10;
    let barrier = Arc::new(Barrier::new(num_indexes));

    let mut handles = vec![];
    for i in 0..num_indexes {
        let ctx_clone = Arc::clone(&ctx);
        let barrier_clone = Arc::clone(&barrier);

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;

            ctx_clone
                .create_test_index(
                    1,
                    &format!("Tag{}", i),
                    "content",
                    Some(EngineType::Bm25),
                )
                .await
        });

        handles.push(handle);
    }

    let mut success_count = 0;
    for handle in handles {
        let result = handle.await.expect("Task panicked");
        if result.is_ok() {
            success_count += 1;
        }
    }

    assert_eq!(
        success_count, num_indexes,
        "All index creations should succeed"
    );

    // Verify all indexes exist
    for i in 0..num_indexes {
        assert!(
            ctx.has_index(1, &format!("Tag{}", i), "content"),
            "Index {} should exist",
            i
        );
    }
}

/// TC-FT-CONC-008: Concurrent Mixed Engine Operations
#[tokio::test]
async fn test_concurrent_mixed_engines() {
    let ctx = Arc::new(FulltextTestContext::new());
    let num_bm25_tasks = 20;
    let num_inv_tasks = 20;
    let barrier = Arc::new(Barrier::new(num_bm25_tasks + num_inv_tasks));

    ctx.create_test_index(1, "Article", "bm25_field", Some(EngineType::Bm25))
        .await
        .expect("Failed to create BM25 index");
    ctx.create_test_index(1, "Article", "inv_field", Some(EngineType::Inversearch))
        .await
        .expect("Failed to create Inversearch index");

    let mut handles = vec![];

    // BM25 insert tasks
    for i in 0..num_bm25_tasks {
        let ctx_clone = Arc::clone(&ctx);
        let barrier_clone = Arc::clone(&barrier);

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;

            ctx_clone
                .insert_test_doc(
                    1,
                    "Article",
                    "bm25_field",
                    &format!("bm25_doc_{}", i),
                    &format!("Mixed engine test content {}", i),
                )
                .await
        });

        handles.push(handle);
    }

    // Inversearch insert tasks
    for i in 0..num_inv_tasks {
        let ctx_clone = Arc::clone(&ctx);
        let barrier_clone = Arc::clone(&barrier);

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;

            ctx_clone
                .insert_test_doc(
                    1,
                    "Article",
                    "inv_field",
                    &format!("inv_doc_{}", i),
                    &format!("Mixed engine test content {}", i),
                )
                .await
        });

        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.expect("Task panicked");
        assert!(result.is_ok(), "Insertion should succeed");
    }

    ctx.commit_all().await.expect("Failed to commit");

    // Search both indexes
    let bm25_results = ctx
        .search(1, "Article", "bm25_field", "test", 50)
        .await
        .expect("BM25 search should succeed");

    let inv_results = ctx
        .search(1, "Article", "inv_field", "test", 50)
        .await
        .expect("Inversearch search should succeed");

    assert_eq!(
        bm25_results.len(),
        num_bm25_tasks,
        "BM25 should have all documents"
    );
    assert_eq!(
        inv_results.len(),
        num_inv_tasks,
        "Inversearch should have all documents"
    );
}

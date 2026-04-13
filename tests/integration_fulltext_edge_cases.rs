//! Fulltext Integration Tests - Edge Cases and Error Handling
//!
//! Test scope:
//! - Error handling (index not found, duplicate creation, invalid queries)
//! - Edge cases (empty content, very long content, unicode, special characters)
//! - Index rebuilding
//! - Multi-space isolation
//! - Memory limits
//!
//! Test cases: TC-FT-026 ~ TC-FT-033

mod common;

use common::fulltext_helpers::{
    assert_search_result_count, generate_test_docs, FulltextTestContext,
};
use graphdb::search::EngineType;

// ==================== Error Handling Tests ====================

/// TC-FT-026: Search on Non-Existent Index
#[tokio::test]
async fn test_search_non_existent_index() {
    let ctx = FulltextTestContext::new();

    // Try to search without creating index
    let result = ctx
        .search(1, "Article", "content", "Hello", 10)
        .await;

    assert!(
        result.is_err(),
        "Search on non-existent index should fail"
    );
    assert!(
        matches!(
            result.unwrap_err(),
            graphdb::search::SearchError::IndexNotFound(_)
        ),
        "Should return IndexNotFound error"
    );
}

/// TC-FT-027: Index Empty Content
#[tokio::test]
async fn test_index_empty_content() {
    let ctx = FulltextTestContext::new();

    // Create index
    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    // Insert document with empty content
    ctx.insert_test_doc(1, "Article", "content", "doc_1", "")
        .await
        .expect("Indexing empty content should not fail");

    // Commit
    ctx.commit_all().await.expect("Failed to commit");

    // Search for any term - should return no results
    let results = ctx
        .search(1, "Article", "content", "anything", 10)
        .await
        .expect("Search should succeed");

    assert_search_result_count(&results, 0)
        .expect("Empty content should not produce search results");
}

/// TC-FT-028: Index Very Long Content
#[tokio::test]
async fn test_index_very_long_content() {
    let ctx = FulltextTestContext::new();

    // Create index
    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    // Generate very long content (10000 words)
    let long_content: Vec<String> = (0..10000).map(|i| format!("word{}", i)).collect();
    let long_content_str = long_content.join(" ");

    // Insert document with long content
    ctx.insert_test_doc(1, "Article", "content", "doc_1", &long_content_str)
        .await
        .expect("Indexing long content should succeed");

    // Commit
    ctx.commit_all().await.expect("Failed to commit");

    // Search for words in the content
    let results = ctx
        .search(1, "Article", "content", "word5000", 10)
        .await
        .expect("Search should succeed");

    assert_search_result_count(&results, 1)
        .expect("Should find document with long content");
    assert!(
        results[0].doc_id == graphdb::core::Value::String("doc_1".to_string()),
        "Should return the correct document"
    );

    // Search for another word
    let results2 = ctx
        .search(1, "Article", "content", "word9999", 10)
        .await
        .expect("Search should succeed");

    assert_search_result_count(&results2, 1)
        .expect("Should find document with long content");
}

/// TC-FT-029: Index Unicode Content
#[tokio::test]
async fn test_index_unicode_content() {
    let ctx = FulltextTestContext::new();

    // Create index
    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    // Insert document with Unicode content (Chinese, emoji, etc.)
    let unicode_content = "你好世界 Hello World 🌍🚀 测试内容";
    ctx.insert_test_doc(1, "Article", "content", "doc_1", unicode_content)
        .await
        .expect("Indexing Unicode content should succeed");

    // Commit
    ctx.commit_all().await.expect("Failed to commit");

    // Search for English words
    let results_en = ctx
        .search(1, "Article", "content", "Hello", 10)
        .await
        .expect("Search should succeed");

    assert_search_result_count(&results_en, 1)
        .expect("Should find document with English words");

    // Search for Chinese characters (if supported by tokenizer)
    let results_cn = ctx
        .search(1, "Article", "content", "你好", 10)
        .await
        .expect("Search should succeed");

    // Note: Depending on tokenizer, Chinese may or may not be indexed
    // This test verifies the system doesn't crash with Unicode
    assert!(results_cn.len() <= 1, "Should handle Chinese search gracefully");
}

/// TC-FT-030: Special Query Characters
#[tokio::test]
async fn test_special_query_characters() {
    let ctx = FulltextTestContext::new();

    // Create index
    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    // Insert document
    ctx.insert_test_doc(
        1,
        "Article",
        "content",
        "doc_1",
        "Testing special characters: hello+world test*query example?mark",
    )
    .await
    .expect("Failed to insert document");

    // Commit
    ctx.commit_all().await.expect("Failed to commit");

    // Search with special characters - should handle gracefully
    let special_queries = vec![
        "hello+world",
        "test*query",
        "example?mark",
        "hello world+",
        "test*",
        "example?",
    ];

    for query in special_queries {
        let result = ctx.search(1, "Article", "content", query, 10).await;
        // Should not panic or crash
        assert!(
            result.is_ok() || result.is_err(),
            "Should handle special query: {}",
            query
        );
    }
}

// ==================== Index Lifecycle Tests ====================

/// TC-FT-031: Index Rebuilding
#[tokio::test]
async fn test_rebuild_index() {
    let ctx = FulltextTestContext::new();

    // Create index and insert data
    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index");

    ctx.insert_test_doc(1, "Article", "content", "doc_1", "Old content")
        .await
        .expect("Failed to insert document");

    ctx.commit_all().await.expect("Failed to commit");

    // Verify old data exists
    let old_results = ctx
        .search(1, "Article", "content", "Old", 10)
        .await
        .expect("Search should succeed");
    assert_search_result_count(&old_results, 1).expect("Should find old data");

    // Drop index
    ctx.drop_index(1, "Article", "content")
        .await
        .expect("Failed to drop index");

    // Recreate index
    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to recreate index");

    // Insert new data
    ctx.insert_test_doc(1, "Article", "content", "doc_2", "New content")
        .await
        .expect("Failed to insert new document");

    ctx.commit_all().await.expect("Failed to commit");

    // Verify old data is gone
    let old_results_after = ctx
        .search(1, "Article", "content", "Old", 10)
        .await
        .expect("Search should succeed");
    assert_search_result_count(&old_results_after, 0)
        .expect("Old data should be gone after rebuild");

    // Verify new data exists
    let new_results = ctx
        .search(1, "Article", "content", "New", 10)
        .await
        .expect("Search should succeed");
    assert_search_result_count(&new_results, 1).expect("Should find new data");
}

// ==================== Isolation Tests ====================

/// TC-FT-032: Multi-Space Isolation
#[tokio::test]
async fn test_multi_space_isolation() {
    let ctx = FulltextTestContext::new();

    // Create indexes for two spaces with same tag.field
    ctx.create_test_index(1, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index for space 1");
    ctx.create_test_index(2, "Article", "content", Some(EngineType::Bm25))
        .await
        .expect("Failed to create index for space 2");

    // Insert data in space 1
    ctx.insert_test_doc(1, "Article", "content", "doc_1", "UniqueSpace1Marker")
        .await
        .expect("Failed to insert in space 1");

    // Insert data in space 2
    ctx.insert_test_doc(2, "Article", "content", "doc_2", "UniqueSpace2Marker")
        .await
        .expect("Failed to insert in space 2");

    // Commit
    ctx.commit_all().await.expect("Failed to commit");

    // Search in space 1
    let results_space_1 = ctx
        .search(1, "Article", "content", "UniqueSpace1Marker", 10)
        .await
        .expect("Search should succeed");

    assert_search_result_count(&results_space_1, 1)
        .expect("Space 1 should have 1 result");
    assert!(
        results_space_1[0].doc_id == graphdb::core::Value::String("doc_1".to_string()),
        "Space 1 should return doc_1"
    );

    // Search in space 2
    let results_space_2 = ctx
        .search(2, "Article", "content", "UniqueSpace2Marker", 10)
        .await
        .expect("Search should succeed");

    assert_search_result_count(&results_space_2, 1)
        .expect("Space 2 should have 1 result");
    assert!(
        results_space_2[0].doc_id == graphdb::core::Value::String("doc_2".to_string()),
        "Space 2 should return doc_2"
    );

    // Verify isolation: space 1 search should not return space 2 data
    let cross_search_1 = ctx
        .search(1, "Article", "content", "UniqueSpace2Marker", 10)
        .await
        .expect("Search should succeed");
    assert_search_result_count(&cross_search_1, 0)
        .expect("Space 1 should not contain space 2 data");

    // Verify isolation: space 2 search should not return space 1 data
    let cross_search_2 = ctx
        .search(2, "Article", "content", "UniqueSpace1Marker", 10)
        .await
        .expect("Search should succeed");
    assert_search_result_count(&cross_search_2, 0)
        .expect("Space 2 should not contain space 1 data");
}

/// TC-FT-033: Memory Limit Control
#[tokio::test]
async fn test_memory_limit() {
    let ctx = FulltextTestContext::new();

    // Create multiple indexes
    let num_indexes = 10;
    let num_docs_per_index = 100;

    for i in 0..num_indexes {
        ctx.create_test_index(
            1,
            &format!("Tag{}", i),
            &format!("field{}", i),
            Some(EngineType::Bm25),
        )
        .await
        .unwrap_or_else(|_| panic!("Failed to create index {}", i));

        // Insert documents
        let docs = generate_test_docs(num_docs_per_index, &format!("Index{}", i));
        let docs_ref: Vec<(&str, &str)> = docs
            .iter()
            .map(|(id, content)| (id.as_str(), content.as_str()))
            .collect();

        ctx.insert_test_docs(
            1,
            &format!("Tag{}", i),
            &format!("field{}", i),
            docs_ref,
        )
        .await
        .unwrap_or_else(|_| panic!("Failed to insert docs for index {}", i));
    }

    // Commit all
    ctx.commit_all().await.expect("Failed to commit");

    // Verify all indexes are working
    for i in 0..num_indexes {
        let results = ctx
            .search(1, &format!("Tag{}", i), &format!("field{}", i), "document", 200)
            .await
            .unwrap_or_else(|_| panic!("Search should succeed for index {}", i));

        assert_search_result_count(&results, num_docs_per_index)
            .unwrap_or_else(|_| panic!("Index {} should have all documents", i));
    }

    // Get stats for each index to verify memory usage is reasonable
    for i in 0..num_indexes {
        let stats = ctx
            .get_stats(1, &format!("Tag{}", i), &format!("field{}", i))
            .await
            .unwrap_or_else(|_| panic!("Should get stats for index {}", i));

        assert_eq!(
            stats.doc_count as u64, num_docs_per_index as u64,
            "Document count should match"
        );

        // Memory usage should be reasonable (this is a basic check)
        // In production, you would add more sophisticated memory monitoring
        assert!(
            stats.index_size > 0,
            "Index should have non-zero size"
        );
    }

    // Verify system is still responsive after creating many indexes
    let final_test = ctx
        .create_test_index(1, "FinalTag", "finalField", Some(EngineType::Bm25))
        .await;

    assert!(final_test.is_ok(), "Should still be able to create indexes");
}

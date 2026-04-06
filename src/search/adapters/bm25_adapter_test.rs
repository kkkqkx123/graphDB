#[cfg(test)]
mod tests {
    use super::super::bm25_adapter::Bm25SearchEngine;
    use crate::core::Value;
    use crate::search::engine::SearchEngine;
    use bm25_service::config::IndexManagerConfig;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_bm25_lifecycle() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let engine =
            Bm25SearchEngine::open_or_create(temp_dir.path(), IndexManagerConfig::default())
                .expect("Failed to create engine");

        engine
            .index("1", "Rust programming language")
            .await
            .expect("Failed to index doc 1");
        engine
            .index("2", "Graph database implementation")
            .await
            .expect("Failed to index doc 2");
        engine
            .index("3", "Rust graph database")
            .await
            .expect("Failed to index doc 3");

        engine.commit().await.expect("Failed to commit");

        let results = engine.search("Rust", 10).await.expect("Failed to search");
        assert_eq!(results.len(), 2, "Expected 2 results for 'Rust'");
        assert!(
            results[0].score >= results[1].score,
            "Results should be sorted by score"
        );

        engine.delete("1").await.expect("Failed to delete doc 1");
        engine
            .commit()
            .await
            .expect("Failed to commit after delete");

        let results = engine
            .search("Rust", 10)
            .await
            .expect("Failed to search after delete");
        assert_eq!(results.len(), 1, "Expected 1 result after deletion");

        engine.close().await.expect("Failed to close engine");
    }

    #[tokio::test]
    async fn test_bm25_batch_operations() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let engine =
            Bm25SearchEngine::open_or_create(temp_dir.path(), IndexManagerConfig::default())
                .expect("Failed to create engine");

        let docs: Vec<(String, String)> = (0..100)
            .map(|i| (i.to_string(), format!("Document content {}", i)))
            .collect();

        engine
            .index_batch(docs)
            .await
            .expect("Failed to batch index");
        engine.commit().await.expect("Failed to commit");

        let results = engine
            .search("Document", 100)
            .await
            .expect("Failed to search");
        assert_eq!(results.len(), 100, "Expected 100 results");
    }

    #[tokio::test]
    async fn test_bm25_persistence() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().to_path_buf();

        {
            let engine = Bm25SearchEngine::open_or_create(&path, IndexManagerConfig::default())
                .expect("Failed to create engine");
            engine
                .index("1", "Persistent data")
                .await
                .expect("Failed to index");
            engine.commit().await.expect("Failed to commit");
            engine.close().await.expect("Failed to close");
        }

        {
            let engine = Bm25SearchEngine::open_or_create(&path, IndexManagerConfig::default())
                .expect("Failed to open engine");
            let results = engine
                .search("Persistent", 10)
                .await
                .expect("Failed to search");
            assert_eq!(results.len(), 1, "Expected 1 result");
            assert_eq!(
                results[0].doc_id,
                Value::from("1"),
                "Expected doc_id to be '1'"
            );
        }
    }

    #[tokio::test]
    async fn test_bm25_search_with_limit() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let engine =
            Bm25SearchEngine::open_or_create(temp_dir.path(), IndexManagerConfig::default())
                .expect("Failed to create engine");

        for i in 0..20 {
            engine
                .index(&i.to_string(), &format!("Common keyword {}", i))
                .await
                .expect("Failed to index");
        }
        engine.commit().await.expect("Failed to commit");

        let results = engine.search("Common", 5).await.expect("Failed to search");
        assert_eq!(results.len(), 5, "Expected 5 results with limit");

        let results = engine.search("Common", 10).await.expect("Failed to search");
        assert_eq!(results.len(), 10, "Expected 10 results with limit");
    }

    #[tokio::test]
    async fn test_bm25_delete_batch() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let engine =
            Bm25SearchEngine::open_or_create(temp_dir.path(), IndexManagerConfig::default())
                .expect("Failed to create engine");

        for i in 0..10 {
            engine
                .index(&i.to_string(), &format!("Content {}", i))
                .await
                .expect("Failed to index");
        }
        engine.commit().await.expect("Failed to commit");

        let doc_ids: Vec<&str> = vec!["0", "1", "2", "3", "4"];
        engine
            .delete_batch(doc_ids)
            .await
            .expect("Failed to batch delete");
        engine.commit().await.expect("Failed to commit");

        let results = engine
            .search("Content", 10)
            .await
            .expect("Failed to search");
        assert_eq!(results.len(), 5, "Expected 5 results after batch delete");
    }

    #[tokio::test]
    async fn test_bm25_stats() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let engine =
            Bm25SearchEngine::open_or_create(temp_dir.path(), IndexManagerConfig::default())
                .expect("Failed to create engine");

        let stats = engine.stats().await.expect("Failed to get stats");
        assert_eq!(stats.doc_count, 0, "Expected 0 docs initially");

        for i in 0..5 {
            engine
                .index(&i.to_string(), &format!("Test content {}", i))
                .await
                .expect("Failed to index");
        }
        engine.commit().await.expect("Failed to commit");

        let stats = engine.stats().await.expect("Failed to get stats");
        assert_eq!(stats.doc_count, 5, "Expected 5 docs after indexing");
        assert!(stats.index_size > 0, "Expected positive index size");
    }

    #[tokio::test]
    async fn test_bm25_empty_search() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let engine =
            Bm25SearchEngine::open_or_create(temp_dir.path(), IndexManagerConfig::default())
                .expect("Failed to create engine");

        let results = engine
            .search("nonexistent", 10)
            .await
            .expect("Failed to search");
        assert!(results.is_empty(), "Expected empty results on empty index");
    }

    #[tokio::test]
    async fn test_bm25_engine_info() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let engine =
            Bm25SearchEngine::open_or_create(temp_dir.path(), IndexManagerConfig::default())
                .expect("Failed to create engine");

        assert_eq!(engine.name(), "bm25", "Expected engine name 'bm25'");
        assert_eq!(engine.version(), "0.1.0", "Expected version '0.1.0'");
    }
}

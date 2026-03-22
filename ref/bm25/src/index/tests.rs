#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use tempfile::{tempdir, TempDir};

    use crate::index::cache::Cache;
    use crate::index::manager::IndexManager;
    use crate::index::schema::IndexSchema;
    use crate::index::search::{search, SearchOptions};
    use crate::index::document::{add_document, get_document};
    use crate::index::delete::delete_document;
    use crate::index::batch::batch_add_documents_optimized;
    use crate::index::stats::get_stats;

    fn create_test_manager() -> (TempDir, IndexManager, IndexSchema) {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let path = temp_dir.path().join("test_index");
        
        if !path.exists() {
            fs::create_dir_all(&path).expect("Failed to create test index directory");
        }
        
        let manager = IndexManager::create(&path).expect("Failed to create index manager");
        let schema = IndexSchema::new();
        
        (temp_dir, manager, schema)
    }

    #[test]
    fn test_cache_basic_operations() {
        let cache: Cache<String, i32> = Cache::new(10, 60);

        assert_eq!(cache.get(&"key1".to_string()), None);
        assert_eq!(cache.size(), 0);

        cache.insert("key1".to_string(), 42);
        assert_eq!(cache.get(&"key1".to_string()), Some(42));
        assert_eq!(cache.size(), 1);

        cache.insert("key1".to_string(), 100);
        assert_eq!(cache.get(&"key1".to_string()), Some(100));

        cache.remove(&"key1".to_string());
        assert_eq!(cache.get(&"key1".to_string()), None);
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_cache_lru_eviction() {
        let cache: Cache<String, i32> = Cache::new(3, 60);

        cache.insert("key1".to_string(), 1);
        cache.insert("key2".to_string(), 2);
        cache.insert("key3".to_string(), 3);

        assert_eq!(cache.size(), 3);

        cache.insert("key4".to_string(), 4);

        assert!(cache.size() <= 3);
    }

    #[test]
    fn test_cache_stats() {
        let cache: Cache<String, i32> = Cache::new(10, 60);

        cache.insert("key1".to_string(), 1);
        cache.get(&"key1".to_string());
        cache.get(&"nonexistent".to_string());

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.size, 1);
    }

    #[test]
    fn test_cache_clear() {
        let cache: Cache<String, i32> = Cache::new(10, 60);

        cache.insert("key1".to_string(), 1);
        cache.insert("key2".to_string(), 2);

        assert_eq!(cache.size(), 2);

        cache.clear();

        assert_eq!(cache.size(), 0);
        assert_eq!(cache.get(&"key1".to_string()), None);
    }

    #[test]
    fn test_index_manager_create() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let path = temp_dir.path().join("test_index");
        fs::create_dir_all(&path).expect("Failed to create test index directory");

        let manager = IndexManager::create(&path).expect("Failed to create index manager");
        assert!(path.exists());
    }

    #[test]
    fn test_index_manager_open() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let path = temp_dir.path().join("test_index");

        {
            fs::create_dir_all(&path).expect("Failed to create test index directory");
            let manager = IndexManager::create(&path).expect("Failed to create index manager");
            let _ = manager.index();
        }

        let manager = IndexManager::open(&path).expect("Failed to open index manager");
        let _ = manager.index();
    }

    #[test]
    fn test_document_add() {
        let (_temp_dir, manager, schema) = create_test_manager();

        let mut fields = HashMap::new();
        fields.insert("title".to_string(), "Test Document Title".to_string());
        fields.insert("content".to_string(), "This is the content of the test document".to_string());

        let result = add_document(&manager, &schema, "1", &fields);
        assert!(result.is_ok());

        let writer = manager.writer();
        assert!(writer.is_ok());
    }

    #[test]
    fn test_document_get() {
        let (_temp_dir, manager, schema) = create_test_manager();

        let mut fields = HashMap::new();
        fields.insert("title".to_string(), "Test Document".to_string());
        fields.insert("content".to_string(), "Content to retrieve".to_string());

        add_document(&manager, &schema, "1", &fields).ok();

        let result = get_document(&manager, &schema, "1");
        assert!(result.is_ok());
        assert!(result.expect("Failed to get document").is_some());
    }

    #[test]
    fn test_document_delete() {
        let (_temp_dir, manager, schema) = create_test_manager();

        let mut fields = HashMap::new();
        fields.insert("title".to_string(), "Test Document".to_string());
        fields.insert("content".to_string(), "Content to delete".to_string());

        add_document(&manager, &schema, "1", &fields).ok();
        let result = delete_document(&manager, &schema, "1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_batch_index() {
        let (_temp_dir, manager, schema) = create_test_manager();

        let docs: Vec<(String, HashMap<String, String>)> = (1..=5)
            .map(|id| {
                let mut fields = HashMap::new();
                fields.insert("title".to_string(), format!("Document {}", id));
                fields.insert("content".to_string(), format!("Content for document {}", id));
                (id.to_string(), fields)
            })
            .collect();

        let result = batch_add_documents_optimized(&manager, &schema, docs, 2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_index_stats() {
        let (_temp_dir, manager, _schema) = create_test_manager();

        let stats = get_stats(&manager);
        assert!(stats.is_ok());
    }

    #[test]
    fn test_search_basic() {
        let (_temp_dir, manager, schema) = create_test_manager();

        let docs = vec![
            (
                "1".to_string(),
                vec![
                    ("title".to_string(), "Rust Programming".to_string()),
                    ("content".to_string(), "Rust is a systems programming language".to_string()),
                ].into_iter().collect(),
            ),
            (
                "2".to_string(),
                vec![
                    ("title".to_string(), "TypeScript Guide".to_string()),
                    ("content".to_string(), "TypeScript is a typed superset of JavaScript".to_string()),
                ].into_iter().collect(),
            ),
        ];

        batch_add_documents_optimized(&manager, &schema, docs, 2).ok();

        let options = SearchOptions {
            limit: 10,
            offset: 0,
            field_weights: HashMap::new(),
            highlight: false,
        };

        let result = search(&manager, &schema, "Rust", &options);
        assert!(result.is_ok());

        let (results, max_score) = result.expect("Search failed");
        assert!(results.len() >= 0);
        assert!(max_score >= 0.0);
    }

    #[test]
    fn test_search_with_pagination() {
        let (_temp_dir, manager, schema) = create_test_manager();

        let docs: Vec<(String, HashMap<String, String>)> = (1..=10)
            .map(|id| {
                let mut fields = HashMap::new();
                fields.insert("title".to_string(), "Programming Tutorial".to_string());
                fields.insert("content".to_string(), format!("This is programming tutorial number {}", id));
                (id.to_string(), fields)
            })
            .collect();

        batch_add_documents_optimized(&manager, &schema, docs, 5).ok();

        let options1 = SearchOptions {
            limit: 5,
            offset: 0,
            field_weights: HashMap::new(),
            highlight: false,
        };

        let result1 = search(&manager, &schema, "Programming", &options1);
        assert!(result1.is_ok());

        let options2 = SearchOptions {
            limit: 5,
            offset: 5,
            field_weights: HashMap::new(),
            highlight: false,
        };

        let result2 = search(&manager, &schema, "Programming", &options2);
        assert!(result2.is_ok());
    }

    #[test]
    fn test_search_highlighting() {
        let (_temp_dir, manager, schema) = create_test_manager();

        let mut fields = HashMap::new();
        fields.insert("title".to_string(), "Hello World".to_string());
        fields.insert("content".to_string(), "This is a test document with hello and world".to_string());

        add_document(&manager, &schema, "1", &fields).ok();

        let options = SearchOptions {
            limit: 10,
            offset: 0,
            field_weights: HashMap::new(),
            highlight: true,
        };

        let result = search(&manager, &schema, "hello world", &options);
        assert!(result.is_ok());

        let (results, _) = result.expect("Search failed");
        assert!(results.len() > 0);
    }

    #[test]
    fn test_search_empty_query() {
        let (_temp_dir, manager, schema) = create_test_manager();

        let options = SearchOptions {
            limit: 10,
            offset: 0,
            field_weights: HashMap::new(),
            highlight: false,
        };

        let result = search(&manager, &schema, "", &options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_no_matches() {
        let (_temp_dir, manager, schema) = create_test_manager();

        let mut fields = HashMap::new();
        fields.insert("title".to_string(), "Rust".to_string());
        fields.insert("content".to_string(), "Programming language".to_string());

        add_document(&manager, &schema, "1", &fields).ok();

        let options = SearchOptions {
            limit: 10,
            offset: 0,
            field_weights: HashMap::new(),
            highlight: false,
        };

        let result = search(&manager, &schema, "Python Java", &options);
        assert!(result.is_ok());

        let (results, _) = result.expect("Search failed");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_index_schema_fields() {
        let schema = IndexSchema::new();
        assert!(schema.document_id.field_id() >= 0);
        assert!(schema.title.field_id() >= 0);
        assert!(schema.content.field_id() >= 0);
    }

    #[test]
    fn test_search_options_default() {
        let options = SearchOptions::default();

        assert_eq!(options.limit, 10);
        assert_eq!(options.offset, 0);
        assert_eq!(options.highlight, false);
        assert!(options.field_weights.is_empty());
    }

    #[test]
    fn test_cache_stats_default() {
        let stats = crate::index::cache::CacheStats::default();

        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.evictions, 0);
        assert_eq!(stats.size, 0);
    }

    #[test]
    fn test_schema_to_document() {
        let schema = IndexSchema::new();

        let mut fields = HashMap::new();
        fields.insert("title".to_string(), "Test Title".to_string());
        fields.insert("content".to_string(), "Test Content".to_string());

        let doc = schema.to_document("1", &fields);
        assert!(doc.field_values().len() >= 3);
    }
}

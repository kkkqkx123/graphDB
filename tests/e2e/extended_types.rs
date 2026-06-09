//! E2E Test Suite for Extended Types
//!
//! Tests extended type functionality including:
//! - Geography/Geospatial types
//! - Vector search
//! - Full-text search

use graphdb::core::stats::StatsManager;
use graphdb::query::optimizer::OptimizerEngine;
use graphdb::query::query_pipeline_manager::QueryPipelineManager;
use std::sync::Arc;

use crate::common::TestStorage;


/// Geography/Geospatial type tests
mod geography {
    use super::*;

    /// Create points using ST_Point
    #[tokio::test]
    async fn test_point_creation() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE e2e_geography (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE e2e_geography")
            .expect("USE should succeed");
        pipeline.execute_query(
            "CREATE TAG location(name: STRING NOT NULL, coord: GEOGRAPHY, address: STRING, category: STRING)"
        ).expect("CREATE TAG should succeed");

        // Insert point
        let result = pipeline.execute_query(
            "INSERT VERTEX location(name, coord, category) VALUES \"loc_test\": (\"Test Location\", ST_Point(116.4, 39.9), \"test\")"
        );
        assert!(result.is_ok(), "INSERT with ST_Point should succeed");
    }

    /// Create points using WKT format
    #[tokio::test]
    async fn test_wkt_creation() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE e2e_geography_wkt (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE e2e_geography_wkt")
            .expect("USE should succeed");
        pipeline.execute_query(
            "CREATE TAG location(name: STRING NOT NULL, coord: GEOGRAPHY, category: STRING)"
        ).expect("CREATE TAG should succeed");

        // Insert point from WKT
        let result = pipeline.execute_query(
            "INSERT VERTEX location(name, coord, category) VALUES \"loc_wkt\": (\"WKT Location\", ST_GeogFromText(\"POINT(116.5 39.8)\"), \"test\")"
        );
        assert!(result.is_ok(), "INSERT with WKT should succeed");
    }

    /// Calculate distance between points
    #[tokio::test]
    async fn test_distance_calculation() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE e2e_geography_dist (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE e2e_geography_dist")
            .expect("USE should succeed");
        pipeline.execute_query(
            "CREATE TAG location(name: STRING NOT NULL, coord: GEOGRAPHY)"
        ).expect("CREATE TAG should succeed");

        // Insert points
        pipeline.execute_query(
            "INSERT VERTEX location(name, coord) VALUES \"loc1\": (\"Tiananmen\", ST_Point(116.3974, 39.9093))"
        ).expect("INSERT should succeed");
        pipeline.execute_query(
            "INSERT VERTEX location(name, coord) VALUES \"loc2\": (\"Forbidden City\", ST_Point(116.3972, 39.9163))"
        ).expect("INSERT should succeed");

        // Calculate distance
        let result = pipeline.execute_query(
            "MATCH (a:location {name: \"Tiananmen\"}), (b:location {name: \"Forbidden City\"}) RETURN ST_Distance(a.coord, b.coord) AS distance_km"
        );
        assert!(result.is_ok(), "ST_Distance should succeed");
    }

    /// Find locations within distance
    #[tokio::test]
    async fn test_within_distance() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE e2e_geography_within (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE e2e_geography_within")
            .expect("USE should succeed");
        pipeline.execute_query(
            "CREATE TAG location(name: STRING NOT NULL, coord: GEOGRAPHY)"
        ).expect("CREATE TAG should succeed");

        // Insert points
        pipeline.execute_query(
            "INSERT VERTEX location(name, coord) VALUES \"center\": (\"Tiananmen\", ST_Point(116.4, 39.9))"
        ).expect("INSERT should succeed");
        pipeline.execute_query(
            "INSERT VERTEX location(name, coord) VALUES \"loc1\": (\"Forbidden City\", ST_Point(116.3972, 39.9163))"
        ).expect("INSERT should succeed");

        // Find within distance
        let result = pipeline.execute_query(
            "MATCH (center:location {name: \"Tiananmen\"}) MATCH (loc:location) WHERE ST_DWithin(center.coord, loc.coord, 5.0) RETURN loc.name, ST_Distance(center.coord, loc.coord) AS distance ORDER BY distance"
        );
        assert!(result.is_ok(), "ST_DWithin should succeed");
    }

    /// EXPLAIN geography query
    #[tokio::test]
    async fn test_explain_geography_query() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE e2e_geography_explain (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE e2e_geography_explain")
            .expect("USE should succeed");
        pipeline.execute_query(
            "CREATE TAG location(name: STRING NOT NULL, coord: GEOGRAPHY)"
        ).expect("CREATE TAG should succeed");

        // Insert data
        pipeline.execute_query(
            "INSERT VERTEX location(name, coord) VALUES \"loc1\": (\"Beijing\", ST_Point(116.4, 39.9))"
        ).expect("INSERT should succeed");

        // EXPLAIN
        let result = pipeline.execute_query(
            "EXPLAIN MATCH (loc:location) WHERE ST_DWithin(ST_Point(116.4, 39.9), loc.coord, 10.0) RETURN loc.name"
        );
        assert!(result.is_ok(), "EXPLAIN geography query should succeed");
    }
}

/// Vector search tests
mod vector {
    use super::*;

    /// Insert vertex with vector
    #[tokio::test]
    async fn test_vector_insertion() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE e2e_vector (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE e2e_vector")
            .expect("USE should succeed");
        pipeline.execute_query(
            "CREATE TAG product_vector(product_id: STRING NOT NULL, name: STRING, category: STRING, embedding: VECTOR(128), price: DOUBLE)"
        ).expect("CREATE TAG should succeed");

        // Insert vector
        let vector = vec![0.1; 128];
        let vector_str = vector.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ");

        let result = pipeline.execute_query(&format!(
            "INSERT VERTEX product_vector(product_id, name, category, embedding, price) VALUES \"pv_test\": (\"TEST001\", \"Test Product\", \"test\", [{}], 99.99)",
            vector_str
        ));
        assert!(result.is_ok(), "INSERT VECTOR should succeed");
    }

    /// Cosine similarity search
    #[tokio::test]
    async fn test_cosine_similarity() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE e2e_vector_search (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE e2e_vector_search")
            .expect("USE should succeed");
        pipeline.execute_query(
            "CREATE TAG product_vector(product_id: STRING NOT NULL, name: STRING, embedding: VECTOR(128))"
        ).expect("CREATE TAG should succeed");

        // Insert products with vectors
        for i in 0..100 {
            let vector: Vec<f64> = (0..128).map(|_| (i as f64) * 0.01).collect();
            let vector_str = vector.iter().map(|v| format!("{:.4}", v)).collect::<Vec<_>>().join(", ");

            pipeline.execute_query(&format!(
                "INSERT VERTEX product_vector(product_id, name, embedding) VALUES \"pv{:03}\": (\"PROD{:03}\", \"Product {}\", [{}])",
                i, i, i, vector_str
            )).expect("INSERT should succeed");
        }

        // Create vector index
        pipeline.execute_query(
            "CREATE VECTOR INDEX idx_product_embedding ON product_vector(embedding) WITH (vector_size=128, distance='cosine')"
        ).expect("CREATE VECTOR INDEX should succeed");

        // Search vector
        let query_vector = vec![0.1; 128];
        let vector_str = query_vector.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ");

        let result = pipeline.execute_query(&format!(
            "SEARCH VECTOR idx_product_embedding WITH vector=[{}] YIELD product_id, name LIMIT 10",
            vector_str
        ));
        assert!(result.is_ok(), "SEARCH VECTOR should succeed");
    }

    /// Vector search with filter
    #[tokio::test]
    async fn test_filtered_vector_search() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE e2e_vector_filtered (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE e2e_vector_filtered")
            .expect("USE should succeed");
        pipeline.execute_query(
            "CREATE TAG product_vector(product_id: STRING NOT NULL, name: STRING, embedding: VECTOR(128), price: DOUBLE)"
        ).expect("CREATE TAG should succeed");

        // Insert data
        for i in 0..50 {
            let vector = vec![0.1; 128];
            let vector_str = vector.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ");

            pipeline.execute_query(&format!(
                "INSERT VERTEX product_vector(product_id, name, embedding, price) VALUES \"pv{:03}\": (\"PROD{:03}\", \"Product {}\", [{}], {}.0)",
                i, i, i, vector_str, i * 10
            )).expect("INSERT should succeed");
        }

        // Create vector index
        pipeline.execute_query(
            "CREATE VECTOR INDEX idx_product_embedding ON product_vector(embedding) WITH (vector_size=128, distance='cosine')"
        ).expect("CREATE VECTOR INDEX should succeed");

        // Search with filter
        let query_vector = vec![0.1; 128];
        let vector_str = query_vector.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ");

        let result = pipeline.execute_query(&format!(
            "SEARCH VECTOR idx_product_embedding WITH vector=[{}] WHERE price < 500 YIELD product_id, name, price LIMIT 5",
            vector_str
        ));
        assert!(result.is_ok(), "SEARCH VECTOR with filter should succeed");
    }

    /// EXPLAIN vector query
    #[tokio::test]
    async fn test_explain_vector_query() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE e2e_vector_explain (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE e2e_vector_explain")
            .expect("USE should succeed");
        pipeline.execute_query(
            "CREATE TAG product_vector(product_id: STRING NOT NULL, name: STRING, embedding: VECTOR(128))"
        ).expect("CREATE TAG should succeed");

        // Create vector index
        pipeline.execute_query(
            "CREATE VECTOR INDEX idx_product_embedding ON product_vector(embedding) WITH (vector_size=128, distance='cosine')"
        ).expect("CREATE VECTOR INDEX should succeed");

        // Insert data
        let vector = vec![0.1; 128];
        let vector_str = vector.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ");
        pipeline.execute_query(&format!(
            "INSERT VERTEX product_vector(product_id, name, embedding) VALUES \"pv001\": (\"PROD001\", \"Product 1\", [{}])",
            vector_str
        )).expect("INSERT should succeed");

        // EXPLAIN
        let result = pipeline.execute_query(&format!(
            "EXPLAIN SEARCH VECTOR idx_product_embedding WITH vector=[{}] YIELD product_id, name LIMIT 10",
            vector_str
        ));
        assert!(result.is_ok(), "EXPLAIN SEARCH VECTOR should succeed");
    }
}

/// Full-text search tests
mod fulltext {
    use super::*;

    /// Create fulltext index
    #[tokio::test]
    async fn test_fulltext_index_creation() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE e2e_fulltext (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE e2e_fulltext")
            .expect("USE should succeed");
        pipeline.execute_query(
            "CREATE TAG article(doc_id: STRING NOT NULL, title: STRING, content: STRING, author: STRING)"
        ).expect("CREATE TAG should succeed");

        // Create fulltext index
        let result = pipeline.execute_query(
            "CREATE FULLTEXT INDEX idx_article_content ON article(content) ENGINE BM25 OPTIONS (analyzer='standard')"
        );
        assert!(result.is_ok(), "CREATE FULLTEXT INDEX should succeed");
    }

    /// Basic fulltext search
    #[tokio::test]
    async fn test_basic_search() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE e2e_fulltext_search (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE e2e_fulltext_search")
            .expect("USE should succeed");
        pipeline.execute_query(
            "CREATE TAG article(doc_id: STRING NOT NULL, title: STRING, content: STRING)"
        ).expect("CREATE TAG should succeed");

        // Create fulltext index
        pipeline.execute_query(
            "CREATE FULLTEXT INDEX idx_article_content ON article(content) ENGINE BM25 OPTIONS (analyzer='standard')"
        ).expect("CREATE FULLTEXT INDEX should succeed");

        // Insert articles
        pipeline.execute_query(
            "INSERT VERTEX article(doc_id, title, content) VALUES \"art001\": (\"art001\", \"Graph Database Introduction\", \"Graph databases are designed for connected data\")"
        ).expect("INSERT should succeed");
        pipeline.execute_query(
            "INSERT VERTEX article(doc_id, title, content) VALUES \"art002\": (\"art002\", \"Query Optimization\", \"Optimizing queries improves performance significantly\")"
        ).expect("INSERT should succeed");

        // Search
        let result = pipeline.execute_query(
            "SEARCH INDEX idx_article_content MATCH 'database' YIELD doc_id, title, score"
        );
        assert!(result.is_ok(), "SEARCH INDEX should succeed");
    }

    /// Boolean query search
    #[tokio::test]
    async fn test_boolean_search() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE e2e_fulltext_bool (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE e2e_fulltext_bool")
            .expect("USE should succeed");
        pipeline.execute_query(
            "CREATE TAG article(doc_id: STRING NOT NULL, title: STRING, content: STRING)"
        ).expect("CREATE TAG should succeed");

        // Create fulltext index
        pipeline.execute_query(
            "CREATE FULLTEXT INDEX idx_article_content ON article(content) ENGINE BM25 OPTIONS (analyzer='standard')"
        ).expect("CREATE FULLTEXT INDEX should succeed");

        // Insert articles
        pipeline.execute_query(
            "INSERT VERTEX article(doc_id, title, content) VALUES \"art001\": (\"art001\", \"Graph Database\", \"Graph databases are designed for connected data\")"
        ).expect("INSERT should succeed");
        pipeline.execute_query(
            "INSERT VERTEX article(doc_id, title, content) VALUES \"art002\": (\"art002\", \"Query Optimization\", \"Optimizing queries improves performance\")"
        ).expect("INSERT should succeed");

        // Boolean search
        let result = pipeline.execute_query(
            "SEARCH INDEX idx_article_content MATCH 'graph AND database' YIELD doc_id, title"
        );
        assert!(result.is_ok(), "SEARCH INDEX with boolean should succeed");
    }

    /// EXPLAIN fulltext search
    #[tokio::test]
    async fn test_explain_fulltext() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE e2e_fulltext_explain (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE e2e_fulltext_explain")
            .expect("USE should succeed");
        pipeline.execute_query(
            "CREATE TAG article(doc_id: STRING NOT NULL, title: STRING, content: STRING)"
        ).expect("CREATE TAG should succeed");

        // Create fulltext index
        pipeline.execute_query(
            "CREATE FULLTEXT INDEX idx_article_content ON article(content) ENGINE BM25 OPTIONS (analyzer='standard')"
        ).expect("CREATE FULLTEXT INDEX should succeed");

        // Insert articles
        pipeline.execute_query(
            "INSERT VERTEX article(doc_id, title, content) VALUES \"art001\": (\"art001\", \"Performance Tuning\", \"Performance tuning is crucial for database performance\")"
        ).expect("INSERT should succeed");

        // EXPLAIN
        let result = pipeline.execute_query(
            "EXPLAIN SEARCH INDEX idx_article_content MATCH 'performance' YIELD doc_id, score"
        );
        assert!(result.is_ok(), "EXPLAIN SEARCH INDEX should succeed");
    }
}

/// Cleanup tests
mod cleanup {
    use super::*;

    /// Drop all test spaces
    #[tokio::test]
    async fn test_cleanup() {
        let spaces = [
            "e2e_geography",
            "e2e_geography_wkt",
            "e2e_geography_dist",
            "e2e_geography_within",
            "e2e_geography_explain",
            "e2e_vector",
            "e2e_vector_search",
            "e2e_vector_filtered",
            "e2e_vector_explain",
            "e2e_fulltext",
            "e2e_fulltext_search",
            "e2e_fulltext_bool",
            "e2e_fulltext_explain",
        ];

        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        for space in &spaces {
            let _ = pipeline.execute_query(&format!("DROP SPACE IF EXISTS {}", space));
        }
    }
}

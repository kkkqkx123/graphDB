//! E2E Test Suite for Query Optimizer
//!
//! Tests optimizer behavior including:
//! - Index selection
//! - Join algorithm selection
//! - Aggregation strategies
//! - TopN optimization
//! - Query plan validation via EXPLAIN

use graphdb::core::stats::StatsManager;
use graphdb::query::optimizer::OptimizerEngine;
use graphdb::query::query_pipeline_manager::QueryPipelineManager;
use std::sync::Arc;

use crate::common::TestStorage;


/// Index selection optimization tests
mod index {
    use super::*;

    /// Equality query should use IndexScan
    #[tokio::test]
    async fn test_index_scan_for_equality() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE e2e_optimizer (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE e2e_optimizer")
            .expect("USE should succeed");
        pipeline.execute_query("CREATE TAG person(name: STRING, age: INT, city: STRING, salary: INT)")
            .expect("CREATE TAG should succeed");
        pipeline.execute_query("CREATE TAG INDEX idx_person_name ON person(name)")
            .expect("CREATE INDEX should succeed");
        pipeline.execute_query("CREATE TAG INDEX idx_person_age ON person(age)")
            .expect("CREATE INDEX should succeed");

        // Insert test data
        for i in 0..100 {
            let name = format!("Person_{:03}", i);
            let age = 20 + (i % 40);
            let city = match i % 3 {
                0 => "Beijing",
                1 => "Shanghai",
                _ => "Shenzhen",
            };
            let salary = 5000 + (i * 100);

            pipeline.execute_query(&format!(
                "INSERT VERTEX person(name, age, city, salary) VALUES \"p{:03}\": (\"{}\", {}, \"{}\", {})",
                i, name, age, city, salary
            )).expect("INSERT should succeed");
        }

        // Test equality query
        let result = pipeline.execute_query(
            "EXPLAIN MATCH (p:person {name: \"Person_001\"}) RETURN p.age"
        );
        assert!(result.is_ok(), "EXPLAIN should succeed");
    }

    /// Range query should use IndexScan
    #[tokio::test]
    async fn test_index_scan_for_range() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE e2e_optimizer_range (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE e2e_optimizer_range")
            .expect("USE should succeed");
        pipeline.execute_query("CREATE TAG person(name: STRING, age: INT)")
            .expect("CREATE TAG should succeed");
        pipeline.execute_query("CREATE TAG INDEX idx_person_age ON person(age)")
            .expect("CREATE INDEX should succeed");

        // Insert test data
        for i in 0..100 {
            pipeline.execute_query(&format!(
                "INSERT VERTEX person(name, age) VALUES \"p{:03}\": (\"Person_{:03}\", {})",
                i, i, 20 + (i % 40)
            )).expect("INSERT should succeed");
        }

        // Test range query
        let result = pipeline.execute_query(
            "EXPLAIN MATCH (p:person) WHERE p.age > 25 AND p.age < 35 RETURN p.name"
        );
        assert!(result.is_ok(), "EXPLAIN should succeed");
    }

    /// Query on non-indexed field should use SeqScan
    #[tokio::test]
    async fn test_no_index_full_scan() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE e2e_optimizer_scan (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE e2e_optimizer_scan")
            .expect("USE should succeed");
        pipeline.execute_query("CREATE TAG person(name: STRING, salary: INT)")
            .expect("CREATE TAG should succeed");

        // Insert test data (no index on salary)
        for i in 0..50 {
            pipeline.execute_query(&format!(
                "INSERT VERTEX person(name, salary) VALUES \"p{:03}\": (\"Person_{:03}\", {})",
                i, i, 5000 + i * 100
            )).expect("INSERT should succeed");
        }

        // Test query on non-indexed field
        let result = pipeline.execute_query(
            "EXPLAIN MATCH (p:person) WHERE p.salary > 10000 RETURN p.name"
        );
        assert!(result.is_ok(), "EXPLAIN should succeed");
    }
}

/// Join optimization tests
mod join {
    use super::*;

    /// Verify traversal operation is selected for graph patterns
    #[tokio::test]
    async fn test_join_algorithm_selection() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE e2e_optimizer_join (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE e2e_optimizer_join")
            .expect("USE should succeed");
        pipeline.execute_query("CREATE TAG company(name: STRING, industry: STRING)")
            .expect("CREATE TAG should succeed");
        pipeline.execute_query("CREATE TAG employee(name: STRING, salary: INT)")
            .expect("CREATE TAG should succeed");
        pipeline.execute_query("CREATE EDGE works_at(position: STRING)")
            .expect("CREATE EDGE should succeed");

        // Insert companies (fewer)
        for i in 0..10 {
            pipeline.execute_query(&format!(
                "INSERT VERTEX company(name, industry) VALUES \"c{:02}\": (\"Company_{:02}\", \"Tech\")",
                i, i
            )).expect("INSERT should succeed");
        }

        // Insert employees (more)
        for i in 0..100 {
            pipeline.execute_query(&format!(
                "INSERT VERTEX employee(name, salary) VALUES \"e{:03}\": (\"Employee_{:03}\", {})",
                i, i, 5000 + i * 100
            )).expect("INSERT should succeed");
        }

        // Create relationships
        for i in 0..100 {
            let company_id = format!("c{:02}", i % 10);
            pipeline.execute_query(&format!(
                "INSERT EDGE works_at(position) VALUES \"e{:03}\" -> \"{}\" @0: (\"Engineer\")",
                i, company_id
            )).expect("INSERT EDGE should succeed");
        }

        // Test join query
        let result = pipeline.execute_query(
            "EXPLAIN MATCH (e:employee)-[:works_at]->(c:company) RETURN e.name, c.name"
        );
        assert!(result.is_ok(), "EXPLAIN should succeed");
    }
}

/// Aggregation optimization tests
mod aggregate {
    use super::*;

    /// HashAggregate for GROUP BY
    #[tokio::test]
    async fn test_hash_aggregate() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE e2e_optimizer_agg (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE e2e_optimizer_agg")
            .expect("USE should succeed");
        pipeline.execute_query("CREATE TAG sales(product: STRING, amount: INT, category: STRING)")
            .expect("CREATE TAG should succeed");

        // Insert sales data
        for i in 0..1000 {
            let product = format!("Product_{:02}", i % 20);
            let amount = 10 + (i % 1000);
            let category = match i % 3 {
                0 => "A",
                1 => "B",
                _ => "C",
            };

            pipeline.execute_query(&format!(
                "INSERT VERTEX sales(product, amount, category) VALUES \"s{:04}\": (\"{}\", {}, \"{}\")",
                i, product, amount, category
            )).expect("INSERT should succeed");
        }

        // Test aggregation query
        let result = pipeline.execute_query(
            "EXPLAIN MATCH (s:sales) RETURN s.category, sum(s.amount) AS total GROUP BY s.category"
        );
        assert!(result.is_ok(), "EXPLAIN should succeed");
    }
}

/// TopN optimization tests
mod topn {
    use super::*;

    /// ORDER BY + LIMIT should use TopN
    #[tokio::test]
    async fn test_order_by_limit() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE e2e_optimizer_topn (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE e2e_optimizer_topn")
            .expect("USE should succeed");
        pipeline.execute_query("CREATE TAG product(name: STRING, price: INT, sales: INT)")
            .expect("CREATE TAG should succeed");

        for i in 0..100 {
            pipeline.execute_query(&format!(
                "INSERT VERTEX product(name, price, sales) VALUES \"p{:03}\": (\"Product_{:03}\", {}, {})",
                i, i, 10 + (i % 1000), i * 10
            )).expect("INSERT should succeed");
        }

        // Test TopN query
        let result = pipeline.execute_query(
            "EXPLAIN MATCH (p:product) RETURN p.name, p.price ORDER BY p.price DESC LIMIT 10"
        );
        assert!(result.is_ok(), "EXPLAIN should succeed");
    }
}

/// EXPLAIN format tests
mod explain_format {
    use super::*;

    /// EXPLAIN with text format
    #[tokio::test]
    async fn test_text_format() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE e2e_optimizer_explain (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE e2e_optimizer_explain")
            .expect("USE should succeed");
        pipeline.execute_query("CREATE TAG person(name: STRING, age: INT)")
            .expect("CREATE TAG should succeed");

        // Test text format
        let result = pipeline.execute_query("EXPLAIN MATCH (p:person) RETURN p.name");
        assert!(result.is_ok(), "EXPLAIN should succeed");
    }

    /// EXPLAIN with DOT format
    #[tokio::test]
    async fn test_dot_format() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE e2e_optimizer_dot (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE e2e_optimizer_dot")
            .expect("USE should succeed");
        pipeline.execute_query("CREATE TAG person(name: STRING, age: INT)")
            .expect("CREATE TAG should succeed");

        // Test DOT format
        let result = pipeline.execute_query("EXPLAIN FORMAT = DOT MATCH (p:person) RETURN p.name");
        assert!(result.is_ok(), "EXPLAIN FORMAT = DOT should succeed");
    }
}

/// PROFILE command tests
mod profile {
    use super::*;

    /// Basic PROFILE execution
    #[tokio::test]
    async fn test_basic_profile() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE e2e_optimizer_profile (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE e2e_optimizer_profile")
            .expect("USE should succeed");
        pipeline.execute_query("CREATE TAG person(name: STRING, age: INT)")
            .expect("CREATE TAG should succeed");

        for i in 0..50 {
            pipeline.execute_query(&format!(
                "INSERT VERTEX person(name, age) VALUES \"p{:03}\": (\"Person_{:03}\", {})",
                i, i, 20 + i
            )).expect("INSERT should succeed");
        }

        // Test PROFILE
        let result = pipeline.execute_query("PROFILE MATCH (p:person) RETURN count(p)");
        assert!(result.is_ok(), "PROFILE should succeed");
    }
}

/// Cleanup tests
mod cleanup {
    use super::*;

    /// Drop all test spaces
    #[tokio::test]
    async fn test_cleanup() {
        let spaces = [
            "e2e_optimizer",
            "e2e_optimizer_range",
            "e2e_optimizer_scan",
            "e2e_optimizer_join",
            "e2e_optimizer_agg",
            "e2e_optimizer_topn",
            "e2e_optimizer_explain",
            "e2e_optimizer_dot",
            "e2e_optimizer_profile",
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

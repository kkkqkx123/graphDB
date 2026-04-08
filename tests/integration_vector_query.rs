//! Vector Query Integration Tests
//!
//! Test scope:
//! - Parser integration for vector queries
//! - Validator integration for vector statements
//! - Planner integration for vector execution plans
//! - Executor integration for vector search operations
//! - End-to-end vector query execution

mod common;

use std::sync::Arc;
use graphdb::core::Context;
use graphdb::query::parser::Parser;
use graphdb::query::validator::Validator;
use graphdb::query::planning::planner::Planner;
use graphdb::query::executor::{Executor, ExecutorFactory, ExecutionContext};
use graphdb::vector::config::VectorConfig;
use graphdb::vector::coordinator::VectorCoordinator;
use graphdb::vector::manager::VectorIndexManager;
use graphdb::storage::{Storage, StorageClient};

/// Test context for vector query integration
struct VectorQueryTestContext {
    storage: Arc<Storage>,
    coordinator: Arc<VectorCoordinator>,
}

impl VectorQueryTestContext {
    /// Create a new test context with in-memory storage
    async fn new() -> Self {
        let storage = Arc::new(
            Storage::new_in_memory()
                .await
                .expect("Failed to create in-memory storage"),
        );

        let vector_config = VectorConfig::default();
        let manager = Arc::new(
            VectorIndexManager::new(vector_config)
                .await
                .expect("Failed to create vector index manager"),
        );
        let coordinator = Arc::new(VectorCoordinator::new(manager));

        Self {
            storage,
            coordinator,
        }
    }

    /// Get a storage client
    fn storage_client(&self) -> Arc<Storage> {
        self.storage.clone()
    }
}

#[tokio::test]
async fn test_parse_vector_search_query() {
    let query = r#"SEARCH VECTOR [0.1, 0.2, 0.3] FROM Document.embedding LIMIT 10"#;
    
    let mut parser = Parser::new();
    let result = parser.parse(query);
    
    assert!(result.is_ok(), "Failed to parse vector search query: {:?}", result.err());
}

#[tokio::test]
async fn test_parse_vector_lookup_query() {
    let query = r#"LOOKUP VECTOR ON Document WHERE embedding SIMILAR TO [0.1, 0.2, 0.3]"#;
    
    let mut parser = Parser::new();
    let result = parser.parse(query);
    
    assert!(result.is_ok(), "Failed to parse vector lookup query: {:?}", result.err());
}

#[tokio::test]
async fn test_parse_vector_match_query() {
    let query = r#"MATCH (n:Document) WHERE n.embedding SIMILAR TO [0.1, 0.2, 0.3] RETURN n"#;
    
    let mut parser = Parser::new();
    let result = parser.parse(query);
    
    assert!(result.is_ok(), "Failed to parse vector match query: {:?}", result.err());
}

#[tokio::test]
async fn test_parse_create_vector_index() {
    let query = r#"CREATE VECTOR INDEX doc_embedding_index ON Document(embedding) OPTIONS {vector_size: 3}"#;
    
    let mut parser = Parser::new();
    let result = parser.parse(query);
    
    assert!(result.is_ok(), "Failed to parse create vector index query: {:?}", result.err());
}

#[tokio::test]
async fn test_parse_drop_vector_index() {
    let query = r#"DROP VECTOR INDEX doc_embedding_index"#;
    
    let mut parser = Parser::new();
    let result = parser.parse(query);
    
    assert!(result.is_ok(), "Failed to parse drop vector index query: {:?}", result.err());
}

#[tokio::test]
async fn test_validate_vector_search_query() {
    let ctx = VectorQueryTestContext::new().await;
    let query = r#"SEARCH VECTOR [0.1, 0.2, 0.3] FROM Document.embedding LIMIT 10"#;
    
    // Parse
    let mut parser = Parser::new();
    let parse_result = parser.parse(query).expect("Failed to parse query");
    
    // Validate
    let mut validator = Validator::new();
    let context = Context::default();
    let validate_result = validator.validate(&parse_result.stmt, &context);
    
    assert!(validate_result.is_ok(), "Failed to validate vector search query: {:?}", validate_result.err());
}

#[tokio::test]
async fn test_plan_vector_search_query() {
    let ctx = VectorQueryTestContext::new().await;
    let query = r#"SEARCH VECTOR [0.1, 0.2, 0.3] FROM Document.embedding LIMIT 10"#;
    
    // Parse
    let mut parser = Parser::new();
    let parse_result = parser.parse(query).expect("Failed to parse query");
    
    // Validate
    let mut validator = Validator::new();
    let context = Context::default();
    let validated_stmt = validator.validate(&parse_result.stmt, &context)
        .expect("Failed to validate query");
    
    // Plan
    let mut planner = Planner::new();
    let plan_result = planner.create_plan(&validated_stmt);
    
    assert!(plan_result.is_ok(), "Failed to create plan for vector search query: {:?}", plan_result.err());
}

#[tokio::test]
async fn test_execute_vector_search_query() {
    let ctx = VectorQueryTestContext::new().await;
    let query = r#"SEARCH VECTOR [0.1, 0.2, 0.3] FROM Document.embedding LIMIT 10"#;
    
    // Parse
    let mut parser = Parser::new();
    let parse_result = parser.parse(query).expect("Failed to parse query");
    
    // Validate
    let mut validator = Validator::new();
    let context = Context::default();
    let validated_stmt = validator.validate(&parse_result.stmt, &context)
        .expect("Failed to validate query");
    
    // Plan
    let mut planner = Planner::new();
    let plan = planner.create_plan(&validated_stmt)
        .expect("Failed to create plan");
    
    // Execute
    let storage = ctx.storage_client();
    let mut executor_factory = ExecutorFactory::new();
    let expression_context = Arc::new(graphdb::query::validator::ExpressionAnalysisContext::default());
    let search_engine = Arc::new(graphdb::query::executor::data_access::fulltext_search::MockSearchEngine::new());
    
    let exec_context = ExecutionContext::with_vector_coordinator(
        expression_context,
        search_engine,
        ctx.coordinator.clone(),
    );
    
    let mut executor = executor_factory.create_executor(&plan, storage, &exec_context)
        .expect("Failed to create executor");
    
    let result = executor.execute();
    
    // Should execute without errors (even if no results)
    assert!(result.is_ok(), "Failed to execute vector search query: {:?}", result.err());
}

#[tokio::test]
async fn test_vector_query_pipeline() {
    let ctx = VectorQueryTestContext::new().await;
    
    // Create vector index
    let create_index_query = r#"CREATE VECTOR INDEX doc_embedding_index ON Document(embedding) OPTIONS {vector_size: 3}"#;
    let mut parser = Parser::new();
    let parse_result = parser.parse(create_index_query).expect("Failed to parse create index query");
    
    let mut validator = Validator::new();
    let context = Context::default();
    let validated_stmt = validator.validate(&parse_result.stmt, &context)
        .expect("Failed to validate create index query");
    
    let mut planner = Planner::new();
    let plan = planner.create_plan(&validated_stmt)
        .expect("Failed to create plan for create index");
    
    let storage = ctx.storage_client();
    let mut executor_factory = ExecutorFactory::new();
    let expression_context = Arc::new(graphdb::query::validator::ExpressionAnalysisContext::default());
    let search_engine = Arc::new(graphdb::query::executor::data_access::fulltext_search::MockSearchEngine::new());
    
    let exec_context = ExecutionContext::with_vector_coordinator(
        expression_context,
        search_engine,
        ctx.coordinator.clone(),
    );
    
    let mut executor = executor_factory.create_executor(&plan, storage.clone(), &exec_context)
        .expect("Failed to create executor for create index");
    
    let result = executor.execute();
    assert!(result.is_ok(), "Failed to execute create vector index: {:?}", result.err());
    
    // Search vector
    let search_query = r#"SEARCH VECTOR [0.1, 0.2, 0.3] FROM Document.embedding LIMIT 10"#;
    let parse_result = parser.parse(search_query).expect("Failed to parse search query");
    let validated_stmt = validator.validate(&parse_result.stmt, &context)
        .expect("Failed to validate search query");
    let plan = planner.create_plan(&validated_stmt)
        .expect("Failed to create plan for search");
    
    let mut executor = executor_factory.create_executor(&plan, storage, &exec_context)
        .expect("Failed to create executor for search");
    
    let result = executor.execute();
    assert!(result.is_ok(), "Failed to execute vector search: {:?}", result.err());
}

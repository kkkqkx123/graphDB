//! Full-Text Search Executor
//!
//! This module implements the executor for full-text search queries,
//! including SEARCH statements and full-text scan operations.

use crate::query::executor::base::{BaseExecutor, DBResult, ExecutionResult, Executor, ExecutorStats, HasStorage};
use crate::query::executor::ExecutionContext;
use crate::query::parser::ast::fulltext::SearchStatement;
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::search::SearchEngine;
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

/// Full-text search executor
pub struct FulltextSearchExecutor<S: StorageClient> {
    /// Base executor
    base: BaseExecutor<S>,
    /// Search statement
    statement: SearchStatement,
    /// Search engine reference
    engine: Arc<dyn SearchEngine>,
    /// Execution context
    context: ExecutionContext,
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient> FulltextSearchExecutor<S> {
    /// Create a new full-text search executor
    pub fn new(
        id: i64,
        statement: SearchStatement,
        engine: Arc<dyn SearchEngine>,
        context: ExecutionContext,
        storage: Arc<Mutex<S>>,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "FulltextSearchExecutor".to_string(), storage, expr_context),
            statement,
            engine,
            context,
            _phantom: std::marker::PhantomData,
        }
    }

}

/// Full-text scan executor for LOOKUP operations
pub struct FulltextScanExecutor<S: StorageClient> {
    /// Base executor
    base: BaseExecutor<S>,
    /// Index name
    index_name: String,
    /// Search query
    query: String,
    /// Search engine reference
    engine: Arc<dyn SearchEngine>,
    /// Execution context
    context: ExecutionContext,
    /// Limit
    limit: Option<usize>,
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient> FulltextScanExecutor<S> {
    /// Create a new full-text scan executor
    pub fn new(
        id: i64,
        index_name: String,
        query: String,
        engine: Arc<dyn SearchEngine>,
        context: ExecutionContext,
        storage: Arc<Mutex<S>>,
        expr_context: Arc<ExpressionAnalysisContext>,
        limit: Option<usize>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "FulltextScanExecutor".to_string(), storage, expr_context),
            index_name,
            query,
            engine,
            context,
            limit,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<S: StorageClient> Executor<S> for FulltextSearchExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        Ok(ExecutionResult::Empty)
    }

    fn open(&mut self) -> DBResult<()> {
        self.base.open()
    }

    fn close(&mut self) -> DBResult<()> {
        self.base.close()
    }

    fn is_open(&self) -> bool {
        self.base.is_open()
    }

    fn id(&self) -> i64 {
        self.base.id()
    }

    fn name(&self) -> &str {
        "FulltextSearchExecutor"
    }

    fn description(&self) -> &str {
        "Fulltext Search Executor"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for FulltextSearchExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient> Executor<S> for FulltextScanExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        Ok(ExecutionResult::Empty)
    }

    fn open(&mut self) -> DBResult<()> {
        self.base.open()
    }

    fn close(&mut self) -> DBResult<()> {
        self.base.close()
    }

    fn is_open(&self) -> bool {
        self.base.is_open()
    }

    fn id(&self) -> i64 {
        self.base.id()
    }

    fn name(&self) -> &str {
        "FulltextScanExecutor"
    }

    fn description(&self) -> &str {
        "Fulltext Scan Executor"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for FulltextScanExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        // Test that executor can be created
        // Note: Actual execution tests require a mock search engine
        let statement = SearchStatement::new(
            "test_index".to_string(),
            FulltextQueryExpr::Simple("test".to_string()),
        );

        // Executor creation test would go here with a mock engine
        assert_eq!(statement.index_name, "test_index");
    }

    #[test]
    fn test_query_conversion() {
        let simple = FulltextQueryExpr::Simple("database".to_string());
        assert!(matches!(simple, FulltextQueryExpr::Simple(_)));

        let field = FulltextQueryExpr::Field("title".to_string(), "database".to_string());
        assert!(matches!(field, FulltextQueryExpr::Field(_, _)));

        let boolean = FulltextQueryExpr::Boolean {
            must: vec![],
            should: vec![],
            must_not: vec![],
        };
        assert!(matches!(boolean, FulltextQueryExpr::Boolean { .. }));
    }
}

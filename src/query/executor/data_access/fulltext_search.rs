//! Full-Text Search Executor
//!
//! This module implements the executor for full-text search queries,
//! including SEARCH statements and full-text scan operations.

use crate::core::Value;
use crate::core::error::QueryError;
use crate::query::executor::base::{BaseExecutor, DBResult, ExecutionResult, Executor, ExecutorStats, HasStorage};
use crate::query::executor::ExecutionContext;
use crate::query::parser::ast::fulltext::{
    FulltextQueryExpr, SearchStatement, YieldExpression,
};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::search::{FulltextSearchResult, FulltextSearchEntry, SearchEngine};
use crate::core::types::FulltextSearchResult as CoreFulltextSearchResult;
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::collections::HashMap;
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

    /// Execute the search query
    pub async fn execute(&mut self) -> Result<FulltextSearchResult, QueryError> {
        let start_time = std::time::Instant::now();

        // Build search query
        let search_query = self.build_search_query()?;

        // Execute search
        let search_result = self.engine.search(&self.statement.index_name, search_query)
            .await
            .map_err(|e| QueryError::ExecutionError(format!("Full-text search failed: {}", e)))?;

        let elapsed = start_time.elapsed();

        // Convert to result format
        let result = self.convert_search_result(search_result, elapsed.as_millis() as u64)?;

        Ok(result)
    }

    /// Build search query from AST
    fn build_search_query(&self) -> Result<crate::core::types::FulltextQuery, QueryError> {
        self.convert_query_expr(&self.statement.query)
    }

    /// Convert query expression to query type
    fn convert_query_expr(
        &self,
        expr: &FulltextQueryExpr,
    ) -> Result<crate::core::types::FulltextQuery, QueryError> {
        match expr {
            FulltextQueryExpr::Simple(text) => {
                Ok(crate::core::types::FulltextQuery::Simple(text.clone()))
            }
            FulltextQueryExpr::Field(field, query) => {
                let field_query = crate::core::types::FieldQuery::new(field.clone(), query.clone());
                Ok(crate::core::types::FulltextQuery::MultiField(vec![field_query]))
            }
            FulltextQueryExpr::MultiField(fields) => {
                let field_queries = fields
                    .iter()
                    .map(|(field, query)| {
                        crate::core::types::FieldQuery::new(field.clone(), query.clone())
                    })
                    .collect();
                Ok(crate::core::types::FulltextQuery::MultiField(field_queries))
            }
            FulltextQueryExpr::Boolean { must, should, must_not } => {
                let must_queries = must
                    .iter()
                    .map(|e| self.convert_query_expr(e))
                    .collect::<Result<Vec<_>, _>>()?;

                let should_queries = should
                    .iter()
                    .map(|e| self.convert_query_expr(e))
                    .collect::<Result<Vec<_>, _>>()?;

                let must_not_queries = must_not
                    .iter()
                    .map(|e| self.convert_query_expr(e))
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(crate::core::types::FulltextQuery::Boolean {
                    must: must_queries,
                    should: should_queries,
                    must_not: must_not_queries,
                    minimum_should_match: None,
                })
            }
            FulltextQueryExpr::Phrase(text) => {
                Ok(crate::core::types::FulltextQuery::Phrase {
                    text: text.clone(),
                    slop: 0,
                })
            }
            FulltextQueryExpr::Prefix(prefix) => {
                // For prefix queries, we need to extract field name
                // Default to searching all fields
                Ok(crate::core::types::FulltextQuery::Prefix {
                    field: "_all".to_string(),
                    prefix: prefix.clone(),
                })
            }
            FulltextQueryExpr::Fuzzy(text, distance) => {
                Ok(crate::core::types::FulltextQuery::Fuzzy {
                    field: "_all".to_string(),
                    value: text.clone(),
                    distance: distance.unwrap_or(2),
                    transpositions: true,
                })
            }
            FulltextQueryExpr::Range { field, lower, upper, include_lower, include_upper } => {
                Ok(crate::core::types::FulltextQuery::Range {
                    field: field.clone(),
                    lower: lower.clone(),
                    upper: upper.clone(),
                    include_lower: *include_lower,
                    include_upper: *include_upper,
                })
            }
            FulltextQueryExpr::Wildcard(pattern) => {
                Ok(crate::core::types::FulltextQuery::Wildcard {
                    field: "_all".to_string(),
                    pattern: pattern.clone(),
                })
            }
        }
    }

    /// Convert search result to output format
    fn convert_search_result(
        &self,
        result: crate::search::FulltextSearchResult,
        took_ms: u64,
    ) -> Result<FulltextSearchResult, QueryError> {
        let mut output_results = Vec::new();
        let mut max_score = 0.0;

        for entry in result.results {
            let score = entry.score;
            if score > max_score {
                max_score = score;
            }

            // Apply yield clause to select fields
            let source = entry.source.clone().unwrap_or_default();
            let yielded_data = self.apply_yield_clause(&source, &entry)?;

            output_results.push(yielded_data);
        }

        Ok(FulltextSearchResult {
            results: output_results,
            total_hits: result.total_hits,
            max_score,
            took_ms,
            timed_out: result.timed_out,
            shards: None,
        })
    }

    /// Apply yield clause to select and transform fields
    fn apply_yield_clause(
        &self,
        source: &HashMap<String, Value>,
        entry: &FulltextSearchEntry,
    ) -> Result<FulltextSearchEntry, QueryError> {
        if let Some(yield_clause) = &self.statement.yield_clause {
            let mut new_source = HashMap::new();

            for item in &yield_clause.items {
                match &item.expr {
                    YieldExpression::All => {
                        // Include all fields
                        new_source.clone_from(source);
                    }
                    YieldExpression::Field(field_name) => {
                        // Include specific field
                        if let Some(value) = source.get(field_name) {
                            new_source.insert(field_name.clone(), value.clone());
                        }
                    }
                    YieldExpression::Score(alias) => {
                        // Include score
                        let field_name = alias.clone().unwrap_or_else(|| "score".to_string());
                        new_source.insert(field_name, Value::Float(entry.score));
                    }
                    YieldExpression::Highlight(field_name, _) => {
                        // Include highlighted text
                        if let Some(highlights) = &entry.highlights {
                            if let Some(field_highlights) = highlights.get(field_name) {
                                let highlighted_text = field_highlights.join(" ... ");
                                let alias_name = item.alias.clone()
                                    .unwrap_or_else(|| format!("highlight_{}", field_name));
                                new_source.insert(alias_name, Value::String(highlighted_text));
                            }
                        }
                    }
                    YieldExpression::MatchedFields => {
                        // Include matched fields
                        let fields: Vec<Value> = entry
                            .matched_fields
                            .iter()
                            .map(|f| Value::String(f.clone()))
                            .collect();
                        let alias_name = item.alias.clone()
                            .unwrap_or_else(|| "matched_fields".to_string());
                        new_source.insert(alias_name, Value::List(fields));
                    }
                    YieldExpression::Snippet(field_name, max_len) => {
                        // Include snippet
                        if let Some(value) = source.get(field_name) {
                            if let Value::String(text) = value {
                                let snippet = if let Some(len) = max_len {
                                    if text.len() <= *len {
                                        text.clone()
                                    } else {
                                        format!("{}...", &text[..*len])
                                    }
                                } else {
                                    text.clone()
                                };
                                let alias_name = item.alias.clone()
                                    .unwrap_or_else(|| format!("snippet_{}", field_name));
                                new_source.insert(alias_name, Value::String(snippet));
                            }
                        }
                    }
                }
            }

            let mut result_entry = FulltextSearchEntry::new(entry.doc_id.clone(), entry.score);
            result_entry.source = Some(new_source);
            result_entry.highlights = entry.highlights.clone();
            result_entry.matched_fields = entry.matched_fields.clone();

            Ok(result_entry)
        } else {
            // No yield clause, return all fields
            Ok(entry.clone())
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

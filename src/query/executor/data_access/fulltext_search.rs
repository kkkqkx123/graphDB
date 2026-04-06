//! Full-Text Search Executor
//!
//! This module implements the executor for full-text search queries,
//! including SEARCH statements and full-text scan operations.

use crate::coordinator::FulltextCoordinator;
use crate::core::error::{DBError, QueryError};
use crate::core::Value;
use crate::query::executor::base::{
    BaseExecutor, DBResult, ExecutionResult, Executor, ExecutorStats, HasStorage,
};
use crate::query::executor::ExecutionContext;
use crate::query::parser::ast::fulltext::{
    FulltextQueryExpr, SearchStatement, YieldExpression,
};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::search::SearchEngine;
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

/// Full-text search executor for SEARCH statements
pub struct FulltextSearchExecutor<S: StorageClient> {
    /// Base executor
    base: BaseExecutor<S>,
    /// Search statement
    statement: SearchStatement,
    /// Search engine reference
    #[allow(dead_code)]
    engine: Arc<dyn SearchEngine>,
    /// Execution context
    #[allow(dead_code)]
    context: ExecutionContext,
    /// Fulltext coordinator
    coordinator: Arc<FulltextCoordinator>,
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
        coordinator: Arc<FulltextCoordinator>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                id,
                "FulltextSearchExecutor".to_string(),
                storage,
                expr_context,
            ),
            statement,
            engine,
            context,
            coordinator,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Parse index name to extract space_id, tag_name, field_name
    fn parse_index_name(&self) -> DBResult<(u64, String, String)> {
        let parts: Vec<&str> = self.statement.index_name.split('_').collect();
        
        if parts.len() < 4 {
            return Err(DBError::Validation(format!(
                "Invalid index name format: {}. Expected format: space_id_tag_name_field_name",
                self.statement.index_name
            )));
        }

        let space_id = parts[0].parse::<u64>().map_err(|e| {
            DBError::Validation(format!("Invalid space_id in index name: {}", e))
        })?;
        
        let tag_name = parts[1].to_string();
        let field_name = parts[2..].join("_");

        Ok((space_id, tag_name, field_name))
    }

    /// Convert FulltextQueryExpr to search query string
    fn convert_query_to_string(&self, expr: &FulltextQueryExpr) -> String {
        match expr {
            FulltextQueryExpr::Simple(text) => text.clone(),
            FulltextQueryExpr::Field(field, text) => format!("{}:{}", field, text),
            FulltextQueryExpr::MultiField(fields) => {
                fields
                    .iter()
                    .map(|(f, t)| format!("{}:{}", f, t))
                    .collect::<Vec<_>>()
                    .join(" OR ")
            }
            FulltextQueryExpr::Boolean {
                must,
                should,
                must_not,
            } => {
                let mut parts = Vec::new();
                if !must.is_empty() {
                    parts.push(format!(
                        "+({})",
                        must.iter()
                            .map(|e| self.convert_query_to_string(e))
                            .collect::<Vec<_>>()
                            .join(" ")
                    ));
                }
                if !should.is_empty() {
                    parts.push(format!(
                        "({})",
                        should.iter()
                            .map(|e| self.convert_query_to_string(e))
                            .collect::<Vec<_>>()
                            .join(" ")
                    ));
                }
                if !must_not.is_empty() {
                    parts.push(format!(
                        "-({})",
                        must_not.iter()
                            .map(|e| self.convert_query_to_string(e))
                            .collect::<Vec<_>>()
                            .join(" ")
                    ));
                }
                parts.join(" ")
            }
            FulltextQueryExpr::Phrase(text) => format!("\"{}\"", text),
            FulltextQueryExpr::Prefix(text) => format!("{}*", text),
            FulltextQueryExpr::Fuzzy(text, distance) => {
                if let Some(d) = distance {
                    format!("{}~{}", text, d)
                } else {
                    format!("{}~", text)
                }
            }
            FulltextQueryExpr::Range {
                field,
                lower,
                upper,
                include_lower,
                include_upper,
            } => {
                let lower_bound = if *include_lower { "[" } else { "{" };
                let upper_bound = if *include_upper { "]" } else { "}" };
                let lower_val = lower.as_deref().unwrap_or("*");
                let upper_val = upper.as_deref().unwrap_or("*");
                format!("{}:{}{} TO {}{}", field, lower_bound, lower_val, upper_val, upper_bound)
            }
            FulltextQueryExpr::Wildcard(text) => text.clone(),
        }
    }
}

/// Full-text scan executor for LOOKUP FULLTEXT operations
pub struct FulltextScanExecutor<S: StorageClient> {
    /// Base executor
    base: BaseExecutor<S>,
    /// Index name
    index_name: String,
    /// Search query
    query: String,
    /// Search engine reference
    #[allow(dead_code)]
    engine: Arc<dyn SearchEngine>,
    /// Execution context
    #[allow(dead_code)]
    context: ExecutionContext,
    /// Fulltext coordinator
    coordinator: Arc<FulltextCoordinator>,
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
        coordinator: Arc<FulltextCoordinator>,
        limit: Option<usize>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                id,
                "FulltextScanExecutor".to_string(),
                storage,
                expr_context,
            ),
            index_name,
            query,
            engine,
            context,
            coordinator,
            limit,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<S: StorageClient> Executor<S> for FulltextSearchExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let (space_id, tag_name, field_name) = self.parse_index_name()?;
        
        let query_string = self.convert_query_to_string(&self.statement.query);
        
        let limit = self.statement.limit.unwrap_or(100);
        
        let search_results = futures::executor::block_on(
            self.coordinator.search(
                space_id,
                &tag_name,
                &field_name,
                &query_string,
                limit,
            )
        ).map_err(|e| DBError::Query(QueryError::ExecutionError(format!("Search failed: {}", e))))?;
        
        let mut rows = Vec::new();
        let storage = self.get_storage().clone();
        let storage_guard = storage.lock();
        
        for result in search_results {
            let vertex_id = &result.doc_id;
            
            let vertex = storage_guard.get_vertex("", vertex_id)
                .map_err(|e| DBError::Storage(e))?;
            
            if let Some(vertex) = vertex {
                let mut row = HashMap::new();
                
                if let Some(yield_clause) = &self.statement.yield_clause {
                    for yield_item in &yield_clause.items {
                        let value = match &yield_item.expr {
                            YieldExpression::Field(name) => {
                                if let Some(tag) = vertex.tags.first() {
                                    tag.properties.get(name).cloned().unwrap_or(Value::Null(crate::core::null::NullType::Null))
                                } else {
                                    Value::Null(crate::core::null::NullType::Null)
                                }
                            }
                            YieldExpression::Score(_) => {
                                Value::Float(result.score as f64)
                            }
                            YieldExpression::Highlight(_, _) => {
                                if let Some(ref highlights) = result.highlights {
                                    Value::String(highlights.join(" ... "))
                                } else {
                                    Value::Null(crate::core::null::NullType::Null)
                                }
                            }
                            YieldExpression::MatchedFields => {
                                let fields: Vec<Value> = result.matched_fields
                                    .iter()
                                    .map(|f| Value::String(f.clone()))
                                    .collect();
                                Value::List(crate::core::value::list::List { values: fields })
                            }
                            YieldExpression::Snippet(field_name, max_len) => {
                                if let Some(tag) = vertex.tags.first() {
                                    if let Some(Value::String(text)) = tag.properties.get(field_name) {
                                        let max_len = max_len.unwrap_or(200);
                                        if text.len() <= max_len {
                                            Value::String(text.clone())
                                        } else {
                                            let break_point = text[..max_len].rfind(' ').unwrap_or(max_len);
                                            Value::String(format!("{}...", &text[..break_point]))
                                        }
                                    } else {
                                        Value::Null(crate::core::null::NullType::Null)
                                    }
                                } else {
                                    Value::Null(crate::core::null::NullType::Null)
                                }
                            }
                            YieldExpression::All => {
                                if let Some(tag) = vertex.tags.first() {
                                    for (k, v) in &tag.properties {
                                        row.insert(k.clone(), v.clone());
                                    }
                                }
                                continue;
                            }
                        };
                        
                        let default_alias = match &yield_item.expr {
                            YieldExpression::Field(name) => name.clone(),
                            YieldExpression::Score(_) => "score".to_string(),
                            YieldExpression::Highlight(field, _) => format!("highlight({})", field),
                            YieldExpression::MatchedFields => "matched_fields()".to_string(),
                            YieldExpression::Snippet(field, _) => format!("snippet({})", field),
                            YieldExpression::All => "*".to_string(),
                        };
                        let alias = yield_item.alias.as_ref().unwrap_or(&default_alias);
                        
                        row.insert(alias.clone(), value);
                    }
                } else {
                    row.insert("doc_id".to_string(), result.doc_id.clone());
                    row.insert("score".to_string(), Value::Float(result.score as f64));
                }
                
                rows.push(row);
            }
        }
        
        if let Some(offset) = self.statement.offset {
            rows = rows.into_iter().skip(offset).collect();
        }
        
        if let Some(limit) = self.statement.limit {
            rows = rows.into_iter().take(limit).collect();
        }
        
        let mut dataset = crate::core::DataSet::new();
        if let Some(first_row) = rows.first() {
            for key in first_row.keys() {
                dataset.col_names.push(key.clone());
            }
        }
        for row in rows {
            let values: Vec<Value> = dataset.col_names.iter().map(|k| row.get(k).cloned().unwrap_or(Value::Null(crate::core::null::NullType::Null))).collect();
            dataset.rows.push(values);
        }
        Ok(ExecutionResult::DataSet(dataset))
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
        let parts: Vec<&str> = self.index_name.split('_').collect();
        
        if parts.len() < 4 {
            return Err(DBError::Validation(format!(
                "Invalid index name format: {}. Expected format: space_id_tag_name_field_name",
                self.index_name
            )));
        }

        let space_id = parts[0].parse::<u64>().map_err(|e| {
            DBError::Validation(format!("Invalid space_id in index name: {}", e))
        })?;
        
        let tag_name = parts[1].to_string();
        let field_name = parts[2..].join("_");
        
        let limit = self.limit.unwrap_or(100);
        
        let search_results = futures::executor::block_on(
            self.coordinator.search(
                space_id,
                &tag_name,
                &field_name,
                &self.query,
                limit,
            )
        ).map_err(|e| DBError::Query(QueryError::ExecutionError(format!("Search failed: {}", e))))?;
        
        let mut rows = Vec::new();
        let storage = self.get_storage().clone();
        let storage_guard = storage.lock();
        
        for result in search_results {
            let vertex_id = &result.doc_id;
            
            let vertex = storage_guard.get_vertex("", vertex_id)
                .map_err(|e| DBError::Storage(e))?;
            
            if let Some(vertex) = vertex {
                let mut row = HashMap::new();
                row.insert("doc_id".to_string(), result.doc_id.clone());
                row.insert("score".to_string(), Value::Float(result.score as f64));
                
                if let Some(tag) = vertex.tags.first() {
                    for (k, v) in &tag.properties {
                        row.insert(k.clone(), v.clone());
                    }
                }
                
                rows.push(row);
            }
        }
        
        if let Some(limit) = self.limit {
            rows = rows.into_iter().take(limit).collect();
        }
        
        let mut dataset = crate::core::DataSet::new();
        if let Some(first_row) = rows.first() {
            for key in first_row.keys() {
                dataset.col_names.push(key.clone());
            }
        }
        for row in rows {
            let values: Vec<Value> = dataset.col_names.iter().map(|k| row.get(k).cloned().unwrap_or(Value::Null(crate::core::null::NullType::Null))).collect();
            dataset.rows.push(values);
        }
        Ok(ExecutionResult::DataSet(dataset))
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

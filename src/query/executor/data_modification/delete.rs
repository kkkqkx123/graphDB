//! Delete the executor.
//!
//! Responsible for deleting vertex and edge data.
//! Supports both standalone deletion and pipe-based deletion (e.g., GO ... | DELETE VERTEX $-.id).

use std::sync::Arc;
use std::time::Instant;

use crate::core::types::expr::contextual::ContextualExpression;
use crate::core::Value;
use crate::query::executor::base::{BaseExecutor, ExecutorStats};
use crate::query::executor::base::{DBResult, ExecutionResult, Executor, HasStorage};
use crate::query::executor::expression::evaluation_context::DefaultExpressionContext;
use crate::query::executor::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::query::executor::expression::evaluator::traits::ExpressionContext;
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::query::DataSet;
use crate::storage::StorageClient;
use parking_lot::Mutex;

/// Delete the executor.
///
/// Responsible for deleting vertices and edges
pub struct DeleteExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    vertex_ids: Option<Vec<Value>>,
    edge_ids: Option<Vec<(Value, Value, String)>>,
    condition: Option<ContextualExpression>,
    with_edge: bool,
    space_name: String,
}

impl<S: StorageClient> DeleteExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        vertex_ids: Option<Vec<Value>>,
        edge_ids: Option<Vec<(Value, Value, String)>>,
        condition: Option<ContextualExpression>,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "DeleteExecutor".to_string(), storage, expr_context),
            vertex_ids,
            edge_ids,
            condition,
            with_edge: false,
            space_name: "default".to_string(),
        }
    }

    pub fn with_edge(mut self, with_edge: bool) -> Self {
        self.with_edge = with_edge;
        self
    }

    pub fn with_space(mut self, space_name: String) -> Self {
        self.space_name = space_name;
        self
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for DeleteExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        let result = self.do_execute();
        let elapsed = start.elapsed();
        self.base.get_stats_mut().add_total_time(elapsed);
        match result {
            Ok(count) => {
                let dataset = DataSet::from_rows(
                    vec![vec![Value::BigInt(count as i64)]],
                    vec!["count".to_string()],
                );
                Ok(ExecutionResult::DataSet(dataset))
            }
            Err(e) => Err(e),
        }
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        "DeleteExecutor"
    }

    fn description(&self) -> &str {
        "Delete executor - deletes vertices and edges from storage"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for DeleteExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient + Send + Sync + 'static> DeleteExecutor<S> {
    fn do_execute(&mut self) -> DBResult<usize> {
        let mut total_deleted = 0;

        let condition_expression = self.condition.as_ref().and_then(|c| c.get_expression());

        if let Some(ids) = &self.vertex_ids {
            let mut storage = self.get_storage().lock();
            for id in ids {
                let should_delete = if let Some(ref expression) = condition_expression {
                    if let Ok(Some(vertex)) = storage.get_vertex(&self.space_name, id) {
                        let mut context = DefaultExpressionContext::new();
                        context.set_variable("VID".to_string(), id.clone());
                        for (key, value) in &vertex.properties {
                            context.set_variable(key.clone(), value.clone());
                        }

                        let result = ExpressionEvaluator::evaluate(expression, &mut context)
                            .map_err(|e| {
                                crate::core::error::DBError::Query(
                                    crate::core::error::QueryError::ExecutionError(format!(
                                        "Condition evaluation failed: {}",
                                        e
                                    )),
                                )
                            })?;

                        match result {
                            crate::core::Value::Bool(b) => b,
                            _ => true,
                        }
                    } else {
                        true
                    }
                } else {
                    true
                };

                if should_delete {
                    if self.with_edge {
                        let edges = storage
                            .get_node_edges(&self.space_name, id, crate::core::EdgeDirection::Both)
                            .map_err(|e| {
                                crate::core::error::DBError::Storage(
                                    crate::core::error::StorageError::StorageError(format!(
                                        "Failed to retrieve associated edges: {}",
                                        e
                                    )),
                                )
                            })?;
                        for edge in edges {
                            storage
                                .delete_edge(
                                    &self.space_name,
                                    &edge.src,
                                    &edge.dst,
                                    &edge.edge_type,
                                    edge.ranking,
                                )
                                .map_err(|e| {
                                    crate::core::error::DBError::Storage(
                                        crate::core::error::StorageError::StorageError(format!(
                                            "Failed to delete the associated edge: {}",
                                            e
                                        )),
                                    )
                                })?;
                            total_deleted += 1;
                        }
                    }

                    if storage.delete_vertex(&self.space_name, id).is_ok() {
                        total_deleted += 1;
                    }
                }
            }
        }

        if let Some(edges) = &self.edge_ids {
            let mut storage = self.get_storage().lock();
            for (src, dst, edge_type) in edges {
                let should_delete = if let Some(ref expression) = condition_expression {
                    if let Ok(Some(edge)) =
                        storage.get_edge(&self.space_name, src, dst, edge_type, 0)
                    {
                        let mut context = DefaultExpressionContext::new();
                        context.set_variable("SRC".to_string(), src.clone());
                        context.set_variable("DST".to_string(), dst.clone());
                        context.set_variable(
                            "edge_type".to_string(),
                            crate::core::Value::String(edge_type.clone()),
                        );
                        for (key, value) in &edge.props {
                            context.set_variable(key.clone(), value.clone());
                        }

                        let result = ExpressionEvaluator::evaluate(expression, &mut context)
                            .map_err(|e| {
                                crate::core::error::DBError::Query(
                                    crate::core::error::QueryError::ExecutionError(format!(
                                        "Condition evaluation failed: {}",
                                        e
                                    )),
                                )
                            })?;

                        match result {
                            crate::core::Value::Bool(b) => b,
                            _ => true,
                        }
                    } else {
                        true
                    }
                } else {
                    true
                };

                if should_delete {
                    let edges = storage
                        .scan_edges_by_type(&self.space_name, edge_type)
                        .map_err(crate::core::error::DBError::Storage)?;
                    for edge in edges {
                        if *edge.src == *src && *edge.dst == *dst {
                            storage
                                .delete_edge(&self.space_name, src, dst, edge_type, edge.ranking)
                                .map_err(crate::core::error::DBError::Storage)?;
                            total_deleted += 1;
                            break;
                        }
                    }
                }
            }
        }

        Ok(total_deleted)
    }
}

/// Pipe Delete Executor
/// 
/// Handles DELETE statements that receive input from a pipe.
/// Evaluates expressions against input rows to determine what to delete.
pub struct PipeDeleteExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    vertex_id_expressions: Vec<ContextualExpression>,
    edge_expressions: Vec<(ContextualExpression, ContextualExpression, Option<ContextualExpression>)>,
    edge_type: Option<String>,
    condition: Option<ContextualExpression>,
    with_edge: bool,
    space_name: String,
    input_data: Option<DataSet>,
}

impl<S: StorageClient> PipeDeleteExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "PipeDeleteExecutor".to_string(), storage, expr_context),
            vertex_id_expressions: vec![],
            edge_expressions: vec![],
            edge_type: None,
            condition: None,
            with_edge: false,
            space_name: "default".to_string(),
            input_data: None,
        }
    }

    pub fn with_vertex_expressions(mut self, expressions: Vec<ContextualExpression>) -> Self {
        self.vertex_id_expressions = expressions;
        self
    }

    pub fn with_edge_expressions(
        mut self,
        expressions: Vec<(ContextualExpression, ContextualExpression, Option<ContextualExpression>)>,
    ) -> Self {
        self.edge_expressions = expressions;
        self
    }

    pub fn with_edge_type(mut self, edge_type: Option<String>) -> Self {
        self.edge_type = edge_type;
        self
    }

    pub fn with_edge_flag(mut self, with_edge: bool) -> Self {
        self.with_edge = with_edge;
        self
    }

    pub fn with_space(mut self, space_name: String) -> Self {
        self.space_name = space_name;
        self
    }

    pub fn with_condition(mut self, condition: Option<ContextualExpression>) -> Self {
        self.condition = condition;
        self
    }

    pub fn with_input_data(mut self, data: DataSet) -> Self {
        self.input_data = Some(data);
        self
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for PipeDeleteExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        let result = self.do_execute();
        let elapsed = start.elapsed();
        self.base.get_stats_mut().add_total_time(elapsed);
        match result {
            Ok(count) => {
                let dataset = DataSet::from_rows(
                    vec![vec![Value::BigInt(count as i64)]],
                    vec!["count".to_string()],
                );
                Ok(ExecutionResult::DataSet(dataset))
            }
            Err(e) => Err(e),
        }
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        "PipeDeleteExecutor"
    }

    fn description(&self) -> &str {
        "Pipe delete executor - deletes vertices and edges based on pipe input"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for PipeDeleteExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient + Send + Sync + 'static> PipeDeleteExecutor<S> {
    fn do_execute(&mut self) -> DBResult<usize> {
        let mut total_deleted = 0;

        let input_data = self.input_data.as_ref().ok_or_else(|| {
            crate::core::error::DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "PipeDeleteExecutor requires input data".to_string(),
                ),
            )
        })?;

        let col_names = &input_data.col_names;

        if !self.vertex_id_expressions.is_empty() {
            let mut storage = self.get_storage().lock();
            
            for row in &input_data.rows {
                for vid_expr in &self.vertex_id_expressions {
                    let id = self.evaluate_expression_with_row(vid_expr, col_names, row)?;
                    
                    let should_delete = self.check_condition(&storage, &id)?;
                    
                    if should_delete {
                        if self.with_edge {
                            let edges = storage
                                .get_node_edges(&self.space_name, &id, crate::core::EdgeDirection::Both)
                                .map_err(|e| {
                                    crate::core::error::DBError::Storage(
                                        crate::core::error::StorageError::StorageError(format!(
                                            "Failed to retrieve associated edges: {}",
                                            e
                                        )),
                                    )
                                })?;
                            for edge in edges {
                                storage
                                    .delete_edge(
                                        &self.space_name,
                                        &edge.src,
                                        &edge.dst,
                                        &edge.edge_type,
                                        edge.ranking,
                                    )
                                    .map_err(|e| {
                                        crate::core::error::DBError::Storage(
                                            crate::core::error::StorageError::StorageError(format!(
                                                "Failed to delete the associated edge: {}",
                                                e
                                            )),
                                        )
                                    })?;
                                total_deleted += 1;
                            }
                        }

                        if storage.delete_vertex(&self.space_name, &id).is_ok() {
                            total_deleted += 1;
                        }
                    }
                }
            }
        }

        if !self.edge_expressions.is_empty() {
            let mut storage = self.get_storage().lock();
            let edge_type = self.edge_type.clone().unwrap_or_else(|| "UNKNOWN".to_string());
            
            for row in &input_data.rows {
                for (src_expr, dst_expr, _rank_expr) in &self.edge_expressions {
                    let src = self.evaluate_expression_with_row(src_expr, col_names, row)?;
                    let dst = self.evaluate_expression_with_row(dst_expr, col_names, row)?;
                    
                    let edges = storage
                        .scan_edges_by_type(&self.space_name, &edge_type)
                        .map_err(crate::core::error::DBError::Storage)?;
                    
                    for edge in edges {
                        if *edge.src == src && *edge.dst == dst {
                            storage
                                .delete_edge(&self.space_name, &src, &dst, &edge_type, edge.ranking)
                                .map_err(crate::core::error::DBError::Storage)?;
                            total_deleted += 1;
                            break;
                        }
                    }
                }
            }
        }

        Ok(total_deleted)
    }

    fn evaluate_expression_with_row(
        &self,
        expr: &ContextualExpression,
        col_names: &[String],
        row: &[Value],
    ) -> DBResult<Value> {
        let expression = expr.get_expression().ok_or_else(|| {
            crate::core::error::DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "Expression not found in ContextualExpression".to_string(),
                ),
            )
        })?;

        let mut context = DefaultExpressionContext::new();
        
        for (i, col_name) in col_names.iter().enumerate() {
            if i < row.len() {
                context.set_variable(col_name.clone(), row[i].clone());
            }
        }

        ExpressionEvaluator::evaluate(&expression, &mut context).map_err(|e| {
            crate::core::error::DBError::Query(
                crate::core::error::QueryError::ExecutionError(format!(
                    "Expression evaluation failed: {}",
                    e
                )),
            )
        })
    }

    fn check_condition(
        &self,
        _storage: &S,
        _id: &Value,
    ) -> DBResult<bool> {
        Ok(true)
    }
}

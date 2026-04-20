//! Delete the executor.
//!
//! Responsible for deleting vertex and edge data

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
    with_edge: bool, // Should the associated edges be deleted in a cascading manner?
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

    /// Set whether to delete associated edges in a cascading manner.
    pub fn with_edge(mut self, with_edge: bool) -> Self {
        self.with_edge = with_edge;
        self
    }

    /// Set the name of the space.
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

        // Obtain the Expression directly from the ContextualExpression.
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
                    // If cascading deletion is enabled, the associated edges are deleted first.
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
                    // Use scan and delete approach for edges without specific rank
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

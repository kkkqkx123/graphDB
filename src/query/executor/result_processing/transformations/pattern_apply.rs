//! PatternApplyExecutor实现
//!
//! 负责处理模式匹配操作，将输入数据与指定模式进行匹配

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult};
use crate::core::types::EdgeDirection;
use crate::core::Expression;
use crate::core::{DataSet, Edge, Path, Value, Vertex};
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::expression::evaluator::traits::ExpressionContext;
use crate::expression::DefaultExpressionContext;
use crate::query::executor::base::BaseExecutor;
use crate::query::executor::traits::{ExecutionResult, Executor};
use crate::storage::StorageEngine;

/// 模式类型
#[derive(Debug, Clone, PartialEq)]
pub enum PatternType {
    /// 节点模式
    Node {
        variable: Option<String>,
        labels: Vec<String>,
        properties: HashMap<String, Expression>,
    },
    /// 边模式
    Edge {
        variable: Option<String>,
        edge_type: Option<String>,
        direction: EdgeDirection,
        properties: HashMap<String, Expression>,
    },
    /// 路径模式
    Path {
        variable: Option<String>,
        length_range: Option<(usize, Option<usize>)>,
    },
}

/// PatternApply执行器
/// 用于将输入数据与指定模式进行匹配
pub struct PatternApplyExecutor<S: StorageEngine + Send + 'static> {
    base: BaseExecutor<S>,
    input_var: String,
    pattern: PatternType,
    col_names: Vec<String>,
    track_prev_path: bool,
}

impl<S: StorageEngine + Send + 'static> PatternApplyExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        input_var: String,
        pattern: PatternType,
        col_names: Vec<String>,
        track_prev_path: bool,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "PatternApplyExecutor".to_string(), storage),
            input_var,
            pattern,
            col_names,
            track_prev_path,
        }
    }

    pub fn with_context(
        id: i64,
        storage: Arc<Mutex<S>>,
        input_var: String,
        pattern: PatternType,
        col_names: Vec<String>,
        track_prev_path: bool,
        context: crate::query::executor::base::ExecutionContext,
    ) -> Self {
        Self {
            base: BaseExecutor::with_context(
                id,
                "PatternApplyExecutor".to_string(),
                storage,
                context,
            ),
            input_var,
            pattern,
            col_names,
            track_prev_path,
        }
    }

    fn match_node_pattern<C: ExpressionContext>(
        &self,
        vertex: &Vertex,
        pattern: &PatternType,
        expr_context: &C,
    ) -> DBResult<bool> {
        if let PatternType::Node {
            labels, properties, ..
        } = pattern
        {
            if !labels.is_empty() {
                let vertex_labels: Vec<String> =
                    vertex.tags.iter().map(|tag| tag.name.clone()).collect();

                for label in labels {
                    if !vertex_labels.contains(label) {
                        return Ok(false);
                    }
                }
            }

            if !properties.is_empty() {
                for (prop_name, prop_expr) in properties {
                    let prop_value = vertex
                        .properties
                        .get(prop_name)
                        .cloned()
                        .unwrap_or(Value::Null(crate::core::NullType::Null));

                    let mut temp_context = DefaultExpressionContext::new();
                    if let Some(variables) = expr_context.get_all_variables() {
                        for (name, value) in variables {
                            temp_context.set_variable(name, value);
                        }
                    }
                    temp_context.set_variable("_".to_string(), prop_value);

                    let result = ExpressionEvaluator::evaluate(prop_expr, &mut temp_context)
                        .map_err(|e| {
                            DBError::Query(crate::core::error::QueryError::ExecutionError(
                                e.to_string(),
                            ))
                        })?;

                    if let Value::Bool(false) = result {
                        return Ok(false);
                    }
                }
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn match_edge_pattern<C: ExpressionContext>(
        &self,
        edge: &Edge,
        pattern: &PatternType,
        expr_context: &C,
    ) -> DBResult<bool> {
        if let PatternType::Edge {
            edge_type,
            direction,
            properties,
            ..
        } = pattern
        {
            if let Some(expected_type) = edge_type {
                if edge.edge_type != *expected_type {
                    return Ok(false);
                }
            }

            if let EdgeDirection::Incoming = direction {
            }

            if !properties.is_empty() {
                for (prop_name, prop_expr) in properties {
                    let prop_value = edge
                        .properties()
                        .get(prop_name)
                        .cloned()
                        .unwrap_or(Value::Null(crate::core::NullType::Null));

                    let mut temp_context = DefaultExpressionContext::new();
                    if let Some(variables) = expr_context.get_all_variables() {
                        for (name, value) in variables {
                            temp_context.set_variable(name, value);
                        }
                    }
                    temp_context.set_variable("_".to_string(), prop_value);

                    let result = ExpressionEvaluator::evaluate(prop_expr, &mut temp_context)
                        .map_err(|e| {
                            DBError::Query(crate::core::error::QueryError::ExecutionError(
                                e.to_string(),
                            ))
                        })?;

                    if let Value::Bool(false) = result {
                        return Ok(false);
                    }
                }
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn match_path_pattern<C: ExpressionContext>(
        &self,
        path: &Path,
        pattern: &PatternType,
        _expr_context: &C,
    ) -> DBResult<bool> {
        if let PatternType::Path { length_range, .. } = pattern {
            let path_length = path.steps.len();

            if let Some((min_len, max_len)) = length_range {
                if path_length < *min_len {
                    return Ok(false);
                }

                if let Some(max) = max_len {
                    if path_length > *max {
                        return Ok(false);
                    }
                }
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn execute_pattern_apply(&mut self) -> DBResult<DataSet> {
        let input_result = self
            .base
            .context
            .get_result(&self.input_var)
            .ok_or_else(|| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                    "Input variable '{}' not found",
                    self.input_var
                )))
            })?;

        let mut expr_context = DefaultExpressionContext::new();

        for (name, value) in &self.base.context.variables.clone() {
            expr_context.set_variable(name.clone(), value.clone());
        }

        let mut dataset = DataSet {
            col_names: self.col_names.clone(),
            rows: Vec::new(),
        };

        match input_result {
            ExecutionResult::Values(values) => {
                for value in values {
                    expr_context.set_variable("_".to_string(), value.clone());

                    let matches = match &self.pattern {
                        PatternType::Node { .. } => {
                            if let Value::Vertex(vertex) = value {
                                self.match_node_pattern(&vertex, &self.pattern, &expr_context)?
                            } else {
                                false
                            }
                        }
                        PatternType::Edge { .. } => {
                            if let Value::Edge(edge) = value {
                                self.match_edge_pattern(&edge, &self.pattern, &expr_context)?
                            } else {
                                false
                            }
                        }
                        PatternType::Path { .. } => {
                            if let Value::Path(path) = value {
                                self.match_path_pattern(&path, &self.pattern, &expr_context)?
                            } else {
                                false
                            }
                        }
                    };

                    if matches {
                        let mut row = Vec::new();
                        row.push(value.clone());
                        dataset.rows.push(row);
                    }
                }
            }
            ExecutionResult::Vertices(vertices) => {
                for vertex in vertices {
                    let vertex_value = Value::Vertex(Box::new(vertex.clone()));
                    expr_context.set_variable("_".to_string(), vertex_value.clone());

                    if let PatternType::Node { .. } = &self.pattern {
                        if self.match_node_pattern(&vertex, &self.pattern, &expr_context)? {
                            let mut row = Vec::new();
                            row.push(vertex_value);
                            dataset.rows.push(row);
                        }
                    }
                }
            }
            ExecutionResult::Edges(edges) => {
                for edge in edges {
                    let edge_value = Value::Edge(edge.clone());
                    expr_context.set_variable("_".to_string(), edge_value.clone());

                    if let PatternType::Edge { .. } = &self.pattern {
                        if self.match_edge_pattern(&edge, &self.pattern, &expr_context)? {
                            let mut row = Vec::new();
                            row.push(edge_value);
                            dataset.rows.push(row);
                        }
                    }
                }
            }
            _ => {
                return Err(DBError::Query(
                    crate::core::error::QueryError::ExecutionError(
                        "Invalid input result type for PatternApply".to_string(),
                    ),
                ));
            }
        }

        Ok(dataset)
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for PatternApplyExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let dataset = self.execute_pattern_apply()?;

        let values: Vec<Value> = dataset
            .rows
            .into_iter()
            .flat_map(|row| row.into_iter())
            .collect();

        Ok(ExecutionResult::Values(values))
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.base.is_open()
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        &self.base.description
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageEngine + Send + 'static> crate::query::executor::traits::HasStorage<S>
    for PatternApplyExecutor<S>
{
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("PatternApplyExecutor storage should be set")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::test_config::test_config;
    use crate::core::{Tag, Value, Vertex};
    use crate::storage::MockStorage;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_pattern_apply_executor() {
        let config = test_config();
        let storage = Arc::new(Mutex::new(MockStorage));

        let tag = Tag {
            name: "Person".to_string(),
            properties: HashMap::new(),
        };

        let vertex = Vertex {
            vid: Box::new(Value::String("vertex1".to_string())),
            id: 1,
            tags: vec![tag],
            properties: HashMap::new(),
        };

        let vertex_value = Value::Vertex(Box::new(vertex));

        let input_result = ExecutionResult::Values(vec![vertex_value.clone()]);

        let mut context = crate::query::executor::base::ExecutionContext::new();
        context.set_result("input".to_string(), input_result);

        let pattern = PatternType::Node {
            variable: Some("v".to_string()),
            labels: vec!["Person".to_string()],
            properties: HashMap::new(),
        };

        let mut executor = PatternApplyExecutor::with_context(
            1,
            storage,
            "input".to_string(),
            pattern,
            vec!["matched".to_string()],
            false,
            context,
        );

        let result = executor
            .execute()
            .await
            .expect("Executor should execute successfully");

        if let ExecutionResult::Values(values) = result {
            assert_eq!(values.len(), 1);
            assert_eq!(values[0], vertex_value);
        } else {
            panic!("Expected Values result");
        }
    }
}

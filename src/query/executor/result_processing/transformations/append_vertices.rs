//! AppendVerticesExecutor实现
//!
//! 负责处理追加顶点操作，根据给定的顶点ID获取顶点信息并追加到结果中

use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult};
use crate::core::Expression;
use crate::core::{DataSet, Value, Vertex};
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::expression::evaluator::traits::ExpressionContext;
use crate::expression::DefaultExpressionContext;
use crate::query::executor::base::BaseExecutor;
use crate::query::executor::traits::{ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;

/// AppendVertices执行器
/// 用于根据顶点ID获取顶点信息并追加到结果中
pub struct AppendVerticesExecutor<S: StorageClient + Send + 'static> {
    base: BaseExecutor<S>,
    /// 输入变量名
    input_var: String,
    /// 源表达式，用于获取顶点ID
    src_expression: Expression,
    /// 顶点过滤表达式
    v_filter: Option<Expression>,
    /// 输出列名
    col_names: Vec<String>,
    /// 是否去重
    dedup: bool,
    /// 是否跟踪前一个路径
    track_prev_path: bool,
    /// 是否需要获取属性
    need_fetch_prop: bool,
}

impl<S: StorageClient + Send + 'static> AppendVerticesExecutor<S> {
    /// 创建新的AppendVerticesExecutor
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        input_var: String,
        src_expression: Expression,
        v_filter: Option<Expression>,
        col_names: Vec<String>,
        dedup: bool,
        track_prev_path: bool,
        need_fetch_prop: bool,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "AppendVerticesExecutor".to_string(), storage),
            input_var,
            src_expression,
            v_filter,
            col_names,
            dedup,
            track_prev_path,
            need_fetch_prop,
        }
    }

    /// 带上下文创建AppendVerticesExecutor
    pub fn with_context(
        id: i64,
        storage: Arc<Mutex<S>>,
        input_var: String,
        src_expression: Expression,
        v_filter: Option<Expression>,
        col_names: Vec<String>,
        dedup: bool,
        track_prev_path: bool,
        need_fetch_prop: bool,
        context: crate::query::executor::base::ExecutionContext,
    ) -> Self {
        Self {
            base: BaseExecutor::with_context(
                id,
                "AppendVerticesExecutor".to_string(),
                storage,
                context,
            ),
            input_var,
            src_expression,
            v_filter,
            col_names,
            dedup,
            track_prev_path,
            need_fetch_prop,
        }
    }

    /// 构建请求数据集
    fn build_request_dataset(&mut self) -> DBResult<Vec<Value>> {
        // 获取输入结果
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

        // 创建表达式上下文
        let mut expr_context = DefaultExpressionContext::new();

        let mut vids = Vec::new();
        let mut seen = if self.dedup {
            Some(std::collections::HashMap::new())
        } else {
            None
        };

        // 根据输入结果类型处理
        match input_result {
            ExecutionResult::Values(values) => {
                for value in values {
                    // 设置当前值到表达式上下文
                    expr_context.set_variable("_".to_string(), value.clone());

                    // 计算源表达式获取顶点ID
                    let vid = ExpressionEvaluator::evaluate(&self.src_expression, &mut expr_context)
                        .map_err(|e| {
                            DBError::Query(crate::core::error::QueryError::ExecutionError(
                                e.to_string(),
                            ))
                        })?;

                    // 检查是否去重
                    if let Some(ref mut seen_map) = seen {
                        if !seen_map.contains_key(&vid) {
                            seen_map.insert(vid.clone(), true);
                            vids.push(vid);
                        }
                    } else {
                        vids.push(vid);
                    }
                }
            }
            ExecutionResult::Vertices(vertices) => {
                for vertex in vertices {
                    let vertex_value = Value::Vertex(Box::new(vertex.clone()));
                    expr_context.set_variable("_".to_string(), vertex_value.clone());

                    let vid = ExpressionEvaluator::evaluate(&self.src_expression, &mut expr_context)
                        .map_err(|e| {
                            DBError::Query(crate::core::error::QueryError::ExecutionError(
                                e.to_string(),
                            ))
                        })?;

                    if let Some(ref mut seen_map) = seen {
                        if !seen_map.contains_key(&vid) {
                            seen_map.insert(vid.clone(), true);
                            vids.push(vid);
                        }
                    } else {
                        vids.push(vid);
                    }
                }
            }
            ExecutionResult::Edges(edges) => {
                for edge in edges {
                    let edge_value = Value::Edge(edge.clone());
                    expr_context.set_variable("_".to_string(), edge_value.clone());

                    let vid = ExpressionEvaluator::evaluate(&self.src_expression, &mut expr_context)
                        .map_err(|e| {
                            DBError::Query(crate::core::error::QueryError::ExecutionError(
                                e.to_string(),
                            ))
                        })?;

                    if let Some(ref mut seen_map) = seen {
                        if !seen_map.contains_key(&vid) {
                            seen_map.insert(vid.clone(), true);
                            vids.push(vid);
                        }
                    } else {
                        vids.push(vid);
                    }
                }
            }
            _ => {
                return Err(DBError::Query(
                    crate::core::error::QueryError::ExecutionError(
                        "Invalid input result type for AppendVertices".to_string(),
                    ),
                ));
            }
        }

        Ok(vids)
    }

    /// 处理空属性情况
    fn handle_null_prop(&mut self, vids: Vec<Value>) -> DBResult<DataSet> {
        let mut dataset = DataSet {
            col_names: self.col_names.clone(),
            rows: Vec::new(),
        };

        let _input_result = self
            .base
            .context
            .get_result(&self.input_var)
            .expect("Context should have input result");

        for vid in vids {
            if vid.is_empty() {
                continue;
            }

            // 创建顶点
            let vertex = Vertex {
                vid: Box::new(vid.clone()),
                id: 0,
                tags: Vec::new(),
                properties: std::collections::HashMap::new(),
            };

            if !self.track_prev_path {
                let mut row = Vec::new();
                row.push(Value::Vertex(Box::new(vertex)));
                dataset.rows.push(row);
            } else {
                let mut row = Vec::new();
                row.push(Value::Vertex(Box::new(vertex)));
                dataset.rows.push(row);
            }
        }

        Ok(dataset)
    }

    /// 从存储中获取顶点属性
    async fn fetch_vertices(&mut self, vids: Vec<Value>) -> DBResult<Vec<Vertex>> {
        let mut vertices = Vec::new();

        let storage = self.get_storage().lock().map_err(|_| {
            DBError::Storage(crate::core::error::StorageError::DbError(
                "Failed to lock storage".to_string(),
            ))
        })?;

        for vid in vids {
            if vid.is_empty() {
                continue;
            }

            let vertex = storage.get_vertex("default", &vid).map_err(|e| DBError::from(e))?;

            if let Some(vertex) = vertex {
                vertices.push(vertex);
            }
        }

        Ok(vertices)
    }

    fn execute_append_vertices(&mut self) -> DBResult<DataSet> {
        if !self.need_fetch_prop {
            let vids = self.build_request_dataset()?;
            return self.handle_null_prop(vids);
        }

        let vids = self.build_request_dataset()?;

        if vids.is_empty() {
            return Ok(DataSet {
                col_names: self.col_names.clone(),
                rows: Vec::new(),
            });
        }

        let vertices = self.fetch_vertices(vids)?;

        let _expr_context = DefaultExpressionContext::new();

        let mut dataset = DataSet {
            col_names: self.col_names.clone(),
            rows: Vec::new(),
        };

        for vertex in vertices {
            let vertex_value = Value::Vertex(Box::new(vertex.clone()));
            let mut row_context = DefaultExpressionContext::new();

            if let Some(ref filter_expression) = self.v_filter {
                let filter_result = ExpressionEvaluator::evaluate(filter_expression, &mut row_context)
                    .map_err(|e| {
                        DBError::Query(crate::core::error::QueryError::ExecutionError(
                            e.to_string(),
                        ))
                    })?;

                if let Value::Bool(false) = filter_result {
                    continue;
                }
            }

            if !self.track_prev_path {
                let mut row = Vec::new();
                row.push(vertex_value);
                dataset.rows.push(row);
            } else {
                let mut row = Vec::new();
                row.push(vertex_value);
                dataset.rows.push(row);
            }
        }

        Ok(dataset)
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for AppendVerticesExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let dataset = self.execute_append_vertices()?;

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

impl<S: StorageClient + Send> HasStorage<S> for AppendVerticesExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("AppendVerticesExecutor storage should be set")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;
    use crate::core::Value;
    use crate::storage::MockStorage;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_append_vertices_executor() {
        let storage = Arc::new(Mutex::new(MockStorage));

        let vids = vec![
            Value::String("vertex1".to_string()),
            Value::String("vertex2".to_string()),
        ];

        let input_result = ExecutionResult::Values(vids);

        let mut context = crate::query::executor::base::ExecutionContext::new();
        context.set_result("input".to_string(), input_result);

        let src_expression = Expression::Variable("_".to_string());
        let mut executor = AppendVerticesExecutor::with_context(
            1,
            storage,
            "input".to_string(),
            src_expression,
            None,
            vec!["vertex".to_string()],
            false,
            false,
            false,
            context,
        );

        let result = executor
            .execute()
            .await
            .expect("Executor should execute successfully");

        if let ExecutionResult::Values(values) = result {
            assert_eq!(values.len(), 2);
            assert!(matches!(values[0], Value::Vertex(_)));
            assert!(matches!(values[1], Value::Vertex(_)));
        } else {
            panic!("Expected Values result");
        }
    }
}

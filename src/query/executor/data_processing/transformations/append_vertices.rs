//! AppendVerticesExecutor实现
//!
//! 负责处理追加顶点操作，根据给定的顶点ID获取顶点信息并追加到结果中

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult};
use crate::core::Expression;
use crate::core::{DataSet, Value, Vertex};
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::expression::evaluator::traits::ExpressionContext;
use crate::expression::DefaultExpressionContext;
use crate::query::executor::base::BaseExecutor;
use crate::query::executor::traits::{ExecutionResult, Executor, HasStorage};
use crate::storage::StorageEngine;

/// AppendVertices执行器
/// 用于根据顶点ID获取顶点信息并追加到结果中
pub struct AppendVerticesExecutor<S: StorageEngine + Send + 'static> {
    base: BaseExecutor<S>,
    /// 输入变量名
    input_var: String,
    /// 源表达式，用于获取顶点ID
    src_expr: Expression,
    /// 要获取的属性列表
    props: Vec<String>,
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

impl<S: StorageEngine + Send + 'static> AppendVerticesExecutor<S> {
    /// 创建新的AppendVerticesExecutor
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        input_var: String,
        src_expr: Expression,
        props: Vec<String>,
        v_filter: Option<Expression>,
        col_names: Vec<String>,
        dedup: bool,
        track_prev_path: bool,
        need_fetch_prop: bool,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "AppendVerticesExecutor".to_string(), storage),
            input_var,
            src_expr,
            props,
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
        src_expr: Expression,
        props: Vec<String>,
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
            src_expr,
            props,
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

        // 从执行上下文中设置变量
        for (name, value) in &self.base.context.variables.clone() {
            expr_context.set_variable(name.clone(), value.clone());
        }

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
                    let vid = ExpressionEvaluator::evaluate(&self.src_expr, &mut expr_context)
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

                    let vid = ExpressionEvaluator::evaluate(&self.src_expr, &mut expr_context)
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

                    let vid = ExpressionEvaluator::evaluate(&self.src_expr, &mut expr_context)
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

        // 创建表达式上下文
        let mut expr_context = DefaultExpressionContext::new();

        // 从执行上下文中设置变量
        for (name, value) in &self.base.context.variables.clone() {
            expr_context.set_variable(name.clone(), value.clone());
        }

        // 获取输入结果
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
                tags: Vec::new(),                             // 空标签
                properties: std::collections::HashMap::new(), // 空属性
            };

            if !self.track_prev_path {
                // 不跟踪前一个路径，只返回顶点
                let mut row = Vec::new();
                row.push(Value::Vertex(Box::new(vertex)));
                dataset.rows.push(row);
            } else {
                // 跟踪前一个路径，需要保留原始行
                // 这里简化处理，实际应该从输入结果中获取原始行
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

        // 获取存储引擎
        let storage = self.get_storage().lock().map_err(|_| {
            DBError::Storage(crate::core::error::StorageError::DbError(
                "Failed to lock storage".to_string(),
            ))
        })?;

        for vid in vids {
            if vid.is_empty() {
                continue;
            }

            // 从存储中获取顶点
            let vertex = storage.get_node(&vid).map_err(|e| DBError::from(e))?;

            if let Some(vertex) = vertex {
                vertices.push(vertex);
            }
        }

        Ok(vertices)
    }

    /// 执行追加顶点操作
    async fn execute_append_vertices(&mut self) -> DBResult<DataSet> {
        // 如果不需要获取属性，直接处理空属性情况
        if !self.need_fetch_prop {
            let vids = self.build_request_dataset()?;
            return self.handle_null_prop(vids);
        }

        // 构建请求数据集
        let vids = self.build_request_dataset()?;

        if vids.is_empty() {
            return Ok(DataSet {
                col_names: self.col_names.clone(),
                rows: Vec::new(),
            });
        }

        // 从存储中获取顶点
        let vertices = self.fetch_vertices(vids).await?;

        // 创建表达式上下文
        let _expr_context = DefaultExpressionContext::new();

        // 创建输出数据集
        let mut dataset = DataSet {
            col_names: self.col_names.clone(),
            rows: Vec::new(),
        };

        // 应用顶点过滤器
        for vertex in vertices {
            let vertex_value = Value::Vertex(Box::new(vertex.clone()));
            let mut row_context = DefaultExpressionContext::new();

            // 如果有顶点过滤器，应用它
            if let Some(ref filter_expr) = self.v_filter {
                let filter_result = ExpressionEvaluator::evaluate(filter_expr, &mut row_context)
                    .map_err(|e| {
                        DBError::Query(crate::core::error::QueryError::ExecutionError(
                            e.to_string(),
                        ))
                    })?;

                if let Value::Bool(false) = filter_result {
                    continue; // 过滤掉这个顶点
                }
            }

            // 添加到结果集
            if !self.track_prev_path {
                // 不跟踪前一个路径，只返回顶点
                let mut row = Vec::new();
                row.push(vertex_value);
                dataset.rows.push(row);
            } else {
                // 跟踪前一个路径，需要保留原始行
                // 这里简化处理，实际应该从输入结果中获取原始行
                let mut row = Vec::new();
                row.push(vertex_value);
                dataset.rows.push(row);
            }
        }

        Ok(dataset)
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for AppendVerticesExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let dataset = self.execute_append_vertices().await?;

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

impl<S: StorageEngine + Send> HasStorage<S> for AppendVerticesExecutor<S> {
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
    use crate::config::test_config::test_config;
    use crate::core::Expression;
    use crate::core::Value;
    use crate::storage::RocksDBStorage;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_append_vertices_executor() {
        let config = test_config();
        let storage = Arc::new(Mutex::new(
            RocksDBStorage::new(config.test_db_path("test_db_append_vertices"))
                .expect("RocksDBStorage should be created successfully"),
        ));

        // 创建输入数据
        let vids = vec![
            Value::String("vertex1".to_string()),
            Value::String("vertex2".to_string()),
        ];

        let input_result = ExecutionResult::Values(vids);

        // 创建执行上下文
        let mut context = crate::query::executor::base::ExecutionContext::new();
        context.set_result("input".to_string(), input_result);

        // 创建AppendVerticesExecutor
        let src_expr = Expression::Variable("_".to_string());
        let mut executor = AppendVerticesExecutor::with_context(
            1,
            storage,
            "input".to_string(),
            src_expr,
            vec![], // 空属性列表
            None,   // 无过滤器
            vec!["vertex".to_string()],
            false, // 不去重
            false, // 不跟踪前一个路径
            false, // 不需要获取属性
            context,
        );

        // 执行追加顶点
        let result = executor
            .execute()
            .await
            .expect("Executor should execute successfully");

        // 检查结果
        if let ExecutionResult::Values(values) = result {
            assert_eq!(values.len(), 2);
            // 验证返回的是顶点值
            assert!(matches!(values[0], Value::Vertex(_)));
            assert!(matches!(values[1], Value::Vertex(_)));
        } else {
            panic!("Expected Values result");
        }
    }
}

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
        /// 变量名
        variable: Option<String>,
        /// 标签列表
        labels: Vec<String>,
        /// 属性条件
        properties: HashMap<String, Expression>,
    },
    /// 边模式
    Edge {
        /// 变量名
        variable: Option<String>,
        /// 边类型
        edge_type: Option<String>,
        /// 方向
        direction: EdgeDirection,
        /// 属性条件
        properties: HashMap<String, Expression>,
    },
    /// 路径模式
    Path {
        /// 路径变量名
        variable: Option<String>,
        /// 路径长度范围
        length_range: Option<(usize, Option<usize>)>,
    },
}

/// PatternApply执行器
/// 用于将输入数据与指定模式进行匹配
pub struct PatternApplyExecutor<S: StorageEngine + Send + 'static> {
    base: BaseExecutor<S>,
    /// 输入变量名
    input_var: String,
    /// 要匹配的模式
    pattern: PatternType,
    /// 输出列名
    col_names: Vec<String>,
    /// 是否跟踪前一个路径
    track_prev_path: bool,
}

impl<S: StorageEngine + Send + 'static> PatternApplyExecutor<S> {
    /// 创建新的PatternApplyExecutor
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

    /// 带上下文创建PatternApplyExecutor
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

    /// 检查顶点是否匹配节点模式
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
            // 检查标签
            if !labels.is_empty() {
                let vertex_labels: Vec<String> =
                    vertex.tags.iter().map(|tag| tag.name.clone()).collect();

                // 检查顶点是否包含所有指定的标签
                for label in labels {
                    if !vertex_labels.contains(label) {
                        return Ok(false);
                    }
                }
            }

            // 检查属性
            if !properties.is_empty() {
                for (prop_name, prop_expr) in properties {
                    // 获取顶点属性值
                    let prop_value = vertex
                        .properties
                        .get(prop_name)
                        .cloned()
                        .unwrap_or(Value::Null(crate::core::NullType::UnknownProp));

                    // 创建临时表达式上下文
                    let mut temp_context = DefaultExpressionContext::new();
                    // 复制变量 - 使用新的变量访问方法
                    if let Some(variables) = expr_context.get_all_variables() {
                        for (name, value) in variables {
                            temp_context.set_variable(name, value);
                        }
                    }
                    temp_context.set_variable("_".to_string(), prop_value);

                    // 评估属性表达式
                    let result = ExpressionEvaluator::evaluate(prop_expr, &mut temp_context)
                        .map_err(|e| {
                            DBError::Query(crate::core::error::QueryError::ExecutionError(
                                e.to_string(),
                            ))
                        })?;

                    // 如果结果是布尔值且为false，则不匹配
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

    /// 检查边是否匹配边模式
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
            // 检查边类型
            if let Some(expected_type) = edge_type {
                if edge.edge_type != *expected_type {
                    return Ok(false);
                }
            }

            // 检查方向（这里简化处理，实际应该根据查询上下文检查）
            if let EdgeDirection::Incoming = direction {
                // 这里应该检查边方向是否匹配
                // 简化处理，假设所有边都匹配
            }

            // 检查属性
            if !properties.is_empty() {
                for (prop_name, prop_expr) in properties {
                    // 获取边属性值
                    let prop_value = edge
                        .properties()
                        .get(prop_name)
                        .cloned()
                        .unwrap_or(Value::Null(crate::core::NullType::UnknownProp));

                    // 创建临时表达式上下文
                    let mut temp_context = DefaultExpressionContext::new();
                    // 复制变量 - 使用新的变量访问方法
                    if let Some(variables) = expr_context.get_all_variables() {
                        for (name, value) in variables {
                            temp_context.set_variable(name, value);
                        }
                    }
                    temp_context.set_variable("_".to_string(), prop_value);

                    // 评估属性表达式
                    let result = ExpressionEvaluator::evaluate(prop_expr, &mut temp_context)
                        .map_err(|e| {
                            DBError::Query(crate::core::error::QueryError::ExecutionError(
                                e.to_string(),
                            ))
                        })?;

                    // 如果结果是布尔值且为false，则不匹配
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

    /// 检查路径是否匹配路径模式
    fn match_path_pattern<C: ExpressionContext>(
        &self,
        path: &Path,
        pattern: &PatternType,
        _expr_context: &C,
    ) -> DBResult<bool> {
        if let PatternType::Path { length_range, .. } = pattern {
            // 检查路径长度
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

    /// 执行模式匹配操作
    fn execute_pattern_apply(&mut self) -> DBResult<DataSet> {
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

        // 创建输出数据集
        let mut dataset = DataSet {
            col_names: self.col_names.clone(),
            rows: Vec::new(),
        };

        // 根据输入结果类型处理
        match input_result {
            ExecutionResult::Values(values) => {
                for value in values {
                    // 设置当前值到表达式上下文
                    expr_context.set_variable("_".to_string(), value.clone());

                    // 根据模式类型进行匹配
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

                    // 如果匹配，添加到结果集
                    if matches {
                        let mut row = Vec::new();

                        if !self.track_prev_path {
                            // 不跟踪前一个路径，只返回匹配的值
                            row.push(value.clone());
                        } else {
                            // 跟踪前一个路径，需要保留原始行
                            // 这里简化处理，实际应该从输入结果中获取原始行
                            row.push(value.clone());
                        }

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
    use crate::storage::NativeStorage;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_pattern_apply_executor() {
        let config = test_config();
        let storage = Arc::new(Mutex::new(
            NativeStorage::new(config.test_db_path("test_db_pattern_apply"))
                .expect("NativeStorage should be created successfully"),
        ));

        // 创建测试顶点
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

        // 创建输入数据
        let input_result = ExecutionResult::Values(vec![vertex_value.clone()]);

        // 创建执行上下文
        let mut context = crate::query::executor::base::ExecutionContext::new();
        context.set_result("input".to_string(), input_result);

        // 创建节点模式
        let pattern = PatternType::Node {
            variable: Some("v".to_string()),
            labels: vec!["Person".to_string()],
            properties: HashMap::new(),
        };

        // 创建PatternApplyExecutor
        let mut executor = PatternApplyExecutor::with_context(
            1,
            storage,
            "input".to_string(),
            pattern,
            vec!["matched".to_string()],
            false, // 不跟踪前一个路径
            context,
        );

        // 执行模式匹配
        let result = executor
            .execute()
            .await
            .expect("Executor should execute successfully");

        // 检查结果
        if let ExecutionResult::Values(values) = result {
            assert_eq!(values.len(), 1);
            assert_eq!(values[0], vertex_value);
        } else {
            panic!("Expected Values result");
        }
    }
}

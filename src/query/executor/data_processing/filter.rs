use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::core::Value;
use crate::graph::expression::{EvalContext, ExpressionV1 as Expression, ExpressionEvaluator};
use crate::query::executor::base::{BaseExecutor, InputExecutor};
use crate::query::executor::traits::{Executor, ExecutionResult, ExecutorCore, ExecutorLifecycle, ExecutorMetadata, DBResult};
use crate::query::QueryError;
use crate::storage::StorageEngine;

/// FilterExecutor - 条件过滤执行器
///
/// 根据指定的条件对输入数据进行过滤，通常用于 WHERE 子句
pub struct FilterExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    condition: Expression, // 条件表达式
    input_executor: Option<Box<dyn Executor<S>>>,
    evaluator: ExpressionEvaluator,
    // 表达式结果缓存，提高性能
    expression_cache: HashMap<String, bool>,
}

// Manual Debug implementation for FilterExecutor to avoid requiring Debug trait for ExpressionEvaluator
impl<S: StorageEngine> std::fmt::Debug for FilterExecutor<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FilterExecutor")
            .field("base", &"BaseExecutor")
            .field("condition", &"Expression")
            .field("input_executor", &"Box<dyn Executor<S>>")
            .field("evaluator", &"ExpressionEvaluator")
            .field("expression_cache", &self.expression_cache)
            .finish()
    }
}

impl<S: StorageEngine> FilterExecutor<S> {
    pub fn new(id: usize, storage: Arc<Mutex<S>>, condition: Expression) -> Self {
        Self {
            base: BaseExecutor::new(id, "FilterExecutor".to_string(), storage),
            condition,
            input_executor: None,
            evaluator: ExpressionEvaluator,
            expression_cache: HashMap::new(),
        }
    }

    /// 评估条件表达式
    async fn evaluate_condition(&mut self, context: &EvalContext<'_>) -> Result<bool, crate::core::error::QueryError> {
        // 评估表达式
        let result = self
            .evaluator
            .evaluate(&self.condition, context)
            .map_err(|e| crate::core::error::QueryError::ExecutionError(e.to_string()))?;

        // 转换为布尔值
        Ok(self.value_to_bool(&result))
    }

    /// 将值转换为布尔值
    fn value_to_bool(&self, value: &Value) -> bool {
        match value {
            Value::Bool(b) => *b,
            Value::Null(_) => false,
            Value::Int(0) => false,
            Value::Float(0.0) => false,
            Value::String(s) if s.is_empty() => false,
            Value::List(l) if l.is_empty() => false,
            Value::Map(m) if m.is_empty() => false,
            _ => true, // 非空、非零值视为 true
        }
    }

    /// 为值创建评估上下文
    fn create_context_for_value<'a>(&self, value: &'a Value) -> EvalContext<'a> {
        let mut context = EvalContext::new();

        match value {
            Value::Vertex(vertex) => {
                context.vertex = Some(vertex);
                // 将顶点属性添加到上下文变量中
                for tag in &vertex.tags {
                    for (prop_name, prop_value) in &tag.properties {
                        context.vars.insert(prop_name.clone(), prop_value.clone());
                    }
                }
                // 添加顶点ID
                context.vars.insert("id".to_string(), *vertex.vid.clone());
            }
            Value::Edge(edge) => {
                context.edge = Some(edge);
                // 将边属性添加到上下文变量中
                for (prop_name, prop_value) in &edge.props {
                    context.vars.insert(prop_name.clone(), prop_value.clone());
                }
                // 添加边的源和目标
                context.vars.insert("src".to_string(), *edge.src.clone());
                context.vars.insert("dst".to_string(), *edge.dst.clone());
                context
                    .vars
                    .insert("type".to_string(), Value::String(edge.edge_type.clone()));
            }
            Value::Map(map) => {
                // 将映射中的所有键值对添加到上下文
                for (key, value) in map {
                    context.vars.insert(key.clone(), value.clone());
                }
            }
            _ => {
                // 将值作为默认变量
                context.vars.insert("_".to_string(), value.clone());
            }
        }

        context
    }

    /// 应用过滤条件
    async fn apply_filter(
        &mut self,
        input: ExecutionResult,
    ) -> Result<ExecutionResult, crate::core::error::QueryError> {
        match input {
            ExecutionResult::Values(values) => {
                let mut filtered_values = Vec::new();

                for value in values {
                    let context = self.create_context_for_value(&value);
                    if self.evaluate_condition(&context).await? {
                        filtered_values.push(value);
                    }
                }

                Ok(ExecutionResult::Values(filtered_values))
            }
            ExecutionResult::Vertices(vertices) => {
                let mut filtered_vertices = Vec::new();

                for vertex in vertices {
                    let value = Value::Vertex(Box::new(vertex.clone()));
                    let context = self.create_context_for_value(&value);
                    if self.evaluate_condition(&context).await? {
                        filtered_vertices.push(vertex);
                    }
                }

                Ok(ExecutionResult::Vertices(filtered_vertices))
            }
            ExecutionResult::Edges(edges) => {
                let mut filtered_edges = Vec::new();

                for edge in edges {
                    let value = Value::Edge(edge.clone());
                    let context = self.create_context_for_value(&value);
                    if self.evaluate_condition(&context).await? {
                        filtered_edges.push(edge);
                    }
                }

                Ok(ExecutionResult::Edges(filtered_edges))
            }
            // DataSet 变体目前不支持，后续可以添加
            // ExecutionResult::DataSet(mut dataset) => {
            //     // 过滤数据集中的行
            //     let mut filtered_rows = Vec::new();

            //     for row in dataset.rows {
            //         // 将行转换为映射
            //         let mut row_map = HashMap::new();
            //         for (i, value) in row.iter().enumerate() {
            //             if let Some(col_name) = dataset.col_names.get(i) {
            //                 row_map.insert(col_name.clone(), value.clone());
            //             }
            //         }

            //         let row_value = Value::Map(row_map);
            //         let context = self.create_context_for_value(&row_value);

            //         if self.evaluate_condition(&context).await? {
            //             filtered_rows.push(row);
            //         }
            //     }

            //     dataset.rows = filtered_rows;
            //     Ok(ExecutionResult::DataSet(dataset))
            // }
            _ => Ok(input),
        }
    }
}

impl<S: StorageEngine> InputExecutor<S> for FilterExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ExecutorCore for FilterExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 清空缓存，避免跨查询的缓存污染
        self.expression_cache.clear();

        // 首先执行输入执行器（如果存在）
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            // 如果没有输入执行器，返回空结果
            ExecutionResult::Values(Vec::new())
        };

        // 应用过滤条件
        self.apply_filter(input_result).await.map_err(|e| crate::core::error::DBError::Query(crate::core::error::QueryError::ExecutionError(e.to_string())))
    }
}

impl<S: StorageEngine + Send + 'static> ExecutorLifecycle for FilterExecutor<S> {
    fn open(&mut self) -> DBResult<()> {
        // 初始化过滤所需的任何资源
        self.expression_cache.clear();

        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        // 清理资源
        self.expression_cache.clear();

        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.close()?;
        }
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }
}

impl<S: StorageEngine + Send + 'static> ExecutorMetadata for FilterExecutor<S> {
    fn id(&self) -> usize {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        "FilterExecutor - filters data based on conditions"
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for FilterExecutor<S> {
    fn storage(&self) -> &Arc<Mutex<S>> {
        &self.base.storage
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::NullType;
    use crate::graph::expression::BinaryOperator;
    use std::sync::{Arc, Mutex};

    // 模拟存储引擎
    struct MockStorage;

    impl StorageEngine for MockStorage {
        fn insert_node(
            &mut self,
            _vertex: crate::core::vertex_edge_path::Vertex,
        ) -> Result<crate::core::Value, crate::storage::StorageError> {
            Ok(crate::core::Value::Null(NullType::NaN))
        }

        fn get_node(
            &self,
            _id: &crate::core::Value,
        ) -> Result<Option<crate::core::vertex_edge_path::Vertex>, crate::storage::StorageError>
        {
            Ok(None)
        }

        fn update_node(
            &mut self,
            _vertex: crate::core::vertex_edge_path::Vertex,
        ) -> Result<(), crate::storage::StorageError> {
            Ok(())
        }

        fn delete_node(
            &mut self,
            _id: &crate::core::Value,
        ) -> Result<(), crate::storage::StorageError> {
            Ok(())
        }

        fn insert_edge(
            &mut self,
            _edge: crate::core::vertex_edge_path::Edge,
        ) -> Result<(), crate::storage::StorageError> {
            Ok(())
        }

        fn get_edge(
            &self,
            _src: &crate::core::Value,
            _dst: &crate::core::Value,
            _edge_type: &str,
        ) -> Result<Option<crate::core::vertex_edge_path::Edge>, crate::storage::StorageError>
        {
            Ok(None)
        }

        fn get_node_edges(
            &self,
            _node_id: &crate::core::Value,
            _direction: crate::core::vertex_edge_path::Direction,
        ) -> Result<Vec<crate::core::vertex_edge_path::Edge>, crate::storage::StorageError>
        {
            Ok(Vec::new())
        }

        fn delete_edge(
            &mut self,
            _src: &crate::core::Value,
            _dst: &crate::core::Value,
            _edge_type: &str,
        ) -> Result<(), crate::storage::StorageError> {
            Ok(())
        }

        fn begin_transaction(&mut self) -> Result<u64, crate::storage::StorageError> {
            Ok(1)
        }

        fn commit_transaction(&mut self, _tx_id: u64) -> Result<(), crate::storage::StorageError> {
            Ok(())
        }

        fn rollback_transaction(
            &mut self,
            _tx_id: u64,
        ) -> Result<(), crate::storage::StorageError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_filter_executor_with_values() {
        let storage = Arc::new(Mutex::new(MockStorage));

        // 创建条件表达式：age > 18
        let condition = Expression::BinaryOp(
            Box::new(Expression::Property("age".to_string())),
            BinaryOperator::Gt,
            Box::new(Expression::Constant(Value::Int(18))),
        );

        let mut executor = FilterExecutor::new(1, storage, condition);

        // 设置测试数据
        let test_data = vec![
            Value::Map(HashMap::from([
                ("name".to_string(), Value::String("Alice".to_string())),
                ("age".to_string(), Value::Int(20)),
            ])),
            Value::Map(HashMap::from([
                ("name".to_string(), Value::String("Bob".to_string())),
                ("age".to_string(), Value::Int(16)),
            ])),
        ];

        let input_result = ExecutionResult::Values(test_data);

        // 创建模拟输入执行器
        struct MockInputExecutor {
            result: ExecutionResult,
        }

        #[async_trait]
        impl crate::query::executor::traits::ExecutorCore for MockInputExecutor {
            async fn execute(&mut self) -> crate::query::executor::traits::DBResult<ExecutionResult> {
                Ok(self.result.clone())
            }
        }
        
        impl crate::query::executor::traits::ExecutorLifecycle for MockInputExecutor {
            fn open(&mut self) -> crate::query::executor::traits::DBResult<()> {
                Ok(())
            }
            fn close(&mut self) -> crate::query::executor::traits::DBResult<()> {
                Ok(())
            }
            fn is_open(&self) -> bool {
                true
            }
        }
        
        impl crate::query::executor::traits::ExecutorMetadata for MockInputExecutor {
            fn id(&self) -> usize {
                0
            }
            fn name(&self) -> &str {
                "MockInputExecutor"
            }
            fn description(&self) -> &str {
                "MockInputExecutor"
            }
        }
        
        #[async_trait::async_trait]
        impl crate::query::executor::traits::Executor<MockStorage> for MockInputExecutor {
            fn storage(&self) -> &Arc<Mutex<MockStorage>> {
                unimplemented!("MockInputExecutor doesn't use storage")
            }
        }

        let input_executor = MockInputExecutor {
            result: input_result,
        };

        executor.set_input(Box::new(input_executor));

        // 执行过滤
        let result = executor.execute().await.unwrap();

        // 验证结果
        match result {
            ExecutionResult::Values(values) => {
                assert_eq!(values.len(), 1);
                if let Value::Map(map) = &values[0] {
                    assert_eq!(map.get("name"), Some(&Value::String("Alice".to_string())));
                    assert_eq!(map.get("age"), Some(&Value::Int(20)));
                } else {
                    panic!("Expected Map value");
                }
            }
            _ => panic!("Expected Values result"),
        }
    }
}

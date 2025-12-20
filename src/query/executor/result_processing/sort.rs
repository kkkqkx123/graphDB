//! 排序执行器
//!
//! 实现对查询结果的排序功能，支持多列排序和自定义排序规则

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult};
use crate::core::{DataSet, Value};
use crate::expression::{Expression, ExpressionEvaluator};
use crate::expression::ExpressionContext;
use crate::query::executor::base::InputExecutor;
use crate::query::executor::result_processing::traits::{
    BaseResultProcessor, ResultProcessor, ResultProcessorContext,
};
use crate::query::executor::traits::{
    ExecutionResult, Executor, ExecutorCore, ExecutorLifecycle, ExecutorMetadata,
};
use crate::storage::StorageEngine;

/// 排序顺序枚举
#[derive(Debug, Clone, PartialEq)]
pub enum SortOrder {
    Asc,
    Desc,
}

/// 排序键定义
#[derive(Debug, Clone)]
pub struct SortKey {
    pub expression: Expression,
    pub order: SortOrder,
}

impl SortKey {
    pub fn new(expression: Expression, order: SortOrder) -> Self {
        Self { expression, order }
    }
}

/// 排序执行器
pub struct SortExecutor<S: StorageEngine + Send + 'static> {
    /// 基础处理器
    base: BaseResultProcessor<S>,
    /// 排序键列表
    sort_keys: Vec<SortKey>,
    /// 限制数量
    limit: Option<usize>,
    /// 输入执行器
    input_executor: Option<Box<dyn Executor<S>>>,
    /// 是否使用磁盘溢出
    use_disk: bool,
}

impl<S: StorageEngine + Send + 'static> SortExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        sort_keys: Vec<SortKey>,
        limit: Option<usize>,
    ) -> Self {
        let base = BaseResultProcessor::new(
            id,
            "SortExecutor".to_string(),
            "Sorts query results based on specified keys and order".to_string(),
            storage,
        );

        Self {
            base,
            sort_keys,
            limit,
            input_executor: None,
            use_disk: false,
        }
    }

    /// 启用磁盘溢出
    pub fn with_disk_spill(mut self, enable: bool) -> Self {
        self.use_disk = enable;
        if enable {
            // 启用磁盘溢出时，自动设置较大的内存限制
            self.base.context.enable_disk_spill = true;
            self.base.context.memory_limit = Some(1024 * 1024 * 1024); // 1GB
        }
        self
    }

    /// 处理输入数据并排序
    async fn process_input(&mut self) -> DBResult<DataSet> {
        if let Some(ref mut input_exec) = self.input_executor {
            let input_result = input_exec.execute().await?;

            match input_result {
                ExecutionResult::DataSet(mut data_set) => {
                    // 执行排序
                    self.sort_dataset(&mut data_set)?;

                    // 应用限制
                    if let Some(limit) = self.limit {
                        data_set.rows.truncate(limit);
                    }

                    Ok(data_set)
                }
                _ => Err(DBError::Query(
                    crate::core::error::QueryError::ExecutionError(
                        "Sort executor expects DataSet input".to_string(),
                    ),
                )),
            }
        } else {
            Err(DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "Sort executor requires input executor".to_string(),
                ),
            ))
        }
    }

    /// 对数据集进行排序
    fn sort_dataset(&self, data_set: &mut DataSet) -> DBResult<()> {
        // 如果没有排序键，直接返回
        if self.sort_keys.is_empty() {
            return Ok(());
        }

        // 为每行计算排序键值
        let mut rows_with_keys: Vec<(Vec<Value>, Vec<Value>)> = Vec::new();
        let evaluator = ExpressionEvaluator;

        for row in &data_set.rows {
            // 构建表达式上下文
            let mut expr_context = ExpressionContext::simple();
            for (i, col_name) in data_set.col_names.iter().enumerate() {
                if i < row.len() {
                    expr_context.set_variable(col_name.clone(), row[i].clone());
                }
            }

            // 计算排序键值
            let mut sort_values = Vec::new();
            for sort_key in &self.sort_keys {
                let sort_value = evaluator
                    .evaluate(&sort_key.expression, &expr_context)
                    .map_err(|e| {
                        DBError::Query(crate::core::error::QueryError::ExecutionError(
                            e.to_string(),
                        ))
                    })?;
                sort_values.push(sort_value);
            }

            rows_with_keys.push((sort_values, row.clone()));
        }

        // 执行排序
        rows_with_keys.sort_by(|a, b| {
            // 逐个比较排序键
            for ((idx, sort_val_a), sort_val_b) in a.0.iter().enumerate().zip(b.0.iter()) {
                let comparison =
                    self.compare_values(sort_val_a, sort_val_b, &self.sort_keys[idx].order);
                if !comparison.is_eq() {
                    return comparison;
                }
            }
            std::cmp::Ordering::Equal
        });

        // 提取排序后的行
        data_set.rows = rows_with_keys.into_iter().map(|(_, row)| row).collect();

        Ok(())
    }

    /// 比较两个值，根据排序方向
    fn compare_values(&self, a: &Value, b: &Value, order: &SortOrder) -> std::cmp::Ordering {
        let comparison = a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal);

        match order {
            SortOrder::Asc => comparison,
            SortOrder::Desc => comparison.reverse(),
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ResultProcessor<S> for SortExecutor<S> {
    async fn process(&mut self, input: ExecutionResult) -> DBResult<ExecutionResult> {
        ResultProcessor::set_input(self, input);
        let dataset = self.process_input().await?;
        Ok(ExecutionResult::DataSet(dataset))
    }

    fn set_input(&mut self, input: ExecutionResult) {
        self.base.input = Some(input);
    }

    fn get_input(&self) -> Option<&ExecutionResult> {
        self.base.input.as_ref()
    }

    fn context(&self) -> &ResultProcessorContext {
        &self.base.context
    }

    fn set_context(&mut self, context: ResultProcessorContext) {
        self.base.context = context;
    }

    fn memory_usage(&self) -> usize {
        self.base.memory_usage
    }

    fn reset(&mut self) {
        self.base.memory_usage = 0;
        self.base.input = None;
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ExecutorCore for SortExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 首先执行输入执行器（如果存在）
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            // 如果没有输入执行器，使用设置的输入数据
            self.base
                .input
                .clone()
                .unwrap_or(ExecutionResult::DataSet(DataSet::new()))
        };

        self.process(input_result).await
    }
}

impl<S: StorageEngine + Send> ExecutorLifecycle for SortExecutor<S> {
    fn open(&mut self) -> DBResult<()> {
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.close()?;
        }
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.base.id > 0 // 简单的状态检查
    }
}

impl<S: StorageEngine + Send> ExecutorMetadata for SortExecutor<S> {
    fn id(&self) -> usize {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        &self.base.description
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for SortExecutor<S> {
    fn storage(&self) -> &Arc<Mutex<S>> {
        &self.base.storage
    }
}

impl<S: StorageEngine + Send + 'static> InputExecutor<S> for SortExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::NullType;

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

        fn scan_all_vertices(
            &self,
        ) -> Result<Vec<crate::core::vertex_edge_path::Vertex>, crate::storage::StorageError>
        {
            Ok(Vec::new())
        }

        fn scan_vertices_by_tag(
            &self,
            _tag: &str,
        ) -> Result<Vec<crate::core::vertex_edge_path::Vertex>, crate::storage::StorageError>
        {
            Ok(Vec::new())
        }
    }

    #[tokio::test]
    async fn test_sort_executor_basic() {
        let storage = Arc::new(Mutex::new(MockStorage));

        // 创建测试数据
        let mut dataset = DataSet::new();
        dataset.col_names = vec!["name".to_string(), "age".to_string()];
        dataset
            .rows
            .push(vec![Value::String("Alice".to_string()), Value::Int(30)]);
        dataset
            .rows
            .push(vec![Value::String("Bob".to_string()), Value::Int(25)]);
        dataset
            .rows
            .push(vec![Value::String("Charlie".to_string()), Value::Int(35)]);

        // 创建排序执行器
        let sort_keys = vec![SortKey::new(
            Expression::Property {
                object: Box::new(Expression::Variable("row".to_string())),
                property: "age".to_string(),
            },
            SortOrder::Asc,
        )];

        let mut executor = SortExecutor::new(1, storage, sort_keys, None);

        // 设置输入数据
        ResultProcessor::set_input(&mut executor, ExecutionResult::DataSet(dataset));

        // 执行排序
        let result = executor
            .process(ExecutionResult::DataSet(DataSet::new()))
            .await
            .unwrap();

        // 验证结果
        match result {
            ExecutionResult::DataSet(sorted_dataset) => {
                assert_eq!(sorted_dataset.rows.len(), 3);
                // 验证按年龄升序排列
                assert_eq!(sorted_dataset.rows[0][1], Value::Int(25)); // Bob
                assert_eq!(sorted_dataset.rows[1][1], Value::Int(30)); // Alice
                assert_eq!(sorted_dataset.rows[2][1], Value::Int(35)); // Charlie
            }
            _ => panic!("Expected DataSet result"),
        }
    }
}

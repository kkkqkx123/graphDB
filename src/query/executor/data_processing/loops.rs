//! 循环执行器模块
//!
//! 包含循环控制相关的执行器

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult};
use crate::expression::DefaultExpressionContext;
use crate::expression::evaluator::traits::ExpressionContext;
use crate::core::Value;
use crate::core::Expression;
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::query::executor::base::BaseExecutor;
use crate::query::executor::traits::{
    ExecutionResult, Executor, ExecutorCore, ExecutorLifecycle, ExecutorMetadata,
};
use crate::storage::StorageEngine;

/// 循环状态
#[derive(Debug, Clone, PartialEq)]
pub enum LoopState {
    NotStarted,
    Running,
    Finished,
    Error(String),
}

/// LoopExecutor - 循环控制执行器
///
/// 实现循环控制逻辑，支持条件循环和计数循环
pub struct LoopExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    condition: Option<Expression>, // 循环条件，None 表示无限循环
    body_executor: Box<dyn Executor<S>>,
    max_iterations: Option<usize>,
    current_iteration: usize,
    loop_state: LoopState,
    evaluator: ExpressionEvaluator,
    // 循环结果收集
    results: Vec<ExecutionResult>,
    // 循环变量上下文
    loop_context: DefaultExpressionContext,
}

impl<S: StorageEngine> LoopExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        condition: Option<Expression>,
        body_executor: Box<dyn Executor<S>>,
        max_iterations: Option<usize>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "LoopExecutor".to_string(), storage),
            condition,
            body_executor,
            max_iterations,
            current_iteration: 0,
            loop_state: LoopState::NotStarted,
            evaluator: ExpressionEvaluator,
            results: Vec::new(),
            loop_context: DefaultExpressionContext::new(),
        }
    }
}

impl<S: StorageEngine + Send + 'static> LoopExecutor<S> {
    /// 验证循环执行器是否存在自引用
    pub fn validate_no_self_reference(&self) -> Result<(), DBError> {
        // 检查body_executor是否指向自身
        if self.body_executor.id() == self.base.id {
            return Err(DBError::Query(crate::core::error::QueryError::ExecutionError(
                "循环执行器不能自引用".to_string()
            )));
        }
        Ok(())
    }

    /// 评估循环条件
    async fn evaluate_condition(&mut self) -> DBResult<bool> {
        match &self.condition {
            Some(expr) => {
                let result = self
                    .evaluator
                    .evaluate(expr, &mut self.loop_context)
                    .map_err(|e| {
                        DBError::Expression(crate::core::error::ExpressionError::function_error(
                            e.to_string(),
                        ))
                    })?;

                Ok(self.value_to_bool(&result))
            }
            None => Ok(true), // 无条件循环
        }
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

    /// 检查是否应该继续循环
    fn should_continue(&self) -> bool {
        if let LoopState::Error(_) = self.loop_state {
            return false;
        }

        if let Some(max_iter) = self.max_iterations {
            if self.current_iteration >= max_iter {
                return false;
            }
        }

        true
    }

    /// 执行单次循环
    async fn execute_iteration(&mut self) -> DBResult<ExecutionResult> {
        self.current_iteration += 1;

        // 更新循环上下文中的迭代变量
        self.loop_context.set_variable(
            "__iteration".to_string(),
            Value::Int(self.current_iteration as i64),
        );

        // 执行循环体
        let result = self.body_executor.execute().await?;

        // 重置循环体状态，为下次迭代做准备
        self.body_executor.close()?;
        self.body_executor.open()?;

        Ok(result)
    }

    /// 收集所有循环结果
    fn collect_results(&self) -> ExecutionResult {
        if self.results.is_empty() {
            return ExecutionResult::Success;
        }

        // 尝试合并同类型的结果
        let mut all_values = Vec::new();
        let mut all_vertices = Vec::new();
        let mut all_edges = Vec::new();
        let mut all_paths = Vec::new();
        let mut all_datasets = Vec::new();

        for result in &self.results {
            match result {
                ExecutionResult::Values(values) => all_values.extend(values.clone()),
                ExecutionResult::Vertices(vertices) => all_vertices.extend(vertices.clone()),
                ExecutionResult::Edges(edges) => all_edges.extend(edges.clone()),
                ExecutionResult::Paths(paths) => all_paths.extend(paths.clone()),
                ExecutionResult::DataSet(dataset) => all_datasets.push(dataset.clone()),
                ExecutionResult::Count(count) => all_values.push(Value::Int(*count as i64)),
                ExecutionResult::Success => {}
                ExecutionResult::Error(_) => {} // Ignore error results or handle as needed
            }
        }

        // 根据内容返回最合适的结果类型
        if !all_values.is_empty()
            && all_vertices.is_empty()
            && all_edges.is_empty()
            && all_paths.is_empty()
        {
            ExecutionResult::Values(all_values)
        } else if !all_vertices.is_empty()
            && all_values.is_empty()
            && all_edges.is_empty()
            && all_paths.is_empty()
        {
            ExecutionResult::Vertices(all_vertices)
        } else if !all_edges.is_empty()
            && all_values.is_empty()
            && all_vertices.is_empty()
            && all_paths.is_empty()
        {
            ExecutionResult::Edges(all_edges)
        } else if !all_paths.is_empty()
            && all_values.is_empty()
            && all_vertices.is_empty()
            && all_edges.is_empty()
        {
            ExecutionResult::Paths(all_paths)
        } else if !all_datasets.is_empty()
            && all_values.is_empty()
            && all_vertices.is_empty()
            && all_edges.is_empty()
            && all_paths.is_empty()
        {
            // 合并数据集
            if all_datasets.len() == 1 {
                ExecutionResult::DataSet(
                    all_datasets
                        .into_iter()
                        .next()
                        .expect("Failed to get next dataset"),
                )
            } else {
                // 简化处理：返回第一个数据集
                ExecutionResult::DataSet(
                    all_datasets
                        .into_iter()
                        .next()
                        .expect("Failed to get next dataset"),
                )
            }
        } else {
            // 混合类型，返回值列表
            let mut mixed_values = Vec::new();
            mixed_values.extend(all_values);
            for vertex in all_vertices {
                mixed_values.push(Value::Vertex(Box::new(vertex)));
            }
            for edge in all_edges {
                mixed_values.push(Value::Edge(edge));
            }
            for path in all_paths {
                mixed_values.push(Value::Path(path));
            }
            ExecutionResult::Values(mixed_values)
        }
    }

    /// 设置循环变量
    pub fn set_loop_variable(&mut self, name: String, value: Value) {
        self.loop_context.set_variable(name, value);
    }

    /// 获取当前迭代次数
    pub fn current_iteration(&self) -> usize {
        self.current_iteration
    }

    /// 获取循环状态
    pub fn loop_state(&self) -> &LoopState {
        &self.loop_state
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ExecutorCore for LoopExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 验证自引用 - 防止无限递归
        self.validate_no_self_reference()?;
        
        self.loop_state = LoopState::Running;
        self.results.clear();
        self.current_iteration = 0;

        // 打开循环体执行器
        self.body_executor.open()?;

        // 执行循环
        while self.should_continue() {
            // 首先设置迭代变量
            self.loop_context.set_variable(
                "__iteration".to_string(),
                Value::Int(self.current_iteration as i64),
            );

            // 评估循环条件
            let should_continue = match self.evaluate_condition().await {
                Ok(continue_flag) => continue_flag,
                Err(e) => {
                    self.loop_state = LoopState::Error(e.to_string());
                    break;
                }
            };

            if !should_continue {
                break;
            }

            // 执行循环体
            match self.execute_iteration().await {
                Ok(result) => {
                    self.results.push(result);
                }
                Err(e) => {
                    self.loop_state = LoopState::Error(e.to_string());
                    break;
                }
            }
        }

        // 关闭循环体执行器
        let _ = self.body_executor.close();

        // 设置最终状态
        if !matches!(self.loop_state, LoopState::Error(_)) {
            self.loop_state = LoopState::Finished;
        }

        // 返回收集的结果
        Ok(self.collect_results())
    }
}

impl<S: StorageEngine + Send> ExecutorLifecycle for LoopExecutor<S> {
    fn open(&mut self) -> DBResult<()> {
        // 初始化循环状态
        self.loop_state = LoopState::NotStarted;
        self.current_iteration = 0;
        self.results.clear();
        self.loop_context = DefaultExpressionContext::new();

        // 打开循环体执行器
        self.body_executor.open()?;
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        // 关闭循环体执行器
        self.body_executor.close()?;

        // 清理资源
        self.results.clear();
        self.loop_context = DefaultExpressionContext::new();

        Ok(())
    }

    fn is_open(&self) -> bool {
        self.base.is_open()
    }
}

impl<S: StorageEngine> ExecutorMetadata for LoopExecutor<S> {
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
    for LoopExecutor<S>
{
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.storage.as_ref().expect("LoopExecutor storage should be set")
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for LoopExecutor<S> {
}

/// WhileLoopExecutor - 条件循环执行器
///
/// 专门用于实现 WHILE 循环
pub struct WhileLoopExecutor<S: StorageEngine> {
    inner: LoopExecutor<S>,
}

impl<S: StorageEngine + Send + 'static> WhileLoopExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        condition: Expression,
        body_executor: Box<dyn Executor<S>>,
        max_iterations: Option<usize>,
    ) -> Self {
        Self {
            inner: LoopExecutor::new(id, storage, Some(condition), body_executor, max_iterations),
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ExecutorCore for WhileLoopExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        self.inner.execute().await
    }
}

impl<S: StorageEngine + Send> ExecutorLifecycle for WhileLoopExecutor<S> {
    fn open(&mut self) -> DBResult<()> {
        self.inner.open()
    }

    fn close(&mut self) -> DBResult<()> {
        self.inner.close()
    }

    fn is_open(&self) -> bool {
        self.inner.is_open()
    }
}

impl<S: StorageEngine> ExecutorMetadata for WhileLoopExecutor<S> {
    fn id(&self) -> i64 {
        self.inner.id()
    }

    fn name(&self) -> &str {
        "WhileLoopExecutor"
    }

    fn description(&self) -> &str {
        &self.inner.description()
    }
}

impl<S: StorageEngine + Send + 'static> crate::query::executor::traits::HasStorage<S>
    for WhileLoopExecutor<S>
{
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.inner.get_storage()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for WhileLoopExecutor<S> {
}

/// ForLoopExecutor - 计数循环执行器
///
/// 专门用于实现 FOR 循环
pub struct ForLoopExecutor<S: StorageEngine> {
    inner: LoopExecutor<S>,
    start: i64,
    end: i64,
    step: i64,
    loop_var: String,
}

impl<S: StorageEngine + Send + 'static> ForLoopExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        loop_var: String,
        start: i64,
        end: i64,
        step: i64,
        body_executor: Box<dyn Executor<S>>,
    ) -> Self {
        let mut executor = LoopExecutor::new(
            id,
            storage,
            None, // 条件在内部处理
            body_executor,
            Some(((end - start).abs() / step.abs() + 1) as usize),
        );

        // 设置循环变量
        executor.set_loop_variable(loop_var.clone(), Value::Int(start));

        Self {
            inner: executor,
            start,
            end,
            step,
            loop_var,
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ExecutorCore for ForLoopExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 初始化循环
        self.inner.open()?;

        let mut current = self.start;
        let mut results = Vec::new();

        // 执行循环
        while (self.step > 0 && current <= self.end) || (self.step < 0 && current >= self.end) {
            // 设置循环变量
            self.inner
                .set_loop_variable(self.loop_var.clone(), Value::Int(current));

            // 执行循环体
            let result = self.inner.execute_iteration().await?;
            results.push(result);

            // 更新循环变量
            current += self.step;
        }

        // 关闭循环
        self.inner.close()?;

        // 返回结果
        self.inner.results = results;
        Ok(self.inner.collect_results())
    }
}

impl<S: StorageEngine + Send> ExecutorLifecycle for ForLoopExecutor<S> {
    fn open(&mut self) -> DBResult<()> {
        self.inner.open()
    }

    fn close(&mut self) -> DBResult<()> {
        self.inner.close()
    }

    fn is_open(&self) -> bool {
        self.inner.is_open()
    }
}

impl<S: StorageEngine> ExecutorMetadata for ForLoopExecutor<S> {
    fn id(&self) -> i64 {
        self.inner.id()
    }

    fn name(&self) -> &str {
        "ForLoopExecutor"
    }

    fn description(&self) -> &str {
        &self.inner.description()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for ForLoopExecutor<S> {
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::NullType;
    use crate::core::BinaryOperator;
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

    // 模拟计数执行器
    struct CountExecutor {
        count: i64,
        max_count: i64,
    }

    #[async_trait]
    impl ExecutorCore for CountExecutor {
        async fn execute(&mut self) -> DBResult<ExecutionResult> {
            if self.count < self.max_count {
                self.count += 1;
                Ok(ExecutionResult::Values(vec![Value::Int(self.count)]))
            } else {
                Ok(ExecutionResult::Success)
            }
        }
    }

    impl ExecutorLifecycle for CountExecutor {
        fn open(&mut self) -> DBResult<()> {
            Ok(())
        }
        fn close(&mut self) -> DBResult<()> {
            Ok(())
        }
        fn is_open(&self) -> bool {
            true
        }
    }

    impl ExecutorMetadata for CountExecutor {
        fn id(&self) -> i64 {
            0
        }
        fn name(&self) -> &str {
            "CountExecutor"
        }
        fn description(&self) -> &str {
            "CountExecutor"
        }
    }

    #[async_trait]
    impl Executor<MockStorage> for CountExecutor {
        fn storage(&self) -> &Arc<Mutex<MockStorage>> {
            // 需要添加base字段或者修改这个实现
            unimplemented!("需要添加base字段到CountExecutor")
        }
    }

    #[tokio::test]
    async fn test_while_loop_executor() {
        let storage = Arc::new(Mutex::new(MockStorage));

        // 创建条件表达式：__iteration < 3
        let condition = Expression::binary(
            Expression::variable("__iteration"),
            BinaryOperator::LessThan,
            Expression::int(3),
        );

        let body_executor = Box::new(CountExecutor {
            count: 0,
            max_count: 10,
        });

        let mut executor = WhileLoopExecutor::new(
            1,
            storage,
            condition,
            body_executor,
            Some(5), // 最大5次迭代
        );

        // 执行循环
        let result = executor.execute().await.expect("Failed to execute");

        // 调试信息
        println!("Loop result: {:?}", result);
        println!("Current iteration: {}", executor.inner.current_iteration());
        println!("Loop state: {:?}", executor.inner.loop_state());

        // 验证结果
        match result {
            ExecutionResult::Values(values) => {
                println!("Values: {:?}", values);
                assert_eq!(values.len(), 3); // 应该执行3次
                assert_eq!(values, vec![Value::Int(1), Value::Int(2), Value::Int(3),]);
            }
            _ => panic!("Expected Values result, got: {:?}", result),
        }

        assert_eq!(executor.inner.current_iteration(), 3);
        assert_eq!(executor.inner.loop_state(), &LoopState::Finished);
    }

    #[tokio::test]
    async fn test_for_loop_executor() {
        let storage = Arc::new(Mutex::new(MockStorage));

        let body_executor = Box::new(CountExecutor {
            count: 0,
            max_count: 10,
        });

        let mut executor =
            ForLoopExecutor::new(1, storage, "i".to_string(), 1, 3, 1, body_executor);

        // 执行循环
        let result = executor.execute().await.expect("Failed to execute");

        // 验证结果
        match result {
            ExecutionResult::Values(values) => {
                assert_eq!(values.len(), 3); // 应该执行3次
                assert_eq!(values, vec![Value::Int(1), Value::Int(2), Value::Int(3),]);
            }
            _ => panic!("Expected Values result"),
        }
    }
}

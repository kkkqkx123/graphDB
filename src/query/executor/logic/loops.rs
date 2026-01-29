//! 循环执行器模块
//!
//! 包含循环控制相关的执行器
//!
//! NebulaGraph 对应实现：
//! nebula-3.8.0/src/graph/executor/logic/LoopExecutor.cpp

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult};
use crate::core::Expression;
use crate::core::Value;
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::expression::evaluator::traits::ExpressionContext;
use crate::expression::DefaultExpressionContext;
use crate::query::executor::base::BaseExecutor;
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::recursion_detector::{
    ExecutorSafetyConfig, ExecutorSafetyValidator, RecursionDetector,
};
use crate::query::executor::traits::{ExecutionResult, Executor, HasStorage};
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
/// 包含递归检测机制，防止循环执行器自引用
pub struct LoopExecutor<S: StorageEngine + Send + 'static> {
    base: BaseExecutor<S>,
    condition: Option<Expression>,
    body_executor: Box<ExecutorEnum<S>>,
    max_iterations: Option<usize>,
    current_iteration: usize,
    loop_state: LoopState,
    results: Vec<ExecutionResult>,
    loop_context: DefaultExpressionContext,
    recursion_detector: RecursionDetector,
    safety_validator: ExecutorSafetyValidator,
}

impl<S: StorageEngine> LoopExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        condition: Option<Expression>,
        body_executor: ExecutorEnum<S>,
        max_iterations: Option<usize>,
    ) -> Self {
        let recursion_detector = RecursionDetector::new(100);
        let safety_validator = ExecutorSafetyValidator::new(ExecutorSafetyConfig::default());

        Self {
            base: BaseExecutor::new(id, "LoopExecutor".to_string(), storage),
            condition,
            body_executor: Box::new(body_executor),
            max_iterations,
            current_iteration: 0,
            loop_state: LoopState::NotStarted,
            results: Vec::new(),
            loop_context: DefaultExpressionContext::new(),
            recursion_detector,
            safety_validator,
        }
    }
}

impl<S: StorageEngine + Send + 'static> LoopExecutor<S> {
    /// 验证循环执行器是否存在自引用
    pub fn validate_no_self_reference(&self) -> Result<(), DBError> {
        if self.body_executor.id() == self.base.id {
            return Err(DBError::Query(
                crate::core::error::QueryError::ExecutionError("循环执行器不能自引用".to_string()),
            ));
        }
        Ok(())
    }

    /// 评估循环条件
    async fn evaluate_condition(&mut self) -> DBResult<bool> {
        match &self.condition {
            Some(expression) => {
                let result =
                    ExpressionEvaluator::evaluate(expression, &mut self.loop_context).map_err(|e| {
                        DBError::Expression(crate::core::error::ExpressionError::function_error(
                            e.to_string(),
                        ))
                    })?;

                Ok(self.value_to_bool(&result))
            }
            None => Ok(true),
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
            _ => true,
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

        self.loop_context.set_variable(
            "__iteration".to_string(),
            Value::Int(self.current_iteration as i64),
        );

        let result = self.body_executor.execute().await?;

        self.body_executor.close()?;
        self.body_executor.open()?;

        Ok(result)
    }

    /// 收集所有循环结果
    fn collect_results(&self) -> ExecutionResult {
        if self.results.is_empty() {
            return ExecutionResult::Success;
        }

        let mut all_values = Vec::new();
        let mut all_vertices = Vec::new();
        let mut all_edges = Vec::new();
        let mut all_paths = Vec::new();
        let mut all_datasets = Vec::new();
        let mut has_success_only = true;

        for result in &self.results {
            match result {
                ExecutionResult::Values(values) => {
                    all_values.extend(values.clone());
                    has_success_only = false;
                }
                ExecutionResult::Vertices(vertices) => {
                    all_vertices.extend(vertices.clone());
                    has_success_only = false;
                }
                ExecutionResult::Edges(edges) => {
                    all_edges.extend(edges.clone());
                    has_success_only = false;
                }
                ExecutionResult::Paths(paths) => {
                    all_paths.extend(paths.clone());
                    has_success_only = false;
                }
                ExecutionResult::DataSet(dataset) => {
                    all_datasets.push(dataset.clone());
                    has_success_only = false;
                }
                ExecutionResult::Count(count) => {
                    all_values.push(Value::Int(*count as i64));
                    has_success_only = false;
                }
                ExecutionResult::Success => {}
                ExecutionResult::Error(_) => {
                    has_success_only = false;
                }
                ExecutionResult::Result(_) => {
                    has_success_only = false;
                }
            }
        }

        if has_success_only {
            return ExecutionResult::Success;
        }

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
            if all_datasets.len() == 1 {
                ExecutionResult::DataSet(all_datasets.into_iter().next().expect("Failed to get next dataset"))
            } else {
                ExecutionResult::DataSet(all_datasets.first().cloned().expect("Failed to get first dataset"))
            }
        } else {
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
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for LoopExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        self.validate_no_self_reference()?;

        self.safety_validator.validate_loop_config(self.max_iterations)?;

        self.recursion_detector.validate_executor(self.body_executor.id(), self.body_executor.name())?;

        self.loop_state = LoopState::Running;
        self.results.clear();
        self.current_iteration = 0;

        self.body_executor.open()?;

        while self.should_continue() {
            self.loop_context.set_variable(
                "__iteration".to_string(),
                Value::Int(self.current_iteration as i64),
            );

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

        let _ = self.body_executor.close();

        self.recursion_detector.leave_executor();

        if !matches!(self.loop_state, LoopState::Error(_)) {
            self.loop_state = LoopState::Finished;
        }

        Ok(self.collect_results())
    }

    fn open(&mut self) -> DBResult<()> {
        self.loop_state = LoopState::NotStarted;
        self.current_iteration = 0;
        self.results.clear();
        self.loop_context = DefaultExpressionContext::new();

        self.body_executor.open()?;
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        self.body_executor.close()?;

        self.results.clear();
        self.loop_context = DefaultExpressionContext::new();

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

impl<S: StorageEngine + Send + 'static> HasStorage<S> for LoopExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

/// WhileLoopExecutor - 条件循环执行器
///
/// 专门用于实现 WHILE 循环
pub struct WhileLoopExecutor<S: StorageEngine + Send + 'static> {
    inner: LoopExecutor<S>,
}

impl<S: StorageEngine + Send + 'static> WhileLoopExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        condition: Expression,
        body_executor: ExecutorEnum<S>,
        max_iterations: Option<usize>,
    ) -> Self {
        Self {
            inner: LoopExecutor::new(id, storage, Some(condition), body_executor, max_iterations),
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for WhileLoopExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        self.inner.execute().await
    }

    fn open(&mut self) -> DBResult<()> {
        self.inner.open()
    }

    fn close(&mut self) -> DBResult<()> {
        self.inner.close()
    }

    fn is_open(&self) -> bool {
        self.inner.is_open()
    }

    fn id(&self) -> i64 {
        self.inner.id()
    }

    fn name(&self) -> &str {
        "WhileLoopExecutor"
    }

    fn description(&self) -> &str {
        &self.inner.description()
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.inner.stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.inner.stats_mut()
    }
}

impl<S: StorageEngine + Send + 'static> HasStorage<S> for WhileLoopExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.inner.get_storage()
    }
}

/// ForLoopExecutor - 计数循环执行器
///
/// 专门用于实现 FOR 循环
pub struct ForLoopExecutor<S: StorageEngine + Send + 'static> {
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
        body_executor: ExecutorEnum<S>,
    ) -> Self {
        let mut executor = LoopExecutor::new(
            id,
            storage,
            None,
            body_executor,
            Some(((end - start).abs() / step.abs() +1) as usize),
        );

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
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for ForLoopExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        self.inner.open()?;

        let mut current = self.start;
        let mut results = Vec::new();

        while (self.step > 0 && current <= self.end) || (self.step < 0 && current >= self.end) {
            self.inner.set_loop_variable(self.loop_var.clone(), Value::Int(current));

            let result = self.inner.execute_iteration().await?;
            results.push(result);

            current += self.step;
        }

        self.inner.close()?;

        self.inner.results = results;
        self.inner.loop_state = LoopState::Finished;
        self.inner.current_iteration = ((self.end - self.start).abs() / self.step.abs() + 1) as usize;
        Ok(self.inner.collect_results())
    }

    fn open(&mut self) -> DBResult<()> {
        self.inner.open()
    }

    fn close(&mut self) -> DBResult<()> {
        self.inner.close()
    }

    fn is_open(&self) -> bool {
        self.inner.is_open()
    }

    fn id(&self) -> i64 {
        self.inner.id()
    }

    fn name(&self) -> &str {
        "ForLoopExecutor"
    }

    fn description(&self) -> &str {
        &self.inner.description()
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.inner.stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.inner.stats_mut()
    }
}

impl<S: StorageEngine + Send + 'static> HasStorage<S> for ForLoopExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.inner.get_storage()
    }
}

/// SelectExecutor - 条件分支执行器
///
/// 实现条件分支逻辑，根据条件选择执行 if 或 else 分支
pub struct SelectExecutor<S: StorageEngine + Send + 'static> {
    base: BaseExecutor<S>,
    condition: Expression,
    if_branch: Box<ExecutorEnum<S>>,
    else_branch: Option<Box<ExecutorEnum<S>>>,
    current_result: Option<ExecutionResult>,
}

impl<S: StorageEngine> SelectExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        condition: Expression,
        if_branch: ExecutorEnum<S>,
        else_branch: Option<ExecutorEnum<S>>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "SelectExecutor".to_string(), storage),
            condition,
            if_branch: Box::new(if_branch),
            else_branch: else_branch.map(Box::new),
            current_result: None,
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for SelectExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let mut context = crate::expression::DefaultExpressionContext::new();

        let condition_result = ExpressionEvaluator::evaluate(&self.condition, &mut context)
            .map_err(|e| DBError::Expression(crate::core::error::ExpressionError::function_error(e.to_string())))?;

        let condition_value = match condition_result {
            Value::Bool(b) => b,
            Value::Int(i) => i != 0,
            Value::Float(f) => f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::List(l) => !l.is_empty(),
            Value::Map(m) => !m.is_empty(),
            Value::Null(_) => false,
            Value::Empty => false,
            _ => true,
        };

        let branch_to_execute = if condition_value {
            &mut self.if_branch
        } else {
            match self.else_branch {
                Some(ref mut branch) => branch,
                None => {
                    return Ok(ExecutionResult::Success);
                }
            }
        };

        branch_to_execute.open()?;
        let result = branch_to_execute.execute().await?;
        branch_to_execute.close()?;

        self.current_result = Some(result.clone());
        Ok(result)
    }

    fn open(&mut self) -> DBResult<()> {
        self.if_branch.open()?;
        if let Some(ref mut else_branch) = self.else_branch {
            else_branch.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        self.if_branch.close()?;
        if let Some(ref mut else_branch) = self.else_branch {
            else_branch.close()?;
        }
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.if_branch.is_open()
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        "Select executor - conditional branch execution"
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageEngine + Send + 'static> HasStorage<S> for SelectExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.storage.as_ref().expect("SelectExecutor storage should be set")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::BinaryOperator;
    use crate::storage::test_mock::MockStorage;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_while_loop_executor() {
        let storage = Arc::new(Mutex::new(MockStorage));
        let storage_clone = storage.clone();

        let condition = Expression::binary(
            Expression::variable("__iteration"),
            BinaryOperator::LessThan,
            Expression::int(3),
        );

        let body_executor = ExecutorEnum::Base(BaseExecutor::new(2, "TestExecutor".to_string(), storage_clone));

        let mut executor = WhileLoopExecutor::new(
            1,
            storage,
            condition,
            body_executor,
            Some(5),
        );

        let result = executor.execute().await.expect("Failed to execute");

        match result {
            ExecutionResult::Success => {
                assert_eq!(executor.inner.current_iteration(), 3);
                assert_eq!(executor.inner.loop_state(), &LoopState::Finished);
            }
            _ => panic!("Expected Success result, got: {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_for_loop_executor() {
        let storage = Arc::new(Mutex::new(MockStorage));
        let storage_clone = storage.clone();

        let body_executor = ExecutorEnum::Base(BaseExecutor::new(2, "TestExecutor".to_string(), storage_clone));

        let mut executor =
            ForLoopExecutor::new(1, storage, "i".to_string(), 1, 3, 1, body_executor);

        let result = executor.execute().await.expect("Failed to execute");

        match result {
            ExecutionResult::Success => {
                assert_eq!(executor.inner.current_iteration(), 3);
                assert_eq!(executor.inner.loop_state(), &LoopState::Finished);
            }
            _ => panic!("Expected Success result"),
        }
    }
}

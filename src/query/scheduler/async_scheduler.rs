use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, Condvar};
use async_trait::async_trait;

use crate::query::executor::{ExecutionContext, ExecutionResult};
use crate::storage::StorageEngine;
use crate::query::QueryError;
use super::types::QueryScheduler;
use super::execution_plan::ExecutionPlan;

// Execution state tracking
#[derive(Debug)]
pub struct ExecutionState {
    pub executing_executors: HashSet<usize>,
    pub execution_results: HashMap<usize, ExecutionResult>,
    pub failed_status: Option<QueryError>,
}

impl ExecutionState {
    pub fn new() -> Self {
        Self {
            executing_executors: HashSet::new(),
            execution_results: HashMap::new(),
            failed_status: None,
        }
    }

    pub fn is_executor_executing(&self, executor_id: usize) -> bool {
        self.executing_executors.contains(&executor_id)
    }

    pub fn is_executor_completed(&self, executor_id: usize) -> bool {
        self.execution_results.contains_key(&executor_id)
    }

    pub fn has_failure(&self) -> bool {
        self.failed_status.is_some()
    }

    pub fn set_failure(&mut self, error: QueryError) {
        self.failed_status = Some(error);
    }

    pub fn take_failure(&mut self) -> Option<QueryError> {
        self.failed_status.take()
    }
}

// Implementation of the AsyncMsgNotifyBasedScheduler in Rust
pub struct AsyncMsgNotifyBasedScheduler<S: StorageEngine> {
    storage: Arc<Mutex<S>>,
    execution_context: Arc<Mutex<ExecutionContext>>,
    execution_state: Arc<Mutex<ExecutionState>>,
    // 用于同步等待执行完成
    completion_notifier: Arc<(Mutex<bool>, Condvar)>,
}

impl<S: StorageEngine + Send + 'static> AsyncMsgNotifyBasedScheduler<S> {
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        Self {
            storage,
            execution_context: Arc::new(Mutex::new(ExecutionContext::new())),
            execution_state: Arc::new(Mutex::new(ExecutionState::new())),
            completion_notifier: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }

    // Execute a single executor
    async fn execute_executor(&self, executor_id: usize, plan: &mut ExecutionPlan<S>) -> Result<ExecutionResult, QueryError> {
        // 从计划中获取执行器
        let mut executor = plan.executors.remove(&executor_id)
            .ok_or_else(|| QueryError::InvalidQuery(format!("Executor {} not found", executor_id)))?;

        {
            let mut state = self.execution_state.lock().unwrap();
            state.executing_executors.insert(executor_id);
        }

        let result = executor.execute().await;

        {
            let mut state = self.execution_state.lock().unwrap();
            state.executing_executors.remove(&executor_id);

            match &result {
                Ok(res) => {
                    state.execution_results.insert(executor_id, res.clone());
                },
                Err(e) => {
                    // Convert the error to QueryError for the state
                    let query_error = QueryError::ExecutionError(e.to_string());
                    state.set_failure(query_error);
                }
            }
        }

        // 将执行器放回计划中
        plan.executors.insert(executor_id, executor);

        result
    }

    // Get executable executors (those with all dependencies satisfied)
    fn get_executable_executors(&self, plan: &ExecutionPlan<S>) -> Vec<usize> {
        let state = self.execution_state.lock().unwrap();
        plan.get_executable_executors(&state.execution_results)
            .into_iter()
            .filter(|id| !state.is_executor_executing(*id))
            .collect()
    }

    // 通知执行器完成
    fn notify_completion(&self) {
        let (ref lock, ref cvar) = *self.completion_notifier;
        let mut completed = lock.lock().unwrap();
        *completed = true;
        cvar.notify_all();
    }

    // 检查是否所有执行器都已完成
    fn all_executors_completed(&self) -> bool {
        let state = self.execution_state.lock().unwrap();
        state.executing_executors.is_empty()
    }

    // Check if there are any failures
    fn has_failure(&self) -> bool {
        self.execution_state.lock().unwrap().has_failure()
    }

    // Wait for all executors to finish
    async fn wait_for_completion(&self) -> Result<(), QueryError> {
        // 使用条件变量进行同步等待
        let (ref lock, ref cvar) = *self.completion_notifier;
        let mut completed = lock.lock().unwrap();
        
        while !*completed && !self.all_executors_completed() {
            completed = cvar.wait(completed).unwrap();
        }

        // 检查是否有执行失败
        let state = self.execution_state.lock().unwrap();
        if let Some(ref error) = state.failed_status {
            return Err(error.clone());
        }

        Ok(())
    }

    // Execute a batch of executors in parallel
    async fn execute_executor_batch(
        &self,
        executor_ids: &[usize],
        plan: &mut ExecutionPlan<S>,
    ) -> Result<Vec<usize>, QueryError> {
        let mut tasks = Vec::new();
        let mut next_executors = Vec::new();

        // Create tasks for parallel execution
        for &executor_id in executor_ids {
            // 从计划中取出执行器
            if let Some(mut executor) = plan.executors.remove(&executor_id) {
                let state = self.execution_state.clone();
                
                let task = tokio::spawn(async move {
                    // 标记执行器正在执行
                    {
                        let mut state_lock = state.lock().unwrap();
                        state_lock.executing_executors.insert(executor_id);
                    }
                    
                    // 执行执行器
                    let result = executor.execute().await;
                    
                    // 返回执行器、ID和结果
                    (executor_id, executor, result)
                });
                
                tasks.push(task);
            } else {
                return Err(QueryError::InvalidQuery(format!("Executor {} not found in plan", executor_id)));
            }
        }

        // 等待所有任务完成并收集结果
        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok((executor_id, executor, result)) => {
                    // 将执行器放回计划中
                    plan.executors.insert(executor_id, executor);
                    results.push((executor_id, result));
                },
                Err(e) => {
                    return Err(QueryError::InvalidQuery(format!("Task execution failed: {}", e)));
                }
            }
        }

        // 更新状态并收集下一个批次的执行器
        {
            let mut state = self.execution_state.lock().unwrap();
            for (executor_id, result) in results {
                state.executing_executors.remove(&executor_id);
                
                match result {
                    Ok(execution_result) => {
                        state.execution_results.insert(executor_id, execution_result);
                        
                        // 添加后继执行器到下一个批次
                        let successors = plan.get_successors(executor_id);
                        for successor_id in successors {
                            if plan.are_dependencies_satisfied(successor_id, &state.execution_results) {
                                next_executors.push(successor_id);
                            }
                        }
                    },
                    Err(error) => {
                        // 设置失败状态
                        state.set_failure(error.clone());
                        return Err(error);
                    }
                }
            }
        }

        // 如果没有执行器在运行，通知完成
        if self.all_executors_completed() {
            self.notify_completion();
        }

        Ok(next_executors)
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> QueryScheduler<S> for AsyncMsgNotifyBasedScheduler<S> {
    async fn schedule(&mut self, mut execution_plan: ExecutionPlan<S>) -> Result<ExecutionResult, QueryError> {
        // 重置完成通知器
        {
            let (ref lock, _) = *self.completion_notifier;
            let mut completed = lock.lock().unwrap();
            *completed = false;
        }

        // 重置执行状态
        {
            let mut state = self.execution_state.lock().unwrap();
            state.executing_executors.clear();
            state.execution_results.clear();
            state.failed_status = None;
        }

        // Validate the execution plan
        execution_plan.validate()?;

        // Start with the root executor
        let mut current_executors = vec![execution_plan.root_executor_id];

        // Execute the plan using a breadth-first approach
        while !current_executors.is_empty() && !self.has_failure() {
            // Execute all currently executable executors in parallel
            current_executors = self.execute_executor_batch(&current_executors, &mut execution_plan).await?;
        }

        // Wait for all executors to finish
        self.wait_for_completion().await?;

        // Check for overall failure
        let state = self.execution_state.lock().unwrap();
        if let Some(error) = &state.failed_status {
            return Err(error.clone());
        }

        // Return the result of the root executor
        match state.execution_results.get(&execution_plan.root_executor_id) {
            Some(result) => Ok(result.clone()),
            None => Ok(ExecutionResult::Success),
        }
    }

    fn wait_finish(&mut self) -> Result<(), QueryError> {
        // 同步版本的等待完成
        // 使用条件变量等待所有执行器完成
        let (ref lock, ref cvar) = *self.completion_notifier;
        let mut completed = lock.lock().unwrap();
        
        while !*completed && !self.all_executors_completed() {
            completed = cvar.wait(completed).unwrap();
        }

        // 检查是否有执行失败
        let state = self.execution_state.lock().unwrap();
        if let Some(ref error) = state.failed_status {
            return Err(error.clone());
        }

        Ok(())
    }
}
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Condvar, Mutex};

use super::execution_schedule::ExecutionSchedule;
use super::types::QueryScheduler;
use crate::query::executor::{ExecutionContext, ExecutionResult};
use crate::query::QueryError;
use crate::storage::StorageEngine;
use crate::utils::safe_lock;

// Execution state tracking
#[derive(Debug)]
pub struct ExecutionState {
    pub executing_executors: HashSet<i64>,
    pub execution_results: HashMap<i64, ExecutionResult>,
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

    pub fn is_executor_executing(&self, executor_id: i64) -> bool {
        self.executing_executors.contains(&executor_id)
    }

    pub fn is_executor_completed(&self, executor_id: i64) -> bool {
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
    async fn execute_executor(
        &self,
        executor_id: i64,
        execution_schedule: &mut ExecutionSchedule<S>,
    ) -> Result<ExecutionResult, QueryError> {
        // 从执行调度中获取执行器
        let mut executor = execution_schedule
            .executors
            .remove(&executor_id)
            .ok_or_else(|| {
                QueryError::InvalidQuery(format!("Executor {} not found", executor_id))
            })?;

        {
            let mut state = safe_lock(&self.execution_state)
                .expect("AsyncScheduler execution_state lock should not be poisoned");
            state.executing_executors.insert(executor_id);
        }

        let result: Result<ExecutionResult, QueryError> =
            executor.execute().await.map_err(|e| match e {
                crate::core::error::DBError::Storage(storage_err) => {
                    QueryError::ExecutionError(storage_err.to_string())
                }
                crate::core::error::DBError::Query(qe) => {
                    QueryError::ExecutionError(qe.to_string())
                }
                crate::core::error::DBError::Expression(expr_err) => {
                    QueryError::ExecutionError(expr_err.to_string())
                }
                crate::core::error::DBError::Plan(plan_err) => {
                    QueryError::ExecutionError(plan_err.to_string())
                }
                crate::core::error::DBError::Lock(lock_err) => {
                    QueryError::ExecutionError(lock_err.to_string())
                }
                crate::core::error::DBError::Manager(manager_err) => {
                    QueryError::ExecutionError(manager_err.to_string())
                }
                crate::core::error::DBError::Validation(msg) => QueryError::InvalidQuery(msg),
                crate::core::error::DBError::Io(io_err) => {
                    QueryError::ExecutionError(io_err.to_string())
                }
                crate::core::error::DBError::TypeDeduction(msg) => QueryError::ExecutionError(msg),
                crate::core::error::DBError::Serialization(msg) => QueryError::ExecutionError(msg),
                crate::core::error::DBError::Index(msg) => QueryError::ExecutionError(msg),
                crate::core::error::DBError::Transaction(msg) => QueryError::ExecutionError(msg),
                crate::core::error::DBError::Internal(msg) => QueryError::ExecutionError(msg),
            });

        {
            let mut state = safe_lock(&self.execution_state)
                .expect("AsyncScheduler execution_state lock should not be poisoned");
            state.executing_executors.remove(&executor_id);

            match &result {
                Ok(res) => {
                    state.execution_results.insert(executor_id, res.clone());
                }
                Err(e) => {
                    // Convert the error to QueryError for the state
                    state.set_failure(e.clone());
                }
            }
        }

        // 将执行器放回执行调度中
        execution_schedule.executors.insert(executor_id, executor);

        result
    }

    // Get executable executors (those with all dependencies satisfied)
    fn get_executable_executors(&self, execution_schedule: &ExecutionSchedule<S>) -> Vec<i64> {
        let state = safe_lock(&self.execution_state)
            .expect("AsyncScheduler execution_state lock should not be poisoned");
        execution_schedule
            .get_executable_executors(&state.execution_results)
            .into_iter()
            .filter(|id| !state.is_executor_executing(*id))
            .collect()
    }

    // 通知执行器完成
    fn notify_completion(&self) {
        let (ref lock, ref cvar) = *self.completion_notifier;
        let mut completed = safe_lock(lock)
            .expect("AsyncScheduler completion_notifier lock should not be poisoned");
        *completed = true;
        cvar.notify_all();
    }

    // 检查是否所有执行器都已完成
    fn all_executors_completed(&self) -> bool {
        let state = safe_lock(&self.execution_state)
            .expect("AsyncScheduler execution_state lock should not be poisoned");
        state.executing_executors.is_empty()
    }

    // Check if there are any failures
    fn has_failure(&self) -> bool {
        let state = safe_lock(&self.execution_state)
            .expect("AsyncScheduler execution_state lock should not be poisoned");
        state.has_failure()
    }

    // Wait for all executors to finish
    async fn wait_for_completion(&self) -> Result<(), QueryError> {
        // 使用条件变量进行同步等待
        let (ref lock, ref cvar) = *self.completion_notifier;
        let mut completed = safe_lock(lock)
            .expect("AsyncScheduler completion_notifier lock should not be poisoned");

        while !*completed && !self.all_executors_completed() {
            completed = cvar
                .wait(completed)
                .expect("AsyncScheduler completion_notifier wait should not be poisoned");
        }

        // 检查是否有执行失败
        let state = safe_lock(&self.execution_state)
            .expect("AsyncScheduler execution_state lock should not be poisoned");
        if let Some(ref error) = state.failed_status {
            return Err(error.clone());
        }

        Ok(())
    }

    // Execute a batch of executors in parallel
    async fn execute_executor_batch(
        &self,
        executor_ids: &[i64],
        execution_schedule: &mut ExecutionSchedule<S>,
    ) -> Result<Vec<i64>, QueryError> {
        let mut tasks = Vec::new();
        let mut next_executors = Vec::new();

        // Create tasks for parallel execution
        for &executor_id in executor_ids {
            // 从执行调度中取出执行器
            if let Some(mut executor) = execution_schedule.executors.remove(&executor_id) {
                let state = self.execution_state.clone();

                let task = tokio::spawn(async move {
                    // 标记执行器正在执行
                    {
                        let mut state_lock = safe_lock(&state)
                            .expect("AsyncScheduler execution_state lock should not be poisoned");
                        state_lock.executing_executors.insert(executor_id);
                    }

                    // 执行执行器
                    let result = executor.execute().await;

                    // 返回执行器、ID和结果
                    (executor_id, executor, result)
                });

                tasks.push(task);
            } else {
                return Err(QueryError::InvalidQuery(format!(
                    "Executor {} not found in execution schedule",
                    executor_id
                )));
            }
        }

        // 等待所有任务完成并收集结果
        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok((executor_id, executor, result)) => {
                    // 将执行器放回执行调度中
                    execution_schedule.executors.insert(executor_id, executor);
                    results.push((executor_id, result));
                }
                Err(e) => {
                    return Err(QueryError::InvalidQuery(format!(
                        "Task execution failed: {}",
                        e
                    )));
                }
            }
        }

        // 更新状态并收集下一个批次的执行器
        {
            let mut state = safe_lock(&self.execution_state)
                .expect("AsyncScheduler execution_state lock should not be poisoned");
            for (executor_id, result) in results {
                state.executing_executors.remove(&executor_id);

                match result {
                    Ok(execution_result) => {
                        state
                            .execution_results
                            .insert(executor_id, execution_result);

                        // 添加后继执行器到下一个批次
                        let successors = execution_schedule.get_successors(executor_id);
                        for successor_id in successors {
                            if execution_schedule
                                .are_dependencies_satisfied(successor_id, &state.execution_results)
                            {
                                next_executors.push(successor_id);
                            }
                        }
                    }
                    Err(error) => {
                        // 设置失败状态
                        let query_error = QueryError::ExecutionError(error.to_string());
                        state.set_failure(query_error.clone());
                        return Err(query_error);
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
    async fn schedule(
        &mut self,
        mut execution_schedule: ExecutionSchedule<S>,
    ) -> Result<ExecutionResult, QueryError> {
        // 重置完成通知器
        {
            let (ref lock, _) = *self.completion_notifier;
            let mut completed = safe_lock(lock)
                .expect("AsyncScheduler completion_notifier lock should not be poisoned");
            *completed = false;
        }

        // 重置执行状态
        {
            let mut state = safe_lock(&self.execution_state)
                .expect("AsyncScheduler execution_state lock should not be poisoned");
            state.executing_executors.clear();
            state.execution_results.clear();
            state.failed_status = None;
        }

        // Validate the execution schedule
        execution_schedule.validate()?;

        // Start with the root executor
        let mut current_executors = vec![execution_schedule.root_executor_id];

        // Execute the schedule using a breadth-first approach
        while !current_executors.is_empty() && !self.has_failure() {
            // Execute all currently executable executors in parallel
            current_executors = self
                .execute_executor_batch(&current_executors, &mut execution_schedule)
                .await?;
        }

        // Wait for all executors to finish
        self.wait_for_completion().await?;

        // Check for overall failure
        let state = safe_lock(&self.execution_state)
            .expect("AsyncScheduler execution_state lock should not be poisoned");
        if let Some(error) = &state.failed_status {
            return Err(error.clone());
        }

        // Return the result of the root executor
        match state
            .execution_results
            .get(&execution_schedule.root_executor_id)
        {
            Some(result) => Ok(result.clone()),
            None => Ok(ExecutionResult::Success),
        }
    }

    fn wait_finish(&mut self) -> Result<(), QueryError> {
        // 同步版本的等待完成
        // 使用条件变量等待所有执行器完成
        let (ref lock, ref cvar) = *self.completion_notifier;
        let mut completed = safe_lock(lock)
            .expect("AsyncScheduler completion_notifier lock should not be poisoned");

        while !*completed && !self.all_executors_completed() {
            completed = cvar
                .wait(completed)
                .expect("AsyncScheduler completion_notifier wait should not be poisoned");
        }

        // 检查是否有执行失败
        let state = safe_lock(&self.execution_state)
            .expect("AsyncScheduler execution_state lock should not be poisoned");
        if let Some(ref error) = state.failed_status {
            return Err(error.clone());
        }

        Ok(())
    }
}

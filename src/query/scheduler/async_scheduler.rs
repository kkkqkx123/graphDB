use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};

use super::execution_schedule::ExecutionSchedule;
use super::types::{ExecutorType, QueryScheduler, SchedulerConfig};
use crate::query::executor::{ExecutionContext, ExecutionResult, Executor};
use crate::query::QueryError;
use crate::storage::StorageEngine;
use crate::utils::safe_lock;

#[derive(Debug)]
pub struct ExecutionState {
    pub executing_count: AtomicUsize,
    pub execution_results: HashMap<i64, ExecutionResult>,
    pub failed_status: Option<QueryError>,
}

impl ExecutionState {
    pub fn new() -> Self {
        Self {
            executing_count: AtomicUsize::new(0),
            execution_results: HashMap::new(),
            failed_status: None,
        }
    }

    pub fn is_executor_executing(&self, executor_id: i64) -> bool {
        self.executing_count.load(Ordering::SeqCst) > 0
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

#[derive(Debug, Clone)]
pub struct AsyncMsgNotifyBasedScheduler<S: StorageEngine> {
    storage: Arc<Mutex<S>>,
    execution_context: Arc<Mutex<ExecutionContext>>,
    execution_state: Arc<Mutex<ExecutionState>>,
    completion_notifier: Arc<(Mutex<bool>, Condvar)>,
    config: SchedulerConfig,
}

impl<S: StorageEngine + Send + 'static> AsyncMsgNotifyBasedScheduler<S> {
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        Self {
            storage,
            execution_context: Arc::new(Mutex::new(ExecutionContext::new())),
            execution_state: Arc::new(Mutex::new(ExecutionState::new())),
            completion_notifier: Arc::new((Mutex::new(false), Condvar::new())),
            config: SchedulerConfig::default(),
        }
    }

    pub fn with_config(storage: Arc<Mutex<S>>, config: SchedulerConfig) -> Self {
        Self {
            storage,
            execution_context: Arc::new(Mutex::new(ExecutionContext::new())),
            execution_state: Arc::new(Mutex::new(ExecutionState::new())),
            completion_notifier: Arc::new((Mutex::new(false), Condvar::new())),
            config,
        }
    }

    fn check_status(&self, statuses: &[Result<(), QueryError>]) -> Result<(), QueryError> {
        for status in statuses {
            if let Err(e) = status {
                return Err(e.clone());
            }
        }
        Ok(())
    }

    fn format_pretty_id(&self, executor_id: i64) -> String {
        format!("[id:{}]", executor_id)
    }

    pub fn format_dependency_tree(&self, execution_schedule: &ExecutionSchedule<S>) -> String {
        execution_schedule.format_dependency_tree(execution_schedule.root_executor_id)
    }

    pub fn to_graphviz(&self, execution_schedule: &ExecutionSchedule<S>) -> String {
        execution_schedule.to_graphviz(execution_schedule.root_executor_id)
    }

    async fn execute_executor(
        &self,
        executor_id: i64,
        execution_schedule: &mut ExecutionSchedule<S>,
    ) -> Result<ExecutionResult, QueryError> {
        let mut executor = execution_schedule
            .executors
            .remove(&executor_id)
            .ok_or_else(|| {
                QueryError::InvalidQuery(format!("Executor {} not found", executor_id))
            })?;

        {
            let mut state = safe_lock(&self.execution_state)
                .expect("AsyncScheduler execution_state lock should not be poisoned");
            state.executing_count.fetch_add(1, Ordering::SeqCst);
        }

        let exec_type = execution_schedule.get_executor_type(executor_id);
        let result = match exec_type {
            ExecutorType::Select => {
                self.execute_select(executor_id, execution_schedule).await
            }
            ExecutorType::Loop => {
                self.execute_loop(executor_id, execution_schedule).await
            }
            ExecutorType::Argument => {
                self.execute_argument(executor_id, execution_schedule).await
            }
            _ => {
                executor.execute().await.map_err(QueryError::from)
            }
        };

        {
            let mut state = safe_lock(&self.execution_state)
                .expect("AsyncScheduler execution_state lock should not be poisoned");
            state.executing_count.fetch_sub(1, Ordering::SeqCst);
        }

        {
            let mut state = safe_lock(&self.execution_state)
                .expect("AsyncScheduler execution_state lock should not be poisoned");
            match &result {
                Ok(res) => {
                    state.execution_results.insert(executor_id, res.clone());
                }
                Err(e) => {
                    state.set_failure(e.clone());
                }
            }
        }

        execution_schedule.executors.insert(executor_id, executor);
        result
    }

    async fn execute_select(
        &self,
        executor_id: i64,
        execution_schedule: &mut ExecutionSchedule<S>,
    ) -> Result<ExecutionResult, QueryError> {
        let mut executor = execution_schedule
            .executors
            .remove(&executor_id)
            .ok_or_else(|| {
                QueryError::InvalidQuery(format!("Select Executor {} not found", executor_id))
            })?;

        let result = executor.execute().await.map_err(QueryError::from)?;

        execution_schedule.executors.insert(executor_id, executor);
        Ok(result)
    }

    async fn execute_loop(
        &self,
        executor_id: i64,
        execution_schedule: &mut ExecutionSchedule<S>,
    ) -> Result<ExecutionResult, QueryError> {
        let mut executor = execution_schedule
            .executors
            .remove(&executor_id)
            .ok_or_else(|| {
                QueryError::InvalidQuery(format!("Loop Executor {} not found", executor_id))
            })?;

        let result = executor.execute().await.map_err(QueryError::from)?;

        execution_schedule.executors.insert(executor_id, executor);
        Ok(result)
    }

    async fn execute_argument(
        &self,
        executor_id: i64,
        execution_schedule: &mut ExecutionSchedule<S>,
    ) -> Result<ExecutionResult, QueryError> {
        let mut executor = execution_schedule
            .executors
            .remove(&executor_id)
            .ok_or_else(|| {
                QueryError::InvalidQuery(format!("Argument Executor {} not found", executor_id))
            })?;

        let result = executor.execute().await.map_err(QueryError::from)?;

        execution_schedule.executors.insert(executor_id, executor);
        Ok(result)
    }

    fn get_executable_executors(&self, execution_schedule: &ExecutionSchedule<S>) -> Vec<i64> {
        let state = safe_lock(&self.execution_state)
            .expect("AsyncScheduler execution_state lock should not be poisoned");
        execution_schedule
            .get_executable_executors(&state.execution_results)
            .into_iter()
            .filter(|id| !state.is_executor_executing(*id))
            .collect()
    }

    fn notify_completion(&self) {
        let (ref lock, ref cvar) = *self.completion_notifier;
        let mut completed = safe_lock(lock)
            .expect("AsyncScheduler completion_notifier lock should not be poisoned");
        *completed = true;
        cvar.notify_all();
    }

    fn all_executors_completed(&self) -> bool {
        let state = safe_lock(&self.execution_state)
            .expect("AsyncScheduler execution_state lock should not be poisoned");
        state.executing_count.load(Ordering::SeqCst) == 0
    }

    fn has_failure(&self) -> bool {
        let state = safe_lock(&self.execution_state)
            .expect("AsyncScheduler execution_state lock should not be poisoned");
        state.has_failure()
    }

    async fn wait_for_completion(&self) -> Result<(), QueryError> {
        let (ref lock, ref cvar) = *self.completion_notifier;
        let mut completed = safe_lock(lock)
            .expect("AsyncScheduler completion_notifier lock should not be poisoned");

        while !*completed && !self.all_executors_completed() {
            completed = cvar
                .wait(completed)
                .expect("AsyncScheduler completion_notifier wait should not be poisoned");
        }

        let state = safe_lock(&self.execution_state)
            .expect("AsyncScheduler execution_state lock should not be poisoned");
        if let Some(ref error) = state.failed_status {
            return Err(error.clone());
        }

        Ok(())
    }

    async fn execute_executor_batch(
        &self,
        executor_ids: &[i64],
        execution_schedule: &mut ExecutionSchedule<S>,
    ) -> Result<Vec<i64>, QueryError> {
        let mut tasks = Vec::new();
        let mut next_executors = Vec::new();

        for &executor_id in executor_ids {
            if let Some(mut executor) = execution_schedule.executors.remove(&executor_id) {
                let state = self.execution_state.clone();

                let task = tokio::spawn(async move {
                    {
                        let mut state_guard = safe_lock(&state)
                            .expect("AsyncScheduler execution_state lock should not be poisoned");
                        state_guard.executing_count.fetch_add(1, Ordering::SeqCst);
                    }
                    let result = executor.execute().await.map_err(QueryError::from);
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

        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok((executor_id, executor, result)) => {
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

        for (executor_id, result) in results {
            {
                let mut state = safe_lock(&self.execution_state)
                    .expect("AsyncScheduler execution_state lock should not be poisoned");
                state.executing_count.fetch_sub(1, Ordering::SeqCst);
            }

            match result {
                Ok(execution_result) => {
                    {
                        let mut state = safe_lock(&self.execution_state)
                            .expect("AsyncScheduler execution_state lock should not be poisoned");
                        state.execution_results.insert(executor_id, execution_result);
                    }

                    let successors = execution_schedule.get_successors(executor_id);
                    for successor_id in successors {
                        let state = safe_lock(&self.execution_state)
                            .expect("AsyncScheduler execution_state lock should not be poisoned");
                        if execution_schedule
                            .are_dependencies_satisfied(successor_id, &state.execution_results)
                        {
                            next_executors.push(successor_id);
                        }
                    }
                }
                Err(error) => {
                    let query_error = QueryError::ExecutionError(error.to_string());
                    {
                        let mut state = safe_lock(&self.execution_state)
                            .expect("AsyncScheduler execution_state lock should not be poisoned");
                        state.set_failure(query_error.clone());
                    }
                    return Err(query_error);
                }
            }
        }

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
        {
            let (ref lock, _) = *self.completion_notifier;
            let mut completed = safe_lock(lock)
                .expect("AsyncScheduler completion_notifier lock should not be poisoned");
            *completed = false;
        }

        {
            let mut state = safe_lock(&self.execution_state)
                .expect("AsyncScheduler execution_state lock should not be poisoned");
            state.execution_results.clear();
            state.failed_status = None;
        }

        if self.config.enable_lifetime_optimize {
            execution_schedule.analyze_lifetime();
        }

        let mut current_executors = vec![execution_schedule.root_executor_id];

        while !current_executors.is_empty() && !self.has_failure() {
            current_executors = self
                .execute_executor_batch(&current_executors, &mut execution_schedule)
                .await?;
        }

        self.wait_for_completion().await?;

        let state = safe_lock(&self.execution_state)
            .expect("AsyncScheduler execution_state lock should not be poisoned");
        if let Some(ref error) = state.failed_status {
            return Err(error.clone());
        }

        match state
            .execution_results
            .get(&execution_schedule.root_executor_id)
        {
            Some(result) => Ok(result.clone()),
            None => Ok(ExecutionResult::Success),
        }
    }

    fn wait_finish(&mut self) -> Result<(), QueryError> {
        let (ref lock, ref cvar) = *self.completion_notifier;
        let mut completed = safe_lock(lock)
            .expect("AsyncScheduler completion_notifier lock should not be poisoned");

        while !*completed && !self.all_executors_completed() {
            completed = cvar
                .wait(completed)
                .expect("AsyncScheduler completion_notifier wait should not be poisoned");
        }

        let state = safe_lock(&self.execution_state)
            .expect("AsyncScheduler execution_state lock should not be poisoned");
        if let Some(ref error) = state.failed_status {
            return Err(error.clone());
        }

        Ok(())
    }
}

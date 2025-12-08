use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use async_trait::async_trait;

use crate::query::executor::{Executor, ExecutionContext, ExecutionResult};
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
}

impl<S: StorageEngine + Send + 'static> AsyncMsgNotifyBasedScheduler<S> {
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        Self {
            storage,
            execution_context: Arc::new(Mutex::new(ExecutionContext::new())),
            execution_state: Arc::new(Mutex::new(ExecutionState::new())),
        }
    }

    // Execute a single executor
    async fn execute_executor(&self, executor: &mut Box<dyn Executor<S>>) -> Result<ExecutionResult, QueryError> {
        {
            let mut state = self.execution_state.lock().unwrap();
            state.executing_executors.insert(executor.id());
        }

        let result = executor.execute().await;

        {
            let mut state = self.execution_state.lock().unwrap();
            state.executing_executors.remove(&executor.id());

            match &result {
                Ok(res) => {
                    state.execution_results.insert(executor.id(), res.clone());
                },
                Err(e) => {
                    state.set_failure(e.clone());
                }
            }
        }

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

    // Check if there are any failures
    fn has_failure(&self) -> bool {
        self.execution_state.lock().unwrap().has_failure()
    }

    // Wait for all executors to finish
    async fn wait_for_completion(&self) -> Result<(), QueryError> {
        loop {
            {
                let state = self.execution_state.lock().unwrap();
                if state.executing_executors.is_empty() {
                    break;
                }
            } // state is dropped here

            // In a real async implementation, we'd use proper async waiting
            // For now, just yield to the task scheduler
            tokio::task::yield_now().await;
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
            if let Some(mut executor) = plan.executors.remove(&executor_id) {
                let state = self.execution_state.clone();
                
                let task = tokio::spawn(async move {
                    // Execute the executor
                    let result = executor.execute().await;
                    
                    // Update state
                    {
                        let mut state_lock = state.lock().unwrap();
                        state_lock.executing_executors.remove(&executor_id);
                        
                        match result {
                            Ok(ref res) => {
                                state_lock.execution_results.insert(executor_id, res.clone());
                            },
                            Err(ref e) => {
                                state_lock.set_failure(e.clone());
                            }
                        }
                    }
                    
                    (executor_id, executor, result)
                });
                
                tasks.push(task);
            }
        }

        // Wait for all tasks to complete
        for task in tasks {
            match task.await {
                Ok((executor_id, executor, result)) => {
                    // Put the executor back
                    plan.executors.insert(executor_id, executor);
                    
                    // If execution was successful, add successors to next batch
                    if result.is_ok() {
                        let successors = plan.get_successors(executor_id);
                        for successor_id in successors {
                            if plan.are_dependencies_satisfied(successor_id, &self.execution_state.lock().unwrap().execution_results) {
                                next_executors.push(successor_id);
                            }
                        }
                    }
                },
                Err(e) => {
                    return Err(QueryError::InvalidQuery(format!("Task execution failed: {}", e)));
                }
            }
        }

        Ok(next_executors)
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> QueryScheduler<S> for AsyncMsgNotifyBasedScheduler<S> {
    async fn schedule(&mut self, mut execution_plan: ExecutionPlan<S>) -> Result<ExecutionResult, QueryError> {
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

    fn add_dependency(&mut self, from: usize, to: usize) -> Result<(), QueryError> {
        // This would be used during plan construction, not during execution
        // For now, just return OK - dependencies should be set up in the ExecutionPlan
        Ok(())
    }

    fn wait_finish(&mut self) -> Result<(), QueryError> {
        // This is a synchronous version, but in practice we'd use the async version
        // For now, just check if there are any executing executors
        let state = self.execution_state.lock().unwrap();
        if state.executing_executors.is_empty() {
            Ok(())
        } else {
            Err(QueryError::InvalidQuery("Executors are still running".to_string()))
        }
    }
}
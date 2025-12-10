use async_trait::async_trait;

use crate::query::executor::ExecutionResult;
use crate::storage::StorageEngine;
use crate::query::QueryError;

// Executor dependency information
#[derive(Debug, Clone)]
pub struct ExecutorDep {
    pub executor_id: usize,
    pub dependencies: Vec<usize>, // IDs of executors that must execute before this one
    pub successors: Vec<usize>,   // IDs of executors that depend on this one
}

// Scheduler trait that defines how executors are coordinated
#[async_trait]
pub trait QueryScheduler<S: StorageEngine> {
    async fn schedule(&mut self, execution_plan: super::execution_plan::ExecutionPlan<S>) -> Result<ExecutionResult, QueryError>;

    fn wait_finish(&mut self) -> Result<(), QueryError>;
}

// ExecutionPlan is defined in execution_plan.rs to avoid duplication
use async_trait::async_trait;

use crate::query::executor::ExecutionResult;
use crate::query::QueryError;
use crate::storage::StorageEngine;

// Executor dependency information
#[derive(Debug, Clone)]
pub struct ExecutorDep {
    pub executor_id: i64,
    pub dependencies: Vec<i64>, // IDs of executors that must execute before this one
    pub successors: Vec<i64>,   // IDs of executors that depend on this one
}

// Scheduler trait that defines how executors are coordinated
#[async_trait]
pub trait QueryScheduler<S: StorageEngine> {
    async fn schedule(
        &mut self,
        execution_schedule: super::execution_schedule::ExecutionSchedule<S>,
    ) -> Result<ExecutionResult, QueryError>;

    fn wait_finish(&mut self) -> Result<(), QueryError>;
}

// ExecutionSchedule is defined in execution_schedule.rs to avoid duplication

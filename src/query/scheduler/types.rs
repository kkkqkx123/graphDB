use async_trait::async_trait;

use crate::query::executor::ExecutionResult;
use crate::query::QueryError;
use crate::storage::StorageClient;

#[derive(Debug, Clone)]
pub struct ExecutorDep {
    pub executor_id: i64,
    pub dependencies: Vec<i64>,
    pub successors: Vec<i64>,
}

#[derive(Debug, Clone)]
pub struct VariableLifetime {
    pub name: String,
    pub user_count: usize,
    pub loop_layers: usize,
    pub is_root_output: bool,
}

impl VariableLifetime {
    pub fn new(name: String) -> Self {
        Self {
            name,
            user_count: 0,
            loop_layers: 0,
            is_root_output: false,
        }
    }

    pub fn is_unlimited(&self) -> bool {
        self.user_count == usize::MAX
    }
}

#[derive(Debug, Clone)]
pub enum ExecutionEvent {
    Started(i64),
    Completed(i64, bool),
    Failed(i64),
}

#[derive(Debug, Clone)]
pub enum ExecutorType {
    Normal,
    Select,
    Loop,
    Argument,
    Leaf,
}

#[async_trait]
pub trait QueryScheduler<S: StorageClient> {
    async fn schedule(
        &mut self,
        execution_schedule: super::execution_schedule::ExecutionSchedule<S>,
    ) -> Result<ExecutionResult, QueryError>;

    fn wait_finish(&mut self) -> Result<(), QueryError>;
}

#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    pub enable_lifetime_optimize: bool,
    pub max_concurrent_executors: usize,
    pub execution_timeout_ms: u64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            enable_lifetime_optimize: true,
            max_concurrent_executors: 100,
            execution_timeout_ms: 30000,
        }
    }
}

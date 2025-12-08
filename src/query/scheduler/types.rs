use std::collections::HashMap;
use async_trait::async_trait;

use crate::query::executor::{Executor, ExecutionResult};
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

    fn add_dependency(&mut self, from: usize, to: usize) -> Result<(), QueryError>;

    fn wait_finish(&mut self) -> Result<(), QueryError>;
}

// Execution plan containing multiple executors and their dependencies
pub struct ExecutionPlan<S: StorageEngine> {
    pub executors: HashMap<usize, Box<dyn Executor<S>>>,
    pub dependencies: HashMap<usize, ExecutorDep>,
    pub root_executor_id: usize, // The executor that starts the execution
}

impl<S: StorageEngine + Send + 'static> ExecutionPlan<S> {
    pub fn new(root_id: usize) -> Self {
        Self {
            executors: HashMap::new(),
            dependencies: HashMap::new(),
            root_executor_id: root_id,
        }
    }

    pub fn add_executor(&mut self, executor: Box<dyn Executor<S>>) {
        let id = executor.id();
        self.executors.insert(id, executor);

        // Initialize dependency info if not already present
        if !self.dependencies.contains_key(&id) {
            self.dependencies.insert(id, ExecutorDep {
                executor_id: id,
                dependencies: Vec::new(),
                successors: Vec::new(),
            });
        }
    }

    pub fn add_dependency(&mut self, from: usize, to: usize) -> Result<(), QueryError> {
        // Check that both executors exist
        if !self.executors.contains_key(&from) {
            return Err(QueryError::InvalidQuery(format!("Executor {} does not exist", from)));
        }
        if !self.executors.contains_key(&to) {
            return Err(QueryError::InvalidQuery(format!("Executor {} does not exist", to)));
        }

        // Update dependency relationships
        self.dependencies.entry(to).or_insert_with(|| ExecutorDep {
            executor_id: to,
            dependencies: Vec::new(),
            successors: Vec::new(),
        }).dependencies.push(from);

        self.dependencies.entry(from).or_insert_with(|| ExecutorDep {
            executor_id: from,
            dependencies: Vec::new(),
            successors: Vec::new(),
        }).successors.push(to);

        Ok(())
    }
}
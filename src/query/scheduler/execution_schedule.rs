use std::collections::HashMap;

use super::types::ExecutorDep;
use crate::query::executor::{ExecutionResult, Executor};
use crate::query::QueryError;
use crate::storage::StorageEngine;

// Execution schedule containing multiple executors and their dependencies
// This represents the physical execution plan with executor dependencies and scheduling
pub struct ExecutionSchedule<S: StorageEngine> {
    pub executors: HashMap<i64, Box<dyn Executor<S>>>,
    pub dependencies: HashMap<i64, ExecutorDep>,
    pub root_executor_id: i64, // The executor that starts the execution
}

impl<S: StorageEngine + Send + 'static> ExecutionSchedule<S> {
    pub fn new(root_id: i64) -> Self {
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
            self.dependencies.insert(
                id,
                ExecutorDep {
                    executor_id: id,
                    dependencies: Vec::new(),
                    successors: Vec::new(),
                },
            );
        }
    }

    pub fn add_dependency(&mut self, from: i64, to: i64) -> Result<(), QueryError> {
        // Check that both executors exist
        if !self.executors.contains_key(&from) {
            return Err(QueryError::InvalidQuery(format!(
                "Executor {} does not exist",
                from
            )));
        }
        if !self.executors.contains_key(&to) {
            return Err(QueryError::InvalidQuery(format!(
                "Executor {} does not exist",
                to
            )));
        }

        // Update dependency relationships
        self.dependencies
            .entry(to)
            .or_insert_with(|| ExecutorDep {
                executor_id: to,
                dependencies: Vec::new(),
                successors: Vec::new(),
            })
            .dependencies
            .push(from);

        self.dependencies
            .entry(from)
            .or_insert_with(|| ExecutorDep {
                executor_id: from,
                dependencies: Vec::new(),
                successors: Vec::new(),
            })
            .successors
            .push(to);

        Ok(())
    }

    /// Get all executors that can be executed (all dependencies satisfied)
    pub fn get_executable_executors(
        &self,
        completed_executors: &HashMap<i64, ExecutionResult>,
    ) -> Vec<i64> {
        let mut executable = Vec::new();

        for (id, dep_info) in &self.dependencies {
            let all_deps_satisfied = dep_info
                .dependencies
                .iter()
                .all(|dep_id| completed_executors.contains_key(dep_id));

            // Check if executor is not already executed
            if all_deps_satisfied && !completed_executors.contains_key(id) {
                executable.push(*id);
            }
        }

        executable
    }

    /// Get successors of a given executor
    pub fn get_successors(&self, executor_id: i64) -> Vec<i64> {
        self.dependencies
            .get(&executor_id)
            .map(|dep| dep.successors.clone())
            .unwrap_or_default()
    }

    /// Check if all dependencies for an executor are satisfied
    pub fn are_dependencies_satisfied(
        &self,
        executor_id: i64,
        completed_executors: &HashMap<i64, ExecutionResult>,
    ) -> bool {
        self.dependencies
            .get(&executor_id)
            .map(|dep| {
                dep.dependencies
                    .iter()
                    .all(|dep_id| completed_executors.contains_key(dep_id))
            })
            .unwrap_or(true) // No dependencies means satisfied
    }

    /// Validate the execution schedule for cycles and missing dependencies
    pub fn validate(&self) -> Result<(), QueryError> {
        // Check for cycles using DFS
        let mut visited = std::collections::HashSet::new();
        let mut recursion_stack = std::collections::HashSet::new();

        for executor_id in self.executors.keys() {
            if !visited.contains(executor_id) {
                if self.has_cycle(*executor_id, &mut visited, &mut recursion_stack)? {
                    return Err(QueryError::InvalidQuery(
                        "Cycle detected in execution schedule".to_string(),
                    ));
                }
            }
        }

        // Check that root executor exists
        if !self.executors.contains_key(&self.root_executor_id) {
            return Err(QueryError::InvalidQuery(format!(
                "Root executor {} does not exist",
                self.root_executor_id
            )));
        }

        Ok(())
    }

    /// Helper method to detect cycles using DFS
    fn has_cycle(
        &self,
        executor_id: i64,
        visited: &mut std::collections::HashSet<i64>,
        recursion_stack: &mut std::collections::HashSet<i64>,
    ) -> Result<bool, QueryError> {
        visited.insert(executor_id);
        recursion_stack.insert(executor_id);

        if let Some(dep_info) = self.dependencies.get(&executor_id) {
            for &successor_id in &dep_info.successors {
                if !visited.contains(&successor_id) {
                    if self.has_cycle(successor_id, visited, recursion_stack)? {
                        return Ok(true);
                    }
                } else if recursion_stack.contains(&successor_id) {
                    return Ok(true);
                }
            }
        }

        recursion_stack.remove(&executor_id);
        Ok(false)
    }
}

use std::collections::{HashMap, HashSet};

use super::types::{ExecutorDep, ExecutorType, VariableLifetime};
use crate::query::executor::{ExecutionResult, Executor};
use crate::query::QueryError;
use crate::storage::StorageClient;

pub struct ExecutionSchedule<S: StorageClient> {
    pub executors: HashMap<i64, Box<dyn Executor<S>>>,
    pub dependencies: HashMap<i64, ExecutorDep>,
    pub root_executor_id: i64,
    pub executor_types: HashMap<i64, ExecutorType>,
    pub variable_lifetimes: HashMap<String, VariableLifetime>,
    pub loop_layers: HashMap<i64, usize>,
    pub output_variables: HashMap<i64, String>,
}

impl<S: StorageClient + Send + 'static> ExecutionSchedule<S> {
    pub fn new(root_id: i64) -> Self {
        Self {
            executors: HashMap::new(),
            dependencies: HashMap::new(),
            root_executor_id: root_id,
            executor_types: HashMap::new(),
            variable_lifetimes: HashMap::new(),
            loop_layers: HashMap::new(),
            output_variables: HashMap::new(),
        }
    }

    pub fn set_executor_type(&mut self, executor_id: i64, exec_type: ExecutorType) {
        self.executor_types.insert(executor_id, exec_type);
    }

    pub fn get_executor_type(&self, executor_id: i64) -> ExecutorType {
        self.executor_types
            .get(&executor_id)
            .cloned()
            .unwrap_or(ExecutorType::Normal)
    }

    pub fn set_output_variable(&mut self, executor_id: i64, var_name: String) {
        self.output_variables.insert(executor_id, var_name.clone());
        self.variable_lifetimes
            .entry(var_name.clone())
            .or_insert_with(|| VariableLifetime::new(var_name));
    }

    pub fn get_output_variable(&self, executor_id: i64) -> Option<&String> {
        self.output_variables.get(&executor_id)
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

    pub fn analyze_lifetime(&mut self) {
        let mut visited: HashSet<i64> = HashSet::new();
        let mut stack: Vec<(i64, usize)> = Vec::new();
        stack.push((self.root_executor_id, 0));

        while let Some((executor_id, current_loop_layers)) = stack.pop() {
            self.loop_layers.insert(executor_id, current_loop_layers);

            if !visited.insert(executor_id) {
                continue;
            }

            if let Some(dep_info) = self.dependencies.get(&executor_id) {
                for &dep_id in &dep_info.dependencies {
                    if let Some(output_var) = self.get_output_variable(dep_id).cloned() {
                        if let Some(lifetime) = self.variable_lifetimes.get_mut(&output_var) {
                            lifetime.user_count += 1;
                        }
                    }
                    stack.push((dep_id, current_loop_layers));
                }
            }

            let exec_type = self.get_executor_type(executor_id);
            match exec_type {
                ExecutorType::Select => {
                    if let Some(dep_info) = self.dependencies.get(&executor_id) {
                        for &successor_id in &dep_info.successors {
                            stack.push((successor_id, current_loop_layers));
                        }
                    }
                }
                ExecutorType::Loop => {
                    if let Some(dep_info) = self.dependencies.get(&executor_id) {
                        for &successor_id in &dep_info.successors {
                            stack.push((successor_id, current_loop_layers + 1));
                        }
                    }
                }
                _ => {
                    if let Some(dep_info) = self.dependencies.get(&executor_id) {
                        for &successor_id in &dep_info.successors {
                            stack.push((successor_id, current_loop_layers));
                        }
                    }
                }
            }
        }

        if let Some(root_var) = self.get_output_variable(self.root_executor_id).cloned() {
            if let Some(lifetime) = self.variable_lifetimes.get_mut(&root_var) {
                lifetime.is_root_output = true;
                lifetime.user_count = usize::MAX;
            }
        }
    }

    pub fn get_variable_lifetime(&self, var_name: &str) -> Option<&VariableLifetime> {
        self.variable_lifetimes.get(var_name)
    }

    pub fn get_loop_layers(&self, executor_id: i64) -> usize {
        self.loop_layers.get(&executor_id).cloned().unwrap_or(0)
    }

    pub fn is_unlimited_variable(&self, var_name: &str) -> bool {
        self.variable_lifetimes
            .get(var_name)
            .map(|l| l.is_unlimited())
            .unwrap_or(false)
    }

    pub fn format_dependency_tree(&self, root_id: i64) -> String {
        let mut output = String::new();
        self.append_executor(root_id, 0, &mut output);
        output
    }

    fn append_executor(&self, executor_id: i64, spaces: usize, output: &mut String) {
        let indent = " ".repeat(spaces);
        let exec_type = self.get_executor_type(executor_id);
        let loop_layers = self.get_loop_layers(executor_id);
        
        output.push_str(&format!(
            "{}[{}, type:{:?}, loop_layers:{}]\n",
            indent, executor_id, exec_type, loop_layers
        ));

        if let Some(dep_info) = self.dependencies.get(&executor_id) {
            for &dep_id in &dep_info.dependencies {
                self.append_executor(dep_id, spaces + 2, output);
            }
        }
    }

    pub fn to_graphviz(&self, root_id: i64) -> String {
        let mut dot = String::from("digraph ExecutionSchedule {\n");
        dot.push_str("  rankdir=TB;\n");
        dot.push_str("  node [shape=box];\n");

        let mut visited: HashSet<i64> = HashSet::new();
        self.collect_dot_nodes(root_id, &mut dot, &mut visited);

        visited.clear();
        self.collect_dot_edges(root_id, &mut dot, &mut visited);

        dot.push_str("}\n");
        dot
    }

    fn collect_dot_nodes(&self, executor_id: i64, dot: &mut String, visited: &mut HashSet<i64>) {
        if !visited.insert(executor_id) {
            return;
        }

        let exec_type = self.get_executor_type(executor_id);
        let loop_layers = self.get_loop_layers(executor_id);
        dot.push_str(&format!(
            "  {} [label=\"id:{}\ntype:{:?}\nloop:{}\"];\n",
            executor_id, executor_id, exec_type, loop_layers
        ));

        if let Some(dep_info) = self.dependencies.get(&executor_id) {
            for &dep_id in &dep_info.dependencies {
                self.collect_dot_nodes(dep_id, dot, visited);
            }
        }
    }

    fn collect_dot_edges(&self, executor_id: i64, dot: &mut String, visited: &mut HashSet<i64>) {
        if !visited.insert(executor_id) {
            return;
        }

        if let Some(dep_info) = self.dependencies.get(&executor_id) {
            for &dep_id in &dep_info.successors {
                dot.push_str(&format!("  {} -> {};\n", dep_id, executor_id));
                self.collect_dot_edges(dep_id, dot, visited);
            }
        }
    }
}

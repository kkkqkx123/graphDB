//! Recursive detector
//!
//! Responsible for detecting circular references in the execution plan to prevent infinite recursion.

use crate::core::error::DBError;
use std::collections::HashSet;

/// Recursive detector
#[derive(Debug, Clone)]
pub struct RecursionDetector {
    /// The access stack is used to detect loops.
    visit_stack: Vec<(i64, &'static str)>,
    /// Set of visited nodes
    visited: HashSet<i64>,
    /// Maximum recursion depth
    max_depth: usize,
}

impl RecursionDetector {
    /// Create a new recursive detector.
    pub fn new(max_depth: usize) -> Self {
        Self {
            visit_stack: Vec::new(),
            visited: HashSet::new(),
            max_depth,
        }
    }

    /// Reset the detector status.
    pub fn reset(&mut self) {
        self.visit_stack.clear();
        self.visited.clear();
    }

    /// Verify whether the execution of the executor will lead to recursion.
    pub fn validate_executor(
        &mut self,
        node_id: i64,
        node_name: &'static str,
    ) -> Result<(), DBError> {
        // Check whether the maximum depth has been exceeded.
        if self.visit_stack.len() >= self.max_depth {
            return Err(DBError::Internal(format!(
                "Execution plan depth exceeds the maximum limit: Current node {} ({})",
                node_name, node_id
            )));
        }

        // Check for circular references.
        if self.visit_stack.iter().any(|(id, _)| *id == node_id) {
            let cycle_path: Vec<String> = self
                .visit_stack
                .iter()
                .map(|(id, name)| format!("{}({})", name, id))
                .collect();
            return Err(DBError::Internal(format!(
                "Detected a circular reference in the execution plan: {} -> {}({})",
                cycle_path.join(" -> "),
                node_name,
                node_id
            )));
        }

        // Push the current node onto the stack.
        self.visit_stack.push((node_id, node_name));
        self.visited.insert(node_id);

        Ok(())
    }

    /// Leave the current node (pop from the stack)
    pub fn leave_executor(&mut self) {
        self.visit_stack.pop();
    }

    /// Get the current depth
    pub fn current_depth(&self) -> usize {
        self.visit_stack.len()
    }

    /// Has this node been visited before?
    pub fn is_visited(&self, node_id: i64) -> bool {
        self.visited.contains(&node_id)
    }
}

impl Default for RecursionDetector {
    fn default() -> Self {
        Self::new(100)
    }
}

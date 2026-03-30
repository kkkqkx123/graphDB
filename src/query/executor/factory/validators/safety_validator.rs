//! Security Validator
//!
//! Responsible for verifying the security configuration of the executor.

use crate::core::error::QueryError;
use crate::storage::StorageClient;

/// Actuator safety configuration
#[derive(Debug, Clone)]
pub struct ExecutorSafetyConfig {
    /// Maximum recursion depth
    pub max_recursion_depth: usize,
    /// Maximum number of loop iterations
    pub max_loop_iterations: usize,
    /// Should recursive detection be enabled?
    pub enable_recursion_detection: bool,
    /// Maximum number of executors
    pub max_executor_count: usize,
}

impl Default for ExecutorSafetyConfig {
    fn default() -> Self {
        Self {
            max_recursion_depth: 100,
            max_loop_iterations: 10000,
            enable_recursion_detection: true,
            max_executor_count: 1000,
        }
    }
}

/// Security Validator
#[derive(Clone)]
pub struct SafetyValidator<S: StorageClient + Send + 'static> {
    config: ExecutorSafetyConfig,
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient + 'static> SafetyValidator<S> {
    /// Create a new security validator.
    pub fn new(config: ExecutorSafetyConfig) -> Self {
        Self {
            config,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Obtain the configuration.
    pub fn config(&self) -> &ExecutorSafetyConfig {
        &self.config
    }

    /// Verify the extended configuration settings.
    pub fn validate_expand_config(&self, step_limit: usize) -> Result<(), QueryError> {
        if step_limit > self.config.max_recursion_depth {
            return Err(QueryError::ExecutionError(format!(
                "Extension step limit {} exceeds maximum recursion depth {}",
                step_limit, self.config.max_recursion_depth
            )));
        }
        Ok(())
    }

    /// Verify the configuration of the shortest path.
    pub fn validate_shortest_path_config(&self, max_step: usize) -> Result<(), QueryError> {
        if max_step > self.config.max_recursion_depth {
            return Err(QueryError::ExecutionError(format!(
                "Maximum number of steps in shortest path {} over maximum recursion depth {}",
                max_step, self.config.max_recursion_depth
            )));
        }
        Ok(())
    }
}

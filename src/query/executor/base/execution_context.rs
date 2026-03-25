//! Of course! Please provide the text you would like to have translated.
//!
//! Manage the intermediate results and variables during the execution of the executor.

use std::collections::HashMap;
use std::sync::Arc;

use super::execution_result::ExecutionResult;
use crate::core::Value;
use crate::query::executor::expression::functions::global_registry_ref;
use crate::query::executor::expression::functions::OwnedFunctionRef;
use crate::query::validator::context::ExpressionAnalysisContext;

/// Of course! Please provide the text you would like to have translated.
///
/// Used for storing intermediate results and variables during the execution of actuators, and supports data transfer between actuators.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Intermediate results are stored.
    pub results: HashMap<String, ExecutionResult>,
    /// Variable storage
    pub variables: HashMap<String, crate::core::Value>,
    /// Expression context, used for sharing expression information and caching across different stages.
    pub expression_context: Arc<ExpressionAnalysisContext>,
}

impl ExecutionContext {
    /// Create a new execution context.
    pub fn new(expression_context: Arc<ExpressionAnalysisContext>) -> Self {
        Self {
            results: HashMap::new(),
            variables: HashMap::new(),
            expression_context,
        }
    }

    /// Set intermediate results
    pub fn set_result(&mut self, name: String, result: ExecutionResult) {
        self.results.insert(name, result);
    }

    /// Obtain the intermediate results.
    pub fn get_result(&self, name: &str) -> Option<&ExecutionResult> {
        self.results.get(name)
    }

    /// Setting variables
    pub fn set_variable(&mut self, name: String, value: crate::core::Value) {
        self.variables.insert(name, value);
    }

    /// Obtain the variable
    pub fn get_variable(&self, name: &str) -> Option<&crate::core::Value> {
        self.variables.get(name)
    }

    /// Obtain the context of the expression.
    pub fn expression_context(&self) -> &Arc<ExpressionAnalysisContext> {
        &self.expression_context
    }
}

impl Default for ExecutionContext {
    /// Default implementation: Creates a new ExpressionContext.
    fn default() -> Self {
        Self {
            results: HashMap::new(),
            variables: HashMap::new(),
            expression_context: Arc::new(ExpressionAnalysisContext::new()),
        }
    }
}

impl crate::query::executor::expression::evaluator::traits::ExpressionContext for ExecutionContext {
    fn get_variable(&self, name: &str) -> Option<Value> {
        self.variables.get(name).cloned()
    }

    fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    fn get_function(&self, name: &str) -> Option<OwnedFunctionRef> {
        let registry = global_registry_ref();
        registry
            .get_builtin(name)
            .map(|f| OwnedFunctionRef::Builtin(f.clone()))
            .or_else(|| {
                registry
                    .get_custom(name)
                    .map(|f| OwnedFunctionRef::Custom(f.clone()))
            })
    }
}

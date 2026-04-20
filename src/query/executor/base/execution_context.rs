//! Execution Context
//!
//! Manage the intermediate results and variables during the execution of the executor.

use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

use super::execution_result::ExecutionResult;
use crate::core::Value;
use crate::query::executor::expression::functions::global_registry_ref;
use crate::query::executor::expression::functions::OwnedFunctionRef;
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::search::SearchEngine;

/// Execution Context
///
/// Used for storing intermediate results and variables during the execution of actuators, and supports data transfer between actuators.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Intermediate results are stored.
    pub results: Arc<Mutex<HashMap<String, ExecutionResult>>>,
    /// Variable storage
    pub variables: Arc<Mutex<HashMap<String, crate::core::Value>>>,
    /// Expression context, used for sharing expression information and caching across different stages.
    pub expression_context: Arc<ExpressionAnalysisContext>,
    /// Search engine for full-text search
    pub search_engine: Option<Arc<dyn SearchEngine>>,
    /// Query parameters
    pub parameters: Arc<HashMap<String, crate::core::Value>>,
}

impl ExecutionContext {
    /// Create a new execution context.
    pub fn new(expression_context: Arc<ExpressionAnalysisContext>) -> Self {
        Self {
            results: Arc::new(Mutex::new(HashMap::new())),
            variables: Arc::new(Mutex::new(HashMap::new())),
            expression_context,
            search_engine: None,
            parameters: Arc::new(HashMap::new()),
        }
    }

    /// Create a new execution context with parameters.
    pub fn with_parameters(
        expression_context: Arc<ExpressionAnalysisContext>,
        parameters: HashMap<String, crate::core::Value>,
    ) -> Self {
        Self {
            results: Arc::new(Mutex::new(HashMap::new())),
            variables: Arc::new(Mutex::new(HashMap::new())),
            expression_context,
            search_engine: None,
            parameters: Arc::new(parameters),
        }
    }

    /// Create a new execution context with search engine.
    pub fn with_search_engine(
        expression_context: Arc<ExpressionAnalysisContext>,
        search_engine: Arc<dyn SearchEngine>,
    ) -> Self {
        Self {
            results: Arc::new(Mutex::new(HashMap::new())),
            variables: Arc::new(Mutex::new(HashMap::new())),
            expression_context,
            search_engine: Some(search_engine),
            parameters: Arc::new(HashMap::new()),
        }
    }

    /// Set intermediate results
    pub fn set_result(&self, name: String, result: ExecutionResult) {
        self.results.lock().insert(name, result);
    }

    /// Obtain the intermediate results.
    pub fn get_result(&self, name: &str) -> Option<ExecutionResult> {
        self.results.lock().get(name).cloned()
    }

    /// Setting variables
    pub fn set_variable(&self, name: String, value: crate::core::Value) {
        self.variables.lock().insert(name, value);
    }

    /// Obtain the variable
    pub fn get_variable(&self, name: &str) -> Option<crate::core::Value> {
        self.variables.lock().get(name).cloned()
    }

    /// Obtain the context of the expression.
    pub fn expression_context(&self) -> &Arc<ExpressionAnalysisContext> {
        &self.expression_context
    }

    /// Obtain the search engine.
    pub fn search_engine(&self) -> Option<&Arc<dyn SearchEngine>> {
        self.search_engine.as_ref()
    }

    /// Get query parameter
    pub fn get_param(&self, name: &str) -> Option<&crate::core::Value> {
        self.parameters.get(name)
    }

    /// Get current space ID from variables
    pub fn current_space_id(&self) -> Option<u64> {
        self.variables.lock().get("space_id").and_then(|v| match v {
            Value::Int(id) => Some(*id as u64),
            _ => None,
        })
    }

    /// Set current space ID
    pub fn set_space_id(&self, space_id: u64) {
        self.variables
            .lock()
            .insert("space_id".to_string(), Value::BigInt(space_id as i64));
    }
}

impl Default for ExecutionContext {
    /// Default implementation: Creates a new ExpressionContext.
    fn default() -> Self {
        Self {
            results: Arc::new(Mutex::new(HashMap::new())),
            variables: Arc::new(Mutex::new(HashMap::new())),
            expression_context: Arc::new(ExpressionAnalysisContext::new()),
            search_engine: None,
            parameters: Arc::new(HashMap::new()),
        }
    }
}

impl crate::query::executor::expression::evaluator::traits::ExpressionContext for ExecutionContext {
    fn get_variable(&self, name: &str) -> Option<Value> {
        self.variables.lock().get(name).cloned()
    }

    fn set_variable(&mut self, name: String, value: Value) {
        self.variables.lock().insert(name, value);
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

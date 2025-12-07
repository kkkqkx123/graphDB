use std::collections::HashMap;
use crate::core::Value;

/// Error type for expression evaluation
#[derive(Debug, Clone, PartialEq)]
pub enum EvaluationError {
    InvalidOperation(String),
    TypeError(String),
    UndefinedVariable(String),
    DivisionByZero,
    Other(String),
}

impl std::fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvaluationError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            EvaluationError::TypeError(msg) => write!(f, "Type error: {}", msg),
            EvaluationError::UndefinedVariable(var) => write!(f, "Undefined variable: {}", var),
            EvaluationError::DivisionByZero => write!(f, "Division by zero"),
            EvaluationError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for EvaluationError {}

/// The expression evaluation context trait
pub trait ExpressionContext {
    fn get_variable(&self, name: &str) -> Result<Value, EvaluationError>;
}

/// A default implementation of the expression context
#[derive(Default, Debug)]
pub struct DefaultExpressionContext {
    variables: HashMap<String, Value>,
}

impl DefaultExpressionContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn get_variable(&self, name: &str) -> Result<Value, EvaluationError> {
        self.variables.get(name).cloned()
            .ok_or_else(|| EvaluationError::UndefinedVariable(name.to_string()))
    }
}

impl ExpressionContext for DefaultExpressionContext {
    fn get_variable(&self, name: &str) -> Result<Value, EvaluationError> {
        self.get_variable(name)
    }
}
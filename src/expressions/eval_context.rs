use std::collections::HashMap;
use crate::expressions::Expression;
use crate::core::Value;

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

// 从base模块导入EvaluationError，避免重复定义
pub use crate::expressions::base::EvaluationError;
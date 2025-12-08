/// Expression evaluation error
#[derive(Debug, thiserror::Error)]
pub enum ExpressionError {
    #[error("Type error: {0}")]
    TypeError(String),
    #[error("Property not found: {0}")]
    PropertyNotFound(String),
    #[error("Function error: {0}")]
    FunctionError(String),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

use crate::core::{Value, NullType, Vertex, Edge, DateValue, TimeValue, DateTimeValue, GeographyValue, DurationValue};

/// Defines different types of unary operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not,
    Increment,
    Decrement,
}

/// Defines different types of binary operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BinaryOp {
    // Arithmetic operations
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Relational operations
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // Logical operations
    And,
    Or,
    Xor,
    // Other operations
    In,
    NotIn,
    Subscript,
    Attribute,
    Contains,
    StartsWith,
    EndsWith,
}

/// The core Expression enum representing all possible expression types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    /// Constant value expression
    Constant(Value),
    
    /// Unary operation: op operand
    Unary {
        op: UnaryOp,
        operand: Box<Expression>,
    },
    
    /// Binary operation: left op right
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
}

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

impl Expression {
    /// Evaluate the expression within the given context
    pub fn eval(&self, context: &dyn ExpressionContext) -> Result<Value, EvaluationError> {
        match self {
            Expression::Constant(value) => Ok(value.clone()),
            
            Expression::Unary { op, operand } => {
                let value = operand.eval(context)?;
                eval_unary_op(*op, value)
            },
            
            Expression::Binary { op, left, right } => {
                let left_val = left.eval(context)?;
                let right_val = right.eval(context)?;
                eval_binary_op(*op, left_val, right_val)
            },
        }
    }
}

/// Evaluate a unary operation
fn eval_unary_op(op: UnaryOp, operand: Value) -> Result<Value, EvaluationError> {
    match op {
        UnaryOp::Plus => match operand {
            Value::Int(i) => Ok(Value::Int(i)),
            Value::Float(f) => Ok(Value::Float(f)),
            _ => Err(EvaluationError::TypeError(
                format!("Unary plus not supported for {:?}", operand)
            )),
        },
        
        UnaryOp::Minus => match operand {
            Value::Int(i) => Ok(Value::Int(-i)),
            Value::Float(f) => Ok(Value::Float(-f)),
            _ => Err(EvaluationError::TypeError(
                format!("Unary minus not supported for {:?}", operand)
            )),
        },
        
        UnaryOp::Not => match operand {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            Value::Null(_) => Ok(Value::Bool(true)), // null is considered "falsy", so !null = true
            Value::Int(i) => Ok(Value::Bool(i == 0)),
            Value::Float(f) => Ok(Value::Bool(f == 0.0)),
            Value::String(s) => Ok(Value::Bool(s.is_empty())),
            Value::List(l) => Ok(Value::Bool(l.is_empty())),
            _ => Err(EvaluationError::TypeError(
                format!("Unary not not supported for {:?}", operand)
            )),
        },
        
        UnaryOp::Increment => match operand {
            Value::Int(i) => Ok(Value::Int(i + 1)),
            Value::Float(f) => Ok(Value::Float(f + 1.0)),
            _ => Err(EvaluationError::TypeError(
                format!("Increment not supported for {:?}", operand)
            )),
        },
        
        UnaryOp::Decrement => match operand {
            Value::Int(i) => Ok(Value::Int(i - 1)),
            Value::Float(f) => Ok(Value::Float(f - 1.0)),
            _ => Err(EvaluationError::TypeError(
                format!("Decrement not supported for {:?}", operand)
            )),
        },
    }
}

/// Evaluate a binary operation
fn eval_binary_op(op: BinaryOp, left: Value, right: Value) -> Result<Value, EvaluationError> {
    match op {
        // Arithmetic operations
        BinaryOp::Add => match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + b as f64)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            _ => Err(EvaluationError::TypeError(
                format!("Addition not supported between {:?} and {:?}", left, right)
            )),
        },
        
        BinaryOp::Sub => match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 - b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - b as f64)),
            _ => Err(EvaluationError::TypeError(
                format!("Subtraction not supported between {:?} and {:?}", left, right)
            )),
        },
        
        BinaryOp::Mul => match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 * b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * b as f64)),
            _ => Err(EvaluationError::TypeError(
                format!("Multiplication not supported between {:?} and {:?}", left, right)
            )),
        },
        
        BinaryOp::Div => match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                if b == 0 {
                    Err(EvaluationError::DivisionByZero)
                } else {
                    Ok(Value::Float(a as f64 / b as f64))
                }
            },
            (Value::Float(a), Value::Float(b)) => {
                if b == 0.0 {
                    Err(EvaluationError::DivisionByZero)
                } else {
                    Ok(Value::Float(a / b))
                }
            },
            (Value::Int(a), Value::Float(b)) => {
                if b == 0.0 {
                    Err(EvaluationError::DivisionByZero)
                } else {
                    Ok(Value::Float(a as f64 / b))
                }
            },
            (Value::Float(a), Value::Int(b)) => {
                if b == 0 {
                    Err(EvaluationError::DivisionByZero)
                } else {
                    Ok(Value::Float(a / b as f64))
                }
            },
            _ => Err(EvaluationError::TypeError(
                format!("Division not supported between {:?} and {:?}", left, right)
            )),
        },
        
        BinaryOp::Mod => match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                if b == 0 {
                    Err(EvaluationError::DivisionByZero)
                } else {
                    Ok(Value::Int(a % b))
                }
            },
            (Value::Float(a), Value::Float(b)) => {
                if b == 0.0 {
                    Err(EvaluationError::DivisionByZero)
                } else {
                    Ok(Value::Float(a % b))
                }
            },
            (Value::Int(a), Value::Float(b)) => {
                if b == 0.0 {
                    Err(EvaluationError::DivisionByZero)
                } else {
                    Ok(Value::Float(a as f64 % b))
                }
            },
            (Value::Float(a), Value::Int(b)) => {
                if b == 0 {
                    Err(EvaluationError::DivisionByZero)
                } else {
                    Ok(Value::Float(a % b as f64))
                }
            },
            _ => Err(EvaluationError::TypeError(
                format!("Modulo not supported between {:?} and {:?}", left, right)
            )),
        },
        
        // Relational operations
        BinaryOp::Eq => Ok(Value::Bool(left == right)),
        BinaryOp::Ne => Ok(Value::Bool(left != right)),
        BinaryOp::Lt => match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((a as f64) < b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(a < (b as f64))),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a < b)),
            _ => Err(EvaluationError::TypeError(
                format!("Less than not supported between {:?} and {:?}", left, right)
            )),
        },
        BinaryOp::Le => match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((a as f64) <= b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(a <= (b as f64))),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a <= b)),
            _ => Err(EvaluationError::TypeError(
                format!("Less than or equal not supported between {:?} and {:?}", left, right)
            )),
        },
        BinaryOp::Gt => match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((a as f64) > b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(a > (b as f64))),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a > b)),
            _ => Err(EvaluationError::TypeError(
                format!("Greater than not supported between {:?} and {:?}", left, right)
            )),
        },
        BinaryOp::Ge => match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((a as f64) >= b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(a >= (b as f64))),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a >= b)),
            _ => Err(EvaluationError::TypeError(
                format!("Greater than or equal not supported between {:?} and {:?}", left, right)
            )),
        },
        
        // Logical operations
        BinaryOp::And => match (left, right) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
            _ => Err(EvaluationError::TypeError(
                format!("Logical and not supported between {:?} and {:?}", left, right)
            )),
        },
        BinaryOp::Or => match (left, right) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
            _ => Err(EvaluationError::TypeError(
                format!("Logical or not supported between {:?} and {:?}", left, right)
            )),
        },
        BinaryOp::Xor => match (left, right) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a ^ b)),
            _ => Err(EvaluationError::TypeError(
                format!("Logical xor not supported between {:?} and {:?}", left, right)
            )),
        },
        
        // Special operations - placeholder implementations
        BinaryOp::In => Err(EvaluationError::Other("IN operation not yet implemented".to_string())),
        BinaryOp::NotIn => Err(EvaluationError::Other("NOT IN operation not yet implemented".to_string())),
        BinaryOp::Subscript => Err(EvaluationError::Other("Subscript operation not yet implemented".to_string())),
        BinaryOp::Attribute => Err(EvaluationError::Other("Attribute operation not yet implemented".to_string())),
        BinaryOp::Contains => Err(EvaluationError::Other("Contains operation not yet implemented".to_string())),
        BinaryOp::StartsWith => Err(EvaluationError::Other("Starts with operation not yet implemented".to_string())),
        BinaryOp::EndsWith => Err(EvaluationError::Other("Ends with operation not yet implemented".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_expression() {
        let constant_expr = Expression::Constant(Value::Int(42));
        let context = DefaultExpressionContext::new();
        let result = constant_expr.eval(&context).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_unary_plus() {
        let expr = Expression::Unary {
            op: UnaryOp::Plus,
            operand: Box::new(Expression::Constant(Value::Int(-5))),
        };
        let context = DefaultExpressionContext::new();
        let result = expr.eval(&context).unwrap();
        assert_eq!(result, Value::Int(-5));
    }

    #[test]
    fn test_unary_minus() {
        let expr = Expression::Unary {
            op: UnaryOp::Minus,
            operand: Box::new(Expression::Constant(Value::Int(5))),
        };
        let context = DefaultExpressionContext::new();
        let result = expr.eval(&context).unwrap();
        assert_eq!(result, Value::Int(-5));
    }

    #[test]
    fn test_binary_addition() {
        let expr = Expression::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expression::Constant(Value::Int(10))),
            right: Box::new(Expression::Constant(Value::Int(20))),
        };
        let context = DefaultExpressionContext::new();
        let result = expr.eval(&context).unwrap();
        assert_eq!(result, Value::Int(30));
    }

    #[test]
    fn test_binary_multiplication() {
        let expr = Expression::Binary {
            op: BinaryOp::Mul,
            left: Box::new(Expression::Constant(Value::Int(6))),
            right: Box::new(Expression::Constant(Value::Int(7))),
        };
        let context = DefaultExpressionContext::new();
        let result = expr.eval(&context).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_binary_equality() {
        let expr = Expression::Binary {
            op: BinaryOp::Eq,
            left: Box::new(Expression::Constant(Value::Int(5))),
            right: Box::new(Expression::Constant(Value::Int(5))),
        };
        let context = DefaultExpressionContext::new();
        let result = expr.eval(&context).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_division_by_zero() {
        let expr = Expression::Binary {
            op: BinaryOp::Div,
            left: Box::new(Expression::Constant(Value::Int(10))),
            right: Box::new(Expression::Constant(Value::Int(0))),
        };
        let context = DefaultExpressionContext::new();
        let result = expr.eval(&context);
        assert!(matches!(result, Err(EvaluationError::DivisionByZero)));
    }
}
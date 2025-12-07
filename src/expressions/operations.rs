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

/// Evaluate a unary operation
pub fn eval_unary_op(op: UnaryOp, operand: Value) -> Result<Value, super::base::EvaluationError> {
    use super::base::EvaluationError;
    
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
pub fn eval_binary_op(op: BinaryOp, left: Value, right: Value) -> Result<Value, super::base::EvaluationError> {
    use super::base::EvaluationError;
    use std::collections::HashMap;
    
    match op {
        // Arithmetic operations
        BinaryOp::Add => match (&left, &right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(*a + *b as f64)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            _ => Err(EvaluationError::TypeError(
                format!("Addition not supported between {:?} and {:?}", left, right)
            )),
        },

        BinaryOp::Sub => match (&left, &right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(*a - *b as f64)),
            _ => Err(EvaluationError::TypeError(
                format!("Subtraction not supported between {:?} and {:?}", left, right)
            )),
        },

        BinaryOp::Mul => match (&left, &right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(*a * *b as f64)),
            _ => Err(EvaluationError::TypeError(
                format!("Multiplication not supported between {:?} and {:?}", left, right)
            )),
        },

        BinaryOp::Div => match (&left, &right) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(EvaluationError::DivisionByZero)
                } else {
                    Ok(Value::Float(*a as f64 / *b as f64))
                }
            },
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err(EvaluationError::DivisionByZero)
                } else {
                    Ok(Value::Float(*a / *b))
                }
            },
            (Value::Int(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err(EvaluationError::DivisionByZero)
                } else {
                    Ok(Value::Float(*a as f64 / *b))
                }
            },
            (Value::Float(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(EvaluationError::DivisionByZero)
                } else {
                    Ok(Value::Float(*a / *b as f64))
                }
            },
            _ => Err(EvaluationError::TypeError(
                format!("Division not supported between {:?} and {:?}", left, right)
            )),
        },

        BinaryOp::Mod => match (&left, &right) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(EvaluationError::DivisionByZero)
                } else {
                    Ok(Value::Int(*a % *b))
                }
            },
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err(EvaluationError::DivisionByZero)
                } else {
                    Ok(Value::Float(*a % *b))
                }
            },
            (Value::Int(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err(EvaluationError::DivisionByZero)
                } else {
                    Ok(Value::Float(*a as f64 % *b))
                }
            },
            (Value::Float(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(EvaluationError::DivisionByZero)
                } else {
                    Ok(Value::Float(*a % *b as f64))
                }
            },
            _ => Err(EvaluationError::TypeError(
                format!("Modulo not supported between {:?} and {:?}", left, right)
            )),
        },

        // Relational operations
        BinaryOp::Eq => Ok(Value::Bool(left == right)),
        BinaryOp::Ne => Ok(Value::Bool(left != right)),
        BinaryOp::Lt => match (&left, &right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(*a < *b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(*a < *b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) < *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a < (*b as f64))),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a < b)),
            _ => Err(EvaluationError::TypeError(
                format!("Less than not supported between {:?} and {:?}", left, right)
            )),
        },
        BinaryOp::Le => match (&left, &right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(*a <= *b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(*a <= *b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) <= *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a <= (*b as f64))),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a <= b)),
            _ => Err(EvaluationError::TypeError(
                format!("Less than or equal not supported between {:?} and {:?}", left, right)
            )),
        },
        BinaryOp::Gt => match (&left, &right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(*a > *b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(*a > *b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) > *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a > (*b as f64))),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a > b)),
            _ => Err(EvaluationError::TypeError(
                format!("Greater than not supported between {:?} and {:?}", left, right)
            )),
        },
        BinaryOp::Ge => match (&left, &right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(*a >= *b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(*a >= *b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) >= *b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a >= (*b as f64))),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a >= b)),
            _ => Err(EvaluationError::TypeError(
                format!("Greater than or equal not supported between {:?} and {:?}", left, right)
            )),
        },

        // Logical operations
        BinaryOp::And => match (&left, &right) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a && *b)),
            _ => Err(EvaluationError::TypeError(
                format!("Logical and not supported between {:?} and {:?}", left, right)
            )),
        },
        BinaryOp::Or => match (&left, &right) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a || *b)),
            _ => Err(EvaluationError::TypeError(
                format!("Logical or not supported between {:?} and {:?}", left, right)
            )),
        },
        BinaryOp::Xor => match (&left, &right) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a ^ *b)),
            _ => Err(EvaluationError::TypeError(
                format!("Logical xor not supported between {:?} and {:?}", left, right)
            )),
        },

        // Special operations
        BinaryOp::In => match (&left, &right) {
            (value, Value::List(list)) => Ok(Value::Bool(list.contains(value))),
            (value, Value::Set(set)) => Ok(Value::Bool(set.contains(value))),
            _ => Err(EvaluationError::TypeError(
                format!("IN operation not supported for {:?} and {:?}", left, right)
            )),
        },
        BinaryOp::NotIn => match (&left, &right) {
            (Value::List(list), value) => Ok(Value::Bool(!list.contains(value))),
            (Value::Set(set), value) => Ok(Value::Bool(!set.contains(value))),
            _ => Err(EvaluationError::TypeError(
                format!("NOT IN operation not supported for {:?} and {:?}", left, right)
            )),
        },
        BinaryOp::Subscript => match (&left, &right) {
            (Value::List(list), Value::Int(index)) => {
                let idx = if *index >= 0 { *index as usize } else { list.len() - (-*index as usize) };
                list.get(idx)
                    .cloned()
                    .ok_or_else(|| EvaluationError::Other("Index out of bounds".to_string()))
            },
            (Value::String(s), Value::Int(index)) => {
                let idx = if *index >= 0 { *index as usize } else { s.len() - (-*index as usize) };
                s.chars().nth(idx)
                    .map(|c| Value::String(c.to_string()))
                    .ok_or_else(|| EvaluationError::Other("Index out of bounds".to_string()))
            },
            (Value::Map(map), Value::String(key)) => {
                map.get(key)
                    .cloned()
                    .ok_or_else(|| EvaluationError::Other("Key not found".to_string()))
            },
            _ => Err(EvaluationError::TypeError(
                format!("Subscript operation not supported for {:?} and {:?}", left, right)
            )),
        },
        BinaryOp::Attribute => Err(EvaluationError::Other("Attribute operation not yet implemented".to_string())),
        BinaryOp::Contains => match (&left, &right) {
            (Value::String(haystack), Value::String(needle)) => Ok(Value::Bool(haystack.contains(needle))),
            _ => Err(EvaluationError::TypeError(
                format!("Contains operation not supported for {:?} and {:?}", left, right)
            )),
        },
        BinaryOp::StartsWith => match (&left, &right) {
            (Value::String(str), Value::String(prefix)) => Ok(Value::Bool(str.starts_with(prefix))),
            _ => Err(EvaluationError::TypeError(
                format!("Starts with operation not supported for {:?} and {:?}", left, right)
            )),
        },
        BinaryOp::EndsWith => match (&left, &right) {
            (Value::String(str), Value::String(suffix)) => Ok(Value::Bool(str.ends_with(suffix))),
            _ => Err(EvaluationError::TypeError(
                format!("Ends with operation not supported for {:?} and {:?}", left, right)
            )),
        },
    }
}
use crate::core::ExpressionError;
use crate::core::{NullType, Value};
use crate::core::{Expression, ExpressionContext};
use serde::{Deserialize, Serialize};

/// Binary operators for expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BinaryOperator {
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

/// 评估二元操作表达式
pub fn evaluate_binary_op(
    left: &Expression,
    op: &BinaryOperator,
    right: &Expression,
    context: &dyn ExpressionContext,
) -> Result<Value, ExpressionError> {
    let evaluator = super::evaluator::ExpressionEvaluator;
    let left_val = evaluator.evaluate(left, context)?;
    let right_val = evaluator.evaluate(right, context)?;

    match op {
        BinaryOperator::Add => add_values(left_val, right_val),
        BinaryOperator::Sub => sub_values(left_val, right_val),
        BinaryOperator::Mul => mul_values(left_val, right_val),
        BinaryOperator::Div => div_values(left_val, right_val),
        BinaryOperator::Mod => mod_values(left_val, right_val),
        BinaryOperator::Eq => Ok(Value::Bool(left_val == right_val)),
        BinaryOperator::Ne => Ok(Value::Bool(left_val != right_val)),
        BinaryOperator::Lt => cmp_values(left_val, right_val, |a, b| a.less_than(&b)),
        BinaryOperator::Le => cmp_values(left_val, right_val, |a, b| a.less_than_equal(&b)),
        BinaryOperator::Gt => cmp_values(left_val, right_val, |a, b| a.greater_than(&b)),
        BinaryOperator::Ge => cmp_values(left_val, right_val, |a, b| a.greater_than_equal(&b)),
        BinaryOperator::And => and_values(left_val, right_val),
        BinaryOperator::Or => or_values(left_val, right_val),
        BinaryOperator::Xor => xor_values(left_val, right_val),
        BinaryOperator::In => in_values(left_val, right_val),
        BinaryOperator::NotIn => not_in_values(left_val, right_val),
        BinaryOperator::Subscript => subscript_values(left_val, right_val),
        BinaryOperator::Attribute => attribute_values(left_val, right_val),
        BinaryOperator::Contains => contains_values(left_val, right_val),
        BinaryOperator::StartsWith => starts_with_values(left_val, right_val),
        BinaryOperator::EndsWith => ends_with_values(left_val, right_val),
    }
}

pub fn add_values(left: Value, right: Value) -> Result<Value, ExpressionError> {
    match (left, right) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + b as f64)),
        (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 + b)),
        (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
        (Value::String(a), Value::Int(b)) => Ok(Value::String(format!("{}{}", a, b))),
        (Value::Int(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
        // Add more combinations as needed
        _ => Err(ExpressionError::TypeError(
            "Cannot add these value types".to_string(),
        )),
    }
}

pub fn sub_values(left: Value, right: Value) -> Result<Value, ExpressionError> {
    match (left, right) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - b as f64)),
        (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 - b)),
        _ => Err(ExpressionError::TypeError(
            "Cannot subtract these value types".to_string(),
        )),
    }
}

pub fn mul_values(left: Value, right: Value) -> Result<Value, ExpressionError> {
    match (left, right) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * b as f64)),
        (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 * b)),
        _ => Err(ExpressionError::TypeError(
            "Cannot multiply these value types".to_string(),
        )),
    }
}

pub fn div_values(left: Value, right: Value) -> Result<Value, ExpressionError> {
    match (left, right) {
        (Value::Int(a), Value::Int(b)) if b != 0 => Ok(Value::Int(a / b)),
        (Value::Float(a), Value::Float(b)) if b != 0.0 => Ok(Value::Float(a / b)),
        (Value::Float(a), Value::Int(b)) if b != 0 => Ok(Value::Float(a / b as f64)),
        (Value::Int(a), Value::Float(b)) if b != 0.0 => Ok(Value::Float(a as f64 / b)),
        _ => Err(ExpressionError::TypeError(
            "Cannot divide these value types or division by zero".to_string(),
        )),
    }
}

pub fn cmp_values<F>(left: Value, right: Value, cmp_fn: F) -> Result<Value, ExpressionError>
where
    F: Fn(Value, Value) -> bool,
{
    Ok(Value::Bool(cmp_fn(left, right)))
}

pub fn and_values(left: Value, right: Value) -> Result<Value, ExpressionError> {
    let left_bool = value_to_bool(&left);
    let right_bool = value_to_bool(&right);
    Ok(Value::Bool(left_bool && right_bool))
}

pub fn or_values(left: Value, right: Value) -> Result<Value, ExpressionError> {
    let left_bool = value_to_bool(&left);
    let right_bool = value_to_bool(&right);
    Ok(Value::Bool(left_bool || right_bool))
}

pub fn mod_values(left: Value, right: Value) -> Result<Value, ExpressionError> {
    match (left, right) {
        (Value::Int(a), Value::Int(b)) => {
            if b == 0 {
                return Err(ExpressionError::InvalidOperation(
                    "Division by zero".to_string(),
                ));
            }
            Ok(Value::Int(a % b))
        }
        (Value::Float(a), Value::Float(b)) => {
            if b == 0.0 {
                return Err(ExpressionError::InvalidOperation(
                    "Division by zero".to_string(),
                ));
            }
            Ok(Value::Float(a % b))
        }
        (Value::Int(a), Value::Float(b)) => {
            if b == 0.0 {
                return Err(ExpressionError::InvalidOperation(
                    "Division by zero".to_string(),
                ));
            }
            Ok(Value::Float((a as f64) % b))
        }
        (Value::Float(a), Value::Int(b)) => {
            if b == 0 {
                return Err(ExpressionError::InvalidOperation(
                    "Division by zero".to_string(),
                ));
            }
            Ok(Value::Float(a % (b as f64)))
        }
        _ => Err(ExpressionError::TypeError(
            "Cannot perform mod operation on these value types".to_string(),
        )),
    }
}

pub fn xor_values(left: Value, right: Value) -> Result<Value, ExpressionError> {
    let left_bool = value_to_bool(&left);
    let right_bool = value_to_bool(&right);
    Ok(Value::Bool(left_bool ^ right_bool)) // XOR operation
}

pub fn in_values(left: Value, right: Value) -> Result<Value, ExpressionError> {
    match right {
        Value::List(items) => {
            let found = items.iter().any(|item| *item == left);
            Ok(Value::Bool(found))
        }
        Value::Set(items) => Ok(Value::Bool(items.contains(&left))),
        Value::Map(items) => {
            if let Value::String(key) = &left {
                Ok(Value::Bool(items.contains_key(key)))
            } else {
                Err(ExpressionError::TypeError(
                    "Key for 'in' operation on map must be a string".to_string(),
                ))
            }
        }
        _ => Err(ExpressionError::TypeError(
            "Right operand of 'in' must be a list, set, or map".to_string(),
        )),
    }
}

pub fn not_in_values(left: Value, right: Value) -> Result<Value, ExpressionError> {
    match in_values(left, right) {
        Ok(Value::Bool(b)) => Ok(Value::Bool(!b)),
        Ok(_) => Err(ExpressionError::TypeError(
            "in_values should return boolean".to_string(),
        )),
        Err(e) => Err(e),
    }
}

pub fn subscript_values(collection: Value, index: Value) -> Result<Value, ExpressionError> {
    match collection {
        Value::List(items) => {
            if let Value::Int(i) = index {
                if i >= 0 && (i as usize) < items.len() {
                    Ok(items[i as usize].clone())
                } else {
                    Err(ExpressionError::InvalidOperation(
                        "List index out of bounds".to_string(),
                    ))
                }
            } else {
                Err(ExpressionError::TypeError(
                    "List index must be an integer".to_string(),
                ))
            }
        }
        Value::Map(items) => {
            if let Value::String(key) = index {
                match items.get(&key) {
                    Some(value) => Ok(value.clone()),
                    None => Ok(Value::Null(NullType::Null)),
                }
            } else {
                Err(ExpressionError::TypeError(
                    "Map key must be a string".to_string(),
                ))
            }
        }
        _ => Err(ExpressionError::TypeError(
            "Subscript operation requires a list or map".to_string(),
        )),
    }
}

pub fn attribute_values(left: Value, right: Value) -> Result<Value, ExpressionError> {
    // For simplicity, treat this like a subscript operation for now
    // In a real system, this would access object properties
    match (&left, &right) {
        (Value::Map(m), Value::String(key)) => match m.get(key) {
            Some(value) => Ok(value.clone()),
            None => Ok(Value::Null(NullType::Null)),
        },
        _ => Err(ExpressionError::TypeError(
            "Attribute access requires a map and string key".to_string(),
        )),
    }
}

pub fn contains_values(left: Value, right: Value) -> Result<Value, ExpressionError> {
    // Check if 'left' contains 'right'
    match (&left, &right) {
        (Value::List(items), item) => Ok(Value::Bool(items.contains(item))),
        (Value::Set(items), item) => Ok(Value::Bool(items.contains(item))),
        (Value::String(s), Value::String(substring)) => Ok(Value::Bool(s.contains(substring))),
        _ => Err(ExpressionError::TypeError(
            "Contains operation not supported for these types".to_string(),
        )),
    }
}

pub fn starts_with_values(left: Value, right: Value) -> Result<Value, ExpressionError> {
    match (&left, &right) {
        (Value::String(s), Value::String(prefix)) => Ok(Value::Bool(s.starts_with(prefix))),
        _ => Err(ExpressionError::TypeError(
            "Starts with operation requires string operands".to_string(),
        )),
    }
}

pub fn ends_with_values(left: Value, right: Value) -> Result<Value, ExpressionError> {
    match (&left, &right) {
        (Value::String(s), Value::String(suffix)) => Ok(Value::Bool(s.ends_with(suffix))),
        _ => Err(ExpressionError::TypeError(
            "Ends with operation requires string operands".to_string(),
        )),
    }
}

pub fn value_to_bool(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::Int(n) => *n != 0,
        Value::Float(f) => *f != 0.0 && !f.is_nan(),
        Value::String(s) => !s.is_empty(),
        Value::Null(_) => false,
        Value::Empty => false,
        _ => true, // Default to true for other types
    }
}

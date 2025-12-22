use crate::core::{NullType, Value};
use crate::core::{Expression, ExpressionError};
use crate::core::context::expression::default_context::ExpressionContextCore;
use crate::core::types::operators::BinaryOperator as CoreBinaryOperator;
use crate::expression::operators_ext::ExtendedBinaryOperator;
use serde::{Deserialize, Serialize};

/// 为了向后兼容，保留原有的BinaryOperator类型别名
/// 
/// 注意：新代码应该使用ExtendedBinaryOperator
#[deprecated(note = "使用 ExtendedBinaryOperator 替代")]
pub type BinaryOperator = ExtendedBinaryOperator;

/// 评估扩展二元操作表达式
pub fn evaluate_binary_op(
    left: &Expression,
    op: &ExtendedBinaryOperator,
    right: &Expression,
    context: &dyn ExpressionContextCore,
) -> Result<Value, ExpressionError> {
    let evaluator = crate::core::evaluator::ExpressionEvaluator;
    let left_val = evaluator.evaluate(left, context)?;
    let right_val = evaluator.evaluate(right, context)?;

    match op {
        // Core操作符委托给Core求值器
        ExtendedBinaryOperator::Core(core_op) => {
            evaluate_core_binary_op(&left_val, core_op, &right_val)
        }
        
        // 扩展操作符使用Expression模块的实现
        ExtendedBinaryOperator::Xor => xor_values(left_val, right_val),
        ExtendedBinaryOperator::NotIn => not_in_values(left_val, right_val),
        ExtendedBinaryOperator::Subscript => subscript_values(left_val, right_val),
        ExtendedBinaryOperator::Attribute => attribute_values(left_val, right_val),
        ExtendedBinaryOperator::Contains => contains_values(left_val, right_val),
        ExtendedBinaryOperator::StartsWith => starts_with_values(left_val, right_val),
        ExtendedBinaryOperator::EndsWith => ends_with_values(left_val, right_val),
    }
}

/// 评估Core二元操作符
fn evaluate_core_binary_op(
    left: &Value,
    op: &CoreBinaryOperator,
    right: &Value,
) -> Result<Value, ExpressionError> {
    match op {
        CoreBinaryOperator::Add => add_values(left.clone(), right.clone()),
        CoreBinaryOperator::Subtract => sub_values(left.clone(), right.clone()),
        CoreBinaryOperator::Multiply => mul_values(left.clone(), right.clone()),
        CoreBinaryOperator::Divide => div_values(left.clone(), right.clone()),
        CoreBinaryOperator::Modulo => mod_values(left.clone(), right.clone()),
        CoreBinaryOperator::Equal => Ok(Value::Bool(left == right)),
        CoreBinaryOperator::NotEqual => Ok(Value::Bool(left != right)),
        CoreBinaryOperator::LessThan => cmp_values(left.clone(), right.clone(), |a, b| a.less_than(&b)),
        CoreBinaryOperator::LessThanOrEqual => cmp_values(left.clone(), right.clone(), |a, b| a.less_than_equal(&b)),
        CoreBinaryOperator::GreaterThan => cmp_values(left.clone(), right.clone(), |a, b| a.greater_than(&b)),
        CoreBinaryOperator::GreaterThanOrEqual => cmp_values(left.clone(), right.clone(), |a, b| a.greater_than_equal(&b)),
        CoreBinaryOperator::And => and_values(left.clone(), right.clone()),
        CoreBinaryOperator::Or => or_values(left.clone(), right.clone()),
        CoreBinaryOperator::StringConcat => string_concat_values(left.clone(), right.clone()),
        CoreBinaryOperator::Like => like_values(left.clone(), right.clone()),
        CoreBinaryOperator::In => in_values(left.clone(), right.clone()),
        CoreBinaryOperator::Union => union_values(left.clone(), right.clone()),
        CoreBinaryOperator::Intersect => intersect_values(left.clone(), right.clone()),
        CoreBinaryOperator::Except => except_values(left.clone(), right.clone()),
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
        _ => Err(ExpressionError::type_error(
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
        _ => Err(ExpressionError::type_error(
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
        _ => Err(ExpressionError::type_error(
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
        _ => Err(ExpressionError::type_error(
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
                return Err(ExpressionError::invalid_operation(
                    "Division by zero".to_string(),
                ));
            }
            Ok(Value::Int(a % b))
        }
        (Value::Float(a), Value::Float(b)) => {
            if b == 0.0 {
                return Err(ExpressionError::invalid_operation(
                    "Division by zero".to_string(),
                ));
            }
            Ok(Value::Float(a % b))
        }
        (Value::Int(a), Value::Float(b)) => {
            if b == 0.0 {
                return Err(ExpressionError::invalid_operation(
                    "Division by zero".to_string(),
                ));
            }
            Ok(Value::Float((a as f64) % b))
        }
        (Value::Float(a), Value::Int(b)) => {
            if b == 0 {
                return Err(ExpressionError::invalid_operation(
                    "Division by zero".to_string(),
                ));
            }
            Ok(Value::Float(a % (b as f64)))
        }
        _ => Err(ExpressionError::type_error(
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
                Err(ExpressionError::type_error(
                    "Key for 'in' operation on map must be a string".to_string(),
                ))
            }
        }
        _ => Err(ExpressionError::type_error(
            "Right operand of 'in' must be a list, set, or map".to_string(),
        )),
    }
}

pub fn not_in_values(left: Value, right: Value) -> Result<Value, ExpressionError> {
    match in_values(left, right) {
        Ok(Value::Bool(b)) => Ok(Value::Bool(!b)),
        Ok(_) => Err(ExpressionError::type_error(
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
                    Err(ExpressionError::invalid_operation(
                        "List index out of bounds".to_string(),
                    ))
                }
            } else {
                Err(ExpressionError::type_error(
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
                Err(ExpressionError::type_error(
                    "Map key must be a string".to_string(),
                ))
            }
        }
        _ => Err(ExpressionError::type_error(
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
        _ => Err(ExpressionError::type_error(
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
        _ => Err(ExpressionError::type_error(
            "Contains operation not supported for these types".to_string(),
        )),
    }
}

pub fn starts_with_values(left: Value, right: Value) -> Result<Value, ExpressionError> {
    match (&left, &right) {
        (Value::String(s), Value::String(prefix)) => Ok(Value::Bool(s.starts_with(prefix))),
        _ => Err(ExpressionError::type_error(
            "Starts with operation requires string operands".to_string(),
        )),
    }
}

pub fn ends_with_values(left: Value, right: Value) -> Result<Value, ExpressionError> {
    match (&left, &right) {
        (Value::String(s), Value::String(suffix)) => Ok(Value::Bool(s.ends_with(suffix))),
        _ => Err(ExpressionError::type_error(
            "Ends with operation requires string operands".to_string(),
        )),
    }
}

/// 字符串连接操作
pub fn string_concat_values(left: Value, right: Value) -> Result<Value, ExpressionError> {
    match (left, right) {
        (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
        (Value::String(a), Value::Int(b)) => Ok(Value::String(format!("{}{}", a, b))),
        (Value::Int(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
        (Value::Float(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
        (Value::String(a), Value::Float(b)) => Ok(Value::String(format!("{}{}", a, b))),
        _ => Err(ExpressionError::type_error(
            "String concatenation requires string operands".to_string(),
        )),
    }
}

/// LIKE操作
pub fn like_values(left: Value, right: Value) -> Result<Value, ExpressionError> {
    match (&left, &right) {
        (Value::String(s), Value::String(pattern)) => {
            // 简单的LIKE实现，支持%通配符
            let regex_pattern = pattern.replace('%', ".*");
            match regex::Regex::new(&format!("^{}$", regex_pattern)) {
                Ok(re) => Ok(Value::Bool(re.is_match(s))),
                Err(_) => Err(ExpressionError::invalid_operation(
                    "Invalid LIKE pattern".to_string(),
                )),
            }
        }
        _ => Err(ExpressionError::type_error(
            "LIKE operation requires string operands".to_string(),
        )),
    }
}

/// 集合并集操作
pub fn union_values(left: Value, right: Value) -> Result<Value, ExpressionError> {
    match (left, right) {
        (Value::List(mut a), Value::List(b)) => {
            a.extend(b);
            Ok(Value::List(a))
        }
        (Value::Set(mut a), Value::Set(b)) => {
            for item in b {
                a.insert(item);
            }
            Ok(Value::Set(a))
        }
        _ => Err(ExpressionError::type_error(
            "Union operation requires list or set operands".to_string(),
        )),
    }
}

/// 集合交集操作
pub fn intersect_values(left: Value, right: Value) -> Result<Value, ExpressionError> {
    match (left, right) {
        (Value::List(a), Value::List(b)) => {
            let result: Vec<Value> = a.into_iter().filter(|item| b.contains(item)).collect();
            Ok(Value::List(result))
        }
        (Value::Set(a), Value::Set(b)) => {
            let result: std::collections::HashSet<Value> = 
                a.into_iter().filter(|item| b.contains(item)).collect();
            Ok(Value::Set(result))
        }
        _ => Err(ExpressionError::type_error(
            "Intersect operation requires list or set operands".to_string(),
        )),
    }
}

/// 集合差集操作
pub fn except_values(left: Value, right: Value) -> Result<Value, ExpressionError> {
    match (left, right) {
        (Value::List(a), Value::List(b)) => {
            let result: Vec<Value> = a.into_iter().filter(|item| !b.contains(item)).collect();
            Ok(Value::List(result))
        }
        (Value::Set(a), Value::Set(b)) => {
            let result: std::collections::HashSet<Value> = 
                a.into_iter().filter(|item| !b.contains(item)).collect();
            Ok(Value::Set(result))
        }
        _ => Err(ExpressionError::type_error(
            "Except operation requires list or set operands".to_string(),
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

// 为了向后兼容，保留原有的操作符枚举定义
#[deprecated(note = "使用 ExtendedBinaryOperator 替代")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LegacyBinaryOperator {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::expression::Expression;
    use crate::core::context::expression::default_context::DefaultExpressionContext;

    #[test]
    fn test_extended_binary_operator() {
        let context = DefaultExpressionContext::new();
        let left = Expression::int(5);
        let right = Expression::int(3);
        
        // 测试Core操作符
        let result = evaluate_binary_op(
            &left,
            &ExtendedBinaryOperator::Core(CoreBinaryOperator::Add),
            &right,
            &context,
        ).unwrap();
        
        assert_eq!(result, Value::Int(8));
        
        // 测试扩展操作符
        let result = evaluate_binary_op(
            &left,
            &ExtendedBinaryOperator::Xor,
            &right,
            &context,
        ).unwrap();
        
        assert_eq!(result, Value::Bool(false)); // 5 XOR 3 = false (both are truthy)
    }
}
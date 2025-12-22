use crate::core::{ExpressionError, Value};
use crate::core::context::expression::default_context::ExpressionContextCore;
use crate::core::Expression;
use crate::core::types::operators::UnaryOperator as CoreUnaryOperator;
use crate::expression::operators_ext::ExtendedUnaryOperator;
use serde::{Deserialize, Serialize};

/// 为了向后兼容，保留原有的UnaryOperator类型别名
/// 
/// 注意：新代码应该使用ExtendedUnaryOperator
#[deprecated(note = "使用 ExtendedUnaryOperator 替代")]
pub type UnaryOperator = ExtendedUnaryOperator;

/// 评估扩展一元操作表达式
pub fn evaluate_unary_op(
    op: &ExtendedUnaryOperator,
    operand: &Expression,
    context: &dyn ExpressionContextCore,
) -> Result<Value, ExpressionError> {
    let evaluator = crate::core::evaluator::ExpressionEvaluator;
    let operand_val = evaluator.evaluate(operand, context)?;

    match op {
        // Core操作符委托给Core求值器
        ExtendedUnaryOperator::Core(core_op) => {
            evaluate_core_unary_op(core_op, &operand_val)
        }
    }
}

/// 评估Core一元操作符
fn evaluate_core_unary_op(
    op: &CoreUnaryOperator,
    operand_val: &Value,
) -> Result<Value, ExpressionError> {
    match op {
        CoreUnaryOperator::Plus => Ok(operand_val.clone()), // Identity operation
        CoreUnaryOperator::Minus => neg_value(operand_val.clone()),
        CoreUnaryOperator::Not => Ok(Value::Bool(!value_to_bool(operand_val))),
        CoreUnaryOperator::IsNull => Ok(Value::Bool(matches!(operand_val, Value::Null(_)))),
        CoreUnaryOperator::IsNotNull => Ok(Value::Bool(!matches!(operand_val, Value::Null(_)))),
        CoreUnaryOperator::IsEmpty => {
            let is_empty = match operand_val {
                Value::String(s) => s.is_empty(),
                Value::List(items) => items.is_empty(),
                Value::Map(items) => items.is_empty(),
                _ => false,
            };
            Ok(Value::Bool(is_empty))
        }
        CoreUnaryOperator::IsNotEmpty => {
            let is_not_empty = match operand_val {
                Value::String(s) => !s.is_empty(),
                Value::List(items) => !items.is_empty(),
                Value::Map(items) => !items.is_empty(),
                _ => true,
            };
            Ok(Value::Bool(is_not_empty))
        }
        CoreUnaryOperator::Increment => increment_value(operand_val.clone()),
        CoreUnaryOperator::Decrement => decrement_value(operand_val.clone()),
    }
}

// 评估扩展的一元操作表达式（处理Expression枚举中的特定变体）
pub fn evaluate_extended_unary_op(
    expr: &Expression,
    context: &dyn ExpressionContextCore,
) -> Result<Value, ExpressionError> {
    match expr {
        Expression::UnaryPlus(operand) => {
            let evaluator = crate::core::evaluator::ExpressionEvaluator;
            evaluator.evaluate(operand, context)
        }
        Expression::UnaryNegate(operand) => {
            let evaluator = crate::core::evaluator::ExpressionEvaluator;
            let value = evaluator.evaluate(operand, context)?;
            neg_value(value)
        }
        Expression::UnaryNot(operand) => {
            let evaluator = crate::core::evaluator::ExpressionEvaluator;
            let value = evaluator.evaluate(operand, context)?;
            Ok(Value::Bool(!value_to_bool(&value)))
        }
        Expression::UnaryIncr(operand) => {
            let evaluator = crate::core::evaluator::ExpressionEvaluator;
            let value = evaluator.evaluate(operand, context)?;
            increment_value(value)
        }
        Expression::UnaryDecr(operand) => {
            let evaluator = crate::core::evaluator::ExpressionEvaluator;
            let value = evaluator.evaluate(operand, context)?;
            decrement_value(value)
        }
        Expression::IsNull(operand) => {
            let evaluator = crate::core::evaluator::ExpressionEvaluator;
            let value = evaluator.evaluate(operand, context)?;
            Ok(Value::Bool(matches!(value, Value::Null(_))))
        }
        Expression::IsNotNull(operand) => {
            let evaluator = crate::core::evaluator::ExpressionEvaluator;
            let value = evaluator.evaluate(operand, context)?;
            Ok(Value::Bool(!matches!(value, Value::Null(_))))
        }
        Expression::IsEmpty(operand) => {
            let evaluator = crate::core::evaluator::ExpressionEvaluator;
            let value = evaluator.evaluate(operand, context)?;
            let is_empty = match value {
                Value::String(s) => s.is_empty(),
                Value::List(items) => items.is_empty(),
                Value::Set(items) => items.is_empty(),
                Value::Map(items) => items.is_empty(),
                _ => false,
            };
            Ok(Value::Bool(is_empty))
        }
        Expression::IsNotEmpty(operand) => {
            let evaluator = crate::core::evaluator::ExpressionEvaluator;
            let value = evaluator.evaluate(operand, context)?;
            let is_not_empty = match value {
                Value::String(s) => !s.is_empty(),
                Value::List(items) => !items.is_empty(),
                Value::Set(items) => !items.is_empty(),
                Value::Map(items) => !items.is_empty(),
                _ => true,
            };
            Ok(Value::Bool(is_not_empty))
        }
        _ => Err(ExpressionError::type_error(
            "Expression is not an extended unary operation".to_string(),
        )),
    }
}

fn neg_value(value: Value) -> Result<Value, ExpressionError> {
    match value {
        Value::Int(n) => Ok(Value::Int(-n)),
        Value::Float(n) => Ok(Value::Float(-n)),
        _ => Err(ExpressionError::type_error(
            "Cannot negate this value type".to_string(),
        )),
    }
}

fn increment_value(value: Value) -> Result<Value, ExpressionError> {
    match value {
        Value::Int(n) => Ok(Value::Int(n + 1)),
        Value::Float(n) => Ok(Value::Float(n + 1.0)),
        _ => Err(ExpressionError::type_error(
            "Cannot increment non-numeric value".to_string(),
        )),
    }
}

fn decrement_value(value: Value) -> Result<Value, ExpressionError> {
    match value {
        Value::Int(n) => Ok(Value::Int(n - 1)),
        Value::Float(n) => Ok(Value::Float(n - 1.0)),
        _ => Err(ExpressionError::type_error(
            "Cannot decrement non-numeric value".to_string(),
        )),
    }
}

/// 为evaluator提供公共访问
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
#[deprecated(note = "使用 ExtendedUnaryOperator 替代")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LegacyUnaryOperator {
    Plus,
    Minus,
    Negate,
    Not,
    IsNull,
    IsNotNull,
    IsEmpty,
    IsNotEmpty,
    Increment,
    Decrement,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::expression::Expression;
    use crate::core::context::expression::default_context::DefaultExpressionContext;

    #[test]
    fn test_extended_unary_operator() {
        let context = DefaultExpressionContext::new();
        let operand = Expression::int(5);
        
        // 测试Core操作符
        let result = evaluate_unary_op(
            &ExtendedUnaryOperator::Core(CoreUnaryOperator::Minus),
            &operand,
            &context,
        ).unwrap();
        
        assert_eq!(result, Value::Int(-5));
        
        // 测试Core操作符
        let result = evaluate_unary_op(
            &ExtendedUnaryOperator::Core(CoreUnaryOperator::Not),
            &Expression::bool(true),
            &context,
        ).unwrap();
        
        assert_eq!(result, Value::Bool(false));
    }
}
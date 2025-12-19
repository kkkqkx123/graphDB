use super::error::ExpressionError;
use crate::core::Value;
use crate::graph::expression::Expression;
use crate::query::context::EvalContext;
use serde::{Deserialize, Serialize};

/// Unary operators for expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnaryOperator {
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

/// 评估一元操作表达式
pub fn evaluate_unary_op(
    op: &UnaryOperator,
    operand: &Expression,
    context: &EvalContext,
) -> Result<Value, ExpressionError> {
    let evaluator = super::evaluator::ExpressionEvaluator;
    let operand_val = evaluator.evaluate(operand, context)?;

    match op {
        UnaryOperator::Plus => Ok(operand_val), // Identity operation
        UnaryOperator::Minus => neg_value(operand_val),
        UnaryOperator::Negate => neg_value(operand_val),
        UnaryOperator::Not => Ok(Value::Bool(!value_to_bool(&operand_val))),
        UnaryOperator::IsNull => Ok(Value::Bool(matches!(operand_val, Value::Null(_)))),
        UnaryOperator::IsNotNull => Ok(Value::Bool(!matches!(operand_val, Value::Null(_)))),
        UnaryOperator::IsEmpty => {
            let is_empty = match &operand_val {
                Value::String(s) => s.is_empty(),
                Value::List(items) => items.is_empty(),
                Value::Map(items) => items.is_empty(),
                _ => false,
            };
            Ok(Value::Bool(is_empty))
        },
        UnaryOperator::IsNotEmpty => {
            let is_not_empty = match &operand_val {
                Value::String(s) => !s.is_empty(),
                Value::List(items) => !items.is_empty(),
                Value::Map(items) => !items.is_empty(),
                _ => true,
            };
            Ok(Value::Bool(is_not_empty))
        },
        UnaryOperator::Increment => Err(ExpressionError::InvalidOperation(
            "Increment operation not supported".to_string(),
        )),
        UnaryOperator::Decrement => Err(ExpressionError::InvalidOperation(
            "Decrement operation not supported".to_string(),
        )),
    }
}

// 评估扩展的一元操作表达式
pub fn evaluate_extended_unary_op(
    expr: &Expression,
    context: &EvalContext,
) -> Result<Value, ExpressionError> {
    match expr {
        Expression::UnaryPlus(operand) => {
            let evaluator = super::evaluator::ExpressionEvaluator;
            evaluator.evaluate(operand, context)
        }
        Expression::UnaryNegate(operand) => {
            let evaluator = super::evaluator::ExpressionEvaluator;
            let value = evaluator.evaluate(operand, context)?;
            neg_value(value)
        }
        Expression::UnaryNot(operand) => {
            let evaluator = super::evaluator::ExpressionEvaluator;
            let value = evaluator.evaluate(operand, context)?;
            Ok(Value::Bool(!value_to_bool(&value)))
        }
        Expression::UnaryIncr(operand) => {
            let evaluator = super::evaluator::ExpressionEvaluator;
            let value = evaluator.evaluate(operand, context)?;
            match value {
                Value::Int(n) => Ok(Value::Int(n + 1)),
                Value::Float(n) => Ok(Value::Float(n + 1.0)),
                _ => Err(ExpressionError::TypeError(
                    "Cannot increment non-numeric value".to_string(),
                )),
            }
        }
        Expression::UnaryDecr(operand) => {
            let evaluator = super::evaluator::ExpressionEvaluator;
            let value = evaluator.evaluate(operand, context)?;
            match value {
                Value::Int(n) => Ok(Value::Int(n - 1)),
                Value::Float(n) => Ok(Value::Float(n - 1.0)),
                _ => Err(ExpressionError::TypeError(
                    "Cannot decrement non-numeric value".to_string(),
                )),
            }
        }
        Expression::IsNull(operand) => {
            let evaluator = super::evaluator::ExpressionEvaluator;
            let value = evaluator.evaluate(operand, context)?;
            Ok(Value::Bool(matches!(value, Value::Null(_))))
        }
        Expression::IsNotNull(operand) => {
            let evaluator = super::evaluator::ExpressionEvaluator;
            let value = evaluator.evaluate(operand, context)?;
            Ok(Value::Bool(!matches!(value, Value::Null(_))))
        }
        Expression::IsEmpty(operand) => {
            let evaluator = super::evaluator::ExpressionEvaluator;
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
            let evaluator = super::evaluator::ExpressionEvaluator;
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
        _ => Err(ExpressionError::TypeError(
            "Expression is not an extended unary operation".to_string(),
        )),
    }
}

fn neg_value(value: Value) -> Result<Value, ExpressionError> {
    match value {
        Value::Int(n) => Ok(Value::Int(-n)),
        Value::Float(n) => Ok(Value::Float(-n)),
        _ => Err(ExpressionError::TypeError(
            "Cannot negate this value type".to_string(),
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

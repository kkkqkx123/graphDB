use std::collections::{HashMap, HashSet};
use crate::core::{Value, NullType};
use super::base::{EvaluationError, ExpressionContext};
use crate::expressions::value::Expression;

/// Process a list container expression
pub fn eval_list(items: &[Expression], context: &dyn ExpressionContext) -> Result<Value, EvaluationError> {
    let evaluated_items: Result<Vec<Value>, _> =
        items.iter().map(|item| item.eval(context)).collect();
    let items = evaluated_items?;
    Ok(Value::List(items))
}

/// Process a map container expression
pub fn eval_map(items: &[(Expression, Expression)], context: &dyn ExpressionContext) -> Result<Value, EvaluationError> {
    let mut result_map = HashMap::new();
    for (key_expr, value_expr) in items {
        let key = key_expr.eval(context)?;
        let value = value_expr.eval(context)?;
        // Convert key to string for map indexing
        let key_str = match key {
            Value::String(s) => s,
            Value::Int(i) => i.to_string(),
            Value::Bool(b) => b.to_string(),
            _ => return Err(EvaluationError::TypeError(
                format!("Map key must be string, int, or bool, got {:?}", key)
            )),
        };
        result_map.insert(key_str, value);
    }
    Ok(Value::Map(result_map))
}

/// Process a set container expression
pub fn eval_set(items: &[Expression], context: &dyn ExpressionContext) -> Result<Value, EvaluationError> {
    let evaluated_items: Result<Vec<Value>, _> =
        items.iter().map(|item| item.eval(context)).collect();
    let items = evaluated_items?;
    Ok(Value::Set(HashSet::from_iter(items)))
}
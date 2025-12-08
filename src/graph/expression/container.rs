use crate::core::Value;
use crate::graph::expression::Expression;
use super::context::EvalContext;
use super::error::ExpressionError;

/// 评估容器表达式
pub fn evaluate_container(
    expr: &Expression,
    context: &EvalContext,
) -> Result<Value, ExpressionError> {
    match expr {
        Expression::List(items) => {
            let mut result = Vec::new();
            for item in items {
                let evaluator = super::evaluator::ExpressionEvaluator;
                result.push(evaluator.evaluate(item, context)?);
            }
            Ok(Value::List(result))
        },
        Expression::Set(items) => {
            let mut result = std::collections::HashSet::new();
            for item in items {
                let evaluator = super::evaluator::ExpressionEvaluator;
                result.insert(evaluator.evaluate(item, context)?);
            }
            Ok(Value::Set(result))
        },
        Expression::Map(items) => {
            let mut result = std::collections::HashMap::new();
            for (key, value) in items {
                let evaluator = super::evaluator::ExpressionEvaluator;
                let evaluated_value = evaluator.evaluate(value, context)?;
                result.insert(key.clone(), evaluated_value);
            }
            Ok(Value::Map(result))
        },
        _ => Err(ExpressionError::TypeError(
            "Expression is not a container expression".to_string()
        )),
    }
}
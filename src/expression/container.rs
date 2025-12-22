use crate::core::Value;
use crate::core::{Expression, ExpressionError};
use crate::core::expressions::ExpressionContext;

/// 评估容器表达式
pub fn evaluate_container(
    expr: &Expression,
    context: &dyn ExpressionContext,
) -> Result<Value, ExpressionError> {
    match expr {
        Expression::List(items) => {
            let mut result = Vec::new();
            for item in items {
                let evaluator = crate::core::evaluator::ExpressionEvaluator;
                result.push(evaluator.evaluate(item, context)?);
            }
            Ok(Value::List(result))
        }
        Expression::Map(items) => {
            let mut result = std::collections::HashMap::new();
            for (key, value) in items {
                let evaluator = crate::core::evaluator::ExpressionEvaluator;
                let evaluated_value = evaluator.evaluate(value, context)?;
                result.insert(key.clone(), evaluated_value);
            }
            Ok(Value::Map(result))
        }
        _ => Err(ExpressionError::type_error(
            "Expression is not a container expression".to_string(),
        )),
    }
}

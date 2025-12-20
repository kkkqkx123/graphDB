use super::error::ExpressionError;
use crate::core::Value;
use crate::expression::Expression;
use crate::query::context::EvalContext;

/// 评估函数表达式
pub fn evaluate_function(
    func_name: &str,
    args: &[Expression],
    context: &EvalContext,
) -> Result<Value, ExpressionError> {
    match func_name.to_lowercase().as_str() {
        "has_property" => {
            if args.len() != 1 {
                return Err(ExpressionError::FunctionError(
                    "has_property expects 1 argument".to_string(),
                ));
            }

            let prop_expr = &args[0];
            let evaluator = super::evaluator::ExpressionEvaluator;
            let prop_name_val = evaluator.evaluate(prop_expr, context)?;
            let prop_name = match prop_name_val {
                Value::String(name) => name,
                _ => {
                    return Err(ExpressionError::FunctionError(
                        "Property name must be a string".to_string(),
                    ))
                }
            };

            // Check if property exists in vertex
            let exists = if let Some(vertex) = context.vertex {
                vertex
                    .tags
                    .iter()
                    .any(|tag| tag.properties.contains_key(&prop_name))
            } else if let Some(edge) = context.edge {
                edge.props.contains_key(&prop_name)
            } else {
                false
            };

            Ok(Value::Bool(exists))
        }
        "coalesce" => {
            for arg in args {
                let evaluator = super::evaluator::ExpressionEvaluator;
                let val = evaluator.evaluate(arg, context)?;
                if !is_null_value(&val) {
                    return Ok(val);
                }
            }
            Ok(Value::Null(crate::core::NullType::Null))
        }
        // 添加更多函数实现
        _ => Err(ExpressionError::FunctionError(format!(
            "Unknown function: {}",
            func_name
        ))),
    }
}

fn is_null_value(value: &Value) -> bool {
    matches!(value, Value::Null(_))
}

/// 评估聚合函数表达式
pub fn evaluate_aggregate(
    func: &str,
    arg: &Expression,
    distinct: bool,
    context: &EvalContext,
) -> Result<Value, ExpressionError> {
    // 这里需要一个更复杂的聚合函数实现
    // 聚合函数通常在执行时需要累积数据
    match func.to_lowercase().as_str() {
        "count" => {
            let evaluator = super::evaluator::ExpressionEvaluator;
            let arg_val = evaluator.evaluate(arg, context)?;
            // 对于计数，我们只关心是否非空
            if is_null_value(&arg_val) {
                Ok(Value::Int(0))
            } else {
                Ok(Value::Int(if distinct { 1 } else { 1 })) // 简化实现
            }
        }
        "sum" => {
            let evaluator = super::evaluator::ExpressionEvaluator;
            let arg_val = evaluator.evaluate(arg, context)?;
            // 这是一个简化的实现，实际的聚合函数需要更复杂的状态管理
            match arg_val {
                Value::Int(n) => Ok(Value::Int(n)),
                Value::Float(f) => Ok(Value::Float(f)),
                _ => Err(ExpressionError::TypeError(
                    "Sum can only be applied to numeric values".to_string(),
                )),
            }
        }
        "avg" => {
            let evaluator = super::evaluator::ExpressionEvaluator;
            let arg_val = evaluator.evaluate(arg, context)?;
            // 这是一个简化的实现
            match arg_val {
                Value::Int(n) => Ok(Value::Float(n as f64)),
                Value::Float(f) => Ok(Value::Float(f)),
                _ => Err(ExpressionError::TypeError(
                    "Avg can only be applied to numeric values".to_string(),
                )),
            }
        }
        "min" | "max" => {
            let evaluator = super::evaluator::ExpressionEvaluator;
            let arg_val = evaluator.evaluate(arg, context)?;
            // 这是一个简化的实现
            Ok(arg_val)
        }
        _ => Err(ExpressionError::FunctionError(format!(
            "Unknown aggregate function: {}",
            func
        ))),
    }
}

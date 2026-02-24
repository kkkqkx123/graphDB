//! 函数调用求值
//!
//! 提供聚合函数的求值功能

use crate::core::error::{ExpressionError, ExpressionErrorType};
use crate::core::types::operators::AggregateFunction;
use crate::core::Value;
use crate::core::value::dataset::List;

/// 函数求值器
pub struct FunctionEvaluator;

impl FunctionEvaluator {
    /// 求值聚合函数
    pub fn eval_aggregate_function(
        func: &AggregateFunction,
        args: &[Value],
        distinct: bool,
    ) -> Result<Value, ExpressionError> {
        if args.is_empty() {
            return Err(ExpressionError::argument_count_error(1, 0));
        }

        match func {
            AggregateFunction::Count(_) => {
                if distinct {
                    let unique_values: std::collections::HashSet<_> = args.iter().collect();
                    Ok(Value::Int(unique_values.len() as i64))
                } else {
                    Ok(Value::Int(args.len() as i64))
                }
            }
            AggregateFunction::Sum(_) => {
                let mut sum = Value::Int(0);
                for arg in args {
                    sum = sum
                        .add(arg)
                        .map_err(|e| ExpressionError::runtime_error(e))?;
                }
                Ok(sum)
            }
            AggregateFunction::Avg(_) => {
                let sum = Self::eval_aggregate_function(&AggregateFunction::Sum("".to_string()), args, distinct)?;
                let count =
                    Self::eval_aggregate_function(&AggregateFunction::Count(None), args, distinct)?;
                sum.div(&count)
                    .map_err(|e| ExpressionError::runtime_error(e))
            }
            AggregateFunction::Min(_) => {
                let mut min = args[0].clone();
                for arg in args.iter().skip(1) {
                    if arg < &min {
                        min = arg.clone();
                    }
                }
                Ok(min)
            }
            AggregateFunction::Max(_) => {
                let mut max = args[0].clone();
                for arg in args.iter().skip(1) {
                    if arg > &max {
                        max = arg.clone();
                    }
                }
                Ok(max)
            }
            AggregateFunction::Collect(_) => {
                if distinct {
                    let unique_values: std::collections::HashSet<_> =
                        args.iter().cloned().collect();
                    Ok(Value::List(List::from(unique_values.into_iter().collect::<Vec<_>>())))}
                else {
                    Ok(Value::List(List::from(args.to_vec())))
                }
            }
            AggregateFunction::CollectSet(_) => {
                let unique_values: std::collections::HashSet<_> = args.iter().cloned().collect();
                Ok(Value::Set(unique_values))
            }
            AggregateFunction::Distinct(_) => {
                let unique_values: std::collections::HashSet<_> = args.iter().cloned().collect();
                Ok(Value::Set(unique_values))
            }
            AggregateFunction::Percentile(_, _) => {
                if args.len() < 2 {
                    return Err(ExpressionError::argument_count_error(2, args.len()));
                }

                let percentile = match &args[1] {
                    Value::Int(v) => *v as f64,
                    Value::Float(v) => *v,
                    _ => return Err(ExpressionError::type_error("Percentile must be a number")),
                };

                if percentile < 0.0 || percentile > 100.0 {
                    return Err(ExpressionError::new(
                        ExpressionErrorType::InvalidOperation,
                        "Percentile must be between 0 and 100",
                    ));
                }

                let values = match &args[0] {
                    Value::List(list) => list,
                    _ => return Err(ExpressionError::type_error("First argument must be a list")),
                };

                if values.is_empty() {
                    return Ok(Value::Null(crate::core::NullType::NaN));
                }

                let mut numeric_values = Vec::new();
                for value in values {
                    match value {
                        Value::Int(v) => numeric_values.push(*v as f64),
                        Value::Float(v) => numeric_values.push(*v),
                        _ => continue,
                    }
                }

                if numeric_values.is_empty() {
                    return Ok(Value::Null(crate::core::NullType::NaN));
                }

                numeric_values
                    .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

                let index = (percentile / 100.0) * (numeric_values.len() - 1) as f64;
                let lower_index = index.floor() as usize;
                let upper_index = index.ceil() as usize;

                if lower_index == upper_index {
                    Ok(Value::Float(numeric_values[lower_index]))
                } else {
                    let lower_value = numeric_values[lower_index];
                    let upper_value = numeric_values[upper_index];
                    let weight = index - lower_index as f64;
                    let interpolated = lower_value + weight * (upper_value - lower_value);
                    Ok(Value::Float(interpolated))
                }
            }
            AggregateFunction::Std(_) => {
                if args.is_empty() {
                    return Err(ExpressionError::argument_count_error(1, args.len()));
                }

                let values = match &args[0] {
                    Value::List(list) => list,
                    _ => return Err(ExpressionError::type_error("First argument must be a list")),
                };

                if values.is_empty() {
                    return Ok(Value::Null(crate::core::NullType::NaN));
                }

                let mut numeric_values = Vec::new();
                for value in values {
                    match value {
                        Value::Int(v) => numeric_values.push(*v as f64),
                        Value::Float(v) => numeric_values.push(*v),
                        _ => continue,
                    }
                }

                if numeric_values.is_empty() {
                    return Ok(Value::Null(crate::core::NullType::NaN));
                }

                let n = numeric_values.len() as f64;
                let mean: f64 = numeric_values.iter().sum::<f64>() / n;
                let variance: f64 = numeric_values.iter()
                    .map(|value| (value - mean).powi(2))
                    .sum::<f64>() / n;
                let std_dev = variance.sqrt();

                Ok(Value::Float(std_dev))
            }
            AggregateFunction::BitAnd(_) => {
                if args.is_empty() {
                    return Err(ExpressionError::argument_count_error(1, args.len()));
                }

                let values = match &args[0] {
                    Value::List(list) => list,
                    _ => return Err(ExpressionError::type_error("First argument must be a list")),
                };

                if values.is_empty() {
                    return Ok(Value::Null(crate::core::NullType::NaN));
                }

                let mut result = std::i64::MAX;
                for value in values {
                    match value {
                        Value::Int(v) => result &= v,
                        _ => return Err(ExpressionError::type_error("All values must be integers for BIT_AND")),
                    }
                }

                Ok(Value::Int(result))
            }
            AggregateFunction::BitOr(_) => {
                if args.is_empty() {
                    return Err(ExpressionError::argument_count_error(1, args.len()));
                }

                let values = match &args[0] {
                    Value::List(list) => list,
                    _ => return Err(ExpressionError::type_error("First argument must be a list")),
                };

                if values.is_empty() {
                    return Ok(Value::Null(crate::core::NullType::NaN));
                }

                let mut result = 0i64;
                for value in values {
                    match value {
                        Value::Int(v) => result |= v,
                        _ => return Err(ExpressionError::type_error("All values must be integers for BIT_OR")),
                    }
                }

                Ok(Value::Int(result))
            }
            AggregateFunction::GroupConcat(_, separator) => {
                if args.is_empty() {
                    return Err(ExpressionError::argument_count_error(1, args.len()));
                }

                let values = match &args[0] {
                    Value::List(list) => list,
                    _ => return Err(ExpressionError::type_error("First argument must be a list")),
                };

                if values.is_empty() {
                    return Ok(Value::String(String::new()));
                }

                let result: Vec<String> = values.iter()
                    .map(|v| format!("{}", v))
                    .collect();
                Ok(Value::String(result.join(separator)))
            }
        }
    }
}

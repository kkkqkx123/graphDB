//! 函数调用求值
//!
//! 提供各种内置函数的求值功能

use crate::core::error::{ExpressionError, ExpressionErrorType};
use crate::core::types::operators::AggregateFunction;
use crate::core::Value;

/// 函数求值器
pub struct FunctionEvaluator;

impl FunctionEvaluator {
    /// 求值函数调用
    pub fn eval_function_call(&self, name: &str, args: &[Value]) -> Result<Value, ExpressionError> {
        match name {
            // 数学函数
            "abs" => {
                if args.len() != 1 {
                    return Err(ExpressionError::argument_count_error(1, args.len()));
                }
                match &args[0] {
                    Value::Int(i) => Ok(Value::Int(i.abs())),
                    Value::Float(f) => Ok(Value::Float(f.abs())),
                    _ => Err(ExpressionError::type_error("abs函数需要数值类型")),
                }
            }
            "ceil" => {
                if args.len() != 1 {
                    return Err(ExpressionError::argument_count_error(1, args.len()));
                }
                match &args[0] {
                    Value::Int(i) => Ok(Value::Int(*i)),
                    Value::Float(f) => Ok(Value::Float(f.ceil())),
                    _ => Err(ExpressionError::type_error("ceil函数需要数值类型")),
                }
            }
            "floor" => {
                if args.len() != 1 {
                    return Err(ExpressionError::argument_count_error(1, args.len()));
                }
                match &args[0] {
                    Value::Int(i) => Ok(Value::Int(*i)),
                    Value::Float(f) => Ok(Value::Float(f.floor())),
                    _ => Err(ExpressionError::type_error("floor函数需要数值类型")),
                }
            }
            "round" => {
                if args.len() != 1 {
                    return Err(ExpressionError::argument_count_error(1, args.len()));
                }
                match &args[0] {
                    Value::Int(i) => Ok(Value::Int(*i)),
                    Value::Float(f) => Ok(Value::Float(f.round())),
                    _ => Err(ExpressionError::type_error("round函数需要数值类型")),
                }
            }

            // 字符串函数
            "length" => {
                if args.len() != 1 {
                    return Err(ExpressionError::argument_count_error(1, args.len()));
                }
                match &args[0] {
                    Value::String(s) => Ok(Value::Int(s.len() as i64)),
                    Value::List(l) => Ok(Value::Int(l.len() as i64)),
                    Value::Map(m) => Ok(Value::Int(m.len() as i64)),
                    _ => Err(ExpressionError::type_error(
                        "length函数需要字符串、列表或映射类型",
                    )),
                }
            }
            "lower" => {
                if args.len() != 1 {
                    return Err(ExpressionError::argument_count_error(1, args.len()));
                }
                match &args[0] {
                    Value::String(s) => Ok(Value::String(s.to_lowercase())),
                    _ => Err(ExpressionError::type_error("lower函数需要字符串类型")),
                }
            }
            "upper" => {
                if args.len() != 1 {
                    return Err(ExpressionError::argument_count_error(1, args.len()));
                }
                match &args[0] {
                    Value::String(s) => Ok(Value::String(s.to_uppercase())),
                    _ => Err(ExpressionError::type_error("upper函数需要字符串类型")),
                }
            }
            "trim" => {
                if args.len() != 1 {
                    return Err(ExpressionError::argument_count_error(1, args.len()));
                }
                match &args[0] {
                    Value::String(s) => Ok(Value::String(s.trim().to_string())),
                    _ => Err(ExpressionError::type_error("trim函数需要字符串类型")),
                }
            }

            _ => Err(ExpressionError::undefined_function(name)),
        }
    }

    /// 求值聚合函数（单个参数）
    pub fn eval_aggregate_function_single(
        &self,
        func: &AggregateFunction,
        arg: &Value,
        _distinct: bool,
    ) -> Result<Value, ExpressionError> {
        match func {
            AggregateFunction::Count => {
                if arg.is_null() {
                    Ok(Value::Int(0))
                } else {
                    Ok(Value::Int(1))
                }
            }
            AggregateFunction::Sum => {
                if arg.is_null() {
                    Ok(Value::Int(0))
                } else {
                    Ok(arg.clone())
                }
            }
            AggregateFunction::Avg => {
                if arg.is_null() {
                    Ok(Value::Null(crate::core::NullType::Null))
                } else {
                    Ok(arg.clone())
                }
            }
            AggregateFunction::Min => {
                if arg.is_null() {
                    Ok(Value::Null(crate::core::NullType::Null))
                } else {
                    Ok(arg.clone())
                }
            }
            AggregateFunction::Max => {
                if arg.is_null() {
                    Ok(Value::Null(crate::core::NullType::Null))
                } else {
                    Ok(arg.clone())
                }
            }
            AggregateFunction::Collect => Ok(Value::List(vec![arg.clone()])),
            AggregateFunction::Distinct => Ok(Value::List(vec![arg.clone()])),
            AggregateFunction::Percentile => {
                if arg.is_null() {
                    Ok(Value::Null(crate::core::NullType::Null))
                } else {
                    Ok(arg.clone())
                }
            }
        }
    }

    /// 求值聚合函数
    pub fn eval_aggregate_function(
        &self,
        func: &AggregateFunction,
        args: &[Value],
        distinct: bool,
    ) -> Result<Value, ExpressionError> {
        if args.is_empty() {
            return Err(ExpressionError::argument_count_error(1, 0));
        }

        match func {
            AggregateFunction::Count => {
                if distinct {
                    let unique_values: std::collections::HashSet<_> = args.iter().collect();
                    Ok(Value::Int(unique_values.len() as i64))
                } else {
                    Ok(Value::Int(args.len() as i64))
                }
            }
            AggregateFunction::Sum => {
                let mut sum = Value::Int(0);
                for arg in args {
                    sum = sum
                        .add(arg)
                        .map_err(|e| ExpressionError::runtime_error(e))?;
                }
                Ok(sum)
            }
            AggregateFunction::Avg => {
                let sum = self.eval_aggregate_function(&AggregateFunction::Sum, args, distinct)?;
                let count =
                    self.eval_aggregate_function(&AggregateFunction::Count, args, distinct)?;
                sum.div(&count)
                    .map_err(|e| ExpressionError::runtime_error(e))
            }
            AggregateFunction::Min => {
                let mut min = args[0].clone();
                for arg in args.iter().skip(1) {
                    if arg < &min {
                        min = arg.clone();
                    }
                }
                Ok(min)
            }
            AggregateFunction::Max => {
                let mut max = args[0].clone();
                for arg in args.iter().skip(1) {
                    if arg > &max {
                        max = arg.clone();
                    }
                }
                Ok(max)
            }
            AggregateFunction::Collect => {
                if distinct {
                    let unique_values: std::collections::HashSet<_> =
                        args.iter().cloned().collect();
                    Ok(Value::List(unique_values.into_iter().collect()))
                } else {
                    Ok(Value::List(args.to_vec()))
                }
            }
            AggregateFunction::Distinct => {
                let unique_values: std::collections::HashSet<_> = args.iter().cloned().collect();
                Ok(Value::List(unique_values.into_iter().collect()))
            }
            AggregateFunction::Percentile => {
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
        }
    }
}

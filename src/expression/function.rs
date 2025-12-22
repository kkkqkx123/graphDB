use crate::core::{ExpressionError, Value};
use crate::core::{Expression, ExpressionContext};

/// 评估函数调用
pub fn evaluate_function(
    name: &str,
    args: &[Expression],
    context: &dyn ExpressionContext,
) -> Result<Value, ExpressionError> {
    match name {
        // 数学函数
        "abs" => evaluate_abs(args, context),
        "ceil" => evaluate_ceil(args, context),
        "floor" => evaluate_floor(args, context),
        "round" => evaluate_round(args, context),
        "sqrt" => evaluate_sqrt(args, context),
        "pow" => evaluate_pow(args, context),

        // 字符串函数
        "length" => evaluate_length(args, context),
        "substring" => evaluate_substring(args, context),
        "trim" => evaluate_trim(args, context),
        "upper" => evaluate_upper(args, context),
        "lower" => evaluate_lower(args, context),

        // 类型检查函数
        "type" => evaluate_type(args, context),
        "exists" => evaluate_exists(args, context),

        // 其他函数
        "id" => evaluate_id(args, context),
        "labels" => evaluate_labels(args, context),

        _ => Err(ExpressionError::UnknownFunction(name.to_string())),
    }
}

/// 评估聚合函数
pub fn evaluate_aggregate(
    name: &str,
    arg: &Expression,
    distinct: bool,
    context: &dyn ExpressionContext,
) -> Result<Value, ExpressionError> {
    match name {
        "count" => evaluate_count(arg, distinct, context),
        "sum" => evaluate_sum(arg, distinct, context),
        "avg" => evaluate_avg(arg, distinct, context),
        "min" => evaluate_min(arg, distinct, context),
        "max" => evaluate_max(arg, distinct, context),
        _ => Err(ExpressionError::UnknownFunction(name.to_string())),
    }
}

// 数学函数实现
fn evaluate_abs(
    args: &[Expression],
    context: &dyn ExpressionContext,
) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::InvalidArgumentCount("abs".to_string()));
    }

    let value = super::evaluator::ExpressionEvaluator::new().evaluate(&args[0], context)?;
    match value {
        Value::Int(i) => Ok(Value::Int(i.abs())),
        Value::Float(f) => Ok(Value::Float(f.abs())),
        _ => Err(ExpressionError::TypeError(
            "abs expects numeric argument".to_string(),
        )),
    }
}

fn evaluate_ceil(
    args: &[Expression],
    context: &dyn ExpressionContext,
) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::InvalidArgumentCount("ceil".to_string()));
    }

    let value = super::evaluator::ExpressionEvaluator::new().evaluate(&args[0], context)?;
    match value {
        Value::Float(f) => Ok(Value::Int(f.ceil() as i64)),
        Value::Int(i) => Ok(Value::Int(i)),
        _ => Err(ExpressionError::TypeError(
            "ceil expects numeric argument".to_string(),
        )),
    }
}

fn evaluate_floor(
    args: &[Expression],
    context: &dyn ExpressionContext,
) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::InvalidArgumentCount("floor".to_string()));
    }

    let value = super::evaluator::ExpressionEvaluator::new().evaluate(&args[0], context)?;
    match value {
        Value::Float(f) => Ok(Value::Int(f.floor() as i64)),
        Value::Int(i) => Ok(Value::Int(i)),
        _ => Err(ExpressionError::TypeError(
            "floor expects numeric argument".to_string(),
        )),
    }
}

fn evaluate_round(
    args: &[Expression],
    context: &dyn ExpressionContext,
) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::InvalidArgumentCount("round".to_string()));
    }

    let value = super::evaluator::ExpressionEvaluator::new().evaluate(&args[0], context)?;
    match value {
        Value::Float(f) => Ok(Value::Int(f.round() as i64)),
        Value::Int(i) => Ok(Value::Int(i)),
        _ => Err(ExpressionError::TypeError(
            "round expects numeric argument".to_string(),
        )),
    }
}

fn evaluate_sqrt(
    args: &[Expression],
    context: &dyn ExpressionContext,
) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::InvalidArgumentCount("sqrt".to_string()));
    }

    let value = super::evaluator::ExpressionEvaluator::new().evaluate(&args[0], context)?;
    match value {
        Value::Float(f) if f >= 0.0 => Ok(Value::Float(f.sqrt())),
        Value::Int(i) if i >= 0 => Ok(Value::Float((i as f64).sqrt())),
        _ => Err(ExpressionError::TypeError(
            "sqrt expects non-negative numeric argument".to_string(),
        )),
    }
}

fn evaluate_pow(
    args: &[Expression],
    context: &dyn ExpressionContext,
) -> Result<Value, ExpressionError> {
    if args.len() != 2 {
        return Err(ExpressionError::InvalidArgumentCount("pow".to_string()));
    }

    let base = super::evaluator::ExpressionEvaluator::new().evaluate(&args[0], context)?;
    let exp = super::evaluator::ExpressionEvaluator::new().evaluate(&args[1], context)?;

    match (base, exp) {
        (Value::Float(base_f), Value::Float(exp_f)) => Ok(Value::Float(base_f.powf(exp_f))),
        (Value::Int(base_i), Value::Int(exp_i)) => {
            Ok(Value::Float((base_i as f64).powf(exp_i as f64)))
        }
        _ => Err(ExpressionError::TypeError(
            "pow expects numeric arguments".to_string(),
        )),
    }
}

// 字符串函数实现
fn evaluate_length(
    args: &[Expression],
    context: &dyn ExpressionContext,
) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::InvalidArgumentCount("length".to_string()));
    }

    let value = super::evaluator::ExpressionEvaluator::new().evaluate(&args[0], context)?;
    match value {
        Value::String(s) => Ok(Value::Int(s.len() as i64)),
        Value::List(list) => Ok(Value::Int(list.len() as i64)),
        _ => Err(ExpressionError::TypeError(
            "length expects string or list argument".to_string(),
        )),
    }
}

fn evaluate_substring(
    args: &[Expression],
    context: &dyn ExpressionContext,
) -> Result<Value, ExpressionError> {
    if args.len() != 3 {
        return Err(ExpressionError::InvalidArgumentCount(
            "substring".to_string(),
        ));
    }

    let string_val = super::evaluator::ExpressionEvaluator::new().evaluate(&args[0], context)?;
    let start_val = super::evaluator::ExpressionEvaluator::new().evaluate(&args[1], context)?;
    let length_val = super::evaluator::ExpressionEvaluator::new().evaluate(&args[2], context)?;

    match (string_val, start_val, length_val) {
        (Value::String(s), Value::Int(start), Value::Int(length)) => {
            let start = start.max(0) as usize;
            let end = (start + length.max(0) as usize).min(s.len());
            Ok(Value::String(s[start..end].to_string()))
        }
        _ => Err(ExpressionError::TypeError(
            "substring expects (string, int, int) arguments".to_string(),
        )),
    }
}

fn evaluate_trim(
    args: &[Expression],
    context: &dyn ExpressionContext,
) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::InvalidArgumentCount("trim".to_string()));
    }

    let value = super::evaluator::ExpressionEvaluator::new().evaluate(&args[0], context)?;
    match value {
        Value::String(s) => Ok(Value::String(s.trim().to_string())),
        _ => Err(ExpressionError::TypeError(
            "trim expects string argument".to_string(),
        )),
    }
}

fn evaluate_upper(
    args: &[Expression],
    context: &dyn ExpressionContext,
) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::InvalidArgumentCount("upper".to_string()));
    }

    let value = super::evaluator::ExpressionEvaluator::new().evaluate(&args[0], context)?;
    match value {
        Value::String(s) => Ok(Value::String(s.to_uppercase())),
        _ => Err(ExpressionError::TypeError(
            "upper expects string argument".to_string(),
        )),
    }
}

fn evaluate_lower(
    args: &[Expression],
    context: &dyn ExpressionContext,
) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::InvalidArgumentCount("lower".to_string()));
    }

    let value = super::evaluator::ExpressionEvaluator::new().evaluate(&args[0], context)?;
    match value {
        Value::String(s) => Ok(Value::String(s.to_lowercase())),
        _ => Err(ExpressionError::TypeError(
            "lower expects string argument".to_string(),
        )),
    }
}

// 类型检查函数
fn evaluate_type(
    args: &[Expression],
    context: &dyn ExpressionContext,
) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::InvalidArgumentCount("type".to_string()));
    }

    let value = super::evaluator::ExpressionEvaluator::new().evaluate(&args[0], context)?;
    let type_name = match value {
        Value::Null(_) => "NULL",
        Value::Bool(_) => "BOOLEAN",
        Value::Int(_) => "INTEGER",
        Value::Float(_) => "FLOAT",
        Value::String(_) => "STRING",
        Value::List(_) => "LIST",
        Value::Map(_) => "MAP",
        Value::Vertex(_) => "VERTEX",
        Value::Edge(_) => "EDGE",
        Value::Path(_) => "PATH",
        Value::DateTime(_) => "DATETIME",
        Value::Date(_) => "DATE",
        Value::Time(_) => "TIME",
        Value::Geography(_) => "GEOGRAPHY",
        Value::Duration(_) => "DURATION",
        Value::Empty => "EMPTY",
        Value::Set(_) => "SET",
        Value::DataSet(_) => "DataSet",
    };

    Ok(Value::String(type_name.to_string()))
}

fn evaluate_exists(
    args: &[Expression],
    context: &dyn ExpressionContext,
) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::InvalidArgumentCount("exists".to_string()));
    }

    let result = super::evaluator::ExpressionEvaluator::new().evaluate(&args[0], context);
    Ok(Value::Bool(result.is_ok()))
}

// 图数据库特定函数
fn evaluate_id(args: &[Expression], context: &dyn ExpressionContext) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::InvalidArgumentCount("id".to_string()));
    }

    let value = super::evaluator::ExpressionEvaluator::new().evaluate(&args[0], context)?;
    match value {
        Value::Vertex(vertex) => Ok(vertex.id().clone()),
        Value::Edge(edge) => Ok(edge.src().clone()),
        _ => Err(ExpressionError::TypeError(
            "id expects vertex or edge argument".to_string(),
        )),
    }
}

fn evaluate_labels(
    args: &[Expression],
    context: &dyn ExpressionContext,
) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::InvalidArgumentCount("labels".to_string()));
    }

    let value = super::evaluator::ExpressionEvaluator::new().evaluate(&args[0], context)?;
    match value {
        Value::Vertex(vertex) => {
            let labels: Vec<Value> = vertex
                .tags
                .iter()
                .map(|tag| Value::String(tag.name.clone()))
                .collect();
            Ok(Value::List(labels))
        }
        _ => Err(ExpressionError::TypeError(
            "labels expects vertex argument".to_string(),
        )),
    }
}

// 聚合函数实现
fn evaluate_count(
    arg: &Expression,
    _distinct: bool,
    context: &dyn ExpressionContext,
) -> Result<Value, ExpressionError> {
    let value = super::evaluator::ExpressionEvaluator::new().evaluate(arg, context)?;
    match value {
        Value::List(list) => Ok(Value::Int(list.len() as i64)),
        _ => Ok(Value::Int(1)), // 非列表值计数为1
    }
}

fn evaluate_sum(
    arg: &Expression,
    _distinct: bool,
    context: &dyn ExpressionContext,
) -> Result<Value, ExpressionError> {
    let value = super::evaluator::ExpressionEvaluator::new().evaluate(arg, context)?;
    match value {
        Value::List(list) => {
            let mut sum = 0.0;
            for item in list {
                match item {
                    Value::Int(i) => sum += i as f64,
                    Value::Float(f) => sum += f,
                    _ => {
                        return Err(ExpressionError::TypeError(
                            "sum expects numeric list".to_string(),
                        ))
                    }
                }
            }
            Ok(Value::Float(sum))
        }
        Value::Int(i) => Ok(Value::Int(i)),
        Value::Float(f) => Ok(Value::Float(f)),
        _ => Err(ExpressionError::TypeError(
            "sum expects numeric argument".to_string(),
        )),
    }
}

fn evaluate_avg(
    arg: &Expression,
    _distinct: bool,
    context: &dyn ExpressionContext,
) -> Result<Value, ExpressionError> {
    let value = super::evaluator::ExpressionEvaluator::new().evaluate(arg, context)?;
    match value {
        Value::List(list) => {
            if list.is_empty() {
                return Ok(Value::Null(crate::core::NullType::Null));
            }
            let sum = evaluate_sum(arg, false, context)?;
            match sum {
                Value::Float(sum_f) => Ok(Value::Float(sum_f / list.len() as f64)),
                Value::Int(sum_i) => Ok(Value::Float(sum_i as f64 / list.len() as f64)),
                _ => unreachable!(),
            }
        }
        _ => evaluate_sum(arg, false, context), // 单个值的平均值就是其本身
    }
}

fn evaluate_min(
    arg: &Expression,
    _distinct: bool,
    context: &dyn ExpressionContext,
) -> Result<Value, ExpressionError> {
    let value = super::evaluator::ExpressionEvaluator::new().evaluate(arg, context)?;
    match value {
        Value::List(list) => {
            if list.is_empty() {
                return Ok(Value::Null(crate::core::NullType::Null));
            }
            // 简化实现：只支持数值类型
            let mut min = None;
            for item in list {
                match item {
                    Value::Int(i) => {
                        if min.is_none() || min.as_ref().expect("min value should exist") < &Value::Int(i) {
                            min = Some(Value::Int(i));
                        }
                    }
                    Value::Float(f) => {
                        if min.is_none() || min.as_ref().expect("min value should exist") < &Value::Float(f) {
                            min = Some(Value::Float(f));
                        }
                    }
                    _ => {
                        return Err(ExpressionError::TypeError(
                            "min expects numeric list".to_string(),
                        ))
                    }
                }
            }
            min.ok_or_else(|| ExpressionError::TypeError("min: empty list".to_string()))
        }
        _ => Ok(value), // 单个值的最小值就是其本身
    }
}

fn evaluate_max(
    arg: &Expression,
    _distinct: bool,
    context: &dyn ExpressionContext,
) -> Result<Value, ExpressionError> {
    let value = super::evaluator::ExpressionEvaluator::new().evaluate(arg, context)?;
    match value {
        Value::List(list) => {
            if list.is_empty() {
                return Ok(Value::Null(crate::core::NullType::Null));
            }
            // 简化实现：只支持数值类型
            let mut max = None;
            for item in list {
                match item {
                    Value::Int(i) => {
                        if max.is_none() || max.as_ref().expect("max value should exist") > &Value::Int(i) {
                            max = Some(Value::Int(i));
                        }
                    }
                    Value::Float(f) => {
                        if max.is_none() || max.as_ref().expect("max value should exist") > &Value::Float(f) {
                            max = Some(Value::Float(f));
                        }
                    }
                    _ => {
                        return Err(ExpressionError::TypeError(
                            "max expects numeric list".to_string(),
                        ))
                    }
                }
            }
            max.ok_or_else(|| ExpressionError::TypeError("max: empty list".to_string()))
        }
        _ => Ok(value), // 单个值的最大值就是其本身
    }
}

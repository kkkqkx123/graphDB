use super::error::ExpressionError;
use crate::core::Value;
use crate::expression::Expression;
use crate::query::context::EvalContext;
use serde::{Deserialize, Serialize};

// 聚合数据结构，用于累积聚合函数的中间结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggData {
    pub count: i64,
    pub sum: Value,
    pub min: Option<Value>,
    pub max: Option<Value>,
    pub values: Vec<Value>,
}

impl AggData {
    pub fn new() -> Self {
        Self {
            count: 0,
            sum: Value::Int(0),
            min: None,
            max: None,
            values: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.count = 0;
        self.sum = Value::Int(0);
        self.min = None;
        self.max = None;
        self.values.clear();
    }

    pub fn apply(&mut self, value: &Value) {
        self.count += 1;
        self.values.push(value.clone());

        // 更新最小值
        if self.min.as_ref().map_or(true, |min_val| value.lt(min_val)) {
            self.min = Some(value.clone());
        }

        // 更新最大值
        if self.max.as_ref().map_or(true, |max_val| value.gt(max_val)) {
            self.max = Some(value.clone());
        }

        // 更新总和
        if let Value::Int(sum_int) = &self.sum {
            if let Value::Int(val_int) = value {
                self.sum = Value::Int(sum_int + val_int);
            } else if let Value::Float(val_float) = value {
                self.sum = Value::Float(*sum_int as f64 + val_float);
            }
        } else if let Value::Float(sum_float) = &self.sum {
            if let Value::Int(val_int) = value {
                self.sum = Value::Float(*sum_float + *val_int as f64);
            } else if let Value::Float(val_float) = value {
                self.sum = Value::Float(*sum_float + val_float);
            }
        }
    }
}

/// 评估聚合表达式
pub fn evaluate_aggregate_expr(
    func: &str,
    arg: &Expression,
    distinct: bool,
    context: &EvalContext,
) -> Result<Value, ExpressionError> {
    let evaluator = super::evaluator::ExpressionEvaluator;
    let arg_val = evaluator.evaluate(arg, context)?;

    // 在实际实现中，聚合函数通常需要跨多个数据行进行计算
    // 这里我们提供一个简化的实现
    match func.to_lowercase().as_str() {
        "count" => {
            // 如果是计数，直接返回1（表示一个值）
            // 在实际聚合中，这会被累积
            if distinct {
                // 对于去重计数，我们需要检查是否已经存在
                Ok(Value::Int(1))
            } else {
                Ok(Value::Int(1))
            }
        }
        "sum" => {
            // 在实际系统中，这里会累加所有值
            Ok(arg_val)
        }
        "avg" => {
            // 平均值需要计数值和总和
            // 简化实现：返回当前值
            Ok(arg_val)
        }
        "min" => {
            // 在实际系统中，这里会比较并保留最小值
            Ok(arg_val)
        }
        "max" => {
            // 在实际系统中，这里会比较并保留最大值
            Ok(arg_val)
        }
        "collect" => {
            // 收集所有值到列表中
            Ok(Value::List(vec![arg_val]))
        }
        _ => Err(ExpressionError::FunctionError(format!(
            "Unknown aggregate function: {}",
            func
        ))),
    }
}

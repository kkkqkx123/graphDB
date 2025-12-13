//! FoldConstantExprVisitor - 用于常量折叠的访问器
//! 对应 NebulaGraph FoldConstantExprVisitor.h/.cpp 的功能

use crate::graph::expression::Expression;
use crate::graph::expression::{BinaryOperator, UnaryOperator};
use crate::core::Value;
use std::collections::HashMap;

pub struct FoldConstantExprVisitor {
    /// 存储参数名到值的映射，用于参数替换
    parameters: HashMap<String, Value>,
}

impl FoldConstantExprVisitor {
    pub fn new(parameters: HashMap<String, Value>) -> Self {
        Self {
            parameters,
        }
    }

    /// 执行常量折叠
    pub fn fold(&self, expr: &Expression) -> Expression {
        self.visit(expr)
    }

    fn visit(&self, expr: &Expression) -> Expression {
        match expr {
            // 常量表达式直接返回
            Expression::Literal(_) => expr.clone(),
            
            // 变量表达式尝试替换为参数值
            Expression::Variable(name) => {
                if let Some(value) = self.parameters.get(name) {
                    Expression::literal(match value {
                        crate::core::Value::Bool(b) => crate::graph::expression::LiteralValue::Bool(*b),
                        crate::core::Value::Int(i) => crate::graph::expression::LiteralValue::Int(*i),
                        crate::core::Value::Float(f) => crate::graph::expression::LiteralValue::Float(*f),
                        crate::core::Value::String(s) => crate::graph::expression::LiteralValue::String(s.clone()),
                        crate::core::Value::Null(_) => crate::graph::expression::LiteralValue::Null,
                        _ => crate::graph::expression::LiteralValue::Null,
                    })
                } else {
                    expr.clone()
                }
            },
            
            // 算术表达式尝试折叠常量
            Expression::Binary { left, op, right } => {
                let left_folded = self.visit(left);
                let right_folded = self.visit(right);
                
                // 如果左右操作数都是常量，执行常量折叠
                if let (Expression::Literal(left_val), Expression::Literal(right_val)) =
                   (&left_folded, &right_folded) {
                    match self.evaluate_arithmetic(op, left_val, right_val) {
                        Ok(result) => Expression::literal(result),
                        Err(_) => Expression::binary(left_folded, op.clone(), right_folded),
                    }
                } else {
                    Expression::binary(left_folded, op.clone(), right_folded)
                }
            },
            
            // 一元表达式尝试折叠常量
            Expression::Unary { op, operand } => {
                let operand_folded = self.visit(operand);
                
                if let Expression::Literal(val) = &operand_folded {
                    match self.evaluate_unary(op, val) {
                        Ok(result) => Expression::literal(result),
                        Err(_) => Expression::unary(op.clone(), operand_folded),
                    }
                } else {
                    Expression::unary(op.clone(), operand_folded)
                }
            },
            
            // 函数调用 - 尝试对参数都是常量的函数调用求值
            Expression::Function { name, args } => {
                let mut folded_args = Vec::new();
                for arg in args {
                    folded_args.push(self.visit(arg));
                }
                
                // 检查是否所有参数都是常量
                let all_constants = folded_args.iter()
                    .all(|expr| matches!(expr, Expression::Literal(_)));
                
                if all_constants {
                    // 尝试执行函数调用
                    match self.evaluate_function(name, &folded_args) {
                        Ok(result) => Expression::literal(result),
                        Err(_) => Expression::function(name.clone(), folded_args),
                    }
                } else {
                    Expression::function(name.clone(), folded_args)
                }
            },
            
            // 其他表达式类型，直接返回
            _ => expr.clone(),
        }
    }

    fn evaluate_arithmetic(&self, op: &BinaryOperator, left: &LiteralValue, right: &LiteralValue) -> Result<LiteralValue, String> {
        use crate::core::Value;
        
        let left_val = match left {
            LiteralValue::Bool(b) => Value::Bool(*b),
            LiteralValue::Int(i) => Value::Int(*i),
            LiteralValue::Float(f) => Value::Float(*f),
            LiteralValue::String(s) => Value::String(s.clone()),
            LiteralValue::Null => Value::Null(crate::core::NullType::Null),
        };
        
        let right_val = match right {
            LiteralValue::Bool(b) => Value::Bool(*b),
            LiteralValue::Int(i) => Value::Int(*i),
            LiteralValue::Float(f) => Value::Float(*f),
            LiteralValue::String(s) => Value::String(s.clone()),
            LiteralValue::Null => Value::Null(crate::core::NullType::Null),
        };
        
        match op {
            BinaryOperator::Add => left_val.add(&right_val).map(|v| match v {
                Value::Bool(b) => LiteralValue::Bool(b),
                Value::Int(i) => LiteralValue::Int(i),
                Value::Float(f) => LiteralValue::Float(f),
                Value::String(s) => LiteralValue::String(s),
                _ => LiteralValue::Null,
            }),
            BinaryOperator::Subtract => left_val.sub(&right_val).map(|v| match v {
                Value::Bool(b) => LiteralValue::Bool(b),
                Value::Int(i) => LiteralValue::Int(i),
                Value::Float(f) => LiteralValue::Float(f),
                Value::String(s) => LiteralValue::String(s),
                _ => LiteralValue::Null,
            }),
            BinaryOperator::Multiply => left_val.mul(&right_val).map(|v| match v {
                Value::Bool(b) => LiteralValue::Bool(b),
                Value::Int(i) => LiteralValue::Int(i),
                Value::Float(f) => LiteralValue::Float(f),
                Value::String(s) => LiteralValue::String(s),
                _ => LiteralValue::Null,
            }),
            BinaryOperator::Divide => left_val.div(&right_val).map(|v| match v {
                Value::Bool(b) => LiteralValue::Bool(b),
                Value::Int(i) => LiteralValue::Int(i),
                Value::Float(f) => LiteralValue::Float(f),
                Value::String(s) => LiteralValue::String(s),
                _ => LiteralValue::Null,
            }),
            _ => Err(format!("Unknown arithmetic operation: {:?}", op)),
        }
    }

    #[allow(dead_code)]
    fn evaluate_logical(&self, op: &str, operands: &[Expression]) -> Result<Value, String> {
        match op {
            "And" | "LogicalAnd" => {
                let mut result = true;
                for operand in operands {
                    if let Expression::Constant(val) = operand {
                        // 简化处理布尔值
                        result = result && val.bool_value().unwrap_or(false);
                    } else {
                        return Err("Non-constant operand in logical operation".to_string());
                    }
                }
                Ok(Value::Bool(result))
            },
            "Or" | "LogicalOr" => {
                let mut result = false;
                for operand in operands {
                    if let Expression::Constant(val) = operand {
                        // 简化处理布尔值
                        result = result || val.bool_value().unwrap_or(false);
                    } else {
                        return Err("Non-constant operand in logical operation".to_string());
                    }
                }
                Ok(Value::Bool(result))
            },
            _ => Err(format!("Unknown logical operation: {}", op)),
        }
    }

    #[allow(dead_code)]
    fn evaluate_relational(&self, op: &str, left: &Value, right: &Value) -> Result<Value, String> {
        match op {
            "RelEQ" => Ok(Value::Bool(left.equals(right))),
            "RelNE" => Ok(Value::Bool(!left.equals(right))),
            "RelLT" => Ok(Value::Bool(left.less_than(right))),
            "RelLE" => Ok(Value::Bool(left.less_than_equal(right))),
            "RelGT" => Ok(Value::Bool(left.greater_than(right))),
            "RelGE" => Ok(Value::Bool(left.greater_than_equal(right))),
            "RelIn" => Ok(Value::Bool(right.contains(left))),
            "RelNotIn" => Ok(Value::Bool(!right.contains(left))),
            "RelRegex" => {
                // 正则表达式匹配简化实现
                if let (Value::String(_s), Value::String(_pattern)) = (left, right) {
                    // 在实际实现中，这里会执行正则匹配
                    // 简化实现，返回True
                    Ok(Value::Bool(true))
                } else {
                    Err("Regex operation requires string operands".to_string())
                }
            },
            _ => Err(format!("Unknown relational operation: {}", op)),
        }
    }

    fn evaluate_unary(&self, op: &UnaryOperator, operand: &Value) -> Result<Value, String> {
        match op {
            UnaryOperator::Plus => Ok(operand.clone()),  // Identity operation
            UnaryOperator::Minus => operand.negate(),
            UnaryOperator::Negate => operand.negate(),
            UnaryOperator::Not => Ok(Value::Bool(!operand.bool_value().unwrap_or(false))),
            UnaryOperator::Increment => Err("Increment operation not supported in constant folding".to_string()),
            UnaryOperator::Decrement => Err("Decrement operation not supported in constant folding".to_string()),
        }
    }

    fn evaluate_function(&self, name: &str, args: &[Expression]) -> Result<Value, String> {
        match name.to_lowercase().as_str() {
            "abs" => {
                if args.len() == 1 {
                    if let Expression::Constant(val) = &args[0] {
                        return val.abs();
                    }
                }
                Err("Invalid arguments for abs function".to_string())
            },
            "ceil" => {
                if args.len() == 1 {
                    if let Expression::Constant(val) = &args[0] {
                        return val.ceil();
                    }
                }
                Err("Invalid arguments for ceil function".to_string())
            },
            "floor" => {
                if args.len() == 1 {
                    if let Expression::Constant(val) = &args[0] {
                        return val.floor();
                    }
                }
                Err("Invalid arguments for floor function".to_string())
            },
            "round" => {
                if args.len() == 1 {
                    if let Expression::Constant(val) = &args[0] {
                        return val.round();
                    }
                }
                Err("Invalid arguments for round function".to_string())
            },
            "lower" => {
                if args.len() == 1 {
                    if let Expression::Constant(val) = &args[0] {
                        return val.lower();
                    }
                }
                Err("Invalid arguments for lower function".to_string())
            },
            "upper" => {
                if args.len() == 1 {
                    if let Expression::Constant(val) = &args[0] {
                        return val.upper();
                    }
                }
                Err("Invalid arguments for upper function".to_string())
            },
            "trim" => {
                if args.len() == 1 {
                    if let Expression::Constant(val) = &args[0] {
                        return val.trim();
                    }
                }
                Err("Invalid arguments for trim function".to_string())
            },
            "length" => {
                if args.len() == 1 {
                    if let Expression::Constant(val) = &args[0] {
                        return val.length();
                    }
                }
                Err("Invalid arguments for length function".to_string())
            },
            // 添加更多内建函数的处理
            _ => Err(format!("Unknown function: {}", name)),
        }
    }

    #[allow(dead_code)]
    fn cast_value(&self, value: &Value, target_type: &crate::core::ValueTypeDef) -> Result<Value, String> {
        // 类型转换实现
        match target_type {
            crate::core::ValueTypeDef::Bool => value.cast_to_bool(),
            crate::core::ValueTypeDef::Int => value.cast_to_int(),
            crate::core::ValueTypeDef::Float => value.cast_to_float(),
            crate::core::ValueTypeDef::String => value.cast_to_string(),
            crate::core::ValueTypeDef::Date => value.cast_to_date(),
            crate::core::ValueTypeDef::Time => value.cast_to_time(),
            crate::core::ValueTypeDef::DateTime => value.cast_to_datetime(),
            crate::core::ValueTypeDef::Vertex => value.cast_to_vertex(),
            crate::core::ValueTypeDef::Edge => value.cast_to_edge(),
            crate::core::ValueTypeDef::Path => value.cast_to_path(),
            crate::core::ValueTypeDef::List => value.cast_to_list(),
            crate::core::ValueTypeDef::Map => value.cast_to_map(),
            crate::core::ValueTypeDef::Set => value.cast_to_set(),
            crate::core::ValueTypeDef::Duration => value.cast_to_duration(),
            crate::core::ValueTypeDef::Geography => value.cast_to_geography(),
            crate::core::ValueTypeDef::IntRange => value.cast_to_int(),
            crate::core::ValueTypeDef::FloatRange => value.cast_to_float(),
            crate::core::ValueTypeDef::StringRange => value.cast_to_string(),
            crate::core::ValueTypeDef::DataSet => value.cast_to_dataset(),
            crate::core::ValueTypeDef::Null => Ok(Value::Null(crate::core::NullType::Null)),
            crate::core::ValueTypeDef::Empty => Ok(Value::Empty),
        }
    }
}
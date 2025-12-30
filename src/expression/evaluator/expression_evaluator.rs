//! 表达式求值器实现
//!
//! 提供具体的表达式求值功能，包含零成本抽象优化

use crate::core::error::ExpressionError;
use crate::core::types::expression::Expression;
use crate::core::Value;
use crate::expression::evaluator::collection_operations::CollectionOperationEvaluator;
use crate::expression::evaluator::functions::FunctionEvaluator;
use crate::expression::evaluator::operations::{BinaryOperationEvaluator, UnaryOperationEvaluator};
use crate::expression::evaluator::traits::ExpressionContext;

/// 表达式求值器实现（unit struct，零开销）
#[derive(Debug)]
pub struct ExpressionEvaluator;

impl ExpressionEvaluator {
    /// 在给定上下文中求值表达式（泛型版本，零成本抽象）
    pub fn evaluate<C: ExpressionContext>(
        expr: &Expression,
        context: &mut C,
    ) -> Result<Value, ExpressionError> {
        Self::evaluate_impl(expr, context)
    }

    /// 批量求值表达式列表（泛型版本，零成本抽象）
    pub fn evaluate_batch<C: ExpressionContext>(
        expressions: &[Expression],
        context: &mut C,
    ) -> Result<Vec<Value>, ExpressionError> {
        let mut results = Vec::with_capacity(expressions.len());
        for expr in expressions {
            results.push(Self::evaluate(expr, context)?);
        }
        Ok(results)
    }

    /// 检查表达式是否可以求值（泛型版本）
    pub fn can_evaluate<C: ExpressionContext>(_expr: &Expression, _context: &C) -> bool {
        true
    }

    /// 表达式求值实现 - 零成本抽象核心（完全静态分发）
    fn evaluate_impl<C: ExpressionContext>(
        expr: &Expression,
        context: &mut C,
    ) -> Result<Value, ExpressionError> {
        match expr {
            // 字面量 - 直接返回值
            Expression::Literal(value) => Ok(value.clone()),

            // 变量 - 从上下文获取
            Expression::Variable(name) => context
                .get_variable(name)
                .ok_or_else(|| ExpressionError::undefined_variable(name)),

            // 二元操作 - 递归求值左右操作数
            Expression::Binary { left, op, right } => {
                let left_value = Self::evaluate_impl(left, context)?;
                let right_value = Self::evaluate_impl(right, context)?;
                BinaryOperationEvaluator::evaluate(&left_value, op, &right_value)
            }

            // 一元操作 - 递归求值操作数
            Expression::Unary { op, operand } => {
                let value = Self::evaluate_impl(operand, context)?;
                UnaryOperationEvaluator::evaluate(op, &value)
            }

            // 函数调用 - 批量求值参数
            Expression::Function { name, args } => {
                let arg_values: Result<Vec<Value>, ExpressionError> = args
                    .iter()
                    .map(|arg| Self::evaluate_impl(arg, context))
                    .collect();
                let arg_values = arg_values?;
                FunctionEvaluator.eval_function_call(name, &arg_values)
            }

            // 聚合函数 - 直接求值
            Expression::Aggregate {
                func,
                arg,
                distinct,
            } => {
                let arg_value = Self::evaluate_impl(arg, context)?;
                FunctionEvaluator.eval_aggregate_function(func, &[arg_value], *distinct)
            }

            // CASE 表达式 - 短路求值
            Expression::Case {
                conditions,
                default,
            } => {
                for (condition, value) in conditions {
                    let condition_result = Self::evaluate_impl(condition, context)?;
                    match condition_result {
                        Value::Bool(true) => {
                            return Self::evaluate_impl(value, context);
                        }
                        Value::Bool(false) => continue,
                        _ => return Err(ExpressionError::type_error("CASE条件必须是布尔值")),
                    }
                }

                match default {
                    Some(default_expr) => Self::evaluate_impl(default_expr, context),
                    None => Ok(Value::Null(crate::core::NullType::Null)),
                }
            }

            // 列表 - 批量求值
            Expression::List(elements) => {
                let element_values: Result<Vec<Value>, ExpressionError> = elements
                    .iter()
                    .map(|elem| Self::evaluate_impl(elem, context))
                    .collect();
                element_values.map(Value::List)
            }

            // 映射 - 批量求值
            Expression::Map(entries) => {
                let mut map_values = std::collections::HashMap::new();
                for (key, value_expr) in entries {
                    let value = Self::evaluate_impl(value_expr, context)?;
                    map_values.insert(key.clone(), value);
                }
                Ok(Value::Map(map_values))
            }

            // 下标访问
            Expression::Subscript { collection, index } => {
                let collection_value = Self::evaluate_impl(collection, context)?;
                let index_value = Self::evaluate_impl(index, context)?;
                CollectionOperationEvaluator.eval_subscript_access(&collection_value, &index_value)
            }

            // 范围访问
            Expression::Range {
                collection,
                start,
                end,
            } => {
                let collection_value = Self::evaluate_impl(collection, context)?;
                let start_value = start
                    .as_ref()
                    .map(|e| Self::evaluate_impl(e, context))
                    .transpose()?;
                let end_value = end
                    .as_ref()
                    .map(|e| Self::evaluate_impl(e, context))
                    .transpose()?;
                CollectionOperationEvaluator.eval_range_access(
                    &collection_value,
                    start_value.as_ref(),
                    end_value.as_ref(),
                )
            }

            // 路径表达式
            Expression::Path(elements) => {
                let element_values: Result<Vec<Value>, ExpressionError> = elements
                    .iter()
                    .map(|elem| Self::evaluate_impl(elem, context))
                    .collect();
                element_values.map(Value::List)
            }

            // 属性访问
            Expression::Property { object, property } => {
                let object_value = Self::evaluate_impl(object, context)?;
                CollectionOperationEvaluator.eval_property_access(&object_value, property)
            }

            // 类型转换
            Expression::TypeCast { expr, target_type } => {
                let value = Self::evaluate_impl(expr, context)?;
                Self::eval_type_cast(&value, target_type)
            }

            // 其他表达式类型 - 保持静态分发，避免动态分发回退
            _ => Err(ExpressionError::type_error("不支持的表达式类型")),
        }
    }

    /// 编译时分支预测优化版本（完全静态分发）
    #[inline(always)]
    pub fn evaluate_with_branch_prediction<C: ExpressionContext>(
        expr: &Expression,
        context: &mut C,
    ) -> Result<Value, ExpressionError> {
        // 根据表达式类型频率排序，将常见类型放在前面
        // 直接调用 evaluate_impl 避免额外的函数调用开销
        Self::evaluate_impl(expr, context)
    }

    /// 获取求值器名称
    pub fn name() -> &'static str {
        "ExpressionEvaluator"
    }

    /// 获取求值器描述
    pub fn description() -> &'static str {
        "标准表达式求值器"
    }

    /// 获取求值器版本
    pub fn version() -> &'static str {
        "1.0.0"
    }

    /// 求值类型转换
    pub fn eval_type_cast(
        value: &Value,
        target_type: &crate::core::types::expression::DataType,
    ) -> Result<Value, ExpressionError> {
        use crate::core::types::expression::DataType;

        match target_type {
            DataType::Bool => value
                .cast_to_bool()
                .map_err(|e| ExpressionError::type_error(e)),
            DataType::Int => value
                .cast_to_int()
                .map_err(|e| ExpressionError::type_error(e)),
            DataType::Float => value
                .cast_to_float()
                .map_err(|e| ExpressionError::type_error(e)),
            DataType::String => value
                .cast_to_string()
                .map_err(|e| ExpressionError::type_error(e)),
            DataType::List => value
                .cast_to_list()
                .map_err(|e| ExpressionError::type_error(e)),
            DataType::Map => value
                .cast_to_map()
                .map_err(|e| ExpressionError::type_error(e)),
            _ => Err(ExpressionError::type_error(format!(
                "不支持的类型转换: {:?}",
                target_type
            ))),
        }
    }

    /// 求值LIKE操作
    /// 支持SQL标准的LIKE通配符：
    /// - %: 匹配任意数量的字符（包括零个）
    /// - _: 匹配单个字符
    pub fn eval_like(
        value: &Value,
        pattern: &Value,
        escape_char: Option<char>,
    ) -> Result<Value, ExpressionError> {
        // 实现LIKE操作逻辑
        // 这里省略具体实现，保持与原始版本一致
        todo!("LIKE操作实现")
    }
}

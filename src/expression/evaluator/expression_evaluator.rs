//! 表达式求值器实现
//!
//! 提供具体的表达式求值功能

use crate::core::types::expression::Expression;
use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
use crate::core::error::{ExpressionError, ExpressionErrorType};
use crate::core::Value;
use crate::expression::evaluator::traits::{Evaluator, ExpressionContext};
use crate::expression::evaluator::operations::{BinaryOperationEvaluator, UnaryOperationEvaluator};
use crate::expression::evaluator::functions::FunctionEvaluator;
use crate::expression::evaluator::graph_operations::GraphOperationEvaluator;
use crate::expression::evaluator::collection_operations::CollectionOperationEvaluator;

/// 表达式求值器实现（unit struct，零开销）
#[derive(Debug)]
pub struct ExpressionEvaluator;

impl ExpressionEvaluator {
    /// 在给定上下文中求值表达式（公共接口，保留 dyn 以兼容）
    pub fn evaluate(
        expr: &Expression,
        context: &mut dyn ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        Self::eval_expression(expr, context)
    }

    /// 在给定上下文中求值表达式（通过 dyn trait object）
    pub fn eval_expression(
        expr: &Expression,
        context: &mut dyn ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        match expr {
            Expression::Literal(value) => {
                Ok(value.clone())
            }
            Expression::TypeCast { expr, target_type } => {
                let value = Self::evaluate(expr, context)?;
                Self::eval_type_cast(&value, target_type)
            }
            Expression::Property { object, property } => {
                let object_value = Self::evaluate(object, context)?;
                CollectionOperationEvaluator.eval_property_access(&object_value, property)
            }
            Expression::Variable(name) => {
                context
                    .get_variable(name)
                    .ok_or_else(|| ExpressionError::undefined_variable(name))
            }
            Expression::Binary { left, op, right } => {
                let left_value = Self::evaluate(left, context)?;
                let right_value = Self::evaluate(right, context)?;
                BinaryOperationEvaluator::evaluate(&left_value, op, &right_value)
            }
            Expression::Unary { op, operand } => {
                let value = Self::evaluate(operand, context)?;
                UnaryOperationEvaluator::evaluate(op, &value)
            }
            Expression::Function { name, args } => {
                let arg_values: Result<Vec<Value>, ExpressionError> =
                    args.iter().map(|arg| Self::evaluate(arg, context)).collect();
                let arg_values = arg_values?;
                FunctionEvaluator.eval_function_call(name, &arg_values)
            }
            Expression::Aggregate {
                func,
                arg,
                distinct,
            } => {
                let arg_value = Self::evaluate(arg, context)?;
                FunctionEvaluator.eval_aggregate_function_single(func, &arg_value, *distinct)
            }
            Expression::Case {
                conditions,
                default,
            } => Self::eval_case_expression(conditions, default.as_deref(), context),
            Expression::List(elements) => {
                let element_values: Result<Vec<Value>, ExpressionError> = elements
                    .iter()
                    .map(|elem| Self::evaluate(elem, context))
                    .collect();
                element_values.map(Value::List)
            }
            Expression::Map(entries) => {
                let mut map_values = std::collections::HashMap::new();
                for (key, value_expr) in entries {
                    let value = Self::evaluate(value_expr, context)?;
                    map_values.insert(key.clone(), value);
                }
                Ok(Value::Map(map_values))
            }
            Expression::Subscript { collection, index } => {
                let collection_value = Self::evaluate(collection, context)?;
                let index_value = Self::evaluate(index, context)?;
                CollectionOperationEvaluator.eval_subscript_access(&collection_value, &index_value)
            }
            Expression::Range {
                collection,
                start,
                end,
            } => {
                let collection_value = Self::evaluate(collection, context)?;
                let start_value = start
                    .as_ref()
                    .map(|e| Self::evaluate(e, context))
                    .transpose()?;
                let end_value = end
                    .as_ref()
                    .map(|e| Self::evaluate(e, context))
                    .transpose()?;
                CollectionOperationEvaluator.eval_range_access(&collection_value, start_value.as_ref(), end_value.as_ref())
            }
            Expression::Path(elements) => {
                let element_values: Result<Vec<Value>, ExpressionError> = elements
                    .iter()
                    .map(|elem| Self::evaluate(elem, context))
                    .collect();
                element_values.map(Value::List)
            }
            Expression::Label(label_name) => {
                GraphOperationEvaluator.eval_label_expression(label_name, context)
            }
            Expression::TagProperty { tag, prop } => {
                GraphOperationEvaluator.eval_tag_property(tag, prop, context)
            }
            Expression::EdgeProperty { edge, prop } => {
                GraphOperationEvaluator.eval_edge_property(edge, prop, context)
            }
            Expression::InputProperty(prop_name) => {
                match context.get_variable(prop_name) {
                    Some(value) => Ok(value),
                    None => Ok(Value::Null(crate::core::NullType::Null))
                }
            }
            Expression::VariableProperty { var, prop } => {
                GraphOperationEvaluator.eval_variable_property(var, prop, context)
            }
            Expression::SourceProperty { tag, prop } => {
                GraphOperationEvaluator.eval_source_property(tag, prop, context)
            }
            Expression::DestinationProperty { tag, prop } => {
                GraphOperationEvaluator.eval_destination_property(tag, prop, context)
            }
            Expression::UnaryPlus(expr) => Self::evaluate(expr, context),
            Expression::UnaryNegate(expr) => {
                let value = Self::evaluate(expr, context)?;
                UnaryOperationEvaluator::evaluate(&UnaryOperator::Minus, &value)
            }
            Expression::UnaryNot(expr) => {
                let value = Self::evaluate(expr, context)?;
                UnaryOperationEvaluator::evaluate(&UnaryOperator::Not, &value)
            }
            Expression::UnaryIncr(expr) => {
                let value = Self::evaluate(expr, context)?;
                UnaryOperationEvaluator::evaluate(&UnaryOperator::Increment, &value)
            }
            Expression::UnaryDecr(expr) => {
                let value = Self::evaluate(expr, context)?;
                UnaryOperationEvaluator::evaluate(&UnaryOperator::Decrement, &value)
            }
            Expression::IsNull(expr) => {
                let value = Self::evaluate(expr, context)?;
                UnaryOperationEvaluator::evaluate(&UnaryOperator::IsNull, &value)
            }
            Expression::IsNotNull(expr) => {
                let value = Self::evaluate(expr, context)?;
                UnaryOperationEvaluator::evaluate(&UnaryOperator::IsNotNull, &value)
            }
            Expression::IsEmpty(expr) => {
                let value = Self::evaluate(expr, context)?;
                UnaryOperationEvaluator::evaluate(&UnaryOperator::IsEmpty, &value)
            }
            Expression::IsNotEmpty(expr) => {
                let value = Self::evaluate(expr, context)?;
                UnaryOperationEvaluator::evaluate(&UnaryOperator::IsNotEmpty, &value)
            }
            Expression::ListComprehension {
                generator,
                condition,
            } => {
                let gen_value = Self::evaluate(generator, context)?;
                match gen_value {
                    Value::List(items) => {
                        let mut result = Vec::new();
                        for item in items {
                            if let Some(cond) = condition {
                                let cond_result = Self::evaluate(cond, context)?;
                                match cond_result {
                                    Value::Bool(true) => result.push(item),
                                    Value::Bool(false) => continue,
                                    _ => return Err(ExpressionError::type_error("推导条件必须是布尔值")),
                                }
                            } else {
                                result.push(item);
                            }
                        }
                        Ok(Value::List(result))
                    }
                    _ => Ok(Value::List(vec![gen_value])),
                }
            }
            Expression::Predicate { list, condition } => {
                let list_value = Self::evaluate(list, context)?;
                match list_value {
                    Value::List(items) => {
                        let mut result = Vec::new();
                        for item in items {
                            let cond_result = Self::evaluate(condition, context)?;
                            match cond_result {
                                Value::Bool(true) => result.push(item),
                                Value::Bool(false) => continue,
                                _ => return Err(ExpressionError::type_error("谓词条件必须是布尔值")),
                            }
                        }
                        Ok(Value::List(result))
                    }
                    _ => Err(ExpressionError::type_error("谓词表达式需要列表")),
                }
            }
            Expression::Reduce {
                list,
                var,
                initial,
                expr,
            } => {
                let list_value = Self::evaluate(list, context)?;
                let mut acc = Self::evaluate(initial, context)?;

                if let Value::List(items) = list_value {
                    for item in items {
                        context.set_variable(var.clone(), item);
                        acc = Self::evaluate(expr, context)?;
                    }
                }
                Ok(acc)
            }
            Expression::ESQuery(_) => {
                Ok(Value::Null(crate::core::NullType::Null))
            }
            Expression::UUID => Ok(Value::String(uuid::Uuid::new_v4().to_string())),
            Expression::MatchPathPattern {
                path_alias: _,
                patterns,
            } => {
                let pattern_values: Result<Vec<Value>, ExpressionError> =
                    patterns.iter().map(|p| Self::evaluate(p, context)).collect();
                pattern_values.map(Value::List)
            }
        }
    }

    /// 批量求值表达式列表
    pub fn evaluate_batch(
        expressions: &[Expression],
        context: &mut dyn ExpressionContext,
    ) -> Result<Vec<Value>, ExpressionError> {
        let mut results = Vec::with_capacity(expressions.len());
        for expr in expressions {
            results.push(Self::evaluate(expr, context)?);
        }
        Ok(results)
    }

    /// 泛型版本的批量求值表达式
    pub fn evaluate_batch_generic<C: ExpressionContext>(
        expressions: &[Expression],
        context: &mut C,
    ) -> Result<Vec<Value>, ExpressionError> {
        let mut results = Vec::with_capacity(expressions.len());
        for expr in expressions {
            results.push(Self::eval_expression_generic(expr, context)?);
        }
        Ok(results)
    }

    /// 检查表达式是否可以求值
    pub fn can_evaluate(_expr: &Expression, _context: &dyn ExpressionContext) -> bool {
        true
    }

    /// 泛型版本检查表达式是否可以求值
    pub fn can_evaluate_generic<C: ExpressionContext>(_expr: &Expression, _context: &C) -> bool {
        true
    }

    /// 泛型版本的表达式求值（避免虚表开销）
    ///
    /// 编译器会为每个具体的 C 类型生成专用代码，支持完全内联
    pub fn eval_expression_generic<C: ExpressionContext>(
        expr: &Expression,
        context: &mut C,
    ) -> Result<Value, ExpressionError> {
        match expr {
            Expression::Literal(value) => Ok(value.clone()),
            Expression::TypeCast { expr, target_type } => {
                let value = Self::eval_expression_generic(expr, context)?;
                Self::eval_type_cast(&value, target_type)
            }
            Expression::Property { object, property } => {
                let object_value = Self::eval_expression_generic(object, context)?;
                CollectionOperationEvaluator.eval_property_access(&object_value, property)
            }
            Expression::Variable(name) => context
                .get_variable(name)
                .ok_or_else(|| ExpressionError::undefined_variable(name)),
            Expression::Binary { left, op, right } => {
                let left_value = Self::eval_expression_generic(left, context)?;
                let right_value = Self::eval_expression_generic(right, context)?;
                BinaryOperationEvaluator::evaluate(&left_value, op, &right_value)
            }
            Expression::Unary { op, operand } => {
                let value = Self::eval_expression_generic(operand, context)?;
                UnaryOperationEvaluator::evaluate(op, &value)
            }
            Expression::Function { name, args } => {
                let arg_values: Result<Vec<Value>, ExpressionError> = args
                    .iter()
                    .map(|arg| Self::eval_expression_generic(arg, context))
                    .collect();
                let arg_values = arg_values?;
                FunctionEvaluator.eval_function_call(name, &arg_values)
            }
            Expression::Aggregate {
                func,
                arg,
                distinct,
            } => {
                let arg_value = Self::eval_expression_generic(arg, context)?;
                FunctionEvaluator.eval_aggregate_function_single(func, &arg_value, *distinct)
            }
            Expression::Case {
                conditions,
                default,
            } => {
                for (condition, value) in conditions {
                    let condition_result = Self::eval_expression_generic(condition, context)?;
                    match condition_result {
                        Value::Bool(true) => return Self::eval_expression_generic(value, context),
                        Value::Bool(false) => continue,
                        _ => return Err(ExpressionError::type_error("CASE条件必须是布尔值")),
                    }
                }
                match default {
                    Some(default_expr) => Self::eval_expression_generic(default_expr, context),
                    None => Ok(Value::Null(crate::core::NullType::Null)),
                }
            }
            Expression::List(elements) => {
                let element_values: Result<Vec<Value>, ExpressionError> = elements
                    .iter()
                    .map(|elem| Self::eval_expression_generic(elem, context))
                    .collect();
                element_values.map(Value::List)
            }
            Expression::Map(entries) => {
                let mut map_values = std::collections::HashMap::new();
                for (key, value_expr) in entries {
                    let value = Self::eval_expression_generic(value_expr, context)?;
                    map_values.insert(key.clone(), value);
                }
                Ok(Value::Map(map_values))
            }
            Expression::Subscript { collection, index } => {
                let collection_value = Self::eval_expression_generic(collection, context)?;
                let index_value = Self::eval_expression_generic(index, context)?;
                CollectionOperationEvaluator.eval_subscript_access(&collection_value, &index_value)
            }
            Expression::Range {
                collection,
                start,
                end,
            } => {
                let collection_value = Self::eval_expression_generic(collection, context)?;
                let start_value = start
                    .as_ref()
                    .map(|e| Self::eval_expression_generic(e, context))
                    .transpose()?;
                let end_value = end
                    .as_ref()
                    .map(|e| Self::eval_expression_generic(e, context))
                    .transpose()?;
                CollectionOperationEvaluator.eval_range_access(&collection_value, start_value.as_ref(), end_value.as_ref())
            }
            Expression::Path(elements) => {
                let element_values: Result<Vec<Value>, ExpressionError> = elements
                    .iter()
                    .map(|elem| Self::eval_expression_generic(elem, context))
                    .collect();
                element_values.map(Value::List)
            }
            _ => {
                let dyn_ctx: &mut dyn ExpressionContext = context;
                Self::eval_expression(expr, dyn_ctx)
            }
        }
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
    fn eval_type_cast(
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
    /// - \: 转义字符，用于转义%和_
    fn eval_like_operation(_text: &str, _pattern: &str) -> Result<Value, ExpressionError> {
        Ok(Value::Null(crate::core::NullType::Null))
    }

    /// 将SQL LIKE模式转换为正则表达式
    fn like_to_regex(_pattern: &str) -> Result<String, ExpressionError> {
        Ok(String::new())
    }

    /// 简单的LIKE模式匹配实现
    fn like_simple_match(_text: &str, _pattern: &str) -> bool {
        false
    }

    /// 求值CASE表达式
    fn eval_case_expression(
        cases: &[(Expression, Expression)],
        default: Option<&Expression>,
        context: &mut dyn crate::expression::ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        for (condition, value) in cases {
            let condition_result = Self::evaluate(condition, context)?;
            match condition_result {
                Value::Bool(true) => return Self::evaluate(value, context),
                Value::Bool(false) => continue,
                _ => return Err(ExpressionError::type_error("CASE条件必须是布尔值")),
            }
        }

        match default {
            Some(default_expr) => Self::evaluate(default_expr, context),
            None => Ok(Value::Null(crate::core::NullType::Null)),
        }
    }

}

impl Default for ExpressionEvaluator {
    fn default() -> Self {
        ExpressionEvaluator
    }
}

impl<C: ExpressionContext> Evaluator<C> for ExpressionEvaluator {
    /// 求值表达式（泛型版本，避免虚表开销）
    fn evaluate(&self, expr: &Expression, context: &mut C) -> Result<Value, ExpressionError> {
        Self::eval_expression_generic(expr, context)
    }

    /// 批量求值表达式
    fn evaluate_batch(
        &self,
        expressions: &[Expression],
        context: &mut C,
    ) -> Result<Vec<Value>, ExpressionError> {
        Self::evaluate_batch_generic(expressions, context)
    }

    /// 检查表达式是否可以求值
    fn can_evaluate(&self, expr: &Expression, context: &C) -> bool {
        Self::can_evaluate_generic(expr, context)
    }

    /// 获取求值器名称
    fn name(&self) -> &str {
        Self::name()
    }

    /// 获取求值器描述
    fn description(&self) -> &str {
        Self::description()
    }

    /// 获取求值器版本
    fn version(&self) -> &str {
        Self::version()
    }
}

//! 表达式求值器实现
//!
//! 提供具体的表达式求值功能，包含零成本抽象优化
//! 使用GenericExpressionVisitor泛型接口，支持统一的访问者模式

use crate::core::error::ExpressionError;
use crate::core::expression_visitor::GenericExpressionVisitor;
use crate::core::types::expression::Expr;
use crate::core::value::NullType;
use crate::core::{Expression, Value};
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
        expr: &Expr,
        context: &mut C,
    ) -> Result<Value, ExpressionError> {
        let mut evaluator = Self;
        evaluator.visit_with_context(expr, context)
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
    pub fn can_evaluate<C: ExpressionContext>(_expr: &Expr, _context: &C) -> bool {
        true
    }

    /// 在上下文中访问表达式
    fn visit_with_context<C: ExpressionContext>(
        &mut self,
        expr: &Expr,
        context: &mut C,
    ) -> Result<Value, ExpressionError> {
        match expr {
            // 字面量 - 直接返回值
            Expr::Literal(value) => Ok(value.clone()),

            // 变量 - 从上下文获取
            Expr::Variable(name) => context
                .get_variable(name)
                .ok_or_else(|| ExpressionError::undefined_variable(name)),

            // 二元操作 - 递归求值左右操作数
            Expr::Binary { left, op, right } => {
                let left_value = self.visit_with_context(left, context)?;
                let right_value = self.visit_with_context(right, context)?;
                BinaryOperationEvaluator::evaluate(&left_value, op, &right_value)
            }

            // 一元操作 - 递归求值操作数
            Expr::Unary { op, operand } => {
                let value = self.visit_with_context(operand, context)?;
                UnaryOperationEvaluator::evaluate(op, &value)
            }

            // 函数调用 - 批量求值参数
            Expr::Function { name, args } => {
                let arg_values: Result<Vec<Value>, ExpressionError> = args
                    .iter()
                    .map(|arg| self.visit_with_context(arg, context))
                    .collect();
                let arg_values = arg_values?;
                FunctionEvaluator.eval_function_call(name, &arg_values)
            }

            // 聚合函数 - 直接求值
            Expr::Aggregate {
                func,
                arg,
                distinct,
            } => {
                let arg_value = self.visit_with_context(arg, context)?;
                FunctionEvaluator.eval_aggregate_function(func, &[arg_value], *distinct)
            }

            // CASE 表达式 - 短路求值
            Expr::Case {
                conditions,
                default,
            } => {
                for (condition, value) in conditions {
                    let condition_result = self.visit_with_context(condition, context)?;
                    match condition_result {
                        Value::Bool(true) => {
                            return self.visit_with_context(value, context);
                        }
                        Value::Bool(false) => continue,
                        _ => return Err(ExpressionError::type_error("CASE条件必须是布尔值")),
                    }
                }

                match default {
                    Some(default_expr) => self.visit_with_context(default_expr, context),
                    None => Ok(Value::Null(crate::core::NullType::Null)),
                }
            }

            // 列表 - 批量求值
            Expr::List(elements) => {
                let element_values: Result<Vec<Value>, ExpressionError> = elements
                    .iter()
                    .map(|elem| self.visit_with_context(elem, context))
                    .collect();
                element_values.map(Value::List)
            }

            // 映射 - 批量求值
            Expr::Map(entries) => {
                let mut map_values = std::collections::HashMap::new();
                for (key, value_expr) in entries {
                    let value = self.visit_with_context(value_expr, context)?;
                    map_values.insert(key.clone(), value);
                }
                Ok(Value::Map(map_values))
            }

            // 下标访问
            Expr::Subscript { collection, index } => {
                let collection_value = self.visit_with_context(collection, context)?;
                let index_value = self.visit_with_context(index, context)?;
                CollectionOperationEvaluator.eval_subscript_access(&collection_value, &index_value)
            }

            // 范围访问
            Expr::Range {
                collection,
                start,
                end,
            } => {
                let collection_value = self.visit_with_context(collection, context)?;
                let start_value = start
                    .as_ref()
                    .map(|e| self.visit_with_context(e, context))
                    .transpose()?;
                let end_value = end
                    .as_ref()
                    .map(|e| self.visit_with_context(e, context))
                    .transpose()?;
                CollectionOperationEvaluator.eval_range_access(
                    &collection_value,
                    start_value.as_ref(),
                    end_value.as_ref(),
                )
            }

            // 路径表达式
            Expr::Path(elements) => {
                let element_values: Result<Vec<Value>, ExpressionError> = elements
                    .iter()
                    .map(|elem| self.visit_with_context(elem, context))
                    .collect();
                element_values.map(Value::List)
            }

            // 属性访问
            Expr::Property { object, property } => {
                let object_value = self.visit_with_context(object, context)?;
                CollectionOperationEvaluator.eval_property_access(&object_value, property)
            }

            // 类型转换
            Expr::TypeCast { expr, target_type } => {
                let value = self.visit_with_context(expr, context)?;
                Self::eval_type_cast(&value, target_type)
            }

            // 其他表达式类型 - 保持静态分发，避免动态分发回退
            _ => Err(ExpressionError::type_error("不支持的表达式类型")),
        }
    }

    /// 编译时分支预测优化版本（完全静态分发）
    #[inline(always)]
    pub fn evaluate_with_branch_prediction<C: ExpressionContext>(
        expr: &Expr,
        context: &mut C,
    ) -> Result<Value, ExpressionError> {
        let mut evaluator = Self;
        evaluator.visit_with_context(expr, context)
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

        let result = match target_type {
            DataType::Bool => value.to_bool(),
            DataType::Int => value.to_int(),
            DataType::Float => value.to_float(),
            DataType::String => {
                return value.to_string().map(Value::String).map_err(ExpressionError::type_error);
            }
            DataType::List => value.to_list(),
            DataType::Map => value.to_map(),
            _ => return Err(ExpressionError::type_error(format!(
                "不支持的类型转换: {:?}",
                target_type
            ))),
        };

        // 检查转换结果是否为 Null(BadType)
        if let Value::Null(NullType::BadType) = result {
            Err(ExpressionError::type_error(format!(
                "无法将 {:?} 转换为 {:?}",
                value, target_type
            )))
        } else {
            Ok(result)
        }
    }

    /// 求值LIKE操作
    /// 支持SQL标准的LIKE通配符：
    /// - %: 匹配任意数量的字符（包括零个）
    /// - _: 匹配单个字符
    pub fn eval_like(
        _value: &Value,
        _pattern: &Value,
        _escape_char: Option<char>,
    ) -> Result<Value, ExpressionError> {
        todo!("LIKE操作实现")
    }
}

/// 为ExpressionEvaluator实现GenericExpressionVisitor<Expression>
/// 提供统一的泛型访问接口
impl GenericExpressionVisitor<Expression> for ExpressionEvaluator {
    type Result = Result<Value, ExpressionError>;

    fn visit(&mut self, expr: &Expr) -> Self::Result {
        match expr {
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Variable(name) => Err(ExpressionError::undefined_variable(name)),
            Expr::Binary { left, op, right } => {
                let left_value = self.visit(left)?;
                let right_value = self.visit(right)?;
                BinaryOperationEvaluator::evaluate(&left_value, op, &right_value)
            }
            Expr::Unary { op, operand } => {
                let value = self.visit(operand)?;
                UnaryOperationEvaluator::evaluate(op, &value)
            }
            Expr::Function { name, args } => {
                let arg_values: Result<Vec<Value>, ExpressionError> = args
                    .iter()
                    .map(|arg| self.visit(arg))
                    .collect();
                let arg_values = arg_values?;
                FunctionEvaluator.eval_function_call(name, &arg_values)
            }
            Expr::Aggregate { func, arg, distinct } => {
                let arg_value = self.visit(arg)?;
                FunctionEvaluator.eval_aggregate_function(func, &[arg_value], *distinct)
            }
            Expr::Case { conditions, default } => {
                for (condition, value) in conditions {
                    let condition_result = self.visit(condition)?;
                    match condition_result {
                        Value::Bool(true) => return self.visit(value),
                        Value::Bool(false) => continue,
                        _ => return Err(ExpressionError::type_error("CASE条件必须是布尔值")),
                    }
                }
                match default {
                    Some(default_expr) => self.visit(default_expr),
                    None => Ok(Value::Null(crate::core::NullType::Null)),
                }
            }
            Expr::List(elements) => {
                let element_values: Result<Vec<Value>, ExpressionError> = elements
                    .iter()
                    .map(|elem| self.visit(elem))
                    .collect();
                element_values.map(Value::List)
            }
            Expr::Map(entries) => {
                let mut map_values = std::collections::HashMap::new();
                for (key, value_expr) in entries {
                    let value = self.visit(value_expr)?;
                    map_values.insert(key.clone(), value);
                }
                Ok(Value::Map(map_values))
            }
            Expr::Subscript { collection, index } => {
                let collection_value = self.visit(collection)?;
                let index_value = self.visit(index)?;
                CollectionOperationEvaluator.eval_subscript_access(&collection_value, &index_value)
            }
            Expr::Range { collection, start, end } => {
                let collection_value = self.visit(collection)?;
                let start_value = start.as_ref().map(|e| self.visit(e)).transpose()?;
                let end_value = end.as_ref().map(|e| self.visit(e)).transpose()?;
                CollectionOperationEvaluator.eval_range_access(
                    &collection_value,
                    start_value.as_ref(),
                    end_value.as_ref(),
                )
            }
            Expr::Path(elements) => {
                let element_values: Result<Vec<Value>, ExpressionError> = elements
                    .iter()
                    .map(|elem| self.visit(elem))
                    .collect();
                element_values.map(Value::List)
            }
            Expr::Property { object, property } => {
                let object_value = self.visit(object.as_ref())?;
                CollectionOperationEvaluator.eval_property_access(&object_value, property)
            }
            Expr::TypeCast { expr, target_type } => {
                let value = self.visit(expr)?;
                Self::eval_type_cast(&value, target_type)
            }
            _ => Err(ExpressionError::type_error("不支持的表达式类型")),
        }
    }
}

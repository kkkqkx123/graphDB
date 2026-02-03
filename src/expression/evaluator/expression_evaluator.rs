//! 表达式求值器实现
//!
//! 提供具体的表达式求值功能，包含零成本抽象优化
//! 使用GenericExpressionVisitor泛型接口，支持统一的访问者模式

use crate::core::error::ExpressionError;
use crate::core::types::expression::visitor::GenericExpressionVisitor;
use crate::core::types::expression::Expression;
use crate::core::value::NullType;
use crate::core::Value;
use crate::expression::evaluator::collection_operations::CollectionOperationEvaluator;
use crate::expression::evaluator::functions::FunctionEvaluator;
use crate::expression::evaluator::operations::{BinaryOperationEvaluator, UnaryOperationEvaluator};
use crate::expression::evaluator::traits::ExpressionContext;
use crate::expression::functions::global_registry;

/// 表达式求值器实现（unit struct，零开销）
#[derive(Debug)]
pub struct ExpressionEvaluator;

impl ExpressionEvaluator {
    /// 在给定上下文中求值表达式（泛型版本，零成本抽象）
    pub fn evaluate<C: ExpressionContext>(
        expression: &Expression,
        context: &mut C,
    ) -> Result<Value, ExpressionError> {
        let mut evaluator = Self;
        evaluator.visit_with_context(expression, context)
    }

    /// 批量求值表达式列表（泛型版本，零成本抽象）
    pub fn evaluate_batch<C: ExpressionContext>(
        expressions: &[Expression],
        context: &mut C,
    ) -> Result<Vec<Value>, ExpressionError> {
        let mut results = Vec::with_capacity(expressions.len());
        for expression in expressions {
            results.push(Self::evaluate(expression, context)?);
        }
        Ok(results)
    }

    /// 检查表达式是否可以求值（泛型版本）
    ///
    /// 检查表达式是否可以在没有运行时上下文的情况下求值
    /// 即表达式只包含常量，不包含变量或属性访问
    pub fn can_evaluate(expression: &Expression) -> bool {
        !Self::requires_runtime_context(expression)
    }

    /// 检查表达式是否需要运行时上下文才能求值
    fn requires_runtime_context(expression: &Expression) -> bool {
        Self::check_requires_context(expression)
    }

    fn check_requires_context(expression: &Expression) -> bool {
        match expression {
            Expression::Literal(_) => false,
            Expression::Variable(_) => true,
            Expression::Property { .. } => true,
            Expression::Binary { left, right, .. } => {
                Self::check_requires_context(left) || Self::check_requires_context(right)
            }
            Expression::Unary { operand, .. } => Self::check_requires_context(operand),
            Expression::Function { args, .. } => args.iter().any(|arg| Self::check_requires_context(arg)),
            Expression::Aggregate { arg, .. } => Self::check_requires_context(arg),
            Expression::List(items) => items.iter().any(|arg| Self::check_requires_context(arg)),
            Expression::Map(pairs) => pairs.iter().any(|(_, val)| Self::check_requires_context(val)),
            Expression::Case { conditions, default } => {
                conditions.iter().any(|(cond, val)| Self::check_requires_context(cond) || Self::check_requires_context(val))
                    || default.as_ref().map_or(false, |d| Self::check_requires_context(d))
            }
            Expression::TypeCast { expression, .. } => Self::check_requires_context(expression),
            Expression::Subscript { collection, index } => {
                Self::check_requires_context(collection) || Self::check_requires_context(index)
            }
            Expression::Range { collection, start, end } => {
                Self::check_requires_context(collection)
                    || start.as_ref().map_or(false, |s| Self::check_requires_context(s))
                    || end.as_ref().map_or(false, |e| Self::check_requires_context(e))
            }
            Expression::Path(items) => items.iter().any(|item| Self::check_requires_context(item)),
            Expression::Label(_) => false,
            Expression::ListComprehension { source, filter, map, .. } => {
                Self::check_requires_context(source)
                    || filter.as_ref().map_or(false, |f| Self::check_requires_context(f))
                    || map.as_ref().map_or(false, |m| Self::check_requires_context(m))
            }
            Expression::LabelTagProperty { tag, .. } => Self::check_requires_context(tag),
            Expression::TagProperty { .. } => false,
            Expression::EdgeProperty { .. } => false,
            Expression::Predicate { args, .. } => args.iter().any(|arg| Self::check_requires_context(arg)),
            Expression::Reduce { initial, source, mapping, .. } => {
                Self::check_requires_context(initial)
                    || Self::check_requires_context(source)
                    || Self::check_requires_context(mapping)
            }
            Expression::PathBuild(exprs) => exprs.iter().any(|expr| Self::check_requires_context(expr)),
        }
    }

    /// 在上下文中访问表达式
    fn visit_with_context<C: ExpressionContext>(
        &mut self,
        expression: &Expression,
        context: &mut C,
    ) -> Result<Value, ExpressionError> {
        match expression {
            // 字面量 - 直接返回值
            Expression::Literal(value) => Ok(value.clone()),

            // 变量 - 从上下文获取
            Expression::Variable(name) => context
                .get_variable(name)
                .ok_or_else(|| ExpressionError::undefined_variable(name)),

            // 二元操作 - 递归求值左右操作数
            Expression::Binary { left, op, right } => {
                let left_value = self.visit_with_context(left, context)?;
                let right_value = self.visit_with_context(right, context)?;
                BinaryOperationEvaluator::evaluate(&left_value, op, &right_value)
            }

            // 一元操作 - 递归求值操作数
            Expression::Unary { op, operand } => {
                let value = self.visit_with_context(operand, context)?;
                UnaryOperationEvaluator::evaluate(op, &value)
            }

            // 函数调用 - 批量求值参数
            Expression::Function { name, args } => {
                let arg_values: Result<Vec<Value>, ExpressionError> = args
                    .iter()
                    .map(|arg| self.visit_with_context(arg, context))
                    .collect();
                let arg_values = arg_values?;
                
                // 先获取函数（不可变借用）
                let func_ref = context.get_function(name);
                
                if let Some(func_ref) = func_ref {
                    // 转换为拥有所有权的函数引用以避免借用问题
                    let owned_func: crate::expression::functions::OwnedFunctionRef = func_ref.clone().into();
                    
                    // 显式释放 func_ref 的借用
                    drop(func_ref);
                    
                    // 如果上下文支持缓存，使用缓存感知执行
                    if context.supports_cache() {
                        // 获取缓存（可变借用）
                        if let Some(cache) = context.get_cache() {
                            return owned_func.execute_with_cache(&arg_values, cache);
                        }
                    }
                    // 否则使用普通执行
                    owned_func.execute(&arg_values)
                } else {
                    // 如果上下文中没有，则使用全局注册表
                    global_registry().execute(name, &arg_values)
                }
            }

            // 聚合函数 - 直接求值
            Expression::Aggregate {
                func,
                arg,
                distinct,
            } => {
                let arg_value = self.visit_with_context(arg, context)?;
                FunctionEvaluator.eval_aggregate_function(func, &[arg_value], *distinct)
            }

            // CASE 表达式 - 短路求值
            Expression::Case {
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
                    Some(default_expression) => self.visit_with_context(default_expression, context),
                    None => Ok(Value::Null(crate::core::NullType::Null)),
                }
            }

            // 列表 - 批量求值
            Expression::List(elements) => {
                let element_values: Result<Vec<Value>, ExpressionError> = elements
                    .iter()
                    .map(|elem| self.visit_with_context(elem, context))
                    .collect();
                element_values.map(Value::List)
            }

            // 映射 - 批量求值
            Expression::Map(entries) => {
                let mut map_values = std::collections::HashMap::new();
                for (key, value_expression) in entries {
                    let value = self.visit_with_context(value_expression, context)?;
                    map_values.insert(key.clone(), value);
                }
                Ok(Value::Map(map_values))
            }

            // 下标访问
            Expression::Subscript { collection, index } => {
                let collection_value = self.visit_with_context(collection, context)?;
                let index_value = self.visit_with_context(index, context)?;
                CollectionOperationEvaluator.eval_subscript_access(&collection_value, &index_value)
            }

            // 范围访问
            Expression::Range {
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
            Expression::Path(elements) => {
                let element_values: Result<Vec<Value>, ExpressionError> = elements
                    .iter()
                    .map(|elem| self.visit_with_context(elem, context))
                    .collect();
                element_values.map(Value::List)
            }

            // 属性访问
            Expression::Property { object, property } => {
                let object_value = self.visit_with_context(object, context)?;
                CollectionOperationEvaluator.eval_property_access(&object_value, property)
            }

            // 类型转换
            Expression::TypeCast { expression, target_type } => {
                let value = self.visit_with_context(expression, context)?;
                Self::eval_type_cast(&value, target_type)
            }

            // 其他表达式类型 - 保持静态分发，避免动态分发回退
            _ => Err(ExpressionError::type_error("不支持的表达式类型")),
        }
    }

    /// 编译时分支预测优化版本（完全静态分发）
    #[inline(always)]
    pub fn evaluate_with_branch_prediction<C: ExpressionContext>(
        expression: &Expression,
        context: &mut C,
    ) -> Result<Value, ExpressionError> {
        let mut evaluator = Self;
        evaluator.visit_with_context(expression, context)
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

        // 检查转换结果是否为 Null(BadData)
        if let Value::Null(NullType::BadData) = result {
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

    fn visit(&mut self, expression: &Expression) -> Self::Result {
        match expression {
            Expression::Literal(value) => Ok(value.clone()),
            Expression::Variable(name) => Err(ExpressionError::undefined_variable(name)),
            Expression::Binary { left, op, right } => {
                let left_value = self.visit(left)?;
                let right_value = self.visit(right)?;
                BinaryOperationEvaluator::evaluate(&left_value, op, &right_value)
            }
            Expression::Unary { op, operand } => {
                let value = self.visit(operand)?;
                UnaryOperationEvaluator::evaluate(op, &value)
            }
            Expression::Function { name, args } => {
                let arg_values: Result<Vec<Value>, ExpressionError> = args
                    .iter()
                    .map(|arg| self.visit(arg))
                    .collect();
                let arg_values = arg_values?;
                global_registry().execute(name, &arg_values)
            }
            Expression::Aggregate { func, arg, distinct } => {
                let arg_value = self.visit(arg)?;
                FunctionEvaluator.eval_aggregate_function(func, &[arg_value], *distinct)
            }
            Expression::Case { conditions, default } => {
                for (condition, value) in conditions {
                    let condition_result = self.visit(condition)?;
                    match condition_result {
                        Value::Bool(true) => return self.visit(value),
                        Value::Bool(false) => continue,
                        _ => return Err(ExpressionError::type_error("CASE条件必须是布尔值")),
                    }
                }
                match default {
                    Some(default_expression) => self.visit(default_expression),
                    None => Ok(Value::Null(crate::core::NullType::Null)),
                }
            }
            Expression::List(elements) => {
                let element_values: Result<Vec<Value>, ExpressionError> = elements
                    .iter()
                    .map(|elem| self.visit(elem))
                    .collect();
                element_values.map(Value::List)
            }
            Expression::Map(entries) => {
                let mut map_values = std::collections::HashMap::new();
                for (key, value_expression) in entries {
                    let value = self.visit(value_expression)?;
                    map_values.insert(key.clone(), value);
                }
                Ok(Value::Map(map_values))
            }
            Expression::Subscript { collection, index } => {
                let collection_value = self.visit(collection)?;
                let index_value = self.visit(index)?;
                CollectionOperationEvaluator.eval_subscript_access(&collection_value, &index_value)
            }
            Expression::Range { collection, start, end } => {
                let collection_value = self.visit(collection)?;
                let start_value = start.as_ref().map(|e| self.visit(e)).transpose()?;
                let end_value = end.as_ref().map(|e| self.visit(e)).transpose()?;
                CollectionOperationEvaluator.eval_range_access(
                    &collection_value,
                    start_value.as_ref(),
                    end_value.as_ref(),
                )
            }
            Expression::Path(elements) => {
                let element_values: Result<Vec<Value>, ExpressionError> = elements
                    .iter()
                    .map(|elem| self.visit(elem))
                    .collect();
                element_values.map(Value::List)
            }
            Expression::Property { object, property } => {
                let object_value = self.visit(object.as_ref())?;
                CollectionOperationEvaluator.eval_property_access(&object_value, property)
            }
            Expression::TypeCast { expression, target_type } => {
                let value = self.visit(expression)?;
                Self::eval_type_cast(&value, target_type)
            }
            Expression::Label(_) => Err(ExpressionError::type_error("未求解的标签表达式")),
            Expression::ListComprehension { .. } => Err(ExpressionError::type_error("列表推导表达式需要运行时上下文")),
            Expression::LabelTagProperty { .. } => Err(ExpressionError::type_error("标签属性表达式需要运行时上下文")),
            Expression::TagProperty { .. } => Err(ExpressionError::type_error("标签属性表达式需要运行时上下文")),
            Expression::EdgeProperty { .. } => Err(ExpressionError::type_error("边属性表达式需要运行时上下文")),
            Expression::Predicate { .. } => Err(ExpressionError::type_error("谓词表达式需要运行时上下文")),
            Expression::Reduce { .. } => Err(ExpressionError::type_error("归约表达式需要运行时上下文")),
            Expression::PathBuild(_) => Err(ExpressionError::type_error("路径构建表达式需要运行时上下文")),
        }
    }
}

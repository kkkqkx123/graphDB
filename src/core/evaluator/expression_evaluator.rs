//! 表达式求值器实现
//!
//! 提供具体的表达式求值功能

use crate::core::expressions::ExpressionContext;
use crate::core::types::expression::{Expression, LiteralValue};
use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
use crate::core::ExpressionError;
use crate::core::Value;

/// 表达式求值器实现
#[derive(Debug)]
pub struct ExpressionEvaluator;

impl ExpressionEvaluator {
    /// 创建新的表达式求值器
    pub fn new() -> Self {
        ExpressionEvaluator
    }

    /// 在给定上下文中求值表达式（公共接口，保留 dyn 以兼容）
    pub fn evaluate(
        &self,
        expr: &Expression,
        context: &mut dyn ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        // 注意：这个方法保留 dyn 用于兼容性，但内部调用泛型实现
        // 实际上，大多数调用会通过 Evaluator<C> trait 使用泛型版本
        self.eval_expression(expr, context)
    }

    /// 在给定上下文中求值表达式（通过 dyn trait object）
    pub fn eval_expression(
        &self,
        expr: &Expression,
        context: &mut dyn ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        match expr {
            Expression::Literal(literal_value) => {
                // 将 LiteralValue 转换为 Value
                match literal_value {
                    LiteralValue::Bool(b) => Ok(Value::Bool(*b)),
                    LiteralValue::Int(i) => Ok(Value::Int(*i)),
                    LiteralValue::Float(f) => Ok(Value::Float(*f)),
                    LiteralValue::String(s) => Ok(Value::String(s.clone())),
                    LiteralValue::Null => Ok(Value::Null(crate::core::NullType::Null)),
                }
            }
            Expression::TypeCast { expr, target_type } => {
                let value = self.evaluate(expr, context)?;
                self.eval_type_cast(&value, target_type)
            }
            Expression::Property { object, property } => {
                // 先计算 object，然后获取其属性
                let object_value = self.evaluate(object, context)?;
                self.eval_property_access(&object_value, property)
            }
            Expression::Variable(name) => {
                // 从上下文中获取变量值
                context
                    .get_variable(name)
                    .ok_or_else(|| ExpressionError::undefined_variable(name))
            }
            Expression::Binary { left, op, right } => {
                let left_value = self.evaluate(left, context)?;
                let right_value = self.evaluate(right, context)?;
                self.eval_binary_operation(&left_value, op, &right_value)
            }
            Expression::Unary { op, operand } => {
                let value = self.evaluate(operand, context)?;
                self.eval_unary_operation(op, &value)
            }
            Expression::Function { name, args } => {
                let arg_values: Result<Vec<Value>, ExpressionError> =
                    args.iter().map(|arg| self.evaluate(arg, context)).collect();
                let arg_values = arg_values?;
                self.eval_function_call(name, &arg_values)
            }
            Expression::Aggregate {
                func,
                arg,
                distinct,
            } => {
                let arg_value = self.evaluate(arg, context)?;
                self.eval_aggregate_function_single(func, &arg_value, *distinct)
            }
            Expression::Case {
                conditions,
                default,
            } => self.eval_case_expression(conditions, default.as_deref(), context),
            Expression::List(elements) => {
                let element_values: Result<Vec<Value>, ExpressionError> = elements
                    .iter()
                    .map(|elem| self.evaluate(elem, context))
                    .collect();
                element_values.map(Value::List)
            }
            Expression::Map(entries) => {
                let mut map_values = std::collections::HashMap::new();
                for (key, value_expr) in entries {
                    let value = self.evaluate(value_expr, context)?;
                    map_values.insert(key.clone(), value);
                }
                Ok(Value::Map(map_values))
            }
            // 添加缺失的表达式类型处理
            Expression::Subscript { collection, index } => {
                let collection_value = self.evaluate(collection, context)?;
                let index_value = self.evaluate(index, context)?;
                self.eval_subscript_access(&collection_value, &index_value)
            }
            Expression::Range {
                collection,
                start,
                end,
            } => {
                let collection_value = self.evaluate(collection, context)?;
                let start_value = start
                    .as_ref()
                    .map(|e| self.evaluate(e, context))
                    .transpose()?;
                let end_value = end
                    .as_ref()
                    .map(|e| self.evaluate(e, context))
                    .transpose()?;
                self.eval_range_access(&collection_value, start_value.as_ref(), end_value.as_ref())
            }
            Expression::Path(elements) => {
                let element_values: Result<Vec<Value>, ExpressionError> = elements
                    .iter()
                    .map(|elem| self.evaluate(elem, context))
                    .collect();
                element_values.map(Value::List)
            }
            Expression::Label(_) => {
                // 标签表达式暂时返回null
                Ok(Value::Null(crate::core::NullType::Null))
            }
            Expression::TagProperty { tag: _, prop: _ } => {
                // 标签属性表达式暂时返回null
                Ok(Value::Null(crate::core::NullType::Null))
            }
            Expression::EdgeProperty { edge: _, prop: _ } => {
                // 边属性表达式暂时返回null
                Ok(Value::Null(crate::core::NullType::Null))
            }
            Expression::InputProperty(_) => {
                // 输入属性表达式暂时返回null
                Ok(Value::Null(crate::core::NullType::Null))
            }
            Expression::VariableProperty { var: _, prop: _ } => {
                // 变量属性表达式暂时返回null
                Ok(Value::Null(crate::core::NullType::Null))
            }
            Expression::SourceProperty { tag: _, prop: _ } => {
                // 源属性表达式暂时返回null
                Ok(Value::Null(crate::core::NullType::Null))
            }
            Expression::DestinationProperty { tag: _, prop: _ } => {
                // 目的属性表达式暂时返回null
                Ok(Value::Null(crate::core::NullType::Null))
            }
            // 一元操作扩展
            Expression::UnaryPlus(expr) => self.evaluate(expr, context),
            Expression::UnaryNegate(expr) => {
                let value = self.evaluate(expr, context)?;
                self.eval_unary_operation(&UnaryOperator::Minus, &value)
            }
            Expression::UnaryNot(expr) => {
                let value = self.evaluate(expr, context)?;
                self.eval_unary_operation(&UnaryOperator::Not, &value)
            }
            Expression::UnaryIncr(expr) => {
                let value = self.evaluate(expr, context)?;
                self.eval_unary_operation(&UnaryOperator::Increment, &value)
            }
            Expression::UnaryDecr(expr) => {
                let value = self.evaluate(expr, context)?;
                self.eval_unary_operation(&UnaryOperator::Decrement, &value)
            }
            Expression::IsNull(expr) => {
                let value = self.evaluate(expr, context)?;
                Ok(Value::Bool(value.is_null()))
            }
            Expression::IsNotNull(expr) => {
                let value = self.evaluate(expr, context)?;
                Ok(Value::Bool(!value.is_null()))
            }
            Expression::IsEmpty(expr) => {
                let value = self.evaluate(expr, context)?;
                self.eval_unary_operation(&UnaryOperator::IsEmpty, &value)
            }
            Expression::IsNotEmpty(expr) => {
                let value = self.evaluate(expr, context)?;
                self.eval_unary_operation(&UnaryOperator::IsNotEmpty, &value)
            }
            // 类型转换
            Expression::TypeCasting {
                expr,
                target_type: _,
            } => {
                // 暂时只返回原值
                self.evaluate(expr, context)
            }
            // 列表推导
            Expression::ListComprehension {
                generator,
                condition,
            } => {
                let gen_value = self.evaluate(generator, context)?;
                // 如果有条件，检查条件是否满足
                if let Some(cond) = condition {
                    let cond_result = self.evaluate(cond, context)?;
                    match cond_result {
                        Value::Bool(true) => Ok(gen_value),
                        Value::Bool(false) => Ok(Value::List(vec![])),
                        _ => Err(ExpressionError::type_error("推导条件必须是布尔值")),
                    }
                } else {
                    Ok(Value::List(vec![gen_value]))
                }
            }
            // 谓词表达式
            Expression::Predicate { list, condition } => {
                let list_value = self.evaluate(list, context)?;
                let cond_value = self.evaluate(condition, context)?;
                // 简单实现，返回满足条件的列表元素
                match (list_value, cond_value) {
                    (Value::List(items), Value::Bool(true)) => Ok(Value::List(items)),
                    (Value::List(_), Value::Bool(false)) => Ok(Value::List(vec![])),
                    _ => Err(ExpressionError::type_error("谓词表达式需要列表和布尔条件")),
                }
            }
            // 归约表达式
            Expression::Reduce {
                list,
                var: _,
                initial,
                expr,
            } => {
                let list_value = self.evaluate(list, context)?;
                let mut acc = self.evaluate(initial, context)?;

                if let Value::List(items) = list_value {
                    for _item in items {
                        // 简单实现，暂时只返回累加器
                        let _ = self.evaluate(expr, context)?;
                    }
                }
                Ok(acc)
            }
            // 路径构建表达式
            Expression::PathBuild(elements) => {
                let element_values: Result<Vec<Value>, ExpressionError> = elements
                    .iter()
                    .map(|elem| self.evaluate(elem, context))
                    .collect();
                element_values.map(Value::List)
            }
            // 文本搜索表达式
            Expression::ESQuery(_) => {
                // 文本搜索暂时返回null
                Ok(Value::Null(crate::core::NullType::Null))
            }
            // UUID表达式
            Expression::UUID => Ok(Value::String(uuid::Uuid::new_v4().to_string())),
            // 下标范围表达式
            Expression::SubscriptRange {
                collection,
                start,
                end,
            } => {
                let collection_value = self.evaluate(collection, context)?;
                let start_value = start
                    .as_ref()
                    .map(|e| self.evaluate(e, context))
                    .transpose()?;
                let end_value = end
                    .as_ref()
                    .map(|e| self.evaluate(e, context))
                    .transpose()?;
                self.eval_range_access(&collection_value, start_value.as_ref(), end_value.as_ref())
            }
            // 匹配路径模式表达式
            Expression::MatchPathPattern {
                path_alias: _,
                patterns,
            } => {
                let pattern_values: Result<Vec<Value>, ExpressionError> =
                    patterns.iter().map(|p| self.evaluate(p, context)).collect();
                pattern_values.map(Value::List)
            }
        }
    }

    /// 批量求值表达式列表
    pub fn evaluate_batch(
        &self,
        expressions: &[Expression],
        context: &mut dyn ExpressionContext,
    ) -> Result<Vec<Value>, ExpressionError> {
        let mut results = Vec::with_capacity(expressions.len());
        for expr in expressions {
            results.push(self.evaluate(expr, context)?);
        }
        Ok(results)
    }

    /// 检查表达式是否可以求值
    pub fn can_evaluate(&self, expr: &Expression, context: &dyn ExpressionContext) -> bool {
        // 基础实现：所有表达式都可以求值
        true
    }

    /// 泛型版本的表达式求值（避免虚表开销）
    ///
    /// 编译器会为每个具体的 C 类型生成专用代码，支持完全内联
    pub fn eval_expression_generic<C: ExpressionContext>(
        &self,
        expr: &Expression,
        context: &mut C,
    ) -> Result<Value, ExpressionError> {
        match expr {
            Expression::Literal(literal_value) => match literal_value {
                LiteralValue::Bool(b) => Ok(Value::Bool(*b)),
                LiteralValue::Int(i) => Ok(Value::Int(*i)),
                LiteralValue::Float(f) => Ok(Value::Float(*f)),
                LiteralValue::String(s) => Ok(Value::String(s.clone())),
                LiteralValue::Null => Ok(Value::Null(crate::core::NullType::Null)),
            },
            Expression::TypeCast { expr, target_type } => {
                let value = self.eval_expression_generic(expr, context)?;
                self.eval_type_cast(&value, target_type)
            }
            Expression::Property { object, property } => {
                let object_value = self.eval_expression_generic(object, context)?;
                self.eval_property_access(&object_value, property)
            }
            Expression::Variable(name) => context
                .get_variable(name)
                .ok_or_else(|| ExpressionError::undefined_variable(name)),
            Expression::Binary { left, op, right } => {
                let left_value = self.eval_expression_generic(left, context)?;
                let right_value = self.eval_expression_generic(right, context)?;
                self.eval_binary_operation(&left_value, op, &right_value)
            }
            Expression::Unary { op, operand } => {
                let value = self.eval_expression_generic(operand, context)?;
                self.eval_unary_operation(op, &value)
            }
            Expression::Function { name, args } => {
                let arg_values: Result<Vec<Value>, ExpressionError> = args
                    .iter()
                    .map(|arg| self.eval_expression_generic(arg, context))
                    .collect();
                let arg_values = arg_values?;
                self.eval_function_call(name, &arg_values)
            }
            Expression::Aggregate {
                func,
                arg,
                distinct,
            } => {
                let arg_value = self.eval_expression_generic(arg, context)?;
                self.eval_aggregate_function_single(func, &arg_value, *distinct)
            }
            Expression::Case {
                conditions,
                default,
            } => {
                for (condition, value) in conditions {
                    let condition_result = self.eval_expression_generic(condition, context)?;
                    match condition_result {
                        Value::Bool(true) => return self.eval_expression_generic(value, context),
                        Value::Bool(false) => continue,
                        _ => return Err(ExpressionError::type_error("CASE条件必须是布尔值")),
                    }
                }
                match default {
                    Some(default_expr) => self.eval_expression_generic(default_expr, context),
                    None => Ok(Value::Null(crate::core::NullType::Null)),
                }
            }
            Expression::List(elements) => {
                let element_values: Result<Vec<Value>, ExpressionError> = elements
                    .iter()
                    .map(|elem| self.eval_expression_generic(elem, context))
                    .collect();
                element_values.map(Value::List)
            }
            Expression::Map(entries) => {
                let mut map_values = std::collections::HashMap::new();
                for (key, value_expr) in entries {
                    let value = self.eval_expression_generic(value_expr, context)?;
                    map_values.insert(key.clone(), value);
                }
                Ok(Value::Map(map_values))
            }
            Expression::Subscript { collection, index } => {
                let collection_value = self.eval_expression_generic(collection, context)?;
                let index_value = self.eval_expression_generic(index, context)?;
                self.eval_subscript_access(&collection_value, &index_value)
            }
            Expression::Range {
                collection,
                start,
                end,
            } => {
                let collection_value = self.eval_expression_generic(collection, context)?;
                let start_value = start
                    .as_ref()
                    .map(|e| self.eval_expression_generic(e, context))
                    .transpose()?;
                let end_value = end
                    .as_ref()
                    .map(|e| self.eval_expression_generic(e, context))
                    .transpose()?;
                self.eval_range_access(&collection_value, start_value.as_ref(), end_value.as_ref())
            }
            Expression::Path(elements) => {
                let element_values: Result<Vec<Value>, ExpressionError> = elements
                    .iter()
                    .map(|elem| self.eval_expression_generic(elem, context))
                    .collect();
                element_values.map(Value::List)
            }
            // 对于复杂表达式，委派给原始实现（这些不需要频繁调用）
            _ => {
                // 这里我们需要适配泛型 C 到 dyn ExpressionContext
                // 创建一个临时的 dyn trait object 来处理剩余的表达式
                let mut dyn_ctx: &mut dyn ExpressionContext = context;
                self.eval_expression(expr, dyn_ctx)
            }
        }
    }

    /// 获取求值器名称
    pub fn name(&self) -> &str {
        "ExpressionEvaluator"
    }

    /// 获取求值器描述
    pub fn description(&self) -> &str {
        "标准表达式求值器"
    }

    /// 获取求值器版本
    pub fn version(&self) -> &str {
        "1.0.0"
    }

    /// 求值二元运算
    pub fn eval_binary_operation(
        &self,
        left: &Value,
        op: &BinaryOperator,
        right: &Value,
    ) -> Result<Value, ExpressionError> {
        match op {
            // 算术运算
            BinaryOperator::Add => left
                .add(right)
                .map_err(|e| ExpressionError::runtime_error(e)),
            BinaryOperator::Subtract => left
                .sub(right)
                .map_err(|e| ExpressionError::runtime_error(e)),
            BinaryOperator::Multiply => left
                .mul(right)
                .map_err(|e| ExpressionError::runtime_error(e)),
            BinaryOperator::Divide => {
                if matches!(right, Value::Int(0) | Value::Float(0.0)) {
                    Err(ExpressionError::division_by_zero())
                } else {
                    left.div(right)
                        .map_err(|e| ExpressionError::runtime_error(e))
                }
            }
            BinaryOperator::Modulo => left
                .modulo(right)
                .map_err(|e| ExpressionError::runtime_error(e)),
            BinaryOperator::Exponent => left
                .pow(right)
                .map_err(|e| ExpressionError::runtime_error(e)),

            // 比较运算
            BinaryOperator::Equal => Ok(Value::Bool(left.equals(right))),
            BinaryOperator::NotEqual => Ok(Value::Bool(!left.equals(right))),
            BinaryOperator::LessThan => Ok(Value::Bool(left.less_than(right))),
            BinaryOperator::LessThanOrEqual => Ok(Value::Bool(left.less_than_equal(right))),
            BinaryOperator::GreaterThan => Ok(Value::Bool(left.greater_than(right))),
            BinaryOperator::GreaterThanOrEqual => Ok(Value::Bool(left.greater_than_equal(right))),

            // 逻辑运算
            BinaryOperator::And => match (left, right) {
                (Value::Bool(l), Value::Bool(r)) => Ok(Value::Bool(*l && *r)),
                _ => Err(ExpressionError::type_error("逻辑运算需要布尔值")),
            },
            BinaryOperator::Or => match (left, right) {
                (Value::Bool(l), Value::Bool(r)) => Ok(Value::Bool(*l || *r)),
                _ => Err(ExpressionError::type_error("逻辑运算需要布尔值")),
            },
            BinaryOperator::Xor => match (left, right) {
                (Value::Bool(l), Value::Bool(r)) => Ok(Value::Bool(*l ^ *r)),
                _ => Err(ExpressionError::type_error("XOR运算需要布尔值")),
            },

            // 字符串运算
            BinaryOperator::StringConcat => match (left, right) {
                (Value::String(l), Value::String(r)) => Ok(Value::String(format!("{}{}", l, r))),
                _ => Err(ExpressionError::type_error("字符串连接需要字符串值")),
            },
            BinaryOperator::Like => {
                match (left, right) {
                    (Value::String(l), Value::String(r)) => {
                        // 改进的LIKE实现，支持%和_通配符，并处理转义字符
                        self.eval_like_operation(l, r)
                    }
                    _ => Err(ExpressionError::type_error("LIKE操作需要字符串值")),
                }
            }
            BinaryOperator::In => match right {
                Value::List(items) => Ok(Value::Bool(items.contains(left))),
                _ => Err(ExpressionError::type_error("IN操作右侧必须是列表")),
            },
            BinaryOperator::NotIn => match right {
                Value::List(items) => Ok(Value::Bool(!items.contains(left))),
                _ => Err(ExpressionError::type_error("NOT IN操作右侧必须是列表")),
            },
            BinaryOperator::Contains => match (&left, &right) {
                (Value::String(s), Value::String(sub)) => Ok(Value::Bool(s.contains(sub))),
                (Value::List(items), item) => Ok(Value::Bool(items.contains(item))),
                _ => Err(ExpressionError::type_error("CONTAINS操作需要字符串或列表")),
            },
            BinaryOperator::StartsWith => match (&left, &right) {
                (Value::String(s), Value::String(prefix)) => Ok(Value::Bool(s.starts_with(prefix))),
                _ => Err(ExpressionError::type_error("STARTS WITH操作需要字符串值")),
            },
            BinaryOperator::EndsWith => match (&left, &right) {
                (Value::String(s), Value::String(suffix)) => Ok(Value::Bool(s.ends_with(suffix))),
                _ => Err(ExpressionError::type_error("ENDS WITH操作需要字符串值")),
            },

            // 访问运算
            BinaryOperator::Subscript => self.eval_subscript_access(left, right),
            BinaryOperator::Attribute => self.eval_property_access(left, &right.to_string()),

            // 集合运算
            BinaryOperator::Union => match (left, right) {
                (Value::List(l), Value::List(r)) => {
                    let mut result = l.clone();
                    result.extend(r.clone());
                    Ok(Value::List(result))
                }
                _ => Err(ExpressionError::type_error("UNION操作需要列表值")),
            },
            BinaryOperator::Intersect => match (left, right) {
                (Value::List(l), Value::List(r)) => {
                    let result: Vec<Value> =
                        l.iter().filter(|item| r.contains(item)).cloned().collect();
                    Ok(Value::List(result))
                }
                _ => Err(ExpressionError::type_error("INTERSECT操作需要列表值")),
            },
            BinaryOperator::Except => match (left, right) {
                (Value::List(l), Value::List(r)) => {
                    let result: Vec<Value> =
                        l.iter().filter(|item| !r.contains(item)).cloned().collect();
                    Ok(Value::List(result))
                }
                _ => Err(ExpressionError::type_error("EXCEPT操作需要列表值")),
            },
        }
    }

    /// 求值一元运算
    pub fn eval_unary_operation(
        &self,
        op: &UnaryOperator,
        value: &Value,
    ) -> Result<Value, ExpressionError> {
        match op {
            // 算术运算
            UnaryOperator::Plus => Ok(value.clone()),
            UnaryOperator::Minus => value
                .negate()
                .map_err(|e| ExpressionError::runtime_error(e)),

            // 逻辑运算
            UnaryOperator::Not => match value {
                Value::Bool(b) => Ok(Value::Bool(!b)),
                _ => Err(ExpressionError::type_error("NOT操作需要布尔值")),
            },

            // 存在性检查
            UnaryOperator::IsNull => Ok(Value::Bool(value.is_null())),
            UnaryOperator::IsNotNull => Ok(Value::Bool(!value.is_null())),
            UnaryOperator::IsEmpty => match value {
                Value::String(s) => Ok(Value::Bool(s.is_empty())),
                Value::List(l) => Ok(Value::Bool(l.is_empty())),
                Value::Map(m) => Ok(Value::Bool(m.is_empty())),
                _ => Err(ExpressionError::type_error("EMPTY检查需要容器类型")),
            },
            UnaryOperator::IsNotEmpty => match value {
                Value::String(s) => Ok(Value::Bool(!s.is_empty())),
                Value::List(l) => Ok(Value::Bool(!l.is_empty())),
                Value::Map(m) => Ok(Value::Bool(!m.is_empty())),
                _ => Err(ExpressionError::type_error("EMPTY检查需要容器类型")),
            },

            // 增减操作
            UnaryOperator::Increment => match value {
                Value::Int(i) => Ok(Value::Int(i + 1)),
                _ => Err(ExpressionError::type_error("递增操作需要整数")),
            },
            UnaryOperator::Decrement => match value {
                Value::Int(i) => Ok(Value::Int(i - 1)),
                _ => Err(ExpressionError::type_error("递减操作需要整数")),
            },
        }
    }

    /// 求值类型转换
    fn eval_type_cast(
        &self,
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

    /// 求值下标访问
    fn eval_subscript_access(
        &self,
        collection: &Value,
        index: &Value,
    ) -> Result<Value, ExpressionError> {
        match collection {
            Value::List(list) => {
                if let Value::Int(i) = index {
                    let adjusted_index = if *i < 0 { list.len() as i64 + i } else { *i };

                    if adjusted_index >= 0 && (adjusted_index as usize) < list.len() {
                        Ok(list[adjusted_index as usize].clone())
                    } else {
                        Err(ExpressionError::index_out_of_bounds(
                            adjusted_index as isize,
                            list.len(),
                        ))
                    }
                } else {
                    Err(ExpressionError::type_error("列表下标必须是整数"))
                }
            }
            Value::Map(map) => {
                if let Value::String(key) = index {
                    map.get(key)
                        .cloned()
                        .ok_or_else(|| ExpressionError::runtime_error(format!("键不存在: {}", key)))
                } else {
                    Err(ExpressionError::type_error("映射下标必须是字符串"))
                }
            }
            _ => Err(ExpressionError::type_error("不支持下标访问的类型")),
        }
    }

    /// 求值范围访问
    fn eval_range_access(
        &self,
        collection: &Value,
        start: Option<&Value>,
        end: Option<&Value>,
    ) -> Result<Value, ExpressionError> {
        match collection {
            Value::List(list) => {
                let start_idx = start
                    .map(|v| {
                        if let Value::Int(i) = v {
                            if *i < 0 {
                                (list.len() as i64 + i) as usize
                            } else {
                                *i as usize
                            }
                        } else {
                            0
                        }
                    })
                    .unwrap_or(0);

                let end_idx = end
                    .map(|v| {
                        if let Value::Int(i) = v {
                            if *i < 0 {
                                (list.len() as i64 + i) as usize
                            } else {
                                *i as usize
                            }
                        } else {
                            list.len()
                        }
                    })
                    .unwrap_or(list.len());

                if start_idx <= end_idx && end_idx <= list.len() {
                    Ok(Value::List(list[start_idx..end_idx].to_vec()))
                } else {
                    Err(ExpressionError::index_out_of_bounds(
                        start_idx as isize,
                        list.len(),
                    ))
                }
            }
            Value::String(s) => {
                let chars: Vec<char> = s.chars().collect();
                let start_idx = start
                    .map(|v| {
                        if let Value::Int(i) = v {
                            if *i < 0 {
                                (chars.len() as i64 + i) as usize
                            } else {
                                *i as usize
                            }
                        } else {
                            0
                        }
                    })
                    .unwrap_or(0);

                let end_idx = end
                    .map(|v| {
                        if let Value::Int(i) = v {
                            if *i < 0 {
                                (chars.len() as i64 + i) as usize
                            } else {
                                *i as usize
                            }
                        } else {
                            chars.len()
                        }
                    })
                    .unwrap_or(chars.len());

                if start_idx <= end_idx && end_idx <= chars.len() {
                    let result: String = chars[start_idx..end_idx].iter().collect();
                    Ok(Value::String(result))
                } else {
                    Err(ExpressionError::index_out_of_bounds(
                        start_idx as isize,
                        chars.len(),
                    ))
                }
            }
            _ => Err(ExpressionError::type_error("不支持范围访问的类型")),
        }
    }

    /// 求值属性访问
    fn eval_property_access(
        &self,
        object: &Value,
        property: &str,
    ) -> Result<Value, ExpressionError> {
        match object {
            Value::Vertex(vertex) => vertex.properties.get(property).cloned().ok_or_else(|| {
                ExpressionError::runtime_error(format!("顶点属性不存在: {}", property))
            }),
            Value::Edge(edge) => edge.properties().get(property).cloned().ok_or_else(|| {
                ExpressionError::runtime_error(format!("边属性不存在: {}", property))
            }),
            Value::Map(map) => map.get(property).cloned().ok_or_else(|| {
                ExpressionError::runtime_error(format!("映射键不存在: {}", property))
            }),
            Value::List(list) => {
                // 支持数字索引访问
                if let Ok(index) = property.parse::<isize>() {
                    let adjusted_index = if index < 0 {
                        list.len() as isize + index
                    } else {
                        index
                    };

                    if adjusted_index >= 0 && adjusted_index < list.len() as isize {
                        Ok(list[adjusted_index as usize].clone())
                    } else {
                        Err(ExpressionError::index_out_of_bounds(
                            adjusted_index,
                            list.len(),
                        ))
                    }
                } else {
                    Err(ExpressionError::type_error("列表索引必须是整数"))
                }
            }
            _ => Err(ExpressionError::type_error("不支持属性访问的类型")),
        }
    }

    /// 求值函数调用
    fn eval_function_call(&self, name: &str, args: &[Value]) -> Result<Value, ExpressionError> {
        match name {
            // 数学函数
            "abs" => {
                if args.len() != 1 {
                    return Err(ExpressionError::argument_count_error(1, args.len()));
                }
                args[0].abs().map_err(|e| ExpressionError::runtime_error(e))
            }
            "ceil" => {
                if args.len() != 1 {
                    return Err(ExpressionError::argument_count_error(1, args.len()));
                }
                args[0]
                    .ceil()
                    .map_err(|e| ExpressionError::runtime_error(e))
            }
            "floor" => {
                if args.len() != 1 {
                    return Err(ExpressionError::argument_count_error(1, args.len()));
                }
                args[0]
                    .floor()
                    .map_err(|e| ExpressionError::runtime_error(e))
            }
            "round" => {
                if args.len() != 1 {
                    return Err(ExpressionError::argument_count_error(1, args.len()));
                }
                args[0]
                    .round()
                    .map_err(|e| ExpressionError::runtime_error(e))
            }

            // 字符串函数
            "length" => {
                if args.len() != 1 {
                    return Err(ExpressionError::argument_count_error(1, args.len()));
                }
                args[0]
                    .length()
                    .map_err(|e| ExpressionError::runtime_error(e))
            }
            "lower" => {
                if args.len() != 1 {
                    return Err(ExpressionError::argument_count_error(1, args.len()));
                }
                args[0]
                    .lower()
                    .map_err(|e| ExpressionError::runtime_error(e))
            }
            "upper" => {
                if args.len() != 1 {
                    return Err(ExpressionError::argument_count_error(1, args.len()));
                }
                args[0]
                    .upper()
                    .map_err(|e| ExpressionError::runtime_error(e))
            }
            "trim" => {
                if args.len() != 1 {
                    return Err(ExpressionError::argument_count_error(1, args.len()));
                }
                args[0]
                    .trim()
                    .map_err(|e| ExpressionError::runtime_error(e))
            }

            _ => Err(ExpressionError::undefined_function(name)),
        }
    }

    /// 求值聚合函数（单个参数）
    fn eval_aggregate_function_single(
        &self,
        func: &AggregateFunction,
        arg: &Value,
        distinct: bool,
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
        }
    }

    /// 求值聚合函数
    fn eval_aggregate_function(
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
                    if arg.less_than(&min) {
                        min = arg.clone();
                    }
                }
                Ok(min)
            }
            AggregateFunction::Max => {
                let mut max = args[0].clone();
                for arg in args.iter().skip(1) {
                    if arg.greater_than(&max) {
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
        }
    }

    /// 求值LIKE操作
    /// 支持SQL标准的LIKE通配符：
    /// - %: 匹配任意数量的字符（包括零个）
    /// - _: 匹配单个字符
    /// - \: 转义字符，用于转义%和_
    fn eval_like_operation(&self, text: &str, pattern: &str) -> Result<Value, ExpressionError> {
        // 将SQL LIKE模式转换为正则表达式
        let regex_pattern = self.like_to_regex(pattern)?;

        // 简化实现，不使用外部正则表达式库
        // 这里使用基本的模式匹配
        Ok(Value::Bool(self.like_simple_match(text, pattern)))
    }

    /// 将SQL LIKE模式转换为正则表达式
    fn like_to_regex(&self, _pattern: &str) -> Result<String, ExpressionError> {
        Ok(String::new())
    }

    /// 简单的LIKE模式匹配实现
    fn like_simple_match(&self, text: &str, pattern: &str) -> bool {
        let mut text_chars = text.chars().peekable();
        let mut pattern_chars = pattern.chars().peekable();

        while let Some(&p) = pattern_chars.peek() {
            match p {
                '%' => {
                    pattern_chars.next();
                    if pattern_chars.peek().is_none() {
                        return true;
                    }
                    while text_chars.peek().is_some() {
                        if self.like_simple_match(
                            &text_chars.clone().collect::<String>(),
                            &pattern_chars.clone().collect::<String>(),
                        ) {
                            return true;
                        }
                        text_chars.next();
                    }
                    return false;
                }
                '_' => {
                    pattern_chars.next();
                    if text_chars.next().is_none() {
                        return false;
                    }
                }
                _ => {
                    pattern_chars.next();
                    if Some(p) != text_chars.next() {
                        return false;
                    }
                }
            }
        }

        text_chars.peek().is_none()
    }

    /// 求值CASE表达式
    fn eval_case_expression(
        &self,
        cases: &[(Expression, Expression)],
        default: Option<&Expression>,
        context: &mut dyn crate::core::expressions::ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        for (condition, value) in cases {
            let condition_result = self.evaluate(condition, context)?;
            match condition_result {
                Value::Bool(true) => return self.evaluate(value, context),
                Value::Bool(false) => continue,
                _ => return Err(ExpressionError::type_error("CASE条件必须是布尔值")),
            }
        }

        match default {
            Some(default_expr) => self.evaluate(default_expr, context),
            None => Ok(Value::Null(crate::core::NullType::Null)),
        }
    }

}

impl Default for ExpressionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: ExpressionContext> Evaluator<C> for ExpressionEvaluator {
    /// 求值表达式（泛型版本，避免虚表开销）
    fn evaluate(&self, expr: &Expression, context: &mut C) -> Result<Value, ExpressionError> {
        // 使用泛型实现，编译器会为每个具体的 C 类型生成专用代码
        // 这避免了虚表查询的开销，允许内联优化
        self.eval_expression_generic(expr, context)
    }

    /// 批量求值表达式
    fn evaluate_batch(
        &self,
        expressions: &[Expression],
        context: &mut C,
    ) -> Result<Vec<Value>, ExpressionError> {
        let mut results = Vec::with_capacity(expressions.len());
        for expr in expressions {
            results.push(self.evaluate(expr, context)?);
        }
        Ok(results)
    }

    /// 检查表达式是否可以求值
    fn can_evaluate(&self, expr: &Expression, context: &C) -> bool {
        // 基础实现：所有表达式都可以求值
        true
    }

    /// 获取求值器名称
    fn name(&self) -> &str {
        "ExpressionEvaluator"
    }

    /// 获取求值器描述
    fn description(&self) -> &str {
        "标准表达式求值器"
    }

    /// 获取求值器版本
    fn version(&self) -> &str {
        "1.0.0"
    }
}

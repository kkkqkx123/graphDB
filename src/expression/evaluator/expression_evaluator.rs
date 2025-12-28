//! 表达式求值器实现
//!
//! 提供具体的表达式求值功能

use crate::core::types::expression::Expression;
use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
use crate::core::error::{ExpressionError, ExpressionErrorType};
use crate::core::Value;
use crate::expression::evaluator::traits::{Evaluator, ExpressionContext};
use crate::expression::evaluator::operations::{BinaryOperationEvaluator, UnaryOperationEvaluator};

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
            Expression::Literal(value) => {
                // 直接返回 Value，无需转换
                Ok(value.clone())
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
                BinaryOperationEvaluator::evaluate(&left_value, op, &right_value)
            }
            Expression::Unary { op, operand } => {
                let value = self.evaluate(operand, context)?;
                UnaryOperationEvaluator::evaluate(op, &value)
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
            Expression::Label(label_name) => {
                self.eval_label_expression(label_name, context)
            }
            Expression::TagProperty { tag, prop } => {
                self.eval_tag_property(tag, prop, context)
            }
            Expression::EdgeProperty { edge, prop } => {
                self.eval_edge_property(edge, prop, context)
            }
            Expression::InputProperty(prop_name) => {
                match context.get_variable(prop_name) {
                    Some(value) => Ok(value),
                    None => Ok(Value::Null(crate::core::NullType::Null))
                }
            }
            Expression::VariableProperty { var, prop } => {
                self.eval_variable_property(var, prop, context)
            }
            Expression::SourceProperty { tag, prop } => {
                self.eval_source_property(tag, prop, context)
            }
            Expression::DestinationProperty { tag, prop } => {
                self.eval_destination_property(tag, prop, context)
            }
            // 一元操作扩展
            Expression::UnaryPlus(expr) => self.evaluate(expr, context),
            Expression::UnaryNegate(expr) => {
                let value = self.evaluate(expr, context)?;
                UnaryOperationEvaluator::evaluate(&UnaryOperator::Minus, &value)
            }
            Expression::UnaryNot(expr) => {
                let value = self.evaluate(expr, context)?;
                UnaryOperationEvaluator::evaluate(&UnaryOperator::Not, &value)
            }
            Expression::UnaryIncr(expr) => {
                let value = self.evaluate(expr, context)?;
                UnaryOperationEvaluator::evaluate(&UnaryOperator::Increment, &value)
            }
            Expression::UnaryDecr(expr) => {
                let value = self.evaluate(expr, context)?;
                UnaryOperationEvaluator::evaluate(&UnaryOperator::Decrement, &value)
            }
            Expression::IsNull(expr) => {
                let value = self.evaluate(expr, context)?;
                UnaryOperationEvaluator::evaluate(&UnaryOperator::IsNull, &value)
            }
            Expression::IsNotNull(expr) => {
                let value = self.evaluate(expr, context)?;
                UnaryOperationEvaluator::evaluate(&UnaryOperator::IsNotNull, &value)
            }
            Expression::IsEmpty(expr) => {
                let value = self.evaluate(expr, context)?;
                UnaryOperationEvaluator::evaluate(&UnaryOperator::IsEmpty, &value)
            }
            Expression::IsNotEmpty(expr) => {
                let value = self.evaluate(expr, context)?;
                UnaryOperationEvaluator::evaluate(&UnaryOperator::IsNotEmpty, &value)
            }
            // 列表推导
            Expression::ListComprehension {
                generator,
                condition,
            } => {
                let gen_value = self.evaluate(generator, context)?;
                match gen_value {
                    Value::List(items) => {
                        let mut result = Vec::new();
                        for item in items {
                            if let Some(cond) = condition {
                                let cond_result = self.evaluate(cond, context)?;
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
            // 谓词表达式
            Expression::Predicate { list, condition } => {
                let list_value = self.evaluate(list, context)?;
                match list_value {
                    Value::List(items) => {
                        let mut result = Vec::new();
                        for item in items {
                            let cond_result = self.evaluate(condition, context)?;
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
            // 归约表达式
            Expression::Reduce {
                list,
                var,
                initial,
                expr,
            } => {
                let list_value = self.evaluate(list, context)?;
                let mut acc = self.evaluate(initial, context)?;

                if let Value::List(items) = list_value {
                    for item in items {
                        context.set_variable(var.clone(), item);
                        acc = self.evaluate(expr, context)?;
                    }
                }
                Ok(acc)
            }
            // 文本搜索表达式
            Expression::ESQuery(_) => {
                // 文本搜索暂时返回null
                Ok(Value::Null(crate::core::NullType::Null))
            }
            // UUID表达式
            Expression::UUID => Ok(Value::String(uuid::Uuid::new_v4().to_string())),
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
            Expression::Literal(value) => Ok(value.clone()),
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
                BinaryOperationEvaluator::evaluate(&left_value, op, &right_value)
            }
            Expression::Unary { op, operand } => {
                let value = self.eval_expression_generic(operand, context)?;
                UnaryOperationEvaluator::evaluate(op, &value)
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
                let dyn_ctx: &mut dyn ExpressionContext = context;
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
                    _ => Err(ExpressionError::type_error("length函数需要字符串、列表或映射类型")),
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
    fn eval_aggregate_function_single(
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
                // 单参数情况下的PERCENTILE，返回原始值
                if arg.is_null() {
                    Ok(Value::Null(crate::core::NullType::Null))
                } else {
                    Ok(arg.clone())
                }
            }
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
                    match arg.lt(&min) {
                        Ok(Value::Bool(true)) => min = arg.clone(),
                        Ok(_) => {}
                        Err(e) => return Err(ExpressionError::runtime_error(e)),
                    }
                }
                Ok(min)
            }
            AggregateFunction::Max => {
                let mut max = args[0].clone();
                for arg in args.iter().skip(1) {
                    match arg.gt(&max) {
                        Ok(Value::Bool(true)) => max = arg.clone(),
                        Ok(_) => {}
                        Err(e) => return Err(ExpressionError::runtime_error(e)),
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
                // PERCENTILE函数 - 需要两个参数：值数组和百分位数
                if args.len() < 2 {
                    return Err(ExpressionError::argument_count_error(2, args.len()));
                }
                
                // 获取百分位数值（第二个参数）
                let percentile = match &args[1] {
                    Value::Int(v) => *v as f64,
                    Value::Float(v) => *v,
                    _ => return Err(ExpressionError::type_error("Percentile must be a number")),
                };
                
                if percentile < 0.0 || percentile > 100.0 {
                    return Err(ExpressionError::new(
                        ExpressionErrorType::InvalidOperation,
                        "Percentile must be between 0 and 100"
                    ));
                }
                
                // 获取值数组（第一个参数）
                let values = match &args[0] {
                    Value::List(list) => list,
                    _ => return Err(ExpressionError::type_error("First argument must be a list")),
                };
                
                if values.is_empty() {
                    return Ok(Value::Null(crate::core::NullType::NaN));
                }
                
                // 提取数值并排序
                let mut numeric_values = Vec::new();
                for value in values {
                    match value {
                        Value::Int(v) => numeric_values.push(*v as f64),
                        Value::Float(v) => numeric_values.push(*v),
                        _ => continue, // 跳过非数值
                    }
                }
                
                if numeric_values.is_empty() {
                    return Ok(Value::Null(crate::core::NullType::NaN));
                }
                
                numeric_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                
                // 计算百分位数
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

    /// 求值标签表达式
    fn eval_label_expression(
        &self,
        label_name: &str,
        context: &mut dyn crate::expression::ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        if let Some(vertex) = context.get_vertex() {
            let label_list: Vec<Value> = vertex.tags.iter()
                .map(|tag| Value::String(tag.name.clone()))
                .collect();
            Ok(Value::List(label_list))
        } else {
            Err(ExpressionError::runtime_error(format!("标签表达式需要顶点上下文: {}", label_name)))
        }
    }

    /// 求值标签属性表达式
    fn eval_tag_property(
        &self,
        tag_name: &str,
        prop_name: &str,
        context: &mut dyn crate::expression::ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        if let Some(vertex) = context.get_vertex() {
            for tag in &vertex.tags {
                if tag.name == tag_name {
                    if let Some(value) = tag.properties.get(prop_name) {
                        return Ok(value.clone());
                    }
                }
            }
            Err(ExpressionError::runtime_error(format!(
                "标签属性不存在: {}.{}", tag_name, prop_name
            )))
        } else {
            Err(ExpressionError::runtime_error(format!(
                "标签属性表达式需要顶点上下文: {}.{}", tag_name, prop_name
            )))
        }
    }

    /// 求值边属性表达式
    fn eval_edge_property(
        &self,
        edge_name: &str,
        prop_name: &str,
        context: &mut dyn crate::expression::ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        if let Some(edge) = context.get_edge() {
            if edge_name.is_empty() || edge.edge_type() == edge_name {
                if let Some(value) = edge.properties().get(prop_name) {
                    return Ok(value.clone());
                }
                Err(ExpressionError::runtime_error(format!(
                    "边属性不存在: {}.{}", edge_name, prop_name
                )))
            } else {
                Err(ExpressionError::runtime_error(format!(
                    "边名称不匹配: 期望 '{}', 实际 '{}'", edge_name, edge.edge_type()
                )))
            }
        } else {
            Err(ExpressionError::runtime_error(format!(
                "边属性表达式需要边上下文: {}.{}", edge_name, prop_name
            )))
        }
    }

    /// 求值变量属性表达式
    fn eval_variable_property(
        &self,
        var_name: &str,
        prop_name: &str,
        context: &mut dyn crate::expression::ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        if let Some(value) = context.get_variable(var_name) {
            match value {
                Value::Vertex(vertex) => {
                    if let Some(prop_value) = vertex.properties.get(prop_name) {
                        Ok(prop_value.clone())
                    } else {
                        Err(ExpressionError::runtime_error(format!(
                            "顶点属性不存在: {}.{}", var_name, prop_name
                        )))
                    }
                }
                Value::Edge(edge) => {
                    if let Some(prop_value) = edge.properties().get(prop_name) {
                        Ok(prop_value.clone())
                    } else {
                        Err(ExpressionError::runtime_error(format!(
                            "边属性不存在: {}.{}", var_name, prop_name
                        )))
                    }
                }
                Value::Map(map) => {
                    if let Some(prop_value) = map.get(prop_name) {
                        Ok(prop_value.clone())
                    } else {
                        Err(ExpressionError::runtime_error(format!(
                            "映射属性不存在: {}.{}", var_name, prop_name
                        )))
                    }
                }
                _ => Err(ExpressionError::type_error(format!(
                    "变量属性访问需要顶点、边或映射类型: {}", var_name
                ))),
            }
        } else {
            Err(ExpressionError::undefined_variable(var_name))
        }
    }

    /// 求值源属性表达式
    fn eval_source_property(
        &self,
        tag_name: &str,
        prop_name: &str,
        context: &mut dyn crate::expression::ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        if let Some(edge) = context.get_edge() {
            let source_var = format!("_src_{}", edge.src());
            if let Some(value) = context.get_variable(&source_var) {
                if let Value::Vertex(vertex) = value {
                    for tag in &vertex.tags {
                        if tag.name == tag_name {
                            if let Some(prop_value) = tag.properties.get(prop_name) {
                                return Ok(prop_value.clone());
                            }
                        }
                    }
                    Err(ExpressionError::runtime_error(format!(
                        "源标签属性不存在: $^.{}.{}", tag_name, prop_name
                    )))
                } else {
                    Err(ExpressionError::type_error(format!(
                        "源属性表达式需要顶点类型: $^.{}.{}", tag_name, prop_name
                    )))
                }
            } else {
                Err(ExpressionError::undefined_variable(&source_var))
            }
        } else {
            Err(ExpressionError::runtime_error(format!(
                "源属性表达式需要边上下文: $^.{}.{}", tag_name, prop_name
            )))
        }
    }

    /// 求值目的属性表达式
    fn eval_destination_property(
        &self,
        tag_name: &str,
        prop_name: &str,
        context: &mut dyn crate::expression::ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        if let Some(edge) = context.get_edge() {
            let dest_var = format!("_dst_{}", edge.dst());
            if let Some(value) = context.get_variable(&dest_var) {
                if let Value::Vertex(vertex) = value {
                    for tag in &vertex.tags {
                        if tag.name == tag_name {
                            if let Some(prop_value) = tag.properties.get(prop_name) {
                                return Ok(prop_value.clone());
                            }
                        }
                    }
                    Err(ExpressionError::runtime_error(format!(
                        "目的标签属性不存在: $$.{}.{}", tag_name, prop_name
                    )))
                } else {
                    Err(ExpressionError::type_error(format!(
                        "目的属性表达式需要顶点类型: $$.{}.{}", tag_name, prop_name
                    )))
                }
            } else {
                Err(ExpressionError::undefined_variable(&dest_var))
            }
        } else {
            Err(ExpressionError::runtime_error(format!(
                "目的属性表达式需要边上下文: $$.{}.{}", tag_name, prop_name
            )))
        }
    }

    /// 求值CASE表达式
    fn eval_case_expression(
        &self,
        cases: &[(Expression, Expression)],
        default: Option<&Expression>,
        context: &mut dyn crate::expression::ExpressionContext,
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

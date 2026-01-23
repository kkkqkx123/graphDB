//! 表达式类型验证系统
//! 负责验证表达式的类型信息（类型推导使用 DeduceTypeVisitor）

use crate::core::Expression;
use crate::core::ValueTypeDef;
use crate::core::AggregateFunction;
use crate::core::BinaryOperator;
use crate::core::UnaryOperator;
use crate::core::Value;
use crate::core::TypeUtils;
use crate::query::validator::base_validator::ValueType;
use crate::query::validator::structs::*;
use crate::query::validator::{ValidationError, ValidationErrorType};
use crate::query::validator::validation_interface::ValidationContext;
use std::collections::HashMap;

/// 表达式验证上下文Trait
/// 定义表达式验证所需的基本接口
pub trait ExpressionValidationContext {
    fn get_aliases(&self) -> &HashMap<String, AliasType>;
    fn get_variable_types(&self) -> Option<&HashMap<String, ValueTypeDef>>;
}

impl<T: ValidationContext> ExpressionValidationContext for T {
    fn get_aliases(&self) -> &HashMap<String, AliasType> {
        self.get_aliases()
    }

    fn get_variable_types(&self) -> Option<&HashMap<String, ValueTypeDef>> {
        None
    }
}

/// 表达式类型验证器
/// 负责验证表达式的类型是否符合预期
pub struct TypeValidator;

impl TypeValidator {
    pub fn new() -> Self {
        Self
    }

    /// 检查类型是否可以用于索引
    /// 使用 TypeUtils 的统一实现
    pub fn is_indexable_type(&self, type_def: &ValueTypeDef) -> bool {
        TypeUtils::is_indexable_type(type_def)
    }

    /// 获取类型的默认值
    /// 使用 TypeUtils 的统一实现
    pub fn get_default_value(&self, type_def: &ValueTypeDef) -> Option<Expression> {
        TypeUtils::get_default_value(type_def).map(|v| Expression::Literal(v))
    }

    /// 验证类型是否可以强制转换
    /// 使用 TypeUtils 的统一实现，确保行为一致
    pub fn can_cast(&self, from: &ValueTypeDef, to: &ValueTypeDef) -> bool {
        TypeUtils::can_cast(from, to)
    }

    /// 获取类型的字符串表示
    /// 使用 TypeUtils 的统一实现
    pub fn type_to_string(&self, type_def: &ValueTypeDef) -> String {
        TypeUtils::type_to_string(type_def)
    }

    /// 检查两个类型是否兼容
    /// 使用 TypeUtils 的统一实现
    pub fn are_types_compatible(&self, left: &ValueTypeDef, right: &ValueTypeDef) -> bool {
        TypeUtils::are_types_compatible(left, right)
    }

    /// 验证表达式类型
    pub fn validate_expression_type<C: ExpressionValidationContext>(
        &self,
        expr: &Expression,
        context: &C,
        expected_type: ValueTypeDef,
    ) -> Result<(), ValidationError> {
        self.validate_expression_type_full(expr, context, expected_type)
    }

    /// 完整的表达式类型验证（使用上下文）
    pub fn validate_expression_type_full<C: ExpressionValidationContext>(
        &self,
        expr: &Expression,
        context: &C,
        expected_type: ValueTypeDef,
    ) -> Result<(), ValidationError> {
        match expr {
            Expression::Literal(value) => {
                let actual_type = value.get_type();
                if self.are_types_compatible(&actual_type, &expected_type) {
                    Ok(())
                } else {
                    Err(ValidationError::new(
                        format!(
                            "表达式类型不匹配: 期望 {:?}, 实际 {:?}, 表达式: {:?}",
                            expected_type, actual_type, expr
                        ),
                        ValidationErrorType::TypeError,
                    ))
                }
            }
            Expression::Binary { op, left, right } => {
                self.validate_binary_expression_type(op, left, right, context, expected_type)
            }
            Expression::Unary { op, operand } => {
                self.validate_unary_expression_type(op, operand, context, expected_type)
            }
            Expression::Function { name, args } => {
                self.validate_function_return_type(name, args, context, expected_type)
            }
            Expression::Aggregate { func, arg: _, distinct: _ } => {
                self.validate_aggregate_return_type(func, expected_type)
            }
            Expression::Variable(name) => {
                self.validate_variable_type(name, context, expected_type)
            }
            _ => {
                let actual_type = self.deduce_expression_type_simple(expr);
                if self.are_types_compatible(&actual_type, &expected_type) {
                    Ok(())
                } else {
                    Err(ValidationError::new(
                        format!(
                            "表达式类型不匹配: 期望 {:?}, 实际 {:?}",
                            expected_type, actual_type
                        ),
                        ValidationErrorType::TypeError,
                    ))
                }
            }
        }
    }

    /// 验证二元表达式类型
    fn validate_binary_expression_type<C: ExpressionValidationContext>(
        &self,
        op: &crate::core::BinaryOperator,
        left: &Expression,
        right: &Expression,
        context: &C,
        expected_type: ValueTypeDef,
    ) -> Result<(), ValidationError> {
        match op {
            crate::core::BinaryOperator::Equal
            | crate::core::BinaryOperator::NotEqual
            | crate::core::BinaryOperator::LessThan
            | crate::core::BinaryOperator::LessThanOrEqual
            | crate::core::BinaryOperator::GreaterThan
            | crate::core::BinaryOperator::GreaterThanOrEqual => {
                if expected_type == ValueTypeDef::Bool {
                    let left_type = self.deduce_expression_type_full(left, context);
                    let right_type = self.deduce_expression_type_full(right, context);
                    if self.are_types_compatible(&left_type, &right_type) {
                        Ok(())
                    } else {
                        Err(ValidationError::new(
                            format!(
                                "比较操作符的操作数类型不匹配: 左 {:?}, 右 {:?}",
                                left_type, right_type
                            ),
                            ValidationErrorType::TypeError,
                        ))
                    }
                } else {
                    Err(ValidationError::new(
                        format!(
                            "比较操作符的结果是布尔值，但期望类型是 {:?}",
                            expected_type
                        ),
                        ValidationErrorType::TypeError,
                    ))
                }
            }
            crate::core::BinaryOperator::And | crate::core::BinaryOperator::Or => {
                if expected_type == ValueTypeDef::Bool {
                    self.validate_expression_type_full(left, context, ValueTypeDef::Bool)?;
                    self.validate_expression_type_full(right, context, ValueTypeDef::Bool)
                } else {
                    Err(ValidationError::new(
                        format!(
                            "逻辑操作符的结果是布尔值，但期望类型是 {:?}",
                            expected_type
                        ),
                        ValidationErrorType::TypeError,
                    ))
                }
            }
            _ => {
                let left_type = self.deduce_expression_type_full(left, context);
                let right_type = self.deduce_expression_type_full(right, context);
                let result_type = self.deduce_binary_expr_type(op, &left_type, &right_type);
                
                if self.are_types_compatible(&result_type, &expected_type) {
                    Ok(())
                } else {
                    Err(ValidationError::new(
                        format!(
                            "算术操作符的结果类型不匹配: 期望 {:?}, 实际 {:?}",
                            expected_type, result_type
                        ),
                        ValidationErrorType::TypeError,
                    ))
                }
            }
        }
    }

    /// 验证一元表达式类型
    fn validate_unary_expression_type<C: ExpressionValidationContext>(
        &self,
        op: &crate::core::UnaryOperator,
        operand: &Expression,
        context: &C,
        expected_type: ValueTypeDef,
    ) -> Result<(), ValidationError> {
        match op {
            crate::core::UnaryOperator::Not => {
                if expected_type == ValueTypeDef::Bool {
                    self.validate_expression_type_full(operand, context, ValueTypeDef::Bool)
                } else {
                    Err(ValidationError::new(
                        format!(
                            "逻辑非的结果是布尔值，但期望类型是 {:?}",
                            expected_type
                        ),
                        ValidationErrorType::TypeError,
                    ))
                }
            }
            crate::core::UnaryOperator::Minus | crate::core::UnaryOperator::Plus => {
                let operand_type = self.deduce_expression_type_full(operand, context);
                if self.are_types_compatible(&operand_type, &expected_type) {
                    Ok(())
                } else {
                    Err(ValidationError::new(
                        format!(
                            "一元操作符的结果类型不匹配: 期望 {:?}, 实际 {:?}",
                            expected_type, operand_type
                        ),
                        ValidationErrorType::TypeError,
                    ))
                }
            }
            _ => Ok(()),
        }
    }

    /// 验证函数返回类型
    fn validate_function_return_type<C: ExpressionValidationContext>(
        &self,
        name: &str,
        args: &[Expression],
        context: &C,
        expected_type: ValueTypeDef,
    ) -> Result<(), ValidationError> {
        let return_type = self.deduce_function_return_type(name, args, context);
        if self.are_types_compatible(&return_type, &expected_type) {
            Ok(())
        } else {
            Err(ValidationError::new(
                format!(
                    "函数 {:?} 的返回类型是 {:?}, 但期望类型是 {:?}",
                    name, return_type, expected_type
                ),
                ValidationErrorType::TypeError,
            ))
        }
    }

    /// 验证聚合函数返回类型
    fn validate_aggregate_return_type(
        &self,
        func: &crate::core::AggregateFunction,
        expected_type: ValueTypeDef,
    ) -> Result<(), ValidationError> {
        let return_type = self.deduce_aggregate_return_type(func);
        if self.are_types_compatible(&return_type, &expected_type) {
            Ok(())
        } else {
            Err(ValidationError::new(
                format!(
                    "聚合函数 {:?} 的返回类型是 {:?}, 但期望类型是 {:?}",
                    func, return_type, expected_type
                ),
                ValidationErrorType::TypeError,
            ))
        }
    }

    /// 验证变量类型
    fn validate_variable_type<C: ExpressionValidationContext>(
        &self,
        name: &str,
        context: &C,
        expected_type: ValueTypeDef,
    ) -> Result<(), ValidationError> {
        if let Some(var_types) = context.get_variable_types() {
            if let Some(var_type) = var_types.get(name) {
                if self.are_types_compatible(var_type, &expected_type) {
                    return Ok(());
                } else {
                    return Err(ValidationError::new(
                        format!(
                            "变量 {:?} 的类型是 {:?}, 但期望类型是 {:?}",
                            name, var_type, expected_type
                        ),
                        ValidationErrorType::TypeError,
                    ));
                }
            }
        }
        Ok(())
    }

    /// 简化的表达式类型推导
    pub fn deduce_expression_type_simple(&self, expr: &Expression) -> ValueTypeDef {
        match expr {
            Expression::Literal(value) => value.get_type(),
            Expression::Variable(_) => ValueTypeDef::Empty,
            Expression::Binary { op, .. } => self.deduce_binary_expr_type_simple(op),
            Expression::Unary { op, .. } => self.deduce_unary_expr_type_simple(op),
            Expression::Function { name, .. } => self.deduce_function_return_type_simple(name),
            Expression::Aggregate { func, .. } => self.deduce_aggregate_return_type(func),
            _ => ValueTypeDef::Empty,
        }
    }

    /// 完整的表达式类型推导（使用上下文）
    pub fn deduce_expression_type_full<C: ExpressionValidationContext>(
        &self,
        expr: &Expression,
        context: &C,
    ) -> ValueTypeDef {
        match expr {
            Expression::Literal(value) => value.get_type(),
            Expression::Variable(name) => {
                if let Some(var_types) = context.get_variable_types() {
                    if let Some(var_type) = var_types.get(name) {
                        return var_type.clone();
                    }
                }
                ValueTypeDef::Empty
            }
            Expression::Binary { op, left, right } => {
                let left_type = self.deduce_expression_type_full(left, context);
                let right_type = self.deduce_expression_type_full(right, context);
                self.deduce_binary_expr_type(op, &left_type, &right_type)
            }
            Expression::Unary { op, operand } => {
                let operand_type = self.deduce_expression_type_full(operand, context);
                self.deduce_unary_expr_type(op, &operand_type)
            }
            Expression::Function { name, args } => {
                self.deduce_function_return_type(name, args, context)
            }
            Expression::Aggregate { func, .. } => self.deduce_aggregate_return_type(func),
            _ => ValueTypeDef::Empty,
        }
    }

    /// 推导二元表达式类型
    fn deduce_binary_expr_type(
        &self,
        op: &crate::core::BinaryOperator,
        left_type: &ValueTypeDef,
        right_type: &ValueTypeDef,
    ) -> ValueTypeDef {
        match op {
            crate::core::BinaryOperator::Equal
            | crate::core::BinaryOperator::NotEqual
            | crate::core::BinaryOperator::LessThan
            | crate::core::BinaryOperator::LessThanOrEqual
            | crate::core::BinaryOperator::GreaterThan
            | crate::core::BinaryOperator::GreaterThanOrEqual => ValueTypeDef::Bool,
            crate::core::BinaryOperator::And | crate::core::BinaryOperator::Or => ValueTypeDef::Bool,
            _ => {
                if *left_type == ValueTypeDef::Float || *right_type == ValueTypeDef::Float {
                    ValueTypeDef::Float
                } else if *left_type == ValueTypeDef::Int || *right_type == ValueTypeDef::Int {
                    ValueTypeDef::Int
                } else {
                    ValueTypeDef::Empty
                }
            }
        }
    }

    /// 推导一元表达式类型
    fn deduce_unary_expr_type(
        &self,
        op: &crate::core::UnaryOperator,
        operand_type: &ValueTypeDef,
    ) -> ValueTypeDef {
        match op {
            crate::core::UnaryOperator::Not => ValueTypeDef::Bool,
            crate::core::UnaryOperator::Minus | crate::core::UnaryOperator::Plus => operand_type.clone(),
            _ => ValueTypeDef::Empty,
        }
    }

    /// 推导函数返回类型
    fn deduce_function_return_type<C: ExpressionValidationContext>(
        &self,
        name: &str,
        _args: &[Expression],
        _context: &C,
    ) -> ValueTypeDef {
        match name.to_lowercase().as_str() {
            "abs" | "length" | "size" => ValueTypeDef::Int,
            "round" | "floor" | "ceil" => ValueTypeDef::Int,
            "sqrt" | "pow" | "sin" | "cos" | "tan" => ValueTypeDef::Float,
            "concat" | "substring" | "trim" | "ltrim" | "rtrim" => ValueTypeDef::String,
            "upper" | "lower" => ValueTypeDef::String,
            "type" => ValueTypeDef::String,
            "id" => ValueTypeDef::Int,
            "properties" => ValueTypeDef::Map,
            "labels" => ValueTypeDef::List,
            "keys" => ValueTypeDef::List,
            "values" => ValueTypeDef::List,
            "range" => ValueTypeDef::List,
            "reverse" => ValueTypeDef::List,
            "head" | "last" | "tail" => ValueTypeDef::Empty,
            _ => ValueTypeDef::Empty,
        }
    }

    /// 推导聚合函数返回类型
    pub fn deduce_aggregate_return_type(
        &self,
        func: &crate::core::AggregateFunction,
    ) -> ValueTypeDef {
        match func {
            crate::core::AggregateFunction::Count(_) => ValueTypeDef::Int,
            crate::core::AggregateFunction::Sum(_) => ValueTypeDef::Float,
            crate::core::AggregateFunction::Avg(_) => ValueTypeDef::Float,
            crate::core::AggregateFunction::Max(_) | crate::core::AggregateFunction::Min(_) => ValueTypeDef::Empty,
            crate::core::AggregateFunction::Collect(_) => ValueTypeDef::List,
            crate::core::AggregateFunction::Distinct(_) => ValueTypeDef::Empty,
            crate::core::AggregateFunction::Percentile(_, _) => ValueTypeDef::Float,
        }
    }

    /// 简化的二元表达式类型推导
    fn deduce_binary_expr_type_simple(&self, op: &crate::core::BinaryOperator) -> ValueTypeDef {
        match op {
            crate::core::BinaryOperator::Equal
            | crate::core::BinaryOperator::NotEqual
            | crate::core::BinaryOperator::LessThan
            | crate::core::BinaryOperator::LessThanOrEqual
            | crate::core::BinaryOperator::GreaterThan
            | crate::core::BinaryOperator::GreaterThanOrEqual => ValueTypeDef::Bool,
            crate::core::BinaryOperator::And | crate::core::BinaryOperator::Or => ValueTypeDef::Bool,
            _ => ValueTypeDef::Empty,
        }
    }

    /// 简化的一元表达式类型推导
    fn deduce_unary_expr_type_simple(&self, op: &crate::core::UnaryOperator) -> ValueTypeDef {
        match op {
            crate::core::UnaryOperator::Not => ValueTypeDef::Bool,
            crate::core::UnaryOperator::Minus | crate::core::UnaryOperator::Plus => ValueTypeDef::Empty,
            _ => ValueTypeDef::Empty,
        }
    }

    /// 简化的函数返回类型推导
    fn deduce_function_return_type_simple(&self, name: &str) -> ValueTypeDef {
        match name.to_lowercase().as_str() {
            "abs" | "length" | "size" => ValueTypeDef::Int,
            "round" | "floor" | "ceil" => ValueTypeDef::Int,
            "sqrt" | "pow" | "sin" | "cos" | "tan" => ValueTypeDef::Float,
            "concat" | "substring" | "trim" | "ltrim" | "rtrim" => ValueTypeDef::String,
            "upper" | "lower" => ValueTypeDef::String,
            "type" => ValueTypeDef::String,
            "id" => ValueTypeDef::Int,
            "properties" => ValueTypeDef::Map,
            "labels" => ValueTypeDef::List,
            "keys" => ValueTypeDef::List,
            "values" => ValueTypeDef::List,
            "range" => ValueTypeDef::List,
            "reverse" => ValueTypeDef::List,
            "head" | "last" | "tail" => ValueTypeDef::Empty,
            _ => ValueTypeDef::Empty,
        }
    }

    /// 检查表达式是否包含聚合函数
    pub fn has_aggregate_expression(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Aggregate { .. } => true,
            Expression::Binary { left, right, .. } => {
                self.has_aggregate_expression(left) || self.has_aggregate_expression(right)
            }
            Expression::Unary { operand, .. } => self.has_aggregate_expression(operand),
            Expression::Function { args, .. } => {
                args.iter().any(|arg| self.has_aggregate_expression(arg))
            }
            Expression::List(items) => items.iter().any(|item| self.has_aggregate_expression(item)),
            Expression::Map(pairs) => {
                pairs.iter().any(|(_, value)| self.has_aggregate_expression(value))
            }
            Expression::Case {
                conditions,
                default,
            } => {
                conditions.iter().any(|(when_expr, then_expr)| {
                    self.has_aggregate_expression(when_expr) || self.has_aggregate_expression(then_expr)
                }) || default.as_ref().map_or(false, |d| self.has_aggregate_expression(d))
            }
            Expression::Property { object, .. } => self.has_aggregate_expression(object),
            Expression::Subscript { collection, index } => {
                self.has_aggregate_expression(collection) || self.has_aggregate_expression(index)
            }
            Expression::Range {
                collection,
                start,
                end,
            } => {
                self.has_aggregate_expression(collection)
                    || start.as_ref().map_or(false, |s| self.has_aggregate_expression(s))
                    || end.as_ref().map_or(false, |e| self.has_aggregate_expression(e))
            }
            Expression::Path(items) => items.iter().any(|item| self.has_aggregate_expression(item)),
            Expression::TypeCast { expr, .. } => self.has_aggregate_expression(expr),
            _ => false,
        }
    }

    /// 验证分组键类型
    pub fn validate_group_key_type<C: ExpressionValidationContext>(
        &self,
        group_key: &Expression,
        context: &C,
    ) -> Result<(), ValidationError> {
        let key_type = self.deduce_expression_type_full(group_key, context);
        
        match key_type {
            ValueTypeDef::Int
            | ValueTypeDef::Float
            | ValueTypeDef::String
            | ValueTypeDef::Bool
            | ValueTypeDef::Date
            | ValueTypeDef::Time
            | ValueTypeDef::DateTime => Ok(()),
            _ => Err(ValidationError::new(
                format!(
                    "分组键的类型 {:?} 不支持，只支持基本类型",
                    key_type
                ),
                ValidationErrorType::TypeError,
            )),
        }
    }

    /// 从 ValueTypeDef 转换为 ValueType
    pub fn value_type_def_to_value_type(type_def: &ValueTypeDef) -> ValueType {
        match type_def {
            ValueTypeDef::Empty => ValueType::Unknown,
            ValueTypeDef::Null => ValueType::Null,
            ValueTypeDef::Bool => ValueType::Bool,
            ValueTypeDef::Int => ValueType::Int,
            ValueTypeDef::Float => ValueType::Float,
            ValueTypeDef::String => ValueType::String,
            ValueTypeDef::Date => ValueType::Date,
            ValueTypeDef::Time => ValueType::Time,
            ValueTypeDef::DateTime => ValueType::DateTime,
            ValueTypeDef::Vertex => ValueType::Vertex,
            ValueTypeDef::Edge => ValueType::Edge,
            ValueTypeDef::Path => ValueType::Path,
            ValueTypeDef::List => ValueType::List,
            ValueTypeDef::Map => ValueType::Map,
            ValueTypeDef::Set => ValueType::Set,
            _ => ValueType::Unknown,
        }
    }

    /// 从 ValueType 转换为 ValueTypeDef
    pub fn value_type_to_value_type_def(type_: &ValueType) -> ValueTypeDef {
        match type_ {
            ValueType::Unknown => ValueTypeDef::Empty,
            ValueType::Bool => ValueTypeDef::Bool,
            ValueType::Int => ValueTypeDef::Int,
            ValueType::Float => ValueTypeDef::Float,
            ValueType::String => ValueTypeDef::String,
            ValueType::Date => ValueTypeDef::Date,
            ValueType::Time => ValueTypeDef::Time,
            ValueType::DateTime => ValueTypeDef::DateTime,
            ValueType::Vertex => ValueTypeDef::Vertex,
            ValueType::Edge => ValueTypeDef::Edge,
            ValueType::Path => ValueTypeDef::Path,
            ValueType::List => ValueTypeDef::List,
            ValueType::Map => ValueTypeDef::Map,
            ValueType::Set => ValueTypeDef::Set,
            ValueType::Null => ValueTypeDef::Null,
        }
    }

    /// 完整的表达式类型推导（增强版）
    pub fn deduce_expr_type(&self, expr: &Expression) -> ValueType {
        match expr {
            Expression::Literal(value) => {
                Self::value_type_def_to_value_type(&value.get_type())
            }
            Expression::Variable(_) => {
                ValueType::Unknown
            }
            Expression::Property { object, property: _ } => {
                self.deduce_expr_type(object)
            }
            Expression::Binary { op, left, right } => {
                let left_type = self.deduce_expr_type(left);
                let right_type = self.deduce_expr_type(right);
                self.deduce_binary_expr_type_enhanced(op, &left_type, &right_type)
            }
            Expression::Unary { op, operand } => {
                let operand_type = self.deduce_expr_type(operand);
                self.deduce_unary_expr_type_enhanced(op, &operand_type)
            }
            Expression::Function { name, args } => {
                self.deduce_function_type_enhanced(name, args)
            }
            Expression::Aggregate { func, arg: _, distinct: _ } => {
                self.deduce_aggregate_type_enhanced(func)
            }
            Expression::List(_) => {
                ValueType::List
            }
            Expression::Map(_) => ValueType::Map,
            Expression::Case { conditions, default } => {
                for (_, then_expr) in conditions {
                    let then_type = self.deduce_expr_type(then_expr);
                    if then_type != ValueType::Unknown {
                        return then_type;
                    }
                }
                if let Some(default_expr) = default {
                    return self.deduce_expr_type(default_expr);
                }
                ValueType::Unknown
            }
            Expression::TypeCast { expr: _, target_type: _ } => {
                ValueType::Unknown
            }
            Expression::Subscript { collection, index: _ } => {
                self.deduce_expr_type(collection)
            }
            Expression::Range { collection, start: _, end: _ } => {
                self.deduce_expr_type(collection)
            }
            Expression::Path(_) => ValueType::Path,
            Expression::Label(_) => ValueType::String,
            _ => ValueType::Unknown,
        }
    }

    /// 增强版二元表达式类型推导
    pub fn deduce_binary_expr_type_enhanced(
        &self,
        op: &BinaryOperator,
        left_type: &ValueType,
        right_type: &ValueType,
    ) -> ValueType {
        if op.is_arithmetic() {
            if *left_type == ValueType::Int && *right_type == ValueType::Int {
                ValueType::Int
            } else if matches!(*left_type, ValueType::Int | ValueType::Float)
                && matches!(*right_type, ValueType::Int | ValueType::Float) {
                ValueType::Float
            } else {
                ValueType::Unknown
            }
        } else if op.is_comparison() {
            ValueType::Bool
        } else if op.is_logical() {
            ValueType::Bool
        } else if matches!(
            op,
            BinaryOperator::StringConcat
                | BinaryOperator::Like
                | BinaryOperator::Contains
                | BinaryOperator::StartsWith
                | BinaryOperator::EndsWith
        ) {
            ValueType::String
        } else {
            ValueType::Unknown
        }
    }

    /// 增强版一元表达式类型推导
    pub fn deduce_unary_expr_type_enhanced(
        &self,
        op: &UnaryOperator,
        operand_type: &ValueType,
    ) -> ValueType {
        match op {
            UnaryOperator::Plus => operand_type.clone(),
            UnaryOperator::Minus => operand_type.clone(),
            UnaryOperator::Not => ValueType::Bool,
            UnaryOperator::IsNull => ValueType::Bool,
            UnaryOperator::IsNotNull => ValueType::Bool,
            UnaryOperator::IsEmpty => ValueType::Bool,
            UnaryOperator::IsNotEmpty => ValueType::Bool,
        }
    }

    /// 增强版函数返回类型推导
    pub fn deduce_function_type_enhanced(&self, name: &str, args: &[Expression]) -> ValueType {
        let name_lower = name.to_lowercase();
        match name_lower.as_str() {
            "id" => ValueType::String,
            "count" | "sum" | "avg" | "min" | "max" => ValueType::Float,
            "length" | "size" => ValueType::Int,
            "to_string" | "string" => ValueType::String,
            "to_int" | "to_integer" | "int" => ValueType::Int,
            "to_float" | "to_double" | "float" => ValueType::Float,
            "abs" => ValueType::Float,
            "floor" | "ceil" | "round" => ValueType::Int,
            "sqrt" | "exp" | "log" => ValueType::Float,
            "now" => ValueType::DateTime,
            "date" | "datetime" => ValueType::DateTime,
            "head" | "tail" | "last" => {
                if !args.is_empty() {
                    self.deduce_expr_type(&args[0])
                } else {
                    ValueType::Unknown
                }
            }
            "keys" => ValueType::List,
            "properties" => ValueType::Map,
            "labels" => ValueType::List,
            "type" => ValueType::String,
            "rank" => ValueType::Int,
            "src" | "dst" => ValueType::String,
            _ => ValueType::Unknown,
        }
    }

    /// 增强版聚合函数类型推导
    pub fn deduce_aggregate_type_enhanced(&self, func: &AggregateFunction) -> ValueType {
        match func {
            AggregateFunction::Count(_) => ValueType::Int,
            AggregateFunction::Sum(_) => ValueType::Float,
            AggregateFunction::Avg(_) => ValueType::Float,
            AggregateFunction::Min(_) => ValueType::Unknown,
            AggregateFunction::Max(_) => ValueType::Unknown,
            AggregateFunction::Collect(_) => ValueType::List,
            AggregateFunction::Distinct(_) => ValueType::Unknown,
            AggregateFunction::Percentile(_, _) => ValueType::Float,
        }
    }

    /// 增强版类型兼容性检查
    pub fn are_types_compatible_enhanced(&self, actual: &ValueType, expected: &ValueType) -> bool {
        if *actual == *expected {
            return true;
        }
        match (actual, expected) {
            (ValueType::Int, ValueType::Float) => true,
            (ValueType::Float, ValueType::Int) => true,
            (ValueType::Unknown, _) => true,
            (_, ValueType::Unknown) => true,
            _ => false,
        }
    }

    /// 增强版过滤条件类型验证
    pub fn validate_filter_type_enhanced(&self, filter: &Expression) -> Result<(), ValidationError> {
        let filter_type = self.deduce_expr_type(filter);
        match filter_type {
            ValueType::Bool => Ok(()),
            ValueType::Null | ValueType::Unknown => Ok(()),
            _ => Err(ValidationError::new(
                format!("过滤条件必须返回布尔类型，实际返回 {:?}", filter_type),
                ValidationErrorType::TypeError,
            )),
        }
    }

    /// 表达式常量折叠（增强版）
    pub fn fold_constant_expr_enhanced(&self, expr: &Expression) -> Option<Expression> {
        match expr {
            Expression::Binary { op, left, right } => {
                if let Some(lit_left) = self.fold_constant_expr_enhanced(left) {
                    if let Some(lit_right) = self.fold_constant_expr_enhanced(right) {
                        return self.evaluate_binary_expr_enhanced(op, &lit_left, &lit_right);
                    }
                }
                None
            }
            Expression::Unary { op, operand } => {
                if let Some(lit_operand) = self.fold_constant_expr_enhanced(operand) {
                    return self.evaluate_unary_expr_enhanced(op, &lit_operand);
                }
                None
            }
            _ => None,
        }
    }

    /// 计算二元表达式的常量值（增强版）
    fn evaluate_binary_expr_enhanced(
        &self,
        op: &BinaryOperator,
        left: &Expression,
        right: &Expression,
    ) -> Option<Expression> {
        match (left, right) {
            (Expression::Literal(l), Expression::Literal(r)) => {
                let result = self.compute_binary_op_enhanced(op, l, r)?;
                Some(Expression::Literal(result))
            }
            _ => None,
        }
    }

    /// 计算一元表达式的常量值（增强版）
    fn evaluate_unary_expr_enhanced(&self, op: &UnaryOperator, operand: &Expression) -> Option<Expression> {
        if let Expression::Literal(val) = operand {
            let result = self.compute_unary_op_enhanced(op, val)?;
            Some(Expression::Literal(result))
        } else {
            None
        }
    }

    /// 计算二元操作（增强版）
    fn compute_binary_op_enhanced(
        &self,
        op: &BinaryOperator,
        left: &Value,
        right: &Value,
    ) -> Option<Value> {
        if op.is_arithmetic() {
            match (left, right) {
                (Value::Int(l), Value::Int(r)) => Some(Value::Int(l + r)),
                (Value::Float(l), Value::Float(r)) => Some(Value::Float(l + r)),
                (Value::Int(l), Value::Float(r)) => Some(Value::Float(*l as f64 + r)),
                (Value::Float(l), Value::Int(r)) => Some(Value::Float(l + *r as f64)),
                _ => None,
            }
        } else {
            None
        }
    }

    /// 计算一元操作（增强版）
    fn compute_unary_op_enhanced(&self, op: &UnaryOperator, val: &Value) -> Option<Value> {
        match op {
            UnaryOperator::Minus => {
                match val {
                    Value::Int(n) => Some(Value::Int(-n)),
                    Value::Float(n) => Some(Value::Float(-n)),
                    _ => None,
                }
            }
            UnaryOperator::Not => {
                match val {
                    Value::Bool(b) => Some(Value::Bool(!b)),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_type_validator_creation() {
        let _type_validator = TypeValidator::new();
        assert!(true);
    }

    #[test]
    fn test_validate_literal_type() {
        let type_validator = TypeValidator::new();
        let literal_expr = Expression::Literal(Value::Bool(true));
        let context = ValidationContextImpl::new();
        
        let result = type_validator.validate_expression_type(&literal_expr, &context, ValueTypeDef::Bool);
        assert!(result.is_ok());
        
        let result = type_validator.validate_expression_type(&literal_expr, &context, ValueTypeDef::Int);
        assert!(result.is_err());
    }

    #[test]
    fn test_deduce_binary_expr_type() {
        let type_validator = TypeValidator::new();
        let op = crate::core::BinaryOperator::Equal;
        let left_type = ValueTypeDef::Int;
        let right_type = ValueTypeDef::Int;
        
        let result = type_validator.deduce_binary_expr_type(&op, &left_type, &right_type);
        assert_eq!(result, ValueTypeDef::Bool);
    }

    #[test]
    fn test_deduce_aggregate_return_type() {
        let type_validator = TypeValidator::new();
        
        let count_type = type_validator.deduce_aggregate_return_type(&crate::core::AggregateFunction::Count(None));
        assert_eq!(count_type, ValueTypeDef::Int);
        
        let sum_type = type_validator.deduce_aggregate_return_type(&crate::core::AggregateFunction::Sum("value".to_string()));
        assert_eq!(sum_type, ValueTypeDef::Float);
        
        let avg_type = type_validator.deduce_aggregate_return_type(&crate::core::AggregateFunction::Avg("value".to_string()));
        assert_eq!(avg_type, ValueTypeDef::Float);
        
        let collect_type = type_validator.deduce_aggregate_return_type(&crate::core::AggregateFunction::Collect("value".to_string()));
        assert_eq!(collect_type, ValueTypeDef::List);
    }

    #[test]
    fn test_are_types_compatible() {
        let type_validator = TypeValidator::new();

        assert!(type_validator.are_types_compatible(&ValueTypeDef::Int, &ValueTypeDef::Int));
        assert!(type_validator.are_types_compatible(&ValueTypeDef::Int, &ValueTypeDef::Float));
        assert!(type_validator.are_types_compatible(&ValueTypeDef::Float, &ValueTypeDef::Int));
        assert!(type_validator.are_types_compatible(&ValueTypeDef::Empty, &ValueTypeDef::Int));
        assert!(type_validator.are_types_compatible(&ValueTypeDef::String, &ValueTypeDef::Empty));
    }
}

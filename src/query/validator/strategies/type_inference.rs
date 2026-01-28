//! 表达式类型验证系统
//! 负责验证表达式的类型信息（类型推导使用 DeduceTypeVisitor）

use crate::core::Expression;
use crate::core::DataType;
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
    fn get_variable_types(&self) -> Option<&HashMap<String, DataType>>;
}

impl<T: ValidationContext> ExpressionValidationContext for T {
    fn get_aliases(&self) -> &HashMap<String, AliasType> {
        self.get_aliases()
    }

    fn get_variable_types(&self) -> Option<&HashMap<String, DataType>> {
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
    pub fn is_indexable_type(&self, type_def: &DataType) -> bool {
        TypeUtils::is_indexable_type(type_def)
    }

    /// 获取类型的默认值
    /// 使用 TypeUtils 的统一实现
    pub fn get_default_value(&self, type_def: &DataType) -> Option<Expression> {
        TypeUtils::get_default_value(type_def).map(|v| Expression::Literal(v))
    }

    /// 验证类型是否可以强制转换
    /// 使用 TypeUtils 的统一实现，确保行为一致
    pub fn can_cast(&self, from: &DataType, to: &DataType) -> bool {
        TypeUtils::can_cast(from, to)
    }

    /// 获取类型的字符串表示
    /// 使用 TypeUtils 的统一实现
    pub fn type_to_string(&self, type_def: &DataType) -> String {
        TypeUtils::type_to_string(type_def)
    }

    /// 检查两个类型是否兼容
    /// 使用 TypeUtils 的统一实现
    pub fn are_types_compatible(&self, left: &DataType, right: &DataType) -> bool {
        TypeUtils::are_types_compatible(left, right)
    }

    /// 验证表达式类型
    pub fn validate_expression_type<C: ExpressionValidationContext>(
        &self,
        expression: &Expression,
        context: &C,
        expected_type: DataType,
    ) -> Result<(), ValidationError> {
        self.validate_expression_type_full(expression, context, expected_type)
    }

    /// 完整的表达式类型验证（使用上下文）
    pub fn validate_expression_type_full<C: ExpressionValidationContext>(
        &self,
        expression: &Expression,
        context: &C,
        expected_type: DataType,
    ) -> Result<(), ValidationError> {
        match expression {
            Expression::Literal(value) => {
                let actual_type = value.get_type();
                if self.are_types_compatible(&actual_type, &expected_type) {
                    Ok(())
                } else {
                    Err(ValidationError::new(
                        format!(
                            "表达式类型不匹配: 期望 {:?}, 实际 {:?}, 表达式: {:?}",
                            expected_type, actual_type, expression
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
                let actual_type = self.deduce_expression_type_simple(expression);
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
        expected_type: DataType,
    ) -> Result<(), ValidationError> {
        match op {
            crate::core::BinaryOperator::Equal
            | crate::core::BinaryOperator::NotEqual
            | crate::core::BinaryOperator::LessThan
            | crate::core::BinaryOperator::LessThanOrEqual
            | crate::core::BinaryOperator::GreaterThan
            | crate::core::BinaryOperator::GreaterThanOrEqual => {
                if expected_type == DataType::Bool {
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
                if expected_type == DataType::Bool {
                    self.validate_expression_type_full(left, context, DataType::Bool)?;
                    self.validate_expression_type_full(right, context, DataType::Bool)
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
        expected_type: DataType,
    ) -> Result<(), ValidationError> {
        match op {
            crate::core::UnaryOperator::Not => {
                if expected_type == DataType::Bool {
                    self.validate_expression_type_full(operand, context, DataType::Bool)
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
        expected_type: DataType,
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
        expected_type: DataType,
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
        expected_type: DataType,
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
    pub fn deduce_expression_type_simple(&self, expression: &Expression) -> DataType {
        match expression {
            Expression::Literal(value) => value.get_type(),
            Expression::Variable(_) => DataType::Empty,
            Expression::Binary { op, .. } => self.deduce_binary_expr_type_simple(op),
            Expression::Unary { op, .. } => self.deduce_unary_expr_type_simple(op),
            Expression::Function { name, .. } => self.deduce_function_return_type_simple(name),
            Expression::Aggregate { func, .. } => self.deduce_aggregate_return_type(func),
            _ => DataType::Empty,
        }
    }

    /// 完整的表达式类型推导（使用上下文）
    pub fn deduce_expression_type_full<C: ExpressionValidationContext>(
        &self,
        expression: &Expression,
        context: &C,
    ) -> DataType {
        match expression {
            Expression::Literal(value) => value.get_type(),
            Expression::Variable(name) => {
                if let Some(var_types) = context.get_variable_types() {
                    if let Some(var_type) = var_types.get(name) {
                        return var_type.clone();
                    }
                }
                DataType::Empty
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
            _ => DataType::Empty,
        }
    }

    /// 推导二元表达式类型
    fn deduce_binary_expr_type(
        &self,
        op: &crate::core::BinaryOperator,
        left_type: &DataType,
        right_type: &DataType,
    ) -> DataType {
        match op {
            crate::core::BinaryOperator::Equal
            | crate::core::BinaryOperator::NotEqual
            | crate::core::BinaryOperator::LessThan
            | crate::core::BinaryOperator::LessThanOrEqual
            | crate::core::BinaryOperator::GreaterThan
            | crate::core::BinaryOperator::GreaterThanOrEqual => DataType::Bool,
            crate::core::BinaryOperator::And | crate::core::BinaryOperator::Or => DataType::Bool,
            _ => {
                if *left_type == DataType::Float || *right_type == DataType::Float {
                    DataType::Float
                } else if *left_type == DataType::Int || *right_type == DataType::Int {
                    DataType::Int
                } else {
                    DataType::Empty
                }
            }
        }
    }

    /// 推导一元表达式类型
    fn deduce_unary_expr_type(
        &self,
        op: &crate::core::UnaryOperator,
        operand_type: &DataType,
    ) -> DataType {
        match op {
            crate::core::UnaryOperator::Not => DataType::Bool,
            crate::core::UnaryOperator::Minus | crate::core::UnaryOperator::Plus => operand_type.clone(),
            _ => DataType::Empty,
        }
    }

    /// 推导函数返回类型
    fn deduce_function_return_type<C: ExpressionValidationContext>(
        &self,
        name: &str,
        _args: &[Expression],
        _context: &C,
    ) -> DataType {
        match name.to_lowercase().as_str() {
            "abs" | "length" | "size" => DataType::Int,
            "round" | "floor" | "ceil" => DataType::Int,
            "sqrt" | "pow" | "sin" | "cos" | "tan" => DataType::Float,
            "concat" | "substring" | "trim" | "ltrim" | "rtrim" => DataType::String,
            "upper" | "lower" => DataType::String,
            "type" => DataType::String,
            "id" => DataType::Int,
            "properties" => DataType::Map,
            "labels" => DataType::List,
            "keys" => DataType::List,
            "values" => DataType::List,
            "range" => DataType::List,
            "reverse" => DataType::List,
            "head" | "last" | "tail" => DataType::Empty,
            _ => DataType::Empty,
        }
    }

    /// 推导聚合函数返回类型
    pub fn deduce_aggregate_return_type(
        &self,
        func: &crate::core::AggregateFunction,
    ) -> DataType {
        match func {
            crate::core::AggregateFunction::Count(_) => DataType::Int,
            crate::core::AggregateFunction::Sum(_) => DataType::Float,
            crate::core::AggregateFunction::Avg(_) => DataType::Float,
            crate::core::AggregateFunction::Max(_) | crate::core::AggregateFunction::Min(_) => DataType::Empty,
            crate::core::AggregateFunction::Collect(_) => DataType::List,
            crate::core::AggregateFunction::Distinct(_) => DataType::Empty,
            crate::core::AggregateFunction::Percentile(_, _) => DataType::Float,
            crate::core::AggregateFunction::Std(_) => DataType::Float,
            crate::core::AggregateFunction::BitAnd(_) | crate::core::AggregateFunction::BitOr(_) => DataType::Int,
            crate::core::AggregateFunction::GroupConcat(_, _) => DataType::String,
        }
    }

    /// 简化的二元表达式类型推导
    fn deduce_binary_expr_type_simple(&self, op: &crate::core::BinaryOperator) -> DataType {
        match op {
            crate::core::BinaryOperator::Equal
            | crate::core::BinaryOperator::NotEqual
            | crate::core::BinaryOperator::LessThan
            | crate::core::BinaryOperator::LessThanOrEqual
            | crate::core::BinaryOperator::GreaterThan
            | crate::core::BinaryOperator::GreaterThanOrEqual => DataType::Bool,
            crate::core::BinaryOperator::And | crate::core::BinaryOperator::Or => DataType::Bool,
            _ => DataType::Empty,
        }
    }

    /// 简化的一元表达式类型推导
    fn deduce_unary_expr_type_simple(&self, op: &crate::core::UnaryOperator) -> DataType {
        match op {
            crate::core::UnaryOperator::Not => DataType::Bool,
            crate::core::UnaryOperator::Minus | crate::core::UnaryOperator::Plus => DataType::Empty,
            _ => DataType::Empty,
        }
    }

    /// 简化的函数返回类型推导
    fn deduce_function_return_type_simple(&self, name: &str) -> DataType {
        match name.to_lowercase().as_str() {
            "abs" | "length" | "size" => DataType::Int,
            "round" | "floor" | "ceil" => DataType::Int,
            "sqrt" | "pow" | "sin" | "cos" | "tan" => DataType::Float,
            "concat" | "substring" | "trim" | "ltrim" | "rtrim" => DataType::String,
            "upper" | "lower" => DataType::String,
            "type" => DataType::String,
            "id" => DataType::Int,
            "properties" => DataType::Map,
            "labels" => DataType::List,
            "keys" => DataType::List,
            "values" => DataType::List,
            "range" => DataType::List,
            "reverse" => DataType::List,
            "head" | "last" | "tail" => DataType::Empty,
            _ => DataType::Empty,
        }
    }

    /// 检查表达式是否包含聚合函数
    pub fn has_aggregate_expression(&self, expression: &Expression) -> bool {
        match expression {
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
                conditions.iter().any(|(when_expression, then_expression)| {
                    self.has_aggregate_expression(when_expression) || self.has_aggregate_expression(then_expression)
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
            Expression::TypeCast { expression, .. } => self.has_aggregate_expression(expression),
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
            DataType::Int
            | DataType::Float
            | DataType::String
            | DataType::Bool
            | DataType::Date
            | DataType::Time
            | DataType::DateTime => Ok(()),
            _ => Err(ValidationError::new(
                format!(
                    "分组键的类型 {:?} 不支持，只支持基本类型",
                    key_type
                ),
                ValidationErrorType::TypeError,
            )),
        }
    }

    /// 从 DataType 转换为 ValueType
    pub fn value_type_def_to_value_type(type_def: &DataType) -> ValueType {
        match type_def {
            DataType::Empty => ValueType::Unknown,
            DataType::Null => ValueType::Null,
            DataType::Bool => ValueType::Bool,
            DataType::Int => ValueType::Int,
            DataType::Float => ValueType::Float,
            DataType::String => ValueType::String,
            DataType::Date => ValueType::Date,
            DataType::Time => ValueType::Time,
            DataType::DateTime => ValueType::DateTime,
            DataType::Vertex => ValueType::Vertex,
            DataType::Edge => ValueType::Edge,
            DataType::Path => ValueType::Path,
            DataType::List => ValueType::List,
            DataType::Map => ValueType::Map,
            DataType::Set => ValueType::Set,
            _ => ValueType::Unknown,
        }
    }

    /// 从 ValueType 转换为 DataType
    pub fn value_type_to_value_type_def(type_: &ValueType) -> DataType {
        match type_ {
            ValueType::Unknown => DataType::Empty,
            ValueType::Bool => DataType::Bool,
            ValueType::Int => DataType::Int,
            ValueType::Float => DataType::Float,
            ValueType::String => DataType::String,
            ValueType::Date => DataType::Date,
            ValueType::Time => DataType::Time,
            ValueType::DateTime => DataType::DateTime,
            ValueType::Vertex => DataType::Vertex,
            ValueType::Edge => DataType::Edge,
            ValueType::Path => DataType::Path,
            ValueType::List => DataType::List,
            ValueType::Map => DataType::Map,
            ValueType::Set => DataType::Set,
            ValueType::Null => DataType::Null,
        }
    }

    /// 完整的表达式类型推导（增强版）
    pub fn deduce_expr_type(&self, expression: &Expression) -> ValueType {
        match expression {
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
                for (_, then_expression) in conditions {
                    let then_type = self.deduce_expr_type(then_expression);
                    if then_type != ValueType::Unknown {
                        return then_type;
                    }
                }
                if let Some(default_expression) = default {
                    return self.deduce_expr_type(default_expression);
                }
                ValueType::Unknown
            }
            Expression::TypeCast { expression: _, target_type: _ } => {
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
            Expression::ListComprehension { .. } => ValueType::List,
            Expression::LabelTagProperty { .. } => ValueType::Unknown,
            Expression::TagProperty { .. } => ValueType::Unknown,
            Expression::EdgeProperty { .. } => ValueType::Unknown,
            Expression::Predicate { .. } => ValueType::Bool,
            Expression::Reduce { .. } => ValueType::Unknown,
            Expression::PathBuild(_) => ValueType::Path,
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
            AggregateFunction::Std(_) => ValueType::Float,
            AggregateFunction::BitAnd(_) | AggregateFunction::BitOr(_) => ValueType::Int,
            AggregateFunction::GroupConcat(_, _) => ValueType::String,
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
    pub fn fold_constant_expr_enhanced(&self, expression: &Expression) -> Option<Expression> {
        match expression {
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
        let literal_expression = Expression::Literal(Value::Bool(true));
        let context = ValidationContextImpl::new();
        
        let result = type_validator.validate_expression_type(&literal_expression, &context, DataType::Bool);
        assert!(result.is_ok());
        
        let result = type_validator.validate_expression_type(&literal_expression, &context, DataType::Int);
        assert!(result.is_err());
    }

    #[test]
    fn test_deduce_binary_expr_type() {
        let type_validator = TypeValidator::new();
        let op = crate::core::BinaryOperator::Equal;
        let left_type = DataType::Int;
        let right_type = DataType::Int;
        
        let result = type_validator.deduce_binary_expr_type(&op, &left_type, &right_type);
        assert_eq!(result, DataType::Bool);
    }

    #[test]
    fn test_deduce_aggregate_return_type() {
        let type_validator = TypeValidator::new();
        
        let count_type = type_validator.deduce_aggregate_return_type(&crate::core::AggregateFunction::Count(None));
        assert_eq!(count_type, DataType::Int);
        
        let sum_type = type_validator.deduce_aggregate_return_type(&crate::core::AggregateFunction::Sum("value".to_string()));
        assert_eq!(sum_type, DataType::Float);
        
        let avg_type = type_validator.deduce_aggregate_return_type(&crate::core::AggregateFunction::Avg("value".to_string()));
        assert_eq!(avg_type, DataType::Float);
        
        let collect_type = type_validator.deduce_aggregate_return_type(&crate::core::AggregateFunction::Collect("value".to_string()));
        assert_eq!(collect_type, DataType::List);
    }

    #[test]
    fn test_are_types_compatible() {
        let type_validator = TypeValidator::new();

        assert!(type_validator.are_types_compatible(&DataType::Int, &DataType::Int));
        assert!(type_validator.are_types_compatible(&DataType::Int, &DataType::Float));
        assert!(type_validator.are_types_compatible(&DataType::Float, &DataType::Int));
        assert!(type_validator.are_types_compatible(&DataType::Empty, &DataType::Int));
        assert!(type_validator.are_types_compatible(&DataType::String, &DataType::Empty));
    }
}

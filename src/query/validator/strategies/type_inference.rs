//! 表达式类型推导系统
//! 负责推导和验证表达式的类型信息

use crate::core::Expression;
use crate::core::ValueTypeDef;
use crate::query::validator::structs::*;
use crate::query::validator::{ValidationError, ValidationErrorType};
use crate::query::validator::validation_interface::ValidationContext;
use std::collections::HashMap;

/// 顶点类型定义
#[derive(Debug, Clone, PartialEq)]
pub struct VertexType {
    pub tag_id: Option<i32>,
    pub tag_name: String,
    pub properties: Vec<PropertyType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyType {
    pub name: String,
    pub type_def: ValueTypeDef,
    pub is_nullable: bool,
}

/// 边类型定义
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeTypeInfo {
    pub edge_type: i32,
    pub edge_name: String,
    pub src_tag: String,
    pub dst_tag: String,
    pub properties: Vec<PropertyType>,
    pub rank_enabled: bool,
}

/// 路径类型定义
#[derive(Debug, Clone, PartialEq)]
pub enum PathType {
    SimplePath,
    AllPaths,
    ShortestPath,
    NonWeightedShortestPath,
    WeightedShortestPath,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathInfo {
    pub path_type: PathType,
    pub steps: Option<(i32, i32)>,
    pub node_types: Vec<VertexType>,
    pub edge_types: Vec<EdgeTypeInfo>,
}

/// 表达式验证上下文Trait
/// 定义表达式验证所需的基本接口
trait ExpressionValidationContext {
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

/// 类型推导器
pub struct TypeInference;

impl TypeInference {
    pub fn new() -> Self {
        Self
    }

    /// 推导顶点类型
    pub fn deduce_vertex_type(&self, tag_name: &str, _tag_id: Option<i32>) -> VertexType {
        VertexType {
            tag_id: None,
            tag_name: tag_name.to_string(),
            properties: Vec::new(),
        }
    }

    /// 推导边类型
    pub fn deduce_edge_type(&self, edge_name: &str, edge_type: i32) -> EdgeTypeInfo {
        EdgeTypeInfo {
            edge_type,
            edge_name: edge_name.to_string(),
            src_tag: String::new(),
            dst_tag: String::new(),
            properties: Vec::new(),
            rank_enabled: true,
        }
    }

    /// 推导路径类型
    pub fn deduce_path_type(
        &self,
        path_type: PathType,
        steps: Option<(i32, i32)>,
    ) -> PathInfo {
        PathInfo {
            path_type,
            steps,
            node_types: Vec::new(),
            edge_types: Vec::new(),
        }
    }

    /// 推导属性类型
    pub fn deduce_property_type(&self, prop_name: &str, _object_type: &str) -> Option<ValueTypeDef> {
        match prop_name.to_lowercase().as_str() {
            "id" => Some(ValueTypeDef::Int),
            "name" | "title" | "desc" | "description" => Some(ValueTypeDef::String),
            "age" | "count" | "size" | "year" | "month" | "day" | "hour" | "minute" | "second" => {
                Some(ValueTypeDef::Int)
            }
            "price" | "score" | "rate" | "ratio" | "percent" | "weight" | "height" | "width" | "length" => {
                Some(ValueTypeDef::Float)
            }
            "created_at" | "updated_at" | "birthday" | "date" | "time" | "datetime" => {
                Some(ValueTypeDef::DateTime)
            }
            "active" | "enabled" | "visible" | "valid" | "exists" => Some(ValueTypeDef::Bool),
            "tags" | "labels" | "categories" => Some(ValueTypeDef::List),
            "properties" | "attrs" | "attributes" => Some(ValueTypeDef::Map),
            _ => None,
        }
    }

    /// 推导列表元素类型
    pub fn deduce_list_element_type(&self, element_type: &ValueTypeDef) -> ValueTypeDef {
        match element_type {
            ValueTypeDef::List => ValueTypeDef::Any,
            ValueTypeDef::Map => ValueTypeDef::Any,
            _ => element_type.clone(),
        }
    }

    /// 推导 Map 值类型
    pub fn deduce_map_value_type(&self, _map_type: &ValueTypeDef) -> ValueTypeDef {
        ValueTypeDef::Any
    }

    /// 推导路径步数类型
    pub fn deduce_step_type(&self) -> ValueTypeDef {
        ValueTypeDef::Int
    }

    /// 推导 ID 类型
    pub fn deduce_id_type(&self) -> ValueTypeDef {
        ValueTypeDef::String
    }

    /// 推导标签列表类型
    pub fn deduce_labels_type(&self) -> ValueTypeDef {
        ValueTypeDef::List
    }

    /// 推导属性映射类型
    pub fn deduce_properties_type(&self) -> ValueTypeDef {
        ValueTypeDef::Map
    }

    /// 推导边类型名称类型
    pub fn deduce_edge_type_name_type(&self) -> ValueTypeDef {
        ValueTypeDef::String
    }

    /// 推导路径类型
    pub fn deduce_path_result_type(&self) -> ValueTypeDef {
        ValueTypeDef::Path
    }

    /// 检查类型是否可以用于索引
    pub fn is_indexable_type(&self, type_def: &ValueTypeDef) -> bool {
        match type_def {
            ValueTypeDef::Bool => true,
            ValueTypeDef::Int => true,
            ValueTypeDef::Float => true,
            ValueTypeDef::String => true,
            ValueTypeDef::DateTime => true,
            ValueTypeDef::Date => true,
            ValueTypeDef::Time => true,
            ValueTypeDef::Duration => true,
            ValueTypeDef::Geography => true,
            _ => false,
        }
    }

    /// 获取类型的默认值
    pub fn get_default_value(&self, type_def: &ValueTypeDef) -> Option<Expression> {
        match type_def {
            ValueTypeDef::Bool => Some(Expression::Literal(crate::core::Value::Bool(false))),
            ValueTypeDef::Int => Some(Expression::Literal(crate::core::Value::Int(0))),
            ValueTypeDef::Float => Some(Expression::Literal(crate::core::Value::Float(0.0))),
            ValueTypeDef::String => Some(Expression::Literal(crate::core::Value::String(String::new()))),
            ValueTypeDef::List => Some(Expression::List(Vec::new())),
            ValueTypeDef::Map => Some(Expression::Map(Vec::new())),
            _ => None,
        }
    }

    /// 验证类型是否可以强制转换
    pub fn can_cast(&self, from: &ValueTypeDef, to: &ValueTypeDef) -> bool {
        match (from, to) {
            (ValueTypeDef::Int, ValueTypeDef::Float) => true,
            (ValueTypeDef::Int, ValueTypeDef::String) => true,
            (ValueTypeDef::Float, ValueTypeDef::String) => true,
            (ValueTypeDef::Bool, ValueTypeDef::String) => true,
            (a, b) => a == b,
        }
    }

    /// 获取类型的字符串表示
    pub fn type_to_string(&self, type_def: &ValueTypeDef) -> String {
        match type_def {
            ValueTypeDef::Empty => "empty".to_string(),
            ValueTypeDef::Null => "null".to_string(),
            ValueTypeDef::Bool => "bool".to_string(),
            ValueTypeDef::Int => "int".to_string(),
            ValueTypeDef::Float => "float".to_string(),
            ValueTypeDef::String => "string".to_string(),
            ValueTypeDef::Date => "date".to_string(),
            ValueTypeDef::Time => "time".to_string(),
            ValueTypeDef::DateTime => "datetime".to_string(),
            ValueTypeDef::Vertex => "vertex".to_string(),
            ValueTypeDef::Edge => "edge".to_string(),
            ValueTypeDef::Path => "path".to_string(),
            ValueTypeDef::List => "list".to_string(),
            ValueTypeDef::Map => "map".to_string(),
            ValueTypeDef::Set => "set".to_string(),
            ValueTypeDef::Geography => "geography".to_string(),
            ValueTypeDef::Duration => "duration".to_string(),
            ValueTypeDef::DataSet => "dataset".to_string(),
            ValueTypeDef::IntRange => "int_range".to_string(),
            ValueTypeDef::FloatRange => "float_range".to_string(),
            ValueTypeDef::StringRange => "string_range".to_string(),
            ValueTypeDef::Any => "any".to_string(),
        }
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
                // 对于其他表达式类型，使用简化的类型推导
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
                // 比较操作符的结果是布尔值
                if expected_type == ValueTypeDef::Bool {
                    // 验证左右操作数类型兼容
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
                // 逻辑操作符的结果是布尔值
                if expected_type == ValueTypeDef::Bool {
                    // 验证左右操作数都是布尔类型
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
                // 算术操作符，推导结果类型
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
                // 逻辑非的结果是布尔值
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
                // 一元加/减操作符的结果类型与操作数相同
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
            Expression::Variable(_) => ValueTypeDef::Any,
            Expression::Binary { op, .. } => self.deduce_binary_expr_type_simple(op),
            Expression::Unary { op, .. } => self.deduce_unary_expr_type_simple(op),
            Expression::Function { name, .. } => self.deduce_function_return_type_simple(name),
            Expression::Aggregate { func, .. } => self.deduce_aggregate_return_type(func),
            _ => ValueTypeDef::Any,
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
                ValueTypeDef::Any
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
            _ => ValueTypeDef::Any,
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
                // 算术操作符，返回更精确的类型
                if *left_type == ValueTypeDef::Float || *right_type == ValueTypeDef::Float {
                    ValueTypeDef::Float
                } else if *left_type == ValueTypeDef::Int || *right_type == ValueTypeDef::Int {
                    ValueTypeDef::Int
                } else {
                    ValueTypeDef::Any
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
            _ => ValueTypeDef::Any,
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
            "head" | "last" | "tail" => ValueTypeDef::Any,
            _ => ValueTypeDef::Any,
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
            crate::core::AggregateFunction::Max(_) | crate::core::AggregateFunction::Min(_) => ValueTypeDef::Any,
            crate::core::AggregateFunction::Collect(_) => ValueTypeDef::List,
            crate::core::AggregateFunction::Distinct(_) => ValueTypeDef::Any,
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
            _ => ValueTypeDef::Any,
        }
    }

    /// 简化的一元表达式类型推导
    fn deduce_unary_expr_type_simple(&self, op: &crate::core::UnaryOperator) -> ValueTypeDef {
        match op {
            crate::core::UnaryOperator::Not => ValueTypeDef::Bool,
            crate::core::UnaryOperator::Minus | crate::core::UnaryOperator::Plus => ValueTypeDef::Any,
            _ => ValueTypeDef::Any,
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
            "head" | "last" | "tail" => ValueTypeDef::Any,
            _ => ValueTypeDef::Any,
        }
    }

    /// 检查类型兼容性
    pub fn are_types_compatible(&self, actual: &ValueTypeDef, expected: &ValueTypeDef) -> bool {
        match (actual, expected) {
            (ValueTypeDef::Any, _) | (_, ValueTypeDef::Any) => true,
            (ValueTypeDef::Int, ValueTypeDef::Float) => true,
            (ValueTypeDef::Float, ValueTypeDef::Int) => false,
            (a, e) => a == e,
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
            Expression::ListComprehension {
                generator,
                condition,
            } => {
                self.has_aggregate_expression(generator)
                    || condition.as_ref().map_or(false, |c| self.has_aggregate_expression(c))
            }
            Expression::Predicate { list, condition } => {
                self.has_aggregate_expression(list) || self.has_aggregate_expression(condition)
            }
            Expression::Reduce {
                initial,
                list,
                expr,
                ..
            } => {
                self.has_aggregate_expression(initial)
                    || self.has_aggregate_expression(list)
                    || self.has_aggregate_expression(expr)
            }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_type_inference_creation() {
        let type_inference = TypeInference::new();
        assert!(true);
    }

    #[test]
    fn test_validate_literal_type() {
        let type_inference = TypeInference::new();
        let literal_expr = Expression::Literal(Value::Bool(true));
        let context = ValidationContextImpl::new();
        
        let result = type_inference.validate_expression_type(&literal_expr, &context, ValueTypeDef::Bool);
        assert!(result.is_ok());
        
        let result = type_inference.validate_expression_type(&literal_expr, &context, ValueTypeDef::Int);
        assert!(result.is_err());
    }

    #[test]
    fn test_deduce_binary_expr_type() {
        let type_inference = TypeInference::new();
        let op = crate::core::BinaryOperator::Equal;
        let left_type = ValueTypeDef::Int;
        let right_type = ValueTypeDef::Int;
        
        let result = type_inference.deduce_binary_expr_type(&op, &left_type, &right_type);
        assert_eq!(result, ValueTypeDef::Bool);
    }

    #[test]
    fn test_deduce_aggregate_return_type() {
        let type_inference = TypeInference::new();
        
        let count_type = type_inference.deduce_aggregate_return_type(&crate::core::AggregateFunction::Count(None));
        assert_eq!(count_type, ValueTypeDef::Int);
        
        let sum_type = type_inference.deduce_aggregate_return_type(&crate::core::AggregateFunction::Sum("value".to_string()));
        assert_eq!(sum_type, ValueTypeDef::Float);
        
        let avg_type = type_inference.deduce_aggregate_return_type(&crate::core::AggregateFunction::Avg("value".to_string()));
        assert_eq!(avg_type, ValueTypeDef::Float);
        
        let collect_type = type_inference.deduce_aggregate_return_type(&crate::core::AggregateFunction::Collect("value".to_string()));
        assert_eq!(collect_type, ValueTypeDef::List);
    }

    #[test]
    fn test_are_types_compatible() {
        let type_inference = TypeInference::new();
        
        assert!(type_inference.are_types_compatible(&ValueTypeDef::Int, &ValueTypeDef::Int));
        assert!(type_inference.are_types_compatible(&ValueTypeDef::Int, &ValueTypeDef::Float));
        assert!(!type_inference.are_types_compatible(&ValueTypeDef::Float, &ValueTypeDef::Int));
        assert!(type_inference.are_types_compatible(&ValueTypeDef::Any, &ValueTypeDef::Int));
        assert!(type_inference.are_types_compatible(&ValueTypeDef::String, &ValueTypeDef::Any));
    }
}
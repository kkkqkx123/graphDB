//! ORDER BY 子句验证器
//! 对应 NebulaGraph OrderByValidator.h/.cpp 的功能
//! 验证 ORDER BY 子句的排序表达式和方向

use super::base_validator::{Validator, ValueType};
use super::ValidationContext;
use crate::core::Expression;
use crate::query::validator::ValidationError;
use crate::query::validator::ValidationErrorType;
use std::collections::HashMap;

pub struct OrderByValidator {
    _base: Validator,
    order_columns: Vec<OrderColumn>,
    input_columns: HashMap<String, ValueType>,
}

#[derive(Debug, Clone)]
pub struct OrderColumn {
    pub expression: Expression,
    pub alias: Option<String>,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
    Default,
}

impl OrderByValidator {
    pub fn new(context: ValidationContext) -> Self {
        Self {
            _base: Validator::new(context),
            order_columns: Vec::new(),
            input_columns: HashMap::new(),
        }
    }

    pub fn validate(&mut self) -> Result<(), ValidationError> {
        self.validate_impl()
    }

    fn validate_impl(&mut self) -> Result<(), ValidationError> {
        self.validate_columns()?;
        self.validate_types()?;
        self.validate_input_compatibility()?;
        Ok(())
    }

    fn validate_columns(&mut self) -> Result<(), ValidationError> {
        if self.order_columns.is_empty() {
            return Err(ValidationError::new(
                "ORDER BY clause must have at least one column".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        for col in &self.order_columns {
            if self.expression_is_empty(&col.expression) {
                return Err(ValidationError::new(
                    "ORDER BY expression cannot be empty".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    fn validate_types(&self) -> Result<(), ValidationError> {
        for col in &self.order_columns {
            let expr_type = self.deduce_expr_type(&col.expression)?;
            if !self.is_comparable_type(&expr_type) {
                return Err(ValidationError::new(
                    format!(
                        "ORDER BY expression type {:?} is not comparable",
                        expr_type
                    ),
                    ValidationErrorType::TypeError,
                ));
            }
        }
        Ok(())
    }

    fn validate_input_compatibility(&self) -> Result<(), ValidationError> {
        for col in &self.order_columns {
            if let Some(alias) = &col.alias {
                if !self.input_columns.contains_key(alias) {
                    return Err(ValidationError::new(
                        format!(
                            "ORDER BY alias '{}' not found in input columns",
                            alias
                        ),
                        ValidationErrorType::SemanticError,
                    ));
                }
            } else {
                let refs = self.get_expression_references(&col.expression);
                for ref_name in refs {
                    if !self.input_columns.contains_key(&ref_name) && ref_name != "$" {
                        return Err(ValidationError::new(
                            format!(
                                "ORDER BY expression references unknown column '{}'",
                                ref_name
                            ),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn expression_is_empty(&self, expression: &Expression) -> bool {
        match expression {
            Expression::Literal(value) => {
                match value {
                    crate::core::Value::Null(_) => true,
                    crate::core::Value::String(s) => s.is_empty(),
                    _ => false,
                }
            },
            Expression::Variable(name) => name.is_empty(),
            Expression::Function { name, args } => name.is_empty() && args.is_empty(),
            Expression::Binary { left, right, .. } => {
                self.expression_is_empty(left) && self.expression_is_empty(right)
            },
            Expression::Unary { operand, .. } => self.expression_is_empty(operand),
            Expression::List(items) => items.is_empty(),
            Expression::Map(pairs) => pairs.is_empty(),
            // 其他表达式类型默认不为空
            _ => false,
        }
    }

    fn deduce_expr_type(&self, expression: &Expression) -> Result<ValueType, ValidationError> {
        match expression {
            Expression::Literal(value) => {
                match value {
                    crate::core::Value::Bool(_) => Ok(ValueType::Bool),
                    crate::core::Value::Int(_) => Ok(ValueType::Int),
                    crate::core::Value::Float(_) => Ok(ValueType::Float),
                    crate::core::Value::String(_) => Ok(ValueType::String),
                    crate::core::Value::Date(_) => Ok(ValueType::Date),
                    crate::core::Value::Time(_) => Ok(ValueType::Time),
                    crate::core::Value::DateTime(_) => Ok(ValueType::DateTime),
                    crate::core::Value::Null(_) => Ok(ValueType::Null),
                    crate::core::Value::Vertex(_) => Ok(ValueType::Vertex),
                    crate::core::Value::Edge(_) => Ok(ValueType::Edge),
                    crate::core::Value::Path(_) => Ok(ValueType::Path),
                    crate::core::Value::List(_) => Ok(ValueType::List),
                    crate::core::Value::Map(_) => Ok(ValueType::Map),
                    crate::core::Value::Set(_) => Ok(ValueType::Set),
                    _ => Ok(ValueType::Unknown),
                }
            },
            Expression::Variable(name) => {
                // 尝试从输入列中获取类型
                if let Some(column_type) = self.input_columns.get(name) {
                    Ok(column_type.clone())
                } else {
                    Ok(ValueType::Unknown) // 如果找不到对应列，则返回未知类型
                }
            },
            Expression::Binary { left, op, right } => {
                // 对于比较操作，结果是布尔类型
                match op {
                    crate::core::BinaryOperator::Equal
                    | crate::core::BinaryOperator::NotEqual
                    | crate::core::BinaryOperator::LessThan
                    | crate::core::BinaryOperator::LessThanOrEqual
                    | crate::core::BinaryOperator::GreaterThan
                    | crate::core::BinaryOperator::GreaterThanOrEqual
                    | crate::core::BinaryOperator::And
                    | crate::core::BinaryOperator::Or
                    | crate::core::BinaryOperator::Xor
                    | crate::core::BinaryOperator::Like
                    | crate::core::BinaryOperator::In
                    | crate::core::BinaryOperator::NotIn
                    | crate::core::BinaryOperator::Contains
                    | crate::core::BinaryOperator::StartsWith
                    | crate::core::BinaryOperator::EndsWith => Ok(ValueType::Bool),
                    // 算术操作通常返回数值类型
                    crate::core::BinaryOperator::Add
                    | crate::core::BinaryOperator::Subtract
                    | crate::core::BinaryOperator::Multiply
                    | crate::core::BinaryOperator::Divide
                    | crate::core::BinaryOperator::Modulo
                    | crate::core::BinaryOperator::Exponent => {
                        let left_type = self.deduce_expr_type(left)?;
                        let right_type = self.deduce_expr_type(right)?;

                        // 如果任一操作数是浮点数，则结果为浮点数
                        if matches!(left_type, ValueType::Float) || matches!(right_type, ValueType::Float) {
                            Ok(ValueType::Float)
                        } else if matches!(left_type, ValueType::Int) || matches!(right_type, ValueType::Int) {
                            Ok(ValueType::Int)
                        } else {
                            Ok(ValueType::Unknown)
                        }
                    },
                    // 字符串连接操作返回字符串
                    crate::core::BinaryOperator::StringConcat => Ok(ValueType::String),
                    // 其他操作返回未知类型
                    _ => Ok(ValueType::Unknown),
                }
            },
            Expression::Unary { op, operand } => {
                match op {
                    crate::core::UnaryOperator::Not => Ok(ValueType::Bool),
                    crate::core::UnaryOperator::IsNull | crate::core::UnaryOperator::IsNotNull => Ok(ValueType::Bool),
                    crate::core::UnaryOperator::IsEmpty | crate::core::UnaryOperator::IsNotEmpty => Ok(ValueType::Bool),
                    crate::core::UnaryOperator::Plus | crate::core::UnaryOperator::Minus => {
                        let operand_type = self.deduce_expr_type(operand)?;
                        Ok(operand_type)
                    },
                    _ => Ok(ValueType::Unknown),
                }
            },
            Expression::Function { name, args: _ } => {
                // 根据函数名推断返回类型
                match name.to_lowercase().as_str() {
                    "id" => Ok(ValueType::String),
                    "count" | "sum" | "avg" | "min" | "max" => Ok(ValueType::Float),
                    "length" | "size" => Ok(ValueType::Int),
                    "to_string" | "string" => Ok(ValueType::String),
                    "abs" => Ok(ValueType::Float),
                    "floor" | "ceil" | "round" => Ok(ValueType::Int),
                    _ => Ok(ValueType::Unknown),
                }
            },
            Expression::Aggregate { func, .. } => {
                match func {
                    crate::core::AggregateFunction::Count(_) => Ok(ValueType::Int),
                    crate::core::AggregateFunction::Sum(_) => Ok(ValueType::Float),
                    crate::core::AggregateFunction::Avg(_) => Ok(ValueType::Float),
                    crate::core::AggregateFunction::Collect(_) => Ok(ValueType::List),
                    _ => Ok(ValueType::Unknown),
                }
            },
            Expression::List(_) => Ok(ValueType::List),
            Expression::Map(_) => Ok(ValueType::Map),
            Expression::Case { .. } => Ok(ValueType::Unknown), // CASE表达式的结果类型取决于其分支
            Expression::TypeCast { target_type, .. } => {
                // 根据目标类型转换
                match target_type {
                    crate::core::DataType::Bool => Ok(ValueType::Bool),
                    crate::core::DataType::Int | crate::core::DataType::Int8 | crate::core::DataType::Int16 |
                    crate::core::DataType::Int32 | crate::core::DataType::Int64 => Ok(ValueType::Int),
                    crate::core::DataType::Float | crate::core::DataType::Double => Ok(ValueType::Float),
                    crate::core::DataType::String => Ok(ValueType::String),
                    crate::core::DataType::Date => Ok(ValueType::Date),
                    crate::core::DataType::Time => Ok(ValueType::Time),
                    crate::core::DataType::DateTime => Ok(ValueType::DateTime),
                    _ => Ok(ValueType::Unknown),
                }
            },
            // 属性表达式统一处理
            Expression::Property { object, property } => {
                if let Expression::Variable(var_name) = object.as_ref() {
                    if let Some(column_type) = self.input_columns.get(var_name) {
                        return Ok(column_type.clone());
                    }
                }
                if let Some(column_type) = self.input_columns.get(property) {
                    Ok(column_type.clone())
                } else {
                    Ok(ValueType::Unknown)
                }
            },
            Expression::Subscript { .. } => Ok(ValueType::Unknown),
            Expression::Range { .. } => Ok(ValueType::List),
            Expression::Path(_) => Ok(ValueType::Path),
            Expression::Label(_) => Ok(ValueType::String),
        }
    }

    fn is_comparable_type(&self, type_: &ValueType) -> bool {
        matches!(
            type_,
            ValueType::Bool | ValueType::Int | ValueType::Float |
            ValueType::String | ValueType::Date | ValueType::Time |
            ValueType::DateTime | ValueType::Null
        )
    }

    fn get_expression_references(&self, expression: &Expression) -> Vec<String> {
        let mut refs = Vec::new();
        self.collect_refs(expression, &mut refs);
        refs
    }

    // 辅助函数：递归收集表达式中的列引用
    fn collect_refs(&self, expression: &Expression, refs: &mut Vec<String>) {
        match expression {
            Expression::Variable(name) => {
                if !refs.contains(name) {
                    refs.push(name.clone());
                }
            },
            Expression::Function { args, .. } => {
                for arg in args {
                    self.collect_refs(arg, refs);
                }
            },
            Expression::Binary { left, right, .. } => {
                self.collect_refs(left, refs);
                self.collect_refs(right, refs);
            },
            Expression::Unary { operand, .. } => {
                self.collect_refs(operand, refs);
            },
            Expression::Aggregate { arg, .. } => {
                self.collect_refs(arg, refs);
            },
            Expression::List(items) => {
                for item in items {
                    self.collect_refs(item, refs);
                }
            },
            Expression::Map(pairs) => {
                for (_, value) in pairs {
                    self.collect_refs(value, refs);
                }
            },
            Expression::Case { conditions, default } => {
                for (condition, value) in conditions {
                    self.collect_refs(condition, refs);
                    self.collect_refs(value, refs);
                }
                if let Some(default_expression) = default {
                    self.collect_refs(default_expression, refs);
                }
            },
            Expression::TypeCast { expression, .. } => {
                self.collect_refs(expression, refs);
            },
            Expression::Subscript { collection, index } => {
                self.collect_refs(collection, refs);
                self.collect_refs(index, refs);
            },
            Expression::Range { collection, start, end } => {
                self.collect_refs(collection, refs);
                if let Some(start_expression) = start {
                    self.collect_refs(start_expression, refs);
                }
                if let Some(end_expression) = end {
                    self.collect_refs(end_expression, refs);
                }
            },
            // 属性表达式统一处理
            Expression::Property { object, property } => {
                self.collect_refs(object, refs);
                if !refs.contains(property) {
                    refs.push(property.clone());
                }
            },
            Expression::Literal(_) => {},
            Expression::Path(_) => {},
            Expression::Label(_) => {},
        }
    }

    pub fn add_order_column(&mut self, col: OrderColumn) {
        self.order_columns.push(col);
    }

    pub fn set_input_columns(&mut self, columns: HashMap<String, ValueType>) {
        self.input_columns = columns;
    }

    pub fn order_columns(&self) -> &[OrderColumn] {
        &self.order_columns
    }
}

impl Validator {
    pub fn validate_order_by(
        &mut self,
        columns: &[OrderColumn],
        input_columns: &HashMap<String, ValueType>,
    ) -> Result<(), ValidationError> {
        let mut validator = OrderByValidator::new(self.context().clone());
        for col in columns {
            validator.add_order_column(col.clone());
        }
        validator.set_input_columns(input_columns.clone());
        validator.validate()
    }
}

//! ORDER BY 子句验证器
//! 验证 ORDER BY 子句的排序表达式和方向
//!
//! 本文件已按照新的 trait + 枚举 验证器体系重构：
//! 1. 实现了 StatementValidator trait，统一接口
//! 2. 保留了原有完整功能：
//!    - 排序列验证
//!    - 类型检查（可比较类型）
//!    - 输入列兼容性验证
//!    - 表达式类型推导
//!    - 表达式引用收集
//! 3. 使用 QueryContext 统一管理上下文

use std::sync::Arc;
use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::Expression;
use crate::core::types::OrderDirection;
use crate::query::context::QueryContext;
use crate::query::validator::validator_trait::{
    StatementType, StatementValidator, ValidationResult, ColumnDef, ValueType,
    ExpressionProps,
};
use std::collections::HashMap;

/// 排序列定义
#[derive(Debug, Clone)]
pub struct OrderColumn {
    pub expression: Expression,
    pub alias: Option<String>,
    pub direction: OrderDirection,
}

/// ORDER BY 验证器 - 新体系实现
///
/// 功能完整性保证：
/// 1. 完整的验证生命周期
/// 2. 输入/输出列管理
/// 3. 表达式属性追踪
/// 4. 排序表达式验证
/// 5. 类型兼容性检查
#[derive(Debug)]
pub struct OrderByValidator {
    // 排序列列表
    order_columns: Vec<OrderColumn>,
    // 输入列定义（来自前序查询）
    input_columns: HashMap<String, ValueType>,
    // 输入列定义（用于 trait 接口）
    inputs: Vec<ColumnDef>,
    // 输出列定义（ORDER BY 不改变输出结构）
    outputs: Vec<ColumnDef>,
    // 表达式属性
    expr_props: ExpressionProps,
    // 用户定义变量
    user_defined_vars: Vec<String>,
    // 验证错误列表
    validation_errors: Vec<ValidationError>,
}

impl OrderByValidator {
    /// 创建新的验证器实例
    pub fn new() -> Self {
        Self {
            order_columns: Vec::new(),
            input_columns: HashMap::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
            validation_errors: Vec::new(),
        }
    }

    /// 添加排序列
    pub fn add_order_column(&mut self, col: OrderColumn) {
        self.order_columns.push(col);
    }

    /// 设置输入列
    pub fn set_input_columns(&mut self, columns: HashMap<String, ValueType>) {
        self.input_columns = columns;
        // 同步到 inputs
        self.inputs = self.input_columns
            .iter()
            .map(|(name, type_)| ColumnDef {
                name: name.clone(),
                type_: type_.clone(),
            })
            .collect();
    }

    /// 获取排序列列表
    pub fn order_columns(&self) -> &[OrderColumn] {
        &self.order_columns
    }

    /// 获取输入列
    pub fn input_columns(&self) -> &HashMap<String, ValueType> {
        &self.input_columns
    }

    /// 清空验证错误
    fn clear_errors(&mut self) {
        self.validation_errors.clear();
    }

    /// 执行验证（传统方式，保持向后兼容）
    pub fn validate_order_by(&mut self) -> Result<(), ValidationError> {
        self.clear_errors();
        self.validate_impl()?;
        Ok(())
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
            Expression::ListComprehension { .. } => false,
            Expression::TagProperty { .. } => false,
            Expression::EdgeProperty { .. } => false,
            Expression::LabelTagProperty { .. } => false,
            Expression::Predicate { .. } => false,
            Expression::Reduce { .. } => false,
            Expression::PathBuild(_) => false,
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
                    }
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
            Expression::ListComprehension { .. } => Ok(ValueType::List),
            Expression::LabelTagProperty { .. } => Ok(ValueType::Unknown),
            Expression::TagProperty { .. } => Ok(ValueType::Unknown),
            Expression::EdgeProperty { .. } => Ok(ValueType::Unknown),
            Expression::Predicate { .. } => Ok(ValueType::Bool),
            Expression::Reduce { .. } => Ok(ValueType::Unknown),
            Expression::PathBuild(_) => Ok(ValueType::Path),
            Expression::Parameter(_) => Ok(ValueType::Unknown),
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
            Expression::Case { test_expr, conditions, default } => {
                if let Some(test_expression) = test_expr {
                    self.collect_refs(test_expression, refs);
                }
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
            Expression::ListComprehension { .. } => {},
            Expression::LabelTagProperty { tag, .. } => {
                self.collect_refs(tag, refs);
            },
            Expression::TagProperty { .. } => {},
            Expression::EdgeProperty { .. } => {},
            Expression::Predicate { args, .. } => {
                for arg in args {
                    self.collect_refs(arg, refs);
                }
            },
            Expression::Reduce { initial, source, mapping, .. } => {
                self.collect_refs(initial, refs);
                self.collect_refs(source, refs);
                self.collect_refs(mapping, refs);
            },
            Expression::PathBuild(exprs) => {
                for expr in exprs {
                    self.collect_refs(expr, refs);
                }
            },
            Expression::Parameter(_) => {},
        }
    }
}

impl Default for OrderByValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// 实现 StatementValidator trait
///
/// # 重构变更
/// - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext
impl StatementValidator for OrderByValidator {
    fn validate(
        &mut self,
        _stmt: &crate::query::parser::ast::Stmt,
        _qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        self.clear_errors();

        // 执行验证
        if let Err(e) = self.validate_impl() {
            return Ok(ValidationResult::failure(vec![e]));
        }

        // ORDER BY 不改变输出结构，输出与输入相同
        self.outputs = self.inputs.clone();

        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::OrderBy
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        // ORDER BY 不是全局语句，需要预先选择空间
        false
    }

    fn expression_props(&self) -> &ExpressionProps {
        &self.expr_props
    }

    fn user_defined_vars(&self) -> &[String] {
        &self.user_defined_vars
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_order_by_validator_new() {
        let validator = OrderByValidator::new();
        assert!(validator.order_columns().is_empty());
        assert!(validator.input_columns().is_empty());
    }

    #[test]
    fn test_add_order_column() {
        let mut validator = OrderByValidator::new();
        let col = OrderColumn {
            expression: Expression::Literal(Value::Int(1)),
            alias: Some("col1".to_string()),
            direction: OrderDirection::Asc,
        };
        validator.add_order_column(col);
        assert_eq!(validator.order_columns().len(), 1);
    }

    #[test]
    fn test_validate_empty_columns() {
        let mut validator = OrderByValidator::new();
        let result = validator.validate_order_by();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_valid_column() {
        let mut validator = OrderByValidator::new();
        let mut input_cols = HashMap::new();
        input_cols.insert("name".to_string(), ValueType::String);
        validator.set_input_columns(input_cols);

        let col = OrderColumn {
            expression: Expression::Variable("name".to_string()),
            alias: None,
            direction: OrderDirection::Asc,
        };
        validator.add_order_column(col);

        let result = validator.validate_order_by();
        assert!(result.is_ok());
    }
}

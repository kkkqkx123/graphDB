//! 基础验证器
//! 对应 NebulaGraph Validator.h/.cpp 的功能
//! 所有验证器的基类
//!
//! 验证生命周期：
//! 1. space_chosen() - 检查是否选择了图空间
//! 2. validate_impl() - 子类实现具体验证逻辑
//! 3. check_permission() - 权限检查
//! 4. to_plan() - 转换为执行计划

use crate::core::error::{DBError, QueryError};
use crate::core::{Expression, Value};
use crate::query::context::validate::ValidationContext;
use crate::query::validator::ValidationError;
use crate::query::validator::ValidationErrorType;

pub struct Validator {
    context: ValidationContext,
    input_var_name: String,
    no_space_required: bool,
    outputs: Vec<ColumnDef>,
    inputs: Vec<ColumnDef>,
    expr_props: ExpressionProps,
    user_defined_vars: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub type_: ValueType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    Unknown,
    Bool,
    Int,
    Float,
    String,
    Date,
    Time,
    DateTime,
    Vertex,
    Edge,
    Path,
    List,
    Map,
    Set,
    Null,
}

#[derive(Debug, Clone, Default)]
pub struct ExpressionProps {
    pub input_props: Vec<InputProperty>,
    pub var_props: Vec<VarProperty>,
    pub tag_props: Vec<TagProperty>,
    pub edge_props: Vec<EdgeProperty>,
}

#[derive(Debug, Clone)]
pub struct InputProperty {
    pub prop_name: String,
    pub type_: ValueType,
}

#[derive(Debug, Clone)]
pub struct VarProperty {
    pub var_name: String,
    pub prop_name: String,
    pub type_: ValueType,
}

#[derive(Debug, Clone)]
pub struct TagProperty {
    pub tag_name: String,
    pub prop_name: String,
    pub type_: ValueType,
}

#[derive(Debug, Clone)]
pub struct EdgeProperty {
    pub edge_type: i32,
    pub prop_name: String,
    pub type_: ValueType,
}

impl Validator {
    pub fn new(context: ValidationContext) -> Self {
        Self {
            context,
            input_var_name: String::new(),
            no_space_required: false,
            outputs: Vec::new(),
            inputs: Vec::new(),
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    pub fn validate(&mut self) -> Result<(), ValidationError> {
        self.validate_lifecycle()
    }

    fn validate_lifecycle(&mut self) -> Result<(), ValidationError> {
        if !self.no_space_required && !self.space_chosen() {
            return Err(ValidationError::new(
                "No space selected. Use `USE <space>` to select a graph space first.".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        self.validate_impl()?;

        if self.context.has_validation_errors() {
            let errors = self.context.get_validation_errors();
            if let Some(first_error) = errors.first() {
                return Err(first_error.clone());
            }
        }

        self.check_permission()?;

        self.to_plan()?;

        Ok(())
    }

    fn space_chosen(&self) -> bool {
        self.context.space_chosen()
    }

    fn validate_impl(&mut self) -> Result<(), ValidationError> {
        Ok(())
    }

    fn check_permission(&self) -> Result<(), ValidationError> {
        Ok(())
    }

    fn to_plan(&mut self) -> Result<(), ValidationError> {
        Ok(())
    }

    pub fn validate_unified(&mut self) -> Result<(), DBError> {
        self.validate_lifecycle().map_err(|e| {
            DBError::Query(QueryError::InvalidQuery(format!("验证失败: {}", e.message)))
        })
    }

    pub fn context_mut(&mut self) -> &mut ValidationContext {
        &mut self.context
    }

    pub fn context(&self) -> &ValidationContext {
        &self.context
    }

    pub fn set_input_var_name(&mut self, name: String) {
        self.input_var_name = name;
    }

    pub fn input_var_name(&self) -> &str {
        &self.input_var_name
    }

    pub fn set_no_space_required(&mut self, required: bool) {
        self.no_space_required = required;
    }

    pub fn no_space_required(&self) -> bool {
        self.no_space_required
    }

    pub fn add_output(&mut self, name: String, type_: ValueType) {
        self.outputs.push(ColumnDef { name, type_ });
    }

    pub fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    pub fn outputs_mut(&mut self) -> &mut Vec<ColumnDef> {
        &mut self.outputs
    }

    pub fn add_input(&mut self, name: String, type_: ValueType) {
        self.inputs.push(ColumnDef { name, type_ });
    }

    pub fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    pub fn add_input_property(&mut self, prop_name: String, type_: ValueType) {
        self.expr_props.input_props.push(InputProperty { prop_name, type_ });
    }

    pub fn add_var_property(&mut self, var_name: String, prop_name: String, type_: ValueType) {
        self.expr_props.var_props.push(VarProperty { var_name, prop_name, type_ });
    }

    pub fn add_tag_property(&mut self, tag_name: String, prop_name: String, type_: ValueType) {
        self.expr_props.tag_props.push(TagProperty { tag_name, prop_name, type_ });
    }

    pub fn add_edge_property(&mut self, edge_type: i32, prop_name: String, type_: ValueType) {
        self.expr_props.edge_props.push(EdgeProperty { edge_type, prop_name, type_ });
    }

    pub fn expr_props(&self) -> &ExpressionProps {
        &self.expr_props
    }

    pub fn expr_props_mut(&mut self) -> &mut ExpressionProps {
        &mut self.expr_props
    }

    pub fn add_user_defined_var(&mut self, var_name: String) {
        self.user_defined_vars.push(var_name);
    }

    pub fn user_defined_vars(&self) -> &[String] {
        &self.user_defined_vars
    }

    pub fn add_error(&mut self, error: ValidationError) {
        self.context.add_validation_error(error);
    }

    pub fn add_semantic_error(&mut self, message: String) {
        self.context.add_validation_error(ValidationError::new(
            message,
            ValidationErrorType::SemanticError,
        ));
    }

    pub fn add_type_error(&mut self, message: String) {
        self.context.add_validation_error(ValidationError::new(
            message,
            ValidationErrorType::TypeError,
        ));
    }

    pub fn add_syntax_error(&mut self, message: String) {
        self.context.add_validation_error(ValidationError::new(
            message,
            ValidationErrorType::SyntaxError,
        ));
    }

    pub fn deduce_expr_type(&self, expression: &Expression) -> ValueType {
        match expression {
            Expression::Literal(value) => {
                match value {
                    Value::Bool(_) => ValueType::Bool,
                    Value::Int(_) => ValueType::Int,
                    Value::Float(_) => ValueType::Float,
                    Value::String(_) => ValueType::String,
                    Value::Null(_) => ValueType::Null,
                    Value::Date(_) => ValueType::Date,
                    Value::Time(_) => ValueType::Time,
                    Value::DateTime(_) => ValueType::DateTime,
                    Value::Vertex(_) => ValueType::Vertex,
                    Value::Edge(_) => ValueType::Edge,
                    Value::Path(_) => ValueType::Path,
                    Value::List(_) => ValueType::List,
                    Value::Map(_) => ValueType::Map,
                    Value::Set(_) => ValueType::Set,
                    _ => ValueType::Unknown,
                }
            }
            Expression::Variable(_) => ValueType::Unknown,
            Expression::Property { .. } => ValueType::Unknown,
            Expression::Binary { op, .. } => {
                match op {
                    crate::core::BinaryOperator::Equal
                    | crate::core::BinaryOperator::NotEqual
                    | crate::core::BinaryOperator::LessThan
                    | crate::core::BinaryOperator::LessThanOrEqual
                    | crate::core::BinaryOperator::GreaterThan
                    | crate::core::BinaryOperator::GreaterThanOrEqual => ValueType::Bool,
                    crate::core::BinaryOperator::And | crate::core::BinaryOperator::Or => ValueType::Bool,
                    _ => ValueType::Unknown,
                }
            }
            Expression::Unary { .. } => ValueType::Unknown,
            Expression::Function { name, .. } => {
                match name.to_lowercase().as_str() {
                    "id" => ValueType::String,
                    "count" | "sum" | "avg" | "min" | "max" => ValueType::Float,
                    "length" | "size" => ValueType::Int,
                    "to_string" | "string" => ValueType::String,
                    "abs" => ValueType::Float,
                    "floor" | "ceil" | "round" => ValueType::Int,
                    _ => ValueType::Unknown,
                }
            }
            Expression::Aggregate { func, .. } => {
                match func {
                    crate::core::AggregateFunction::Count(_) => ValueType::Int,
                    crate::core::AggregateFunction::Sum(_) => ValueType::Float,
                    crate::core::AggregateFunction::Avg(_) => ValueType::Float,
                    crate::core::AggregateFunction::Collect(_) => ValueType::List,
                    _ => ValueType::Unknown,
                }
            }
            Expression::List(_) => ValueType::List,
            Expression::Map(_) => ValueType::Map,
            _ => ValueType::Unknown,
        }
     }
}

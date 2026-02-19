//! GO 语句验证器
//! 对应 NebulaGraph GoValidator.h/.cpp 的功能
//! 验证 GO FROM ... OVER ... WHERE ... YIELD ... 语句

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::{
    AggregateFunction, BinaryOperator, DataType, Expression, UnaryOperator,
};
use crate::core::types::EdgeDirection;
use crate::query::context::validate::ValidationContext;
use crate::query::parser::ast::stmt::GoStmt;
use crate::query::validator::core::{ColumnDef, StatementType, StatementValidator};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct GoContext {
    pub from_source: Option<GoSource>,
    pub over_edges: Vec<OverEdge>,
    pub where_filter: Option<Expression>,
    pub yield_columns: Vec<GoYieldColumn>,
    pub step_range: Option<StepRange>,
    pub inputs: Vec<GoInput>,
    pub outputs: Vec<GoOutput>,
    pub is_truncate: bool,
    pub truncate_columns: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct GoSource {
    pub source_type: GoSourceType,
    pub expression: Expression,
    pub is_variable: bool,
    pub variable_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GoSourceType {
    VertexId,
    Expression,
    Variable,
    Parameter,
}

#[derive(Debug, Clone)]
pub struct OverEdge {
    pub edge_name: String,
    pub edge_type: Option<i32>,
    pub direction: EdgeDirection,
    pub props: Vec<EdgeProperty>,
    pub is_reversible: bool,
    pub is_all: bool,
}

#[derive(Debug, Clone)]
pub struct EdgeProperty {
    pub name: String,
    pub prop_name: String,
    pub prop_type: DataType,
}

#[derive(Debug, Clone)]
pub struct GoYieldColumn {
    pub expression: Expression,
    pub alias: String,
    pub is_distinct: bool,
}

#[derive(Debug, Clone)]
pub struct StepRange {
    pub step_from: i32,
    pub step_to: i32,
}

#[derive(Debug, Clone)]
pub struct GoInput {
    pub name: String,
    pub columns: Vec<InputColumn>,
}

#[derive(Debug, Clone)]
pub struct InputColumn {
    pub name: String,
    pub type_: DataType,
}

#[derive(Debug, Clone)]
pub struct GoOutput {
    pub name: String,
    pub type_: DataType,
    pub alias: String,
}

pub struct GoValidator {
    context: GoContext,
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
}

impl GoValidator {
    pub fn new() -> Self {
        Self {
            context: GoContext {
                from_source: None,
                over_edges: Vec::new(),
                where_filter: None,
                yield_columns: Vec::new(),
                step_range: None,
                inputs: Vec::new(),
                outputs: Vec::new(),
                is_truncate: false,
                truncate_columns: Vec::new(),
            },
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    pub fn validate(&mut self) -> Result<(), ValidationError> {
        self.validate_from_clause()?;
        self.validate_over_clause()?;
        self.validate_where_clause()?;
        self.validate_yield_clause()?;
        self.validate_step_range()?;
        self.build_outputs()?;

        Ok(())
    }

    fn validate_from_clause(&mut self) -> Result<(), ValidationError> {
        if let Some(ref source) = self.context.from_source {
            match &source.source_type {
                GoSourceType::VertexId | GoSourceType::Expression | GoSourceType::Parameter => {
                    self.validate_expression(&source.expression)?;
                }
                GoSourceType::Variable => {
                    if let Expression::Variable(ref var_name) = source.expression {
                        self.validate_variable_reference(var_name)?;
                    } else {
                        return Err(ValidationError::new(
                            "FROM 子句中的变量引用格式不正确".to_string(),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    fn validate_over_clause(&mut self) -> Result<(), ValidationError> {
        if self.context.over_edges.is_empty() {
            return Err(ValidationError::new(
                "OVER 子句必须指定至少一条边".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        for edge in &self.context.over_edges {
            if edge.edge_name.is_empty() {
                return Err(ValidationError::new(
                    "边名称不能为空".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }

            for prop in &edge.props {
                if prop.name.is_empty() || prop.prop_name.is_empty() {
                    return Err(ValidationError::new(
                        "边属性名称不能为空".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate_where_clause(&mut self) -> Result<(), ValidationError> {
        if let Some(ref filter) = self.context.where_filter {
            self.validate_expression(filter)?;
        }

        Ok(())
    }

    fn validate_yield_clause(&mut self) -> Result<(), ValidationError> {
        let mut column_names = HashMap::new();

        for column in &self.context.yield_columns {
            if column_names.get(&column.alias).is_some() {
                return Err(ValidationError::new(
                    format!("YIELD 列别名 '{}' 重复出现", column.alias),
                    ValidationErrorType::DuplicateKey,
                ));
            }
            column_names.insert(column.alias.clone(), true);
            self.validate_expression(&column.expression)?;
        }

        Ok(())
    }

    fn validate_step_range(&mut self) -> Result<(), ValidationError> {
        if let Some(ref range) = self.context.step_range {
            if range.step_from < 0 {
                return Err(ValidationError::new(
                    "步数范围起始值不能为负".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
            if range.step_to < range.step_from {
                return Err(ValidationError::new(
                    "步数范围结束值不能小于起始值".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }

        Ok(())
    }

    fn validate_expression(&self, expression: &Expression) -> Result<(), ValidationError> {
        match expression {
            Expression::Literal(_) => Ok(()),
            Expression::Variable(name) => self.validate_variable_reference(name),
            Expression::Property { object, property } => {
                self.validate_expression(object)?;
                self.validate_property_name(property)?;
                Ok(())
            }
            Expression::Binary { left, op: _, right } => {
                self.validate_expression(left)?;
                self.validate_expression(right)?;
                Ok(())
            }
            Expression::Unary { op: _, operand } => {
                self.validate_expression(operand)?;
                Ok(())
            }
            Expression::Function { name, args } => {
                self.validate_function_name(name)?;
                for arg in args {
                    self.validate_expression(arg)?;
                }
                Ok(())
            }
            Expression::Aggregate { func: _, arg, .. } => {
                self.validate_expression(arg)?;
                Ok(())
            }
            Expression::List(items) => {
                for item in items {
                    self.validate_expression(item)?;
                }
                Ok(())
            }
            Expression::Map(pairs) => {
                for (_key, value) in pairs {
                    self.validate_expression(value)?;
                }
                Ok(())
            }
            Expression::Case { test_expr, conditions, default } => {
                if let Some(test_expression) = test_expr {
                    self.validate_expression(test_expression)?;
                }
                for (condition, result) in conditions {
                    self.validate_expression(condition)?;
                    self.validate_expression(result)?;
                }
                if let Some(default_expression) = default {
                    self.validate_expression(default_expression)?;
                }
                Ok(())
            }
            Expression::TypeCast { expression, .. } => {
                self.validate_expression(expression)?;
                Ok(())
            }
            Expression::Subscript { collection, index } => {
                self.validate_expression(collection)?;
                self.validate_expression(index)?;
                Ok(())
            }
            Expression::Range { collection, start, end } => {
                self.validate_expression(collection)?;
                if let Some(start_expression) = start {
                    self.validate_expression(start_expression)?;
                }
                if let Some(end_expression) = end {
                    self.validate_expression(end_expression)?;
                }
                Ok(())
            }
            Expression::Path(items) => {
                for item in items {
                    self.validate_expression(item)?;
                }
                Ok(())
            }
            Expression::Label(name) => {
                self.validate_label_name(name)?;
                Ok(())
            }
            Expression::ListComprehension { .. } => Ok(()),
            Expression::LabelTagProperty { tag, .. } => self.validate_expression(tag),
            Expression::TagProperty { .. } => Ok(()),
            Expression::EdgeProperty { .. } => Ok(()),
            Expression::Predicate { args, .. } => {
                for arg in args {
                    self.validate_expression(arg)?;
                }
                Ok(())
            }
            Expression::Reduce { initial, source, mapping, .. } => {
                self.validate_expression(initial)?;
                self.validate_expression(source)?;
                self.validate_expression(mapping)?;
                Ok(())
            }
            Expression::PathBuild(exprs) => {
                for expr in exprs {
                    self.validate_expression(expr)?;
                }
                Ok(())
            }
        }
    }

    fn validate_variable_reference(&self, var_name: &str) -> Result<(), ValidationError> {
        if var_name.is_empty() {
            return Err(ValidationError::new(
                "变量名不能为空".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        if var_name == "$-" {
            if let Some(ref source) = self.context.from_source {
                if source.source_type != GoSourceType::Expression {
                    return Err(ValidationError::new(
                        "$- 必须在 FROM 中使用管道输入".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            return Ok(());
        }

        let input_exists = self.context.inputs.iter().any(|input| input.name == var_name);
        if !input_exists {
            return Err(ValidationError::new(
                format!("变量 '{}' 未定义", var_name),
                ValidationErrorType::SemanticError,
            ));
        }

        Ok(())
    }

    fn validate_property_name(&self, prop_name: &str) -> Result<(), ValidationError> {
        if prop_name.is_empty() {
            return Err(ValidationError::new(
                "属性名不能为空".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    fn validate_function_name(&self, func_name: &str) -> Result<(), ValidationError> {
        if func_name.is_empty() {
            return Err(ValidationError::new(
                "函数名不能为空".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    fn validate_label_name(&self, label_name: &str) -> Result<(), ValidationError> {
        if label_name.is_empty() {
            return Err(ValidationError::new(
                "标签名不能为空".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    fn build_outputs(&mut self) -> Result<(), ValidationError> {
        for column in &self.context.yield_columns {
            let output = GoOutput {
                name: column.alias.clone(),
                type_: DataType::String,
                alias: column.alias.clone(),
            };
            self.context.outputs.push(output);
        }

        Ok(())
    }

    pub fn context(&self) -> &GoContext {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut GoContext {
        &mut self.context
    }

    pub fn set_from_source(&mut self, source: GoSource) {
        self.context.from_source = Some(source);
    }

    pub fn add_over_edge(&mut self, edge: OverEdge) {
        self.context.over_edges.push(edge);
    }

    pub fn set_where_filter(&mut self, filter: Expression) {
        self.context.where_filter = Some(filter);
    }

    pub fn add_yield_column(&mut self, column: GoYieldColumn) {
        self.context.yield_columns.push(column);
    }

    pub fn set_step_range(&mut self, range: StepRange) {
        self.context.step_range = Some(range);
    }

    pub fn validate_from_stmt(&mut self, go_stmt: &GoStmt) -> Result<(), ValidationError> {
        self.build_from_context(&go_stmt.from)?;

        if let Some(ref over) = go_stmt.over {
            self.build_over_context(over)?;
        }

        self.context.where_filter = go_stmt.where_clause.clone();

        if let Some(ref yield_clause) = go_stmt.yield_clause {
            self.build_yield_context(yield_clause)?;
        }

        self.build_step_range(&go_stmt.steps)?;

        self.validate()
    }

    fn build_from_context(
        &mut self,
        from: &crate::query::parser::ast::stmt::FromClause,
    ) -> Result<(), ValidationError> {
        if from.vertices.is_empty() {
            return Err(ValidationError::new(
                "GO 语句必须指定 FROM 顶点".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        let first_vertex = from.vertices.first().unwrap();
        
        let source_type = match first_vertex {
            Expression::Variable(_) => GoSourceType::Variable,
            Expression::List(_) => GoSourceType::VertexId,
            _ => GoSourceType::Expression,
        };

        let variable_name = match first_vertex {
            Expression::Variable(name) => Some(name.clone()),
            _ => None,
        };

        self.context.from_source = Some(GoSource {
            source_type,
            expression: first_vertex.clone(),
            is_variable: variable_name.is_some(),
            variable_name,
        });

        Ok(())
    }

    fn build_over_context(
        &mut self,
        over: &crate::query::parser::ast::stmt::OverClause,
    ) -> Result<(), ValidationError> {
        if over.edge_types.is_empty() {
            return Err(ValidationError::new(
                "OVER 子句必须指定至少一条边".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        for edge_type in &over.edge_types {
            self.context.over_edges.push(OverEdge {
                edge_name: edge_type.clone(),
                edge_type: None,
                direction: over.direction,
                props: Vec::new(),
                is_reversible: over.direction == EdgeDirection::Both,
                is_all: false,
            });
        }

        Ok(())
    }

    fn build_yield_context(
        &mut self,
        yield_clause: &crate::query::parser::ast::stmt::YieldClause,
    ) -> Result<(), ValidationError> {
        for item in &yield_clause.items {
            let column = GoYieldColumn {
                expression: item.expression.clone(),
                alias: item.alias.clone().unwrap_or_else(|| {
                    format!("col_{}", self.context.yield_columns.len())
                }),
                is_distinct: false,
            };
            self.context.yield_columns.push(column);
        }

        Ok(())
    }

    fn build_step_range(
        &mut self,
        steps: &crate::query::parser::ast::stmt::Steps,
    ) -> Result<(), ValidationError> {
        let (from, to) = match steps {
            crate::query::parser::ast::stmt::Steps::Fixed(n) => (*n as i32, *n as i32),
            crate::query::parser::ast::stmt::Steps::Range { min, max } => (*min as i32, *max as i32),
            crate::query::parser::ast::stmt::Steps::Variable(_) => {
                return Err(ValidationError::new(
                    "GO 语句不支持变量步数".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        self.context.step_range = Some(StepRange {
            step_from: from,
            step_to: to,
        });

        Ok(())
    }
}

impl Default for GoValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl StatementValidator for GoValidator {
    fn validate(&mut self, _ctx: &mut ValidationContext) -> Result<(), ValidationError> {
        self.validate()
    }

    fn statement_type(&self) -> StatementType {
        StatementType::Go
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn add_input(&mut self, col: ColumnDef) {
        self.inputs.push(col);
    }

    fn add_output(&mut self, col: ColumnDef) {
        self.outputs.push(col);
    }
}

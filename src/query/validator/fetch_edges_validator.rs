//! 边获取验证器
//! 对应 NebulaGraph FetchEdgesValidator.h/.cpp 的功能
//! 验证 FETCH PROP ON ... 语句

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::Expression;
use crate::core::DataType;
use crate::query::context::validate::ValidationContext;
use crate::query::validator::core::{ColumnDef, StatementType, StatementValidator};

#[derive(Debug, Clone)]
pub struct FetchEdgesContext {
    pub edge_keys: Vec<FetchEdgeKey>,
    pub edge_name: String,
    pub edge_type: Option<i32>,
    pub yield_columns: Vec<FetchEdgeYieldColumn>,
    pub outputs: Vec<FetchEdgeOutput>,
    pub schema: Option<FetchEdgeSchema>,
    pub is_system: bool,
}

#[derive(Debug, Clone)]
pub struct FetchEdgeKey {
    pub src_id: EdgeEndpoint,
    pub dst_id: EdgeEndpoint,
    pub rank: Option<Expression>,
    pub is_variable: bool,
}

#[derive(Debug, Clone)]
pub struct EdgeEndpoint {
    pub id_type: EndpointType,
    pub expression: Expression,
    pub is_variable: bool,
    pub variable_name: Option<String>,
}

#[derive(Debug, Clone)]
pub enum EndpointType {
    Constant,
    Expression,
    Variable,
    Parameter,
}

#[derive(Debug, Clone)]
pub struct FetchEdgeYieldColumn {
    pub expression: Expression,
    pub alias: Option<String>,
    pub prop_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FetchEdgeOutput {
    pub name: String,
    pub type_: DataType,
    pub alias: String,
}

#[derive(Debug, Clone)]
pub struct FetchEdgeSchema {
    pub edge_type: i32,
    pub edge_name: String,
    pub props: Vec<FetchEdgePropDef>,
}

#[derive(Debug, Clone)]
pub struct FetchEdgePropDef {
    pub name: String,
    pub type_: DataType,
    pub is_nullable: bool,
    pub default_value: Option<Expression>,
}

pub struct FetchEdgesValidator {
    context: FetchEdgesContext,
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
}

impl FetchEdgesValidator {
    pub fn new() -> Self {
        Self {
            context: FetchEdgesContext {
                edge_keys: Vec::new(),
                edge_name: String::new(),
                edge_type: None,
                yield_columns: Vec::new(),
                outputs: Vec::new(),
                schema: None,
                is_system: false,
            },
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    pub fn validate(&mut self) -> Result<(), ValidationError> {
        self.validate_edge_name()?;
        self.validate_edge_keys()?;
        self.validate_yield_clause()?;
        self.validate_edge_props()?;
        self.build_outputs()?;

        Ok(())
    }

    fn validate_edge_name(&mut self) -> Result<(), ValidationError> {
        if self.context.edge_name.is_empty() {
            return Err(ValidationError::new(
                "必须指定边类型名称".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    fn validate_edge_keys(&mut self) -> Result<(), ValidationError> {
        if self.context.edge_keys.is_empty() {
            return Err(ValidationError::new(
                "必须指定至少一个边键".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        for edge_key in &self.context.edge_keys {
            match &edge_key.src_id.expression {
                Expression::Literal(value) => {
                    if value.is_null() || value.is_empty() {
                        return Err(ValidationError::new(
                            "边键的源顶点 ID 不能为空".to_string(),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                }
                _ => {}
            }

            match &edge_key.dst_id.expression {
                Expression::Literal(value) => {
                    if value.is_null() || value.is_empty() {
                        return Err(ValidationError::new(
                            "边键的目标顶点 ID 不能为空".to_string(),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                }
                _ => {}
            }

            if let Some(ref rank_expression) = edge_key.rank {
                if !rank_expression.is_constant() {
                    return Err(ValidationError::new(
                        "rank 值必须为常量".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }

                if let Expression::Literal(value) = rank_expression {
                    match value {
                        crate::core::Value::Int(i) if *i >= 0 => {}
                        crate::core::Value::Int(_) => {
                            return Err(ValidationError::new(
                                "rank 值必须为非负整数".to_string(),
                                ValidationErrorType::SemanticError,
                            ));
                        }
                        _ => {
                            return Err(ValidationError::new(
                                "rank 值必须为整数类型".to_string(),
                                ValidationErrorType::SemanticError,
                            ));
                        }
                    }
                } else {
                    return Err(ValidationError::new(
                        "rank 值必须为常量".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate_yield_clause(&mut self) -> Result<(), ValidationError> {
        let mut column_names = std::collections::HashMap::new();

        for column in &self.context.yield_columns {
            if let Some(ref alias) = column.alias {
                if column_names.get(alias).is_some() {
                    return Err(ValidationError::new(
                        format!("YIELD 列别名 '{}' 重复出现", alias),
                        ValidationErrorType::DuplicateKey,
                    ));
                }
                column_names.insert(alias.clone(), true);
            }
        }

        Ok(())
    }

    fn validate_edge_props(&mut self) -> Result<(), ValidationError> {
        for column in &self.context.yield_columns {
            if let Some(ref prop_name) = column.prop_name {
                if let Some(ref schema) = self.context.schema {
                    let prop_exists = schema.props.iter().any(|p| &p.name == prop_name);
                    if !prop_exists {
                        return Err(ValidationError::new(
                            format!("属性 '{}' 不在边类型 '{}' 中定义", prop_name, schema.edge_name),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    fn build_outputs(&mut self) -> Result<(), ValidationError> {
        for column in &self.context.yield_columns {
            let alias_name = column.alias.clone().unwrap_or_default();
            let output = FetchEdgeOutput {
                name: alias_name.clone(),
                type_: DataType::String,
                alias: alias_name,
            };
            self.context.outputs.push(output);
        }

        Ok(())
    }

    pub fn context(&self) -> &FetchEdgesContext {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut FetchEdgesContext {
        &mut self.context
    }

    pub fn set_edge_name(&mut self, edge_name: String) {
        self.context.edge_name = edge_name;
    }

    pub fn add_edge_key(&mut self, edge_key: FetchEdgeKey) {
        self.context.edge_keys.push(edge_key);
    }

    pub fn add_yield_column(&mut self, column: FetchEdgeYieldColumn) {
        self.context.yield_columns.push(column);
    }

    pub fn set_schema(&mut self, schema: FetchEdgeSchema) {
        self.context.schema = Some(schema);
    }
}

impl Default for FetchEdgesValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl StatementValidator for FetchEdgesValidator {
    fn validate(&mut self, _ctx: &mut ValidationContext) -> Result<(), ValidationError> {
        self.validate()
    }

    fn statement_type(&self) -> StatementType {
        StatementType::FetchEdges
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

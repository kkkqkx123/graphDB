//! 顶点获取验证器
//! 对应 NebulaGraph FetchVerticesValidator.h/.cpp 的功能
//! 验证 FETCH PROP ON ... 语句

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::Expression;
use crate::core::DataType;
use crate::query::context::validate::ValidationContext;
use crate::query::validator::core::{ColumnDef, StatementType, StatementValidator};

#[derive(Debug, Clone)]
pub struct FetchVerticesContext {
    pub vertex_ids: Vec<FetchVertexId>,
    pub tag_names: Vec<String>,
    pub tag_ids: Vec<i32>,
    pub yield_columns: Vec<FetchYieldColumn>,
    pub outputs: Vec<FetchOutput>,
    pub schemas: Vec<FetchSchema>,
    pub is_system: bool,
}

#[derive(Debug, Clone)]
pub struct FetchVertexId {
    pub id_type: VertexIdType,
    pub expression: Expression,
    pub is_variable: bool,
    pub variable_name: Option<String>,
}

#[derive(Debug, Clone)]
pub enum VertexIdType {
    Constant,
    Expression,
    Variable,
    Parameter,
}

#[derive(Debug, Clone)]
pub struct FetchYieldColumn {
    pub expression: Expression,
    pub alias: String,
    pub tag_name: Option<String>,
    pub prop_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FetchOutput {
    pub name: String,
    pub type_: DataType,
    pub alias: String,
    pub tag_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FetchSchema {
    pub tag_id: i32,
    pub tag_name: String,
    pub schema: Vec<FetchPropDef>,
}

#[derive(Debug, Clone)]
pub struct FetchPropDef {
    pub name: String,
    pub type_: DataType,
    pub is_nullable: bool,
    pub default_value: Option<Expression>,
}

pub struct FetchVerticesValidator {
    context: FetchVerticesContext,
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
}

impl FetchVerticesValidator {
    pub fn new() -> Self {
        Self {
            context: FetchVerticesContext {
                vertex_ids: Vec::new(),
                tag_names: Vec::new(),
                tag_ids: Vec::new(),
                yield_columns: Vec::new(),
                outputs: Vec::new(),
                schemas: Vec::new(),
                is_system: false,
            },
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    pub fn validate(&mut self) -> Result<(), ValidationError> {
        self.validate_vertex_ids()?;
        self.validate_tag_clause()?;
        self.validate_yield_clause()?;
        self.validate_tag_props()?;
        self.build_outputs()?;

        Ok(())
    }

    fn validate_vertex_ids(&mut self) -> Result<(), ValidationError> {
        if self.context.vertex_ids.is_empty() {
            return Err(ValidationError::new(
                "必须指定至少一个顶点 ID".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        for vertex_id in &self.context.vertex_ids {
            if vertex_id.is_variable {
                if vertex_id.variable_name.is_none() {
                    return Err(ValidationError::new(
                        "变量引用必须指定变量名".to_string(),
                        ValidationErrorType::VariableNotFound,
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate_tag_clause(&mut self) -> Result<(), ValidationError> {
        let mut tag_set = std::collections::HashSet::new();

        for tag_name in &self.context.tag_names {
            if !tag_set.insert(tag_name) {
                return Err(ValidationError::new(
                    format!("标签 '{}' 重复出现", tag_name),
                    ValidationErrorType::DuplicateKey,
                ));
            }
        }

        if self.context.tag_names.is_empty() {
            return Err(ValidationError::new(
                "必须指定至少一个标签".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        Ok(())
    }

    fn validate_yield_clause(&mut self) -> Result<(), ValidationError> {
        let mut column_names = std::collections::HashMap::new();

        for column in &self.context.yield_columns {
            if column_names.get(&column.alias).is_some() {
                return Err(ValidationError::new(
                    format!("YIELD 列别名 '{}' 重复出现", column.alias),
                    ValidationErrorType::DuplicateKey,
                ));
            }
            column_names.insert(column.alias.clone(), true);
        }

        Ok(())
    }

    fn validate_tag_props(&mut self) -> Result<(), ValidationError> {
        for column in &self.context.yield_columns {
            if let (Some(tag_name), Some(prop_name)) = (&column.tag_name, &column.prop_name) {
                let tag_exists = self.context.tag_names.contains(tag_name);
                if !tag_exists {
                    return Err(ValidationError::new(
                        format!("标签 '{}' 不在查询的标签列表中", tag_name),
                        ValidationErrorType::SemanticError,
                    ));
                }

                let prop_exists = self.context.schemas.iter()
                    .find(|schema| &schema.tag_name == tag_name)
                    .map_or(false, |schema| {
                        schema.schema.iter()
                            .any(|prop_def| prop_def.name == *prop_name)
                    });

                if !prop_exists {
                    return Err(ValidationError::new(
                        format!("属性 '{}' 在标签 '{}' 中不存在", prop_name, tag_name),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
        }

        Ok(())
    }

    fn build_outputs(&mut self) -> Result<(), ValidationError> {
        for column in &self.context.yield_columns {
            let output = FetchOutput {
                name: column.alias.clone(),
                type_: DataType::String,
                alias: column.alias.clone(),
                tag_name: column.tag_name.clone(),
            };
            self.context.outputs.push(output);
        }

        Ok(())
    }

    pub fn context(&self) -> &FetchVerticesContext {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut FetchVerticesContext {
        &mut self.context
    }

    pub fn add_vertex_id(&mut self, vertex_id: FetchVertexId) {
        self.context.vertex_ids.push(vertex_id);
    }

    pub fn add_tag_name(&mut self, tag_name: String) {
        self.context.tag_names.push(tag_name);
    }

    pub fn add_yield_column(&mut self, column: FetchYieldColumn) {
        self.context.yield_columns.push(column);
    }

    pub fn add_schema(&mut self, schema: FetchSchema) {
        self.context.schemas.push(schema);
    }
}

impl Default for FetchVerticesValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl StatementValidator for FetchVerticesValidator {
    fn validate(&mut self, _ctx: &mut ValidationContext) -> Result<(), ValidationError> {
        self.validate()
    }

    fn statement_type(&self) -> StatementType {
        StatementType::FetchVertices
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

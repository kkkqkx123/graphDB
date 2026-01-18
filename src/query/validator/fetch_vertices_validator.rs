//! 顶点获取验证器
//! 对应 NebulaGraph FetchVerticesValidator.h/.cpp 的功能
//! 验证 FETCH PROP ON ... 语句

use super::base_validator::Validator;
use super::validation_interface::{ValidationError, ValidationErrorType};
use crate::core::Expression;

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
    pub type_: String,
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
    pub type_: String,
    pub is_nullable: bool,
    pub default_value: Option<Expression>,
}

pub struct FetchVerticesValidator {
    base: Validator,
    context: FetchVerticesContext,
}

impl FetchVerticesValidator {
    pub fn new(context: super::ValidationContext) -> Self {
        Self {
            base: Validator::new(context),
            context: FetchVerticesContext {
                vertex_ids: Vec::new(),
                tag_names: Vec::new(),
                tag_ids: Vec::new(),
                yield_columns: Vec::new(),
                outputs: Vec::new(),
                schemas: Vec::new(),
                is_system: false,
            },
        }
    }

    pub fn validate(&mut self) -> Result<(), ValidationError> {
        self.validate_vertex_ids()?;
        self.validate_tag_clause()?;
        self.validate_yield_clause()?;
        self.validate_tag_props()?;
        self.build_outputs()?;

        if self.base.context().has_validation_errors() {
            let errors = self.base.context().get_validation_errors();
            if let Some(first_error) = errors.first() {
                return Err(first_error.clone());
            }
        }

        Ok(())
    }

    fn validate_vertex_ids(&mut self) -> Result<(), ValidationError> {
        // 验证顶点 ID 列表
        // 需要检查：
        // 1. 顶点 ID 不能为空
        // 2. 如果是变量引用，变量必须存在且类型正确

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
        // 验证标签子句
        // 需要检查：
        // 1. 标签必须存在
        // 2. 标签不能重复

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
        // 验证 YIELD 子句
        // 需要检查：
        // 1. 引用的属性必须在标签中存在
        // 2. 别名不能重复

        let mut column_names = std::collections::HashMap::new();

        for column in &self.context.yield_columns {
            if let Some(_) = column_names.get(&column.alias) {
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
        // 验证标签属性
        // 需要检查：
        // 1. 属性必须在标签 Schema 中定义
        // 2. 只能引用指定标签的属性

        for column in &self.context.yield_columns {
            if let (Some(tag_name), Some(prop_name)) = (&column.tag_name, &column.prop_name) {
                let tag_exists = self.context.tag_names.contains(tag_name);
                if !tag_exists {
                    return Err(ValidationError::new(
                        format!("标签 '{}' 不在查询的标签列表中", tag_name),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
        }

        Ok(())
    }

    fn build_outputs(&mut self) -> Result<(), ValidationError> {
        // 构建输出列
        // 每个 YIELD 列对应一个输出

        for column in &self.context.yield_columns {
            let output = FetchOutput {
                name: column.alias.clone(),
                type_: String::new(),
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

impl super::validation_interface::ValidationStrategy for FetchVerticesValidator {
    fn validate(&self, _context: &dyn super::validation_interface::ValidationContext) -> Result<(), ValidationError> {
        Ok(())
    }

    fn strategy_type(&self) -> super::validation_interface::ValidationStrategyType {
        super::validation_interface::ValidationStrategyType::Clause
    }

    fn strategy_name(&self) -> &'static str {
        "FetchVerticesValidator"
    }
}

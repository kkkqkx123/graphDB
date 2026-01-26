//! 边获取验证器
//! 对应 NebulaGraph FetchEdgesValidator.h/.cpp 的功能
//! 验证 FETCH PROP ON ... 语句

use super::base_validator::Validator;
use super::validation_interface::{ValidationError, ValidationErrorType};
use crate::core::Expression;
use crate::core::types::DataType;

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
    base: Validator,
    context: FetchEdgesContext,
}

impl FetchEdgesValidator {
    pub fn new(context: super::ValidationContext) -> Self {
        Self {
            base: Validator::with_context(context),
            context: FetchEdgesContext {
                edge_keys: Vec::new(),
                edge_name: String::new(),
                edge_type: None,
                yield_columns: Vec::new(),
                outputs: Vec::new(),
                schema: None,
                is_system: false,
            },
        }
    }

    pub fn validate(&mut self) -> Result<(), ValidationError> {
        self.validate_edge_name()?;
        self.validate_edge_keys()?;
        self.validate_yield_clause()?;
        self.validate_edge_props()?;
        self.build_outputs()?;

        if self.base.context().has_validation_errors() {
            let errors = self.base.context().get_validation_errors();
            if let Some(first_error) = errors.first() {
                return Err(first_error.clone());
            }
        }

        Ok(())
    }

    fn validate_edge_name(&mut self) -> Result<(), ValidationError> {
        // 验证边类型名称
        // 需要检查：
        // 1. 边类型名称不能为空
        // 2. 边类型必须存在

        if self.context.edge_name.is_empty() {
            return Err(ValidationError::new(
                "必须指定边类型名称".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // TODO: 检查边类型是否存在
        // 需要通过 SchemaManager 查询边类型是否存在

        Ok(())
    }

    fn validate_edge_keys(&mut self) -> Result<(), ValidationError> {
        // 验证边键列表
        // 需要检查：
        // 1. 边键不能为空
        // 2. 源顶点和目标顶点必须有效
        // 3. rank 值必须为非负整数

        if self.context.edge_keys.is_empty() {
            return Err(ValidationError::new(
                "必须指定至少一个边键".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        for edge_key in &self.context.edge_keys {
            // 验证源顶点表达式是否有效
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

            // 验证目标顶点表达式是否有效
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

            // 验证 rank 值
            if let Some(ref rank_expression) = edge_key.rank {
                // 检查 rank 表达式是否为常量
                if !rank_expression.is_constant() {
                    return Err(ValidationError::new(
                        "rank 值必须为常量".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }

                // 检查 rank 值是否为非负整数
                if let Expression::Literal(value) = rank_expression {
                    match value {
                        crate::core::Value::Int(i) if *i >= 0 => {
                            // 非负整数，验证通过
                        }
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
        // 验证 YIELD 子句
        // 需要检查：
        // 1. 引用的属性必须在边 Schema 中定义
        // 2. 别名不能重复

        let mut column_names = std::collections::HashMap::new();

        for column in &self.context.yield_columns {
            if let Some(ref alias) = column.alias {
                if let Some(_) = column_names.get(alias) {
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
        // 验证边属性
        // 需要检查：
        // 1. 属性必须在边 Schema 中定义

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
        // 构建输出列
        // 每个 YIELD 列对应一个输出

        for column in &self.context.yield_columns {
            let alias_name = column.alias.clone().unwrap_or_else(|| String::new());
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

impl super::validation_interface::ValidationStrategy for FetchEdgesValidator {
    fn validate(&self, _context: &dyn super::validation_interface::ValidationContext) -> Result<(), ValidationError> {
        Ok(())
    }

    fn strategy_type(&self) -> super::validation_interface::ValidationStrategyType {
        super::validation_interface::ValidationStrategyType::Clause
    }

    fn strategy_name(&self) -> &'static str {
        "FetchEdgesValidator"
    }
}

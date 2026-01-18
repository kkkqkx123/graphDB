//! GO 语句验证器
//! 对应 NebulaGraph GoValidator.h/.cpp 的功能
//! 验证 GO FROM ... OVER ... WHERE ... YIELD ... 语句

use super::base_validator::Validator;
use super::validation_interface::{ValidationError, ValidationErrorType};
use crate::core::Expression;
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

#[derive(Debug, Clone)]
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
pub enum EdgeDirection {
    Forward,
    Backward,
    Both,
}

#[derive(Debug, Clone)]
pub struct EdgeProperty {
    pub name: String,
    pub prop_name: String,
    pub prop_type: String,
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
    pub type_: String,
}

#[derive(Debug, Clone)]
pub struct GoOutput {
    pub name: String,
    pub type_: String,
    pub alias: String,
}

pub struct GoValidator {
    base: Validator,
    context: GoContext,
}

impl GoValidator {
    pub fn new(context: super::ValidationContext) -> Self {
        Self {
            base: Validator::new(context),
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
        }
    }

    pub fn validate(&mut self) -> Result<(), ValidationError> {
        self.validate_from_clause()?;
        self.validate_over_clause()?;
        self.validate_where_clause()?;
        self.validate_yield_clause()?;
        self.validate_step_range()?;
        self.build_outputs()?;

        if self.base.context().has_validation_errors() {
            let errors = self.base.context().get_validation_errors();
            if let Some(first_error) = errors.first() {
                return Err(first_error.clone());
            }
        }

        Ok(())
    }

    fn validate_from_clause(&mut self) -> Result<(), ValidationError> {
        // 验证 FROM 子句
        // 需要检查：
        // 1. 起始点表达式是否有效
        // 2. 如果是变量引用，变量是否存在
        // 3. 如果是常量表达式，类型是否正确

        // 子类重写此方法进行具体验证
        Ok(())
    }

    fn validate_over_clause(&mut self) -> Result<(), ValidationError> {
        // 验证 OVER 子句
        // 需要检查：
        // 1. 边类型是否存在
        // 2. 方向是否有效
        // 3. 属性引用是否正确

        // 子类重写此方法进行具体验证
        Ok(())
    }

    fn validate_where_clause(&mut self) -> Result<(), ValidationError> {
        // 验证 WHERE 子句
        // 需要检查：
        // 1. 过滤表达式是否有效
        // 2. 引用的属性是否存在
        // 3. 类型兼容性

        if let Some(ref filter) = self.context.where_filter {
            // 验证过滤表达式
            // TODO: 实现表达式验证
        }

        Ok(())
    }

    fn validate_yield_clause(&mut self) -> Result<(), ValidationError> {
        // 验证 YIELD 子句
        // 需要检查：
        // 1. 返回列表达式是否有效
        // 2. 别名是否重复
        // 3. 属性引用是否正确

        let mut column_names = HashMap::new();

        for column in &self.context.yield_columns {
            if let Some(existing) = column_names.get(&column.alias) {
                return Err(ValidationError::new(
                    format!("YIELD 列别名 '{}' 重复出现", column.alias),
                    ValidationErrorType::DuplicateKey,
                ));
            }
            column_names.insert(column.alias.clone(), true);
        }

        Ok(())
    }

    fn validate_step_range(&mut self) -> Result<(), ValidationError> {
        // 验证步数范围
        // 需要检查：
        // 1. 起始步数是否为正
        // 2. 结束步数是否大于等于起始步数

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

    fn build_outputs(&mut self) -> Result<(), ValidationError> {
        // 构建输出列
        // 根据 YIELD 子句构建输出定义

        for column in &self.context.yield_columns {
            let output = GoOutput {
                name: column.alias.clone(),
                type_: String::new(), // 需要通过类型推导确定
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
}

impl super::validation_interface::ValidationStrategy for GoValidator {
    fn validate(&self, _context: &dyn super::validation_interface::ValidationContext) -> Result<(), ValidationError> {
        Ok(())
    }

    fn strategy_type(&self) -> super::validation_interface::ValidationStrategyType {
        super::validation_interface::ValidationStrategyType::Clause
    }

    fn strategy_name(&self) -> &'static str {
        "GoValidator"
    }
}

//! 变量赋值语句验证器
//! 对应 NebulaGraph AssignmentValidator 的功能
//! 验证变量赋值语句的合法性，如 $var = GO FROM ...
//!
//! 设计原则：
//! 1. 实现了 StatementValidator trait，统一接口
//! 2. 赋值语句包装其他语句，需要递归验证内部语句
//! 3. 变量名验证（必须以$开头）

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::query::context::ast::AstContext;
use crate::query::parser::ast::stmt::AssignmentStmt;
use crate::query::validator::validator_trait::{
    StatementType, StatementValidator, ValidationResult, ColumnDef,
    ExpressionProps,
};
use crate::query::validator::validator_enum::Validator;

/// 验证后的赋值信息
#[derive(Debug, Clone)]
pub struct ValidatedAssignment {
    pub variable: String,
    pub inner_statement_type: String,
}

/// 赋值语句验证器
#[derive(Debug)]
pub struct AssignmentValidator {
    variable: String,
    inner_validator: Option<Box<Validator>>,
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expr_props: ExpressionProps,
    user_defined_vars: Vec<String>,
}

impl AssignmentValidator {
    pub fn new() -> Self {
        Self {
            variable: String::new(),
            inner_validator: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    fn validate_impl(&mut self, stmt: &AssignmentStmt) -> Result<(), ValidationError> {
        // 验证变量名
        self.variable = stmt.variable.clone();
        self.validate_variable_name(&self.variable)?;

        // 创建内部语句验证器
        self.inner_validator = Some(Box::new(
            Validator::from_stmt(&stmt.statement)
                .ok_or_else(|| ValidationError::new(
                    "Failed to create validator for inner statement".to_string(),
                    ValidationErrorType::SemanticError,
                ))?
        ));

        Ok(())
    }

    fn validate_variable_name(&self, name: &str) -> Result<(), ValidationError> {
        // 变量名不能为空
        if name.is_empty() {
            return Err(ValidationError::new(
                "Variable name cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 变量名必须以字母或下划线开头
        let first_char = name.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() && first_char != '_' {
            return Err(ValidationError::new(
                format!("Variable name '{}' must start with a letter or underscore", name),
                ValidationErrorType::SemanticError,
            ));
        }

        // 变量名只能包含字母、数字和下划线
        for (i, c) in name.chars().enumerate() {
            if i > 0 && !c.is_ascii_alphanumeric() && c != '_' {
                return Err(ValidationError::new(
                    format!("Variable name '{}' contains invalid character '{}'", name, c),
                    ValidationErrorType::SemanticError,
                ));
            }
        }

        Ok(())
    }

    /// 获取变量名
    pub fn variable(&self) -> &str {
        &self.variable
    }

    /// 获取内部验证器
    pub fn inner_validator(&self) -> Option<&Validator> {
        self.inner_validator.as_deref()
    }

    pub fn validated_result(&self) -> ValidatedAssignment {
        ValidatedAssignment {
            variable: self.variable.clone(),
            inner_statement_type: self.inner_validator.as_ref()
                .map(|v| v.statement_type().as_str().to_string())
                .unwrap_or_default(),
        }
    }
}

impl StatementValidator for AssignmentValidator {
    fn validate(&mut self, ast: &mut AstContext) -> Result<ValidationResult, ValidationError> {
        let stmt = ast.sentence.as_ref()
            .and_then(|s| s.as_assignment())
            .ok_or_else(|| ValidationError::new(
                "Expected ASSIGNMENT statement".to_string(),
                ValidationErrorType::SemanticError,
            ))?;
        
        self.validate_impl(stmt)?;
        
        // 验证内部语句
        if let Some(ref mut inner) = self.inner_validator {
            let mut inner_ast = AstContext::new(ast.qctx.clone(), Some(*stmt.statement.clone()));
            let result = inner.validate(&mut inner_ast)?;
            
            // 赋值语句的输出与内部语句相同
            self.inputs = result.inputs.clone();
            self.outputs = result.outputs.clone();
            
            // 添加变量到用户定义变量列表
            if !self.user_defined_vars.contains(&self.variable) {
                self.user_defined_vars.push(self.variable.clone());
            }
        }
        
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::Assignment
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        // 赋值语句是否为全局语句取决于内部语句
        self.inner_validator.as_ref()
            .map(|v| v.is_global_statement())
            .unwrap_or(false)
    }

    fn expression_props(&self) -> &ExpressionProps {
        &self.expr_props
    }

    fn user_defined_vars(&self) -> &[String] {
        &self.user_defined_vars
    }
}

impl Default for AssignmentValidator {
    fn default() -> Self {
        Self::new()
    }
}

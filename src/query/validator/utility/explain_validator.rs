//! Explain/Profile 语句验证器
//! 对应 NebulaGraph ExplainValidator 的功能
//! 验证 EXPLAIN 和 PROFILE 语句
//!
//! 设计原则：
//! 1. 实现了 StatementValidator trait，统一接口
//! 2. EXPLAIN/PROFILE 包装其他语句，需要递归验证内部语句
//! 3. 支持多种输出格式（row, dot）

use std::sync::Arc;

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::query::QueryContext;
use crate::query::parser::ast::stmt::{ExplainStmt, ProfileStmt, ExplainFormat};
use crate::query::validator::validator_trait::{
    StatementType, StatementValidator, ValidationResult, ColumnDef, ValueType,
    ExpressionProps,
};
use crate::query::validator::validator_enum::Validator;

/// 验证后的 EXPLAIN 信息
#[derive(Debug, Clone)]
pub struct ValidatedExplain {
    pub format: ExplainFormat,
    pub inner_statement_type: String,
}

/// EXPLAIN 语句验证器
#[derive(Debug)]
pub struct ExplainValidator {
    format: ExplainFormat,
    inner_validator: Option<Box<Validator>>,
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expr_props: ExpressionProps,
    user_defined_vars: Vec<String>,
}

impl ExplainValidator {
    pub fn new() -> Self {
        Self {
            format: ExplainFormat::Table,
            inner_validator: None,
            inputs: Vec::new(),
            outputs: vec![
                ColumnDef { name: "id".to_string(), type_: ValueType::Int },
                ColumnDef { name: "name".to_string(), type_: ValueType::String },
                ColumnDef { name: "dependencies".to_string(), type_: ValueType::String },
                ColumnDef { name: "profiling_data".to_string(), type_: ValueType::String },
                ColumnDef { name: "operator info".to_string(), type_: ValueType::String },
            ],
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    fn validate_impl(&mut self, stmt: &ExplainStmt) -> Result<(), ValidationError> {
        self.format = stmt.format.clone();

        // 验证内部语句
        self.inner_validator = Some(Box::new(
            Validator::from_stmt(&stmt.statement)
                .ok_or_else(|| ValidationError::new(
                    "Failed to create validator for inner statement".to_string(),
                    ValidationErrorType::SemanticError,
                ))?
        ));

        Ok(())
    }

    /// 获取内部验证器
    pub fn inner_validator(&self) -> Option<&Validator> {
        self.inner_validator.as_deref()
    }

    /// 获取格式类型
    pub fn format(&self) -> &ExplainFormat {
        &self.format
    }

    pub fn validated_result(&self) -> ValidatedExplain {
        ValidatedExplain {
            format: self.format.clone(),
            inner_statement_type: self.inner_validator.as_ref()
                .map(|v| v.statement_type().as_str().to_string())
                .unwrap_or_default(),
        }
    }
}

/// 实现 StatementValidator trait
///
/// # 重构变更
/// - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext
/// - 内部语句验证直接调用 validate 方法，传入 stmt 和 qctx
impl StatementValidator for ExplainValidator {
    fn validate(
        &mut self,
        stmt: &crate::query::parser::ast::Stmt,
        qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        let explain_stmt = match stmt {
            crate::query::parser::ast::Stmt::Explain(explain_stmt) => explain_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected EXPLAIN statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        self.validate_impl(explain_stmt)?;

        // 验证内部语句
        if let Some(ref mut inner) = self.inner_validator {
            inner.validate(&explain_stmt.statement, qctx)?;
        }

        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::Explain
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        // EXPLAIN 是否为全局语句取决于内部语句
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

impl Default for ExplainValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// PROFILE 语句验证器
/// PROFILE 与 EXPLAIN 类似，但会实际执行并收集性能数据
#[derive(Debug)]
pub struct ProfileValidator {
    format: ExplainFormat,
    inner_validator: Option<Box<Validator>>,
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expr_props: ExpressionProps,
    user_defined_vars: Vec<String>,
}

impl ProfileValidator {
    pub fn new() -> Self {
        Self {
            format: ExplainFormat::Table,
            inner_validator: None,
            inputs: Vec::new(),
            outputs: vec![
                ColumnDef { name: "id".to_string(), type_: ValueType::Int },
                ColumnDef { name: "name".to_string(), type_: ValueType::String },
                ColumnDef { name: "dependencies".to_string(), type_: ValueType::String },
                ColumnDef { name: "profiling_data".to_string(), type_: ValueType::String },
                ColumnDef { name: "operator info".to_string(), type_: ValueType::String },
            ],
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    fn validate_impl(&mut self, stmt: &ProfileStmt) -> Result<(), ValidationError> {
        self.format = stmt.format.clone();

        // 验证内部语句
        self.inner_validator = Some(Box::new(
            Validator::from_stmt(&stmt.statement)
                .ok_or_else(|| ValidationError::new(
                    "Failed to create validator for inner statement".to_string(),
                    ValidationErrorType::SemanticError,
                ))?
        ));

        Ok(())
    }

    /// 获取内部验证器
    pub fn inner_validator(&self) -> Option<&Validator> {
        self.inner_validator.as_deref()
    }

    /// 获取格式类型
    pub fn format(&self) -> &ExplainFormat {
        &self.format
    }

    pub fn validated_result(&self) -> ValidatedExplain {
        ValidatedExplain {
            format: self.format.clone(),
            inner_statement_type: self.inner_validator.as_ref()
                .map(|v| v.statement_type().as_str().to_string())
                .unwrap_or_default(),
        }
    }
}

/// 实现 StatementValidator trait
///
/// # 重构变更
/// - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext
/// - 内部语句验证直接调用 validate 方法，传入 stmt 和 qctx
impl StatementValidator for ProfileValidator {
    fn validate(
        &mut self,
        stmt: &crate::query::parser::ast::Stmt,
        qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        let profile_stmt = match stmt {
            crate::query::parser::ast::Stmt::Profile(profile_stmt) => profile_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected PROFILE statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        self.validate_impl(profile_stmt)?;

        // 验证内部语句
        if let Some(ref mut inner) = self.inner_validator {
            inner.validate(&profile_stmt.statement, qctx)?;
        }

        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::Profile
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        // PROFILE 是否为全局语句取决于内部语句
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

impl Default for ProfileValidator {
    fn default() -> Self {
        Self::new()
    }
}

//! Query 语句验证器
//! 用于验证顶层查询语句（QueryStmt）
//! Query 语句是一个包装器，包含实际的查询语句

use std::sync::Arc;
use crate::core::error::{ValidationError, ValidationErrorType};
use crate::query::QueryContext;
use crate::query::parser::ast::stmt::QueryStmt;
use crate::query::validator::validator_enum::Validator;
use crate::query::validator::validator_trait::{
    ColumnDef, ExpressionProps, StatementType, StatementValidator, ValidationResult,
};

/// Query 语句验证器
#[derive(Debug)]
pub struct QueryValidator {
    inner_validator: Option<Box<crate::query::validator::validator_enum::Validator>>,
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expr_props: ExpressionProps,
    user_defined_vars: Vec<String>,
}

impl QueryValidator {
    /// 创建新的 Query 验证器
    pub fn new() -> Self {
        Self {
            inner_validator: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    fn validate_impl(&mut self, stmt: &QueryStmt) -> Result<(), ValidationError> {
        if stmt.statements.is_empty() {
            return Err(ValidationError::new(
                "Query must contain at least one statement".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        let first_stmt = &stmt.statements[0];
        let validator = Validator::create_from_stmt(first_stmt)
            .ok_or_else(|| ValidationError::new(
                format!("Unsupported statement type in QUERY: {:?}", first_stmt.kind()),
                ValidationErrorType::SemanticError,
            ))?;

        self.inner_validator = Some(Box::new(validator));
        self.setup_outputs();

        Ok(())
    }

    fn setup_outputs(&mut self) {
        // Query 语句的输出与内部语句相同
        // 在验证后从内部验证器复制
        if let Some(ref inner) = self.inner_validator {
            self.outputs = inner.get_outputs().to_vec();
        }
    }
}

impl Default for QueryValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// 实现 StatementValidator trait
///
/// # 重构变更
/// - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext
impl StatementValidator for QueryValidator {
    fn validate(
        &mut self,
        stmt: &crate::query::parser::ast::Stmt,
        qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        let query_stmt = match stmt {
            crate::query::parser::ast::Stmt::Query(query_stmt) => query_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected QUERY statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        self.validate_impl(query_stmt)?;

        let inner = self.inner_validator.as_mut()
            .expect("inner_validator should be set after validate_impl");

        let first_stmt = query_stmt.statements.first()
            .expect("validate_impl already checked statements is not empty");

        let result = inner.validate(first_stmt, qctx.clone());

        if result.success {
            self.inputs = result.inputs;
            self.outputs = result.outputs;
            Ok(ValidationResult::success(
                self.inputs.clone(),
                self.outputs.clone(),
            ))
        } else {
            Err(result.errors.into_iter().next().unwrap_or_else(|| {
                ValidationError::new(
                    "Internal validation failed".to_string(),
                    ValidationErrorType::SemanticError,
                )
            }))
        }
    }

    fn statement_type(&self) -> StatementType {
        StatementType::Query
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        if let Some(ref inner) = self.inner_validator {
            inner.get_type().is_global_statement()
        } else {
            false
        }
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
    use crate::query::parser::ast::{QueryStmt, Stmt, Span};

    #[test]
    fn test_query_validator_new() {
        let validator = QueryValidator::new();
        assert_eq!(validator.statement_type(), StatementType::Query);
    }

    #[test]
    fn test_query_validator_with_match() {
        use crate::query::parser::ast::MatchStmt;

        let mut validator = QueryValidator::new();
        let query_stmt = QueryStmt {
            span: Span::default(),
            statements: vec![Stmt::Match(MatchStmt {
                span: Span::default(),
                patterns: vec![],
                where_clause: None,
                return_clause: None,
                order_by: None,
                limit: None,
                skip: None,
                optional: false,
            })],
        };

        // 验证实现应该成功创建内部验证器
        assert!(validator.validate_impl(&query_stmt).is_ok());
        assert!(validator.inner_validator.is_some());
    }
}

//! LIMIT 子句验证器
//! 对应 NebulaGraph LimitValidator.h/.cpp 的功能
//! 验证 LIMIT 和 SKIP 子句的表达式

use std::sync::Arc;
use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::Expression;
use crate::query::context::QueryContext;
use crate::query::validator::validator_trait::{
    ColumnDef, ExpressionProps, StatementType, StatementValidator, ValidationResult, ValueType,
};
use crate::storage::metadata::schema_manager::SchemaManager;

/// 验证后的 LIMIT 信息
#[derive(Debug, Clone)]
pub struct ValidatedLimit {
    pub space_id: u64,
    pub skip: Option<u64>,
    pub limit: Option<u64>,
    pub count: Option<u64>,
}

#[derive(Debug)]
pub struct LimitValidator {
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expression_props: ExpressionProps,
    user_defined_vars: Vec<String>,
    validated_result: Option<ValidatedLimit>,
    schema_manager: Option<Arc<dyn SchemaManager>>,
    skip_expr: Option<Expression>,
    limit_expr: Option<Expression>,
    count: Option<u64>,
}

impl LimitValidator {
    pub fn new() -> Self {
        Self {
            inputs: Vec::new(),
            outputs: Vec::new(),
            expression_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
            validated_result: None,
            schema_manager: None,
            skip_expr: None,
            limit_expr: None,
            count: None,
        }
    }

    pub fn with_schema_manager(mut self, schema_manager: Arc<dyn SchemaManager>) -> Self {
        self.schema_manager = Some(schema_manager);
        self
    }

    pub fn set_skip(mut self, skip: Expression) -> Self {
        self.skip_expr = Some(skip);
        self
    }

    pub fn set_limit(mut self, limit: Expression) -> Self {
        self.limit_expr = Some(limit);
        self
    }

    pub fn set_count(mut self, count: u64) -> Self {
        self.count = Some(count);
        self
    }

    /// 验证 SKIP 表达式
    fn validate_skip(&self, skip: &Option<Expression>) -> Result<Option<u64>, ValidationError> {
        if let Some(skip_expr) = skip {
            // 验证类型是否为整数
            if !self.is_integer_expression(skip_expr) {
                return Err(ValidationError::new(
                    "SKIP value must be integer type".to_string(),
                    ValidationErrorType::TypeError,
                ));
            }

            // 评估表达式
            let skip_val = self.evaluate_expression(skip_expr)?;
            if skip_val < 0 {
                return Err(ValidationError::new(
                    "SKIP value cannot be negative".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
            Ok(Some(skip_val as u64))
        } else {
            Ok(None)
        }
    }

    /// 验证 LIMIT 表达式
    fn validate_limit(&self, limit: &Option<Expression>) -> Result<Option<u64>, ValidationError> {
        if let Some(limit_expr) = limit {
            // 验证类型是否为整数
            if !self.is_integer_expression(limit_expr) {
                return Err(ValidationError::new(
                    "LIMIT value must be integer type".to_string(),
                    ValidationErrorType::TypeError,
                ));
            }

            // 评估表达式
            let limit_val = self.evaluate_expression(limit_expr)?;
            if limit_val < 0 {
                return Err(ValidationError::new(
                    "LIMIT value cannot be negative".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
            Ok(Some(limit_val as u64))
        } else {
            Ok(None)
        }
    }

    /// 验证范围
    fn validate_range(&self, skip: Option<u64>, limit: Option<u64>) -> Result<(), ValidationError> {
        let skip_val = skip.unwrap_or(0);
        let limit_val = limit.unwrap_or(0);

        if skip_val == 0 && limit_val == 0 {
            return Err(ValidationError::new(
                "At least one of SKIP or LIMIT must be greater than zero".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        Ok(())
    }

    /// 验证 count
    fn validate_count(&self, count: Option<u64>) -> Result<(), ValidationError> {
        if let Some(c) = count {
            if c > u64::MAX / 2 {
                return Err(ValidationError::new(
                    "LIMIT value is too large".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    /// 检查表达式是否为整数类型
    fn is_integer_expression(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Literal(val) => matches!(val, crate::core::Value::Int(_)),
            Expression::Variable(_) => true, // 变量在运行时检查
            _ => false,
        }
    }

    /// 评估表达式
    fn evaluate_expression(&self, expr: &Expression) -> Result<i64, ValidationError> {
        match expr {
            Expression::Literal(crate::core::Value::Int(n)) => Ok(*n),
            Expression::Variable(_) => Ok(0), // 变量在运行时解析
            _ => Err(ValidationError::new(
                "Cannot evaluate expression".to_string(),
                ValidationErrorType::SemanticError,
            )),
        }
    }

    /// 生成输出列
    fn generate_output_columns(&mut self) {
        self.outputs.clear();
        self.outputs.push(ColumnDef {
            name: "LIMIT_RESULT".to_string(),
            type_: ValueType::List,
        });
    }
}

impl Default for LimitValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// 实现 StatementValidator trait
///
/// # 重构变更
/// - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext
impl StatementValidator for LimitValidator {
    fn validate(
        &mut self,
        _stmt: &crate::query::parser::ast::Stmt,
        qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        // 1. 检查是否需要空间
        if !self.is_global_statement() && qctx.space_id().is_none() {
            return Err(ValidationError::new(
                "未选择图空间，请先执行 USE <space>".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 2. 获取 LIMIT 语句（如果存在）
        // QueryStmt 没有 skip/limit 字段，使用验证器预设的值
        let (skip_opt, limit_opt) = (self.skip_expr.clone(), self.limit_expr.clone());

        // 3. 验证 SKIP
        let skip_val = self.validate_skip(&skip_opt)?;

        // 4. 验证 LIMIT
        let limit_val = self.validate_limit(&limit_opt)?;

        // 5. 验证范围
        self.validate_range(skip_val, limit_val)?;

        // 6. 验证 count
        self.validate_count(self.count)?;

        // 7. 获取 space_id
        let space_id = qctx.space_id().unwrap_or(0);

        // 8. 创建验证结果
        let validated = ValidatedLimit {
            space_id,
            skip: skip_val,
            limit: limit_val,
            count: self.count,
        };

        self.validated_result = Some(validated);

        // 9. 生成输出列
        self.generate_output_columns();

        // 10. 返回验证结果
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::Limit
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        // LIMIT 不是全局语句，需要预先选择空间
        false
    }

    fn expression_props(&self) -> &ExpressionProps {
        &self.expression_props
    }

    fn user_defined_vars(&self) -> &[String] {
        &self.user_defined_vars
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::parser::ast::Stmt;

    #[test]
    fn test_limit_validator_basic() {
        let mut validator = LimitValidator::new()
            .set_limit(Expression::literal(10));

        let qctx = Arc::new(QueryContext::default());
        let use_stmt = crate::query::parser::ast::UseStmt {
            span: crate::core::types::Span::default(),
            space: "test".to_string(),
        };
        let result = validator.validate(&Stmt::Use(use_stmt), qctx);
        assert!(result.is_ok());

        let validated = validator.validated_result.unwrap();
        assert_eq!(validated.limit, Some(10));
    }

    #[test]
    fn test_limit_validator_with_skip() {
        let mut validator = LimitValidator::new()
            .set_skip(Expression::literal(5))
            .set_limit(Expression::literal(10));

        let qctx = Arc::new(QueryContext::default());
        let use_stmt = crate::query::parser::ast::UseStmt {
            span: crate::core::types::Span::default(),
            space: "test".to_string(),
        };
        let result = validator.validate(&Stmt::Use(use_stmt), qctx);
        assert!(result.is_ok());

        let validated = validator.validated_result.unwrap();
        assert_eq!(validated.skip, Some(5));
        assert_eq!(validated.limit, Some(10));
    }

    #[test]
    fn test_limit_validator_negative_skip() {
        let mut validator = LimitValidator::new()
            .set_skip(Expression::literal(-1));

        let qctx = Arc::new(QueryContext::default());
        let use_stmt = crate::query::parser::ast::UseStmt {
            span: crate::core::types::Span::default(),
            space: "test".to_string(),
        };
        let result = validator.validate(&Stmt::Use(use_stmt), qctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("cannot be negative"));
    }

    #[test]
    fn test_limit_validator_negative_limit() {
        let mut validator = LimitValidator::new()
            .set_limit(Expression::literal(-5));

        let qctx = Arc::new(QueryContext::default());
        let use_stmt = crate::query::parser::ast::UseStmt {
            span: crate::core::types::Span::default(),
            space: "test".to_string(),
        };
        let result = validator.validate(&Stmt::Use(use_stmt), qctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("cannot be negative"));
    }

    #[test]
    fn test_limit_validator_zero_skip_and_limit() {
        let mut validator = LimitValidator::new()
            .set_skip(Expression::literal(0))
            .set_limit(Expression::literal(0));

        let qctx = Arc::new(QueryContext::default());
        let use_stmt = crate::query::parser::ast::UseStmt {
            span: crate::core::types::Span::default(),
            space: "test".to_string(),
        };
        let result = validator.validate(&Stmt::Use(use_stmt), qctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("greater than zero"));
    }

    #[test]
    fn test_limit_validator_non_integer() {
        let mut validator = LimitValidator::new()
            .set_limit(Expression::literal("invalid"));

        let qctx = Arc::new(QueryContext::default());
        let use_stmt = crate::query::parser::ast::UseStmt {
            span: crate::core::types::Span::default(),
            space: "test".to_string(),
        };
        let result = validator.validate(&Stmt::Use(use_stmt), qctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("must be integer"));
    }

    #[test]
    fn test_limit_validator_trait_interface() {
        let validator = LimitValidator::new();

        assert_eq!(validator.statement_type(), StatementType::Limit);
        assert!(validator.inputs().is_empty());
        assert!(validator.user_defined_vars().is_empty());
    }

    #[test]
    fn test_limit_validator_count() {
        let mut validator = LimitValidator::new()
            .set_limit(Expression::literal(10))
            .set_count(100);

        let qctx = Arc::new(QueryContext::default());
        let use_stmt = crate::query::parser::ast::UseStmt {
            span: crate::core::types::Span::default(),
            space: "test".to_string(),
        };
        let result = validator.validate(&Stmt::Use(use_stmt), qctx);
        assert!(result.is_ok());

        let validated = validator.validated_result.unwrap();
        assert_eq!(validated.count, Some(100));
    }
}

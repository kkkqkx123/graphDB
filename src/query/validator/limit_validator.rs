//! LIMIT 子句验证器
//! 对应 NebulaGraph LimitValidator.h/.cpp 的功能
//! 验证 LIMIT 和 SKIP 子句的表达式

use super::base_validator::{Validator, ValueType};
use super::ValidationContext;
use crate::core::Expression;
use crate::query::validator::ValidationError;
use crate::query::validator::ValidationErrorType;

pub struct LimitValidator {
    base: Validator,
    skip: Option<Expression>,
    limit: Option<Expression>,
    count: Option<u64>,
}

impl LimitValidator {
    pub fn new(context: ValidationContext) -> Self {
        Self {
            base: Validator::with_context(context),
            skip: None,
            limit: None,
            count: None,
        }
    }

    pub fn validate(&mut self) -> Result<(), ValidationError> {
        self.validate_impl()
    }

    fn validate_impl(&mut self) -> Result<(), ValidationError> {
        self.validate_skip()?;
        self.validate_limit()?;
        self.validate_range()?;
        self.validate_count()?;
        Ok(())
    }

    fn validate_skip(&mut self) -> Result<(), ValidationError> {
        if let Some(skip_expression) = &self.skip {
            let skip_type = self.deduce_expr_type(skip_expression)?;
            if skip_type != ValueType::Int {
                return Err(ValidationError::new(
                    format!(
                        "SKIP value must be integer type, got {:?}",
                        skip_type
                    ),
                    ValidationErrorType::TypeError,
                ));
            }

            if !self.is_constant_or_parameter(skip_expression) {
                if let Some(input_cols) = self.base.inputs().first() {
                    if input_cols.type_ != ValueType::Int {
                        return Err(ValidationError::new(
                            "SKIP value must be integer type".to_string(),
                            ValidationErrorType::TypeError,
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_limit(&mut self) -> Result<(), ValidationError> {
        if let Some(limit_expression) = &self.limit {
            let limit_type = self.deduce_expr_type(limit_expression)?;
            if limit_type != ValueType::Int {
                return Err(ValidationError::new(
                    format!(
                        "LIMIT value must be integer type, got {:?}",
                        limit_type
                    ),
                    ValidationErrorType::TypeError,
                ));
            }

            if !self.is_constant_or_parameter(limit_expression) {
                if let Some(input_cols) = self.base.inputs().first() {
                    if input_cols.type_ != ValueType::Int {
                        return Err(ValidationError::new(
                            "LIMIT value must be integer type".to_string(),
                            ValidationErrorType::TypeError,
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_range(&self) -> Result<(), ValidationError> {
        let skip_value = self.evaluate_skip()?;
        let limit_value = self.evaluate_limit()?;

        if skip_value < 0 {
            return Err(ValidationError::new(
                "SKIP value cannot be negative".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        if limit_value < 0 {
            return Err(ValidationError::new(
                "LIMIT value cannot be negative".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        if skip_value == 0 && limit_value == 0 {
            return Err(ValidationError::new(
                "At least one of SKIP or LIMIT must be greater than zero".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        Ok(())
    }

    fn validate_count(&self) -> Result<(), ValidationError> {
        if let Some(count) = self.count {
            if count > u64::MAX / 2 {
                return Err(ValidationError::new(
                    "LIMIT value is too large".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    fn deduce_expr_type(&self, _expression: &Expression) -> Result<ValueType, ValidationError> {
        Ok(ValueType::Int)
    }

    fn is_constant_or_parameter(&self, _expression: &Expression) -> bool {
        true
    }

    fn evaluate_skip(&self) -> Result<i64, ValidationError> {
        Ok(0)
    }

    fn evaluate_limit(&self) -> Result<i64, ValidationError> {
        Ok(10)
    }

    pub fn set_skip(&mut self, skip: Expression) {
        self.skip = Some(skip);
    }

    pub fn set_limit(&mut self, limit: Expression) {
        self.limit = Some(limit);
    }

    pub fn set_count(&mut self, count: u64) {
        self.count = Some(count);
    }

    pub fn skip(&self) -> Option<&Expression> {
        self.skip.as_ref()
    }

    pub fn limit(&self) -> Option<&Expression> {
        self.limit.as_ref()
    }
}

impl Validator {
    pub fn validate_limit_clause(
        &mut self,
        skip: Option<&Expression>,
        limit: Option<&Expression>,
    ) -> Result<(), ValidationError> {
        let mut validator = LimitValidator::new(self.context().clone());
        if let Some(skip_expression) = skip {
            validator.set_skip(skip_expression.clone());
        }
        if let Some(limit_expression) = limit {
            validator.set_limit(limit_expression.clone());
        }
        validator.validate()
    }
}

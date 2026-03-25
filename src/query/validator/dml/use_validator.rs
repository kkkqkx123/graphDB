//! USE Statement Validator – New System Version
//! 对应 NebulaGraph UseValidator.h/.cpp 的功能
//! Verify the USE <space> statement
//!
//! This document has been restructured in accordance with the new trait + enumeration validator framework.
//! The StatementValidator trait has been implemented to unify the interface.
//! 2. All original functions have been retained.
//! Space name validation (must not be empty, must not start with a digit, there are length restrictions, etc.)
//! Special character check
//! 3. Use QueryContext to manage the context in a unified manner.

use std::sync::Arc;

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::query::parser::ast::stmt::Ast;
use crate::query::parser::ast::Stmt;
use crate::query::validator::structs::validation_info::ValidationInfo;
use crate::query::validator::validator_trait::{
    ColumnDef, ExpressionProps, StatementType, StatementValidator, ValidationResult,
};
use crate::query::QueryContext;

/// Verified USE information
#[derive(Debug, Clone)]
pub struct ValidatedUse {
    pub space_name: String,
}

/// USE Validator – New implementation of the system
///
/// Functionality integrity assurance:
/// 1. Complete validation lifecycle
/// 2. Management of input/output columns
/// 3. Expression property tracing
/// 4. Support for global statements (no need to pre-select a scope).
#[derive(Debug)]
pub struct UseValidator {
    // Space name
    space_name: String,
    // Input column definition
    inputs: Vec<ColumnDef>,
    // Column definition
    outputs: Vec<ColumnDef>,
    // Expression property
    expr_props: ExpressionProps,
    // User-defined variables
    user_defined_vars: Vec<String>,
    // List of validation errors
    validation_errors: Vec<ValidationError>,
    // Cache validation results
    validated_result: Option<ValidatedUse>,
}

impl UseValidator {
    /// Create a new instance of the validator.
    pub fn new() -> Self {
        Self {
            space_name: String::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
            validation_errors: Vec::new(),
            validated_result: None,
        }
    }

    /// Obtain the verification results.
    pub fn validated_result(&self) -> Option<&ValidatedUse> {
        self.validated_result.as_ref()
    }

    /// Obtain the list of validation errors.
    pub fn validation_errors(&self) -> &[ValidationError] {
        &self.validation_errors
    }

    /// Add verification errors.
    fn add_error(&mut self, error: ValidationError) {
        self.validation_errors.push(error);
    }

    /// Clear the verification errors.
    fn clear_errors(&mut self) {
        self.validation_errors.clear();
    }

    /// Check for any validation errors.
    fn has_errors(&self) -> bool {
        !self.validation_errors.is_empty()
    }

    /// Setting the space name
    pub fn set_space_name(&mut self, name: String) {
        self.space_name = name;
    }

    /// Obtain the space name
    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    /// Verify the USE statement (traditional method, maintaining backward compatibility)
    pub fn validate_use(&mut self) -> Result<ValidatedUse, ValidationError> {
        self.validate_space_name()?;
        self.validate_space_exists()?;

        let result = ValidatedUse {
            space_name: self.space_name.clone(),
        };

        self.validated_result = Some(result.clone());
        Ok(result)
    }

    /// Verify the space name
    fn validate_space_name(&self) -> Result<(), ValidationError> {
        if self.space_name.is_empty() {
            return Err(ValidationError::new(
                "USE 语句需要指定空间名".to_string(),
                ValidationErrorType::SyntaxError,
            ));
        }

        if self.space_name.starts_with('_') {
            return Err(ValidationError::new(
                format!("空间名 '{}' 不能以下划线开头", self.space_name),
                ValidationErrorType::SemanticError,
            ));
        }

        if self
            .space_name
            .chars()
            .next()
            .unwrap_or_default()
            .is_ascii_digit()
        {
            return Err(ValidationError::new(
                format!("空间名 '{}' 不能以数字开头", self.space_name),
                ValidationErrorType::SemanticError,
            ));
        }

        let invalid_chars: Vec<char> = vec![' ', '\t', '\n', '\r', ',', ';', '(', ')', '[', ']'];
        for c in self.space_name.chars() {
            if invalid_chars.contains(&c) {
                return Err(ValidationError::new(
                    format!("空间名 '{}' 包含非法字符 '{}'", self.space_name, c),
                    ValidationErrorType::SemanticError,
                ));
            }
        }

        if self.space_name.len() > 64 {
            return Err(ValidationError::new(
                format!("空间名 '{}' 超过最大长度 64 个字符", self.space_name),
                ValidationErrorType::SemanticError,
            ));
        }

        Ok(())
    }

    /// Verify whether the space exists.
    fn validate_space_exists(&self) -> Result<(), ValidationError> {
        // In the actual implementation, the SchemaManager should be checked here.
        // However, due to the special nature of the USE statement (which is used for selecting a space),
        // It is possible that we were not yet connected to the specific space at the time of verification.
        // So, for now, we’ll just say “Ok”. The actual verification will take place during the execution phase.
        Ok(())
    }
}

impl Default for UseValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementing the StatementValidator trait
///
/// # Refactoring Changes
/// The `validate` method accepts `Arc<Ast>` and `Arc<QueryContext>` as arguments.
impl StatementValidator for UseValidator {
    fn validate(
        &mut self,
        ast: Arc<Ast>,
        _qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        // Clear the previous state.
        self.outputs.clear();
        self.inputs.clear();
        self.expr_props = ExpressionProps::default();
        self.clear_errors();

        // Extract USE statement information from Ast.
        if let Stmt::Use(use_stmt) = &ast.stmt {
            self.space_name = use_stmt.space.clone();
        } else {
            return Err(ValidationError::new(
                "期望 USE 语句".to_string(),
                crate::core::error::ValidationErrorType::SemanticError,
            ));
        }

        // Perform the specific validation logic.
        if let Err(e) = self.validate_use() {
            self.add_error(e);
        }

        // If there are validation errors, return a failure result.
        if self.has_errors() {
            let errors = self.validation_errors.clone();
            return Ok(ValidationResult::failure(errors));
        }

        let info = ValidationInfo::new();

        // Return the successful validation results.
        Ok(ValidationResult::success_with_info(info))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::Use
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        // The `USE` statement is a global statement; therefore, there is no need to pre-select a database or schema.
        true
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

    #[test]
    fn test_use_validator_new() {
        let validator = UseValidator::new();
        assert!(validator.inputs().is_empty());
        assert!(validator.outputs().is_empty());
        assert!(validator.validated_result().is_none());
        assert!(validator.validation_errors().is_empty());
    }

    #[test]
    fn test_use_validator_default() {
        let validator: UseValidator = Default::default();
        assert!(validator.inputs().is_empty());
        assert!(validator.outputs().is_empty());
    }

    #[test]
    fn test_statement_type() {
        let validator = UseValidator::new();
        assert_eq!(validator.statement_type(), StatementType::Use);
    }

    #[test]
    fn test_use_validation() {
        let mut validator = UseValidator::new();

        // Set an effective space name.
        validator.set_space_name("test_space".to_string());

        let result = validator.validate_use();
        assert!(result.is_ok());

        let validated = result.expect("Failed to validate use");
        assert_eq!(validated.space_name, "test_space");
    }

    #[test]
    fn test_use_empty_space_name() {
        let mut validator = UseValidator::new();

        // Do not set the space name.
        let result = validator.validate_use();
        assert!(result.is_err());
    }

    #[test]
    fn test_use_invalid_space_name_start_with_digit() {
        let mut validator = UseValidator::new();

        // Space names that start with a number
        validator.set_space_name("1space".to_string());
        let result = validator.validate_use();
        assert!(result.is_err());
    }

    #[test]
    fn test_use_invalid_space_name_start_with_underscore() {
        let mut validator = UseValidator::new();

        // The space names that start with an underscore (_).
        validator.set_space_name("_space".to_string());
        let result = validator.validate_use();
        assert!(result.is_err());
    }

    #[test]
    fn test_use_invalid_space_name_with_space() {
        let mut validator = UseValidator::new();

        // Domain names that contain spaces
        validator.set_space_name("test space".to_string());
        let result = validator.validate_use();
        assert!(result.is_err());
    }

    #[test]
    fn test_use_invalid_space_name_too_long() {
        let mut validator = UseValidator::new();

        // Domain names that contain more than 64 characters
        let long_name = "a".repeat(65);
        validator.set_space_name(long_name);
        let result = validator.validate_use();
        assert!(result.is_err());
    }

    #[test]
    fn test_is_global_statement() {
        let validator = UseValidator::new();
        assert!(validator.is_global_statement());
    }
}

//! USE 语句验证器
//! 对应 NebulaGraph UseValidator.h/.cpp 的功能
//! 验证 USE <space> 语句

use super::base_validator::Validator;
use super::ValidationContext;
use crate::query::validator::ValidationError;
use crate::query::validator::ValidationErrorType;

pub struct UseValidator {
    base: Validator,
    space_name: String,
}

impl UseValidator {
    pub fn new(context: ValidationContext) -> Self {
        Self {
            base: Validator::with_context(context),
            space_name: String::new(),
        }
    }

    pub fn validate(&mut self) -> Result<(), ValidationError> {
        self.validate_impl()
    }

    fn validate_impl(&mut self) -> Result<(), ValidationError> {
        self.validate_space_name()?;
        self.validate_space_exists()?;
        self.set_no_space_required(true);
        Ok(())
    }

    fn validate_space_name(&self) -> Result<(), ValidationError> {
        if self.space_name.is_empty() {
            return Err(ValidationError::new(
                "USE statement requires a space name".to_string(),
                ValidationErrorType::SyntaxError,
            ));
        }

        if self.space_name.starts_with('_') {
            return Err(ValidationError::new(
                format!(
                    "Space name '{}' cannot start with underscore",
                    self.space_name
                ),
                ValidationErrorType::SemanticError,
            ));
        }

        if self.space_name.chars().next().unwrap_or_default().is_ascii_digit() {
            return Err(ValidationError::new(
                format!(
                    "Space name '{}' cannot start with a digit",
                    self.space_name
                ),
                ValidationErrorType::SemanticError,
            ));
        }

        let invalid_chars: Vec<char> = vec![' ', '\t', '\n', '\r', ',', ';', '(', ')', '[', ']'];
        for c in self.space_name.chars() {
            if invalid_chars.contains(&c) {
                return Err(ValidationError::new(
                    format!(
                        "Space name '{}' contains invalid character '{}'",
                        self.space_name, c
                    ),
                    ValidationErrorType::SemanticError,
                ));
            }
        }

        if self.space_name.len() > 64 {
            return Err(ValidationError::new(
                format!(
                    "Space name '{}' exceeds maximum length of 64 characters",
                    self.space_name
                ),
                ValidationErrorType::SemanticError,
            ));
        }

        Ok(())
    }

    fn validate_space_exists(&self) -> Result<(), ValidationError> {
        let schema_manager = self.base.context().get_schema_manager();
        if schema_manager.is_none() {
            return Ok(());
        }

        Ok(())
    }

    fn set_no_space_required(&mut self, required: bool) {
        self.base.set_no_space_required(required);
    }

    pub fn set_space_name(&mut self, name: String) {
        self.space_name = name;
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }
}

impl Validator {
    pub fn validate_use_space(&mut self, space_name: String) -> Result<(), ValidationError> {
        let mut validator = UseValidator::new(self.context().clone());
        validator.set_space_name(space_name);
        validator.validate()
    }
}

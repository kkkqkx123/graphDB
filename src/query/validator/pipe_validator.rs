//! 管道操作验证器
//! 对应 NebulaGraph PipeValidator.h/.cpp 的功能
//! 验证管道操作符 `|` 连接的前后查询兼容性

use super::base_validator::{Validator, ValueType};
use super::ValidationContext;
use crate::core::Expression;
use crate::query::validator::ValidationError;
use crate::query::validator::ValidationErrorType;

pub struct PipeValidator {
    base: Validator,
    left_output_cols: Vec<ColumnInfo>,
    right_input_cols: Vec<ColumnInfo>,
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub type_: ValueType,
    pub alias: Option<String>,
}

impl PipeValidator {
    pub fn new(context: ValidationContext) -> Self {
        Self {
            base: Validator::new(context),
            left_output_cols: Vec::new(),
            right_input_cols: Vec::new(),
        }
    }

    pub fn validate(&mut self) -> Result<(), ValidationError> {
        self.validate_impl()
    }

    fn validate_impl(&mut self) -> Result<(), ValidationError> {
        self.validate_left_output()?;
        self.validate_right_input()?;
        self.validate_compatibility()?;
        self.validate_pipe_connection()?;
        Ok(())
    }

    fn validate_left_output(&mut self) -> Result<(), ValidationError> {
        for col in &self.left_output_cols {
            if col.name.is_empty() {
                return Err(ValidationError::new(
                    "Pipe left side has empty column name".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    fn validate_right_input(&mut self) -> Result<(), ValidationError> {
        for col in &self.right_input_cols {
            if col.name.is_empty() {
                return Err(ValidationError::new(
                    "Pipe right side has empty column reference".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    fn validate_compatibility(&self) -> Result<(), ValidationError> {
        if self.left_output_cols.is_empty() && !self.right_input_cols.is_empty() {
            return Err(ValidationError::new(
                "Pipe left side has no output columns but right side requires input".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        for right_col in &self.right_input_cols {
            let mut found = false;
            for left_col in &self.left_output_cols {
                if right_col.name == left_col.name {
                    if right_col.type_ != left_col.type_ && left_col.type_ != ValueType::Unknown {
                        return Err(ValidationError::new(
                            format!(
                                "Column type mismatch for '{}': left output is {:?}, right input requires {:?}",
                                right_col.name, left_col.type_, right_col.type_
                            ),
                            ValidationErrorType::TypeError,
                        ));
                    }
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(ValidationError::new(
                    format!(
                        "Column '{}' referenced in pipe right side not found in left output",
                        right_col.name
                    ),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    fn validate_pipe_connection(&self) -> Result<(), ValidationError> {
        if self.left_output_cols.is_empty() && self.right_input_cols.is_empty() {
            return Ok(());
        }

        if !self.right_input_cols.is_empty() && self.left_output_cols.is_empty() {
            return Err(ValidationError::new(
                "Pipe requires input from previous query but previous query has no output".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    pub fn set_left_output(&mut self, cols: Vec<ColumnInfo>) {
        self.left_output_cols = cols;
    }

    pub fn set_right_input(&mut self, cols: Vec<ColumnInfo>) {
        self.right_input_cols = cols;
    }

    pub fn add_left_output(&mut self, col: ColumnInfo) {
        self.left_output_cols.push(col);
    }

    pub fn add_right_input(&mut self, col: ColumnInfo) {
        self.right_input_cols.push(col);
    }
}

impl Validator {
    pub fn validate_pipe(
        &mut self,
        left_outputs: &[ColumnInfo],
        right_inputs: &[ColumnInfo],
    ) -> Result<(), ValidationError> {
        let mut pipe_validator = PipeValidator::new(self.context().clone());
        pipe_validator.set_left_output(left_outputs.to_vec());
        pipe_validator.set_right_input(right_inputs.to_vec());
        pipe_validator.validate()
    }
}

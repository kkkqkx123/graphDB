//! YIELD 子句验证器
//! 对应 NebulaGraph YieldValidator.h/.cpp 的功能
//! 验证 YIELD 子句的表达式和列定义

use super::base_validator::{Validator, ValueType};
use super::ValidationContext;
use crate::query::validator::ValidationError;
use crate::query::validator::ValidationErrorType;
use crate::query::validator::structs::YieldColumn;
use std::collections::HashMap;

pub struct YieldValidator {
    base: Validator,
    yield_columns: Vec<YieldColumn>,
    distinct: bool,
    aliases_available: HashMap<String, ValueType>,
}

impl YieldValidator {
    pub fn new(context: ValidationContext) -> Self {
        Self {
            base: Validator::with_context(context),
            yield_columns: Vec::new(),
            distinct: false,
            aliases_available: HashMap::new(),
        }
    }

    pub fn validate(&mut self) -> Result<(), ValidationError> {
        self.validate_impl()
    }

    fn validate_impl(&mut self) -> Result<(), ValidationError> {
        self.validate_columns()?;
        self.validate_aliases()?;
        self.validate_types()?;
        self.validate_distinct()?;
        Ok(())
    }

    fn validate_columns(&mut self) -> Result<(), ValidationError> {
        if self.yield_columns.is_empty() {
            return Err(ValidationError::new(
                "YIELD clause must have at least one column".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        let mut seen_names: HashMap<String, usize> = HashMap::new();
        for col in &self.yield_columns {
            let name = col.name().to_string();
            if name.is_empty() {
                return Err(ValidationError::new(
                    "YIELD column must have a name or alias".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }

            let count = seen_names.entry(name.clone()).or_insert(0);
            *count += 1;

            if *count > 1 {
                return Err(ValidationError::new(
                    format!("Duplicate column name '{}' in YIELD clause", name),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    fn validate_aliases(&self) -> Result<(), ValidationError> {
        for col in &self.yield_columns {
            let alias = col.name();
            if !alias.starts_with('_') && alias.chars().next().unwrap_or_default().is_ascii_digit() {
                return Err(ValidationError::new(
                    format!("Alias '{}' cannot start with a digit", alias),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    fn validate_types(&mut self) -> Result<(), ValidationError> {
        for col in &self.yield_columns {
            let expr_type = self.deduce_expr_type(&col.expression)?;
            if expr_type == ValueType::Unknown {
                self.base.add_type_error(format!(
                    "Cannot deduce type for expression in YIELD column '{}'",
                    col.name()
                ));
            }
        }
        Ok(())
    }

    fn validate_distinct(&self) -> Result<(), ValidationError> {
        if self.distinct && self.yield_columns.len() > 1 {
            let has_non_comparable = self.yield_columns.iter().any(|col| {
                let col_type = self.deduce_expr_type(&col.expression).unwrap_or(ValueType::Unknown);
                !matches!(col_type, ValueType::Bool | ValueType::Int | ValueType::Float | ValueType::String)
            });
            if has_non_comparable {
                return Err(ValidationError::new(
                    "DISTINCT on YIELD with non-comparable types is not supported".to_string(),
                    ValidationErrorType::TypeError,
                ));
            }
        }
        Ok(())
    }

    fn deduce_expr_type(&self, _expression: &crate::core::Expression) -> Result<ValueType, ValidationError> {
        Ok(ValueType::Unknown)
    }

    pub fn add_yield_column(&mut self, col: YieldColumn) {
        self.yield_columns.push(col);
    }

    pub fn set_distinct(&mut self, distinct: bool) {
        self.distinct = distinct;
    }

    pub fn set_aliases_available(&mut self, aliases: HashMap<String, ValueType>) {
        self.aliases_available = aliases;
    }

    pub fn yield_columns(&self) -> &[YieldColumn] {
        &self.yield_columns
    }

    pub fn is_distinct(&self) -> bool {
        self.distinct
    }
}

impl Validator {
    pub fn validate_yield_columns(
        &mut self,
        columns: &[YieldColumn],
        distinct: bool,
    ) -> Result<(), ValidationError> {
        let mut validator = YieldValidator::new(self.context().clone());
        for col in columns {
            validator.add_yield_column(col.clone());
        }
        validator.set_distinct(distinct);
        validator.validate()
    }
}

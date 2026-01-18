//! ORDER BY 子句验证器
//! 对应 NebulaGraph OrderByValidator.h/.cpp 的功能
//! 验证 ORDER BY 子句的排序表达式和方向

use super::base_validator::{Validator, ValueType};
use super::ValidationContext;
use crate::core::Expression;
use crate::query::validator::ValidationError;
use crate::query::validator::ValidationErrorType;
use std::collections::HashMap;

pub struct OrderByValidator {
    base: Validator,
    order_columns: Vec<OrderColumn>,
    input_columns: HashMap<String, ValueType>,
}

#[derive(Debug, Clone)]
pub struct OrderColumn {
    pub expression: Expression,
    pub alias: Option<String>,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
    Default,
}

impl OrderByValidator {
    pub fn new(context: ValidationContext) -> Self {
        Self {
            base: Validator::new(context),
            order_columns: Vec::new(),
            input_columns: HashMap::new(),
        }
    }

    pub fn validate(&mut self) -> Result<(), ValidationError> {
        self.validate_impl()
    }

    fn validate_impl(&mut self) -> Result<(), ValidationError> {
        self.validate_columns()?;
        self.validate_types()?;
        self.validate_input_compatibility()?;
        Ok(())
    }

    fn validate_columns(&mut self) -> Result<(), ValidationError> {
        if self.order_columns.is_empty() {
            return Err(ValidationError::new(
                "ORDER BY clause must have at least one column".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        for col in &self.order_columns {
            if self.expression_is_empty(&col.expression) {
                return Err(ValidationError::new(
                    "ORDER BY expression cannot be empty".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    fn validate_types(&self) -> Result<(), ValidationError> {
        for col in &self.order_columns {
            let expr_type = self.deduce_expr_type(&col.expression)?;
            if !self.is_comparable_type(&expr_type) {
                return Err(ValidationError::new(
                    format!(
                        "ORDER BY expression type {:?} is not comparable",
                        expr_type
                    ),
                    ValidationErrorType::TypeError,
                ));
            }
        }
        Ok(())
    }

    fn validate_input_compatibility(&self) -> Result<(), ValidationError> {
        for col in &self.order_columns {
            if let Some(alias) = &col.alias {
                if !self.input_columns.contains_key(alias) {
                    return Err(ValidationError::new(
                        format!(
                            "ORDER BY alias '{}' not found in input columns",
                            alias
                        ),
                        ValidationErrorType::SemanticError,
                    ));
                }
            } else {
                let refs = self.get_expression_references(&col.expression);
                for ref_name in refs {
                    if !self.input_columns.contains_key(&ref_name) && ref_name != "$" {
                        return Err(ValidationError::new(
                            format!(
                                "ORDER BY expression references unknown column '{}'",
                                ref_name
                            ),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn expression_is_empty(&self, expr: &Expression) -> bool {
        false
    }

    fn deduce_expr_type(&self, _expr: &Expression) -> Result<ValueType, ValidationError> {
        Ok(ValueType::Unknown)
    }

    fn is_comparable_type(&self, type_: &ValueType) -> bool {
        matches!(
            type_,
            ValueType::Bool | ValueType::Int | ValueType::Float | 
            ValueType::String | ValueType::Date | ValueType::Time | 
            ValueType::DateTime | ValueType::Null
        )
    }

    fn get_expression_references(&self, _expr: &Expression) -> Vec<String> {
        Vec::new()
    }

    pub fn add_order_column(&mut self, col: OrderColumn) {
        self.order_columns.push(col);
    }

    pub fn set_input_columns(&mut self, columns: HashMap<String, ValueType>) {
        self.input_columns = columns;
    }

    pub fn order_columns(&self) -> &[OrderColumn] {
        &self.order_columns
    }
}

impl Validator {
    pub fn validate_order_by(
        &mut self,
        columns: &[OrderColumn],
        input_columns: &HashMap<String, ValueType>,
    ) -> Result<(), ValidationError> {
        let mut validator = OrderByValidator::new(self.context().clone());
        for col in columns {
            validator.add_order_column(col.clone());
        }
        validator.set_input_columns(input_columns.clone());
        validator.validate()
    }
}

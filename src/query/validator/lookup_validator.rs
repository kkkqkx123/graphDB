//! LOOKUP 语句验证器
//! 对应 NebulaGraph LookupValidator.h/.cpp 的功能
//! 验证 LOOKUP 语句的合法性

use super::base_validator::{Validator, ValueType};
use super::ValidationContext;
use super::strategies::type_inference::TypeValidator;
use crate::core::Expression;
use crate::query::validator::ValidationError;
use crate::query::validator::ValidationErrorType;
use crate::query::validator::structs::{LookupTarget, LookupIndexType};
use std::collections::HashMap;

pub struct LookupValidator {
    base: Validator,
    lookup_target: LookupTarget,
    filter_expression: Option<Expression>,
    yield_columns: Vec<super::structs::YieldColumn>,
    is_yield_all: bool,
}

impl LookupValidator {
    pub fn new(context: ValidationContext) -> Self {
        Self {
            base: Validator::new(context),
            lookup_target: LookupTarget {
                label: String::new(),
                index_type: LookupIndexType::None,
                properties: HashMap::new(),
            },
            filter_expression: None,
            yield_columns: Vec::new(),
            is_yield_all: false,
        }
    }

    pub fn validate(&mut self) -> Result<(), ValidationError> {
        self.validate_impl()
    }

    fn validate_impl(&mut self) -> Result<(), ValidationError> {
        self.validate_lookup_target()?;
        self.validate_filter()?;
        self.validate_yields()?;
        Ok(())
    }

    fn validate_lookup_target(&mut self) -> Result<(), ValidationError> {
        if self.lookup_target.label.is_empty() {
            return Err(ValidationError::new(
                "LOOKUP must specify a label".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        match &self.lookup_target.index_type {
            LookupIndexType::None => {
                return Err(ValidationError::new(
                    format!("No index found for label '{}'", self.lookup_target.label),
                    ValidationErrorType::SemanticError,
                ));
            }
            LookupIndexType::Single(prop_name) => {
                if self.lookup_target.properties.get(prop_name).is_none() {
                    return Err(ValidationError::new(
                        format!("Index on property '{}' does not exist", prop_name),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            LookupIndexType::Composite(prop_names) => {
                for prop_name in prop_names {
                    if self.lookup_target.properties.get(prop_name).is_none() {
                        return Err(ValidationError::new(
                            format!("Index on property '{}' does not exist", prop_name),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_filter(&mut self) -> Result<(), ValidationError> {
        if let Some(filter) = &self.filter_expression {
            self.validate_filter_type(filter)?;

            if self.has_aggregate_expr(filter) {
                return Err(ValidationError::new(
                    "LOOKUP filter cannot contain aggregate expressions".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    fn validate_filter_type(&self, filter: &Expression) -> Result<(), ValidationError> {
        let filter_type = self.base.deduce_expr_type(filter);
        match filter_type {
            ValueType::Bool => Ok(()),
            ValueType::Null | ValueType::Unknown => Ok(()),
            _ => Err(ValidationError::new(
                format!("Filter expression must return bool type, actual returns {:?}", filter_type),
                ValidationErrorType::TypeError,
            )),
        }
    }

    fn has_aggregate_expr(&self, expr: &Expression) -> bool {
        let type_validator = TypeValidator::new();
        type_validator.has_aggregate_expression(expr)
    }

    fn validate_yields(&self) -> Result<(), ValidationError> {
        if self.is_yield_all {
            return Ok(());
        }

        if self.yield_columns.is_empty() {
            return Err(ValidationError::new(
                "LOOKUP must have YIELD clause or YIELD *".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        let mut seen_names: HashMap<String, usize> = HashMap::new();
        for col in &self.yield_columns {
            let name = col.name();
            let count = seen_names.entry(name.to_string()).or_insert(0);
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

    pub fn set_lookup_target(&mut self, target: LookupTarget) {
        self.lookup_target = target;
    }

    pub fn set_filter(&mut self, filter: Expression) {
        self.filter_expression = Some(filter);
    }

    pub fn set_yield_all(&mut self) {
        self.is_yield_all = true;
    }

    pub fn add_yield_column(&mut self, col: super::structs::YieldColumn) {
        self.yield_columns.push(col);
    }
}

impl Validator {
    pub fn validate_lookup(
        &mut self,
        target: LookupTarget,
        filter: Option<Expression>,
        yield_columns: &[super::structs::YieldColumn],
        yield_all: bool,
    ) -> Result<(), ValidationError> {
        let mut validator = LookupValidator::new(self.context().clone());
        validator.set_lookup_target(target);
        if let Some(filter_expr) = filter {
            validator.set_filter(filter_expr);
        }
        for col in yield_columns {
            validator.add_yield_column(col.clone());
        }
        if yield_all {
            validator.set_yield_all();
        }
        validator.validate()
    }
}

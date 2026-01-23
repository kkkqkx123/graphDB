//! UNWIND 子句验证器
//! 对应 NebulaGraph UnwindValidator.h/.cpp 的功能
//! 验证 UNWIND <expression> AS <variable> 语句

use super::base_validator::{Validator, ValueType};
use super::ValidationContext;
use crate::core::{Expr, Value, NullType};
use crate::query::validator::ValidationError;
use crate::query::validator::ValidationErrorType;
use std::collections::HashMap;

pub struct UnwindValidator {
    base: Validator,
    unwind_expression: Expr,
    variable_name: String,
    aliases_available: HashMap<String, ValueType>,
}

impl UnwindValidator {
    pub fn new(context: ValidationContext) -> Self {
        Self {
            base: Validator::new(context),
            unwind_expression: Expr::Literal(Value::Null(NullType::Null)),
            variable_name: String::new(),
            aliases_available: HashMap::new(),
        }
    }

    pub fn validate(&mut self) -> Result<(), ValidationError> {
        self.validate_impl()
    }

    fn validate_impl(&mut self) -> Result<(), ValidationError> {
        self.validate_expression()?;
        self.validate_variable()?;
        self.validate_type()?;
        self.validate_aliases()?;
        Ok(())
    }

    fn validate_expression(&self) -> Result<(), ValidationError> {
        if self.expression_is_empty(&self.unwind_expression) {
            return Err(ValidationError::new(
                "UNWIND expression cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        let expr_type = self.deduce_expr_type(&self.unwind_expression)?;
        if expr_type != ValueType::List && expr_type != ValueType::Set {
            return Err(ValidationError::new(
                format!(
                    "UNWIND expression must evaluate to a list or set, got {:?}",
                    expr_type
                ),
                ValidationErrorType::TypeError,
            ));
        }

        Ok(())
    }

    fn validate_variable(&self) -> Result<(), ValidationError> {
        if self.variable_name.is_empty() {
            return Err(ValidationError::new(
                "UNWIND requires an AS clause to specify variable name".to_string(),
                ValidationErrorType::SyntaxError,
            ));
        }

        if self.variable_name.starts_with('_') && !self.variable_name.starts_with("__") {
            return Err(ValidationError::new(
                format!(
                    "Variable name '{}' should not start with single underscore (reserved for internal use)",
                    self.variable_name
                ),
                ValidationErrorType::SemanticError,
            ));
        }

        if self.variable_name.chars().next().unwrap_or_default().is_ascii_digit() {
            return Err(ValidationError::new(
                format!(
                    "Variable name '{}' cannot start with a digit",
                    self.variable_name
                ),
                ValidationErrorType::SemanticError,
            ));
        }

        if self.aliases_available.contains_key(&self.variable_name) {
            return Err(ValidationError::new(
                format!(
                    "Variable '{}' is already defined in the query",
                    self.variable_name
                ),
                ValidationErrorType::SemanticError,
            ));
        }

        Ok(())
    }

    fn validate_type(&mut self) -> Result<(), ValidationError> {
        let list_type = self.deduce_list_element_type(&self.unwind_expression)?;
        if list_type == ValueType::Unknown {
            self.base.add_type_error(
                "Cannot deduce element type of UNWIND expression".to_string(),
            );
        }
        Ok(())
    }

    fn validate_aliases(&self) -> Result<(), ValidationError> {
        let refs = self.get_expression_references(&self.unwind_expression);
        for ref_name in refs {
            if !self.aliases_available.contains_key(&ref_name) && ref_name != "$" && ref_name != "$$" {
                return Err(ValidationError::new(
                    format!(
                        "UNWIND expression references undefined variable '{}'",
                        ref_name
                    ),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    fn expression_is_empty(&self, _expr: &Expression) -> bool {
        false
    }

    fn deduce_expr_type(&self, _expr: &Expression) -> Result<ValueType, ValidationError> {
        Ok(ValueType::List)
    }

    fn deduce_list_element_type(&self, _expr: &Expression) -> Result<ValueType, ValidationError> {
        Ok(ValueType::Unknown)
    }

    fn get_expression_references(&self, _expr: &Expression) -> Vec<String> {
        Vec::new()
    }

    pub fn set_unwind_expression(&mut self, expr: Expression) {
        self.unwind_expression = expr;
    }

    pub fn set_variable_name(&mut self, name: String) {
        self.variable_name = name;
    }

    pub fn set_aliases_available(&mut self, aliases: HashMap<String, ValueType>) {
        self.aliases_available = aliases;
    }

    pub fn unwind_expression(&self) -> &Expression {
        &self.unwind_expression
    }

    pub fn variable_name(&self) -> &str {
        &self.variable_name
    }
}

impl Validator {
    pub fn validate_unwind(
        &mut self,
        expr: Expression,
        var_name: String,
        aliases: &HashMap<String, ValueType>,
    ) -> Result<(), ValidationError> {
        let mut validator = UnwindValidator::new(self.context().clone());
        validator.set_unwind_expression(expr);
        validator.set_variable_name(var_name);
        validator.set_aliases_available(aliases.clone());
        validator.validate()
    }
}

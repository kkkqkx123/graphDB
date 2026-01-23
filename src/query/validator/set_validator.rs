//! SET/GET/SHOW 语句验证器
//! 对应 NebulaGraph SetValidator.h/.cpp 的功能
//! 验证 SET/GET/SHOW 语句的合法性

use super::base_validator::Validator;
use super::ValidationContext;
use crate::core::Expression;
use crate::query::validator::ValidationError;
use crate::query::validator::ValidationErrorType;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum SetStatementType {
    SetVariable,
    SetTag,
    SetEdge,
    SetPriority,
}

#[derive(Debug, Clone)]
pub struct SetItem {
    pub statement_type: SetStatementType,
    pub target: Expression,
    pub value: Expression,
}

pub struct SetValidator {
    base: Validator,
    set_items: Vec<SetItem>,
    variables: HashMap<String, Expression>,
}

impl SetValidator {
    pub fn new(context: ValidationContext) -> Self {
        Self {
            base: Validator::new(context),
            set_items: Vec::new(),
            variables: HashMap::new(),
        }
    }

    pub fn validate(&mut self) -> Result<(), ValidationError> {
        self.validate_impl()
    }

    fn validate_impl(&mut self) -> Result<(), ValidationError> {
        for item in &self.set_items {
            self.validate_set_item(item)?;
        }
        self.validate_variables()?;
        Ok(())
    }

    fn validate_set_item(&self, item: &SetItem) -> Result<(), ValidationError> {
        match item.statement_type {
            SetStatementType::SetVariable => {
                if let Expression::Variable(name) = &item.target {
                    if name.is_empty() {
                        return Err(ValidationError::new(
                            "Variable name cannot be empty".to_string(),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                    if !name.starts_with('$') {
                        return Err(ValidationError::new(
                            format!("Variable name '{}' must start with '$'", name),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                } else {
                    return Err(ValidationError::new(
                        "SET must target a variable".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            SetStatementType::SetTag => {
                if !matches!(&item.target, Expression::Property { .. }) {
                    return Err(ValidationError::new(
                        "SET tag must target a property expression".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            SetStatementType::SetEdge => {
                if !matches!(&item.target, Expression::Property { .. }) {
                    return Err(ValidationError::new(
                        "SET edge must target a property expression".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            SetStatementType::SetPriority => {
                self.validate_priority_value(&item.value)?;
            }
        }
        Ok(())
    }

    fn validate_priority_value(&self, value: &Expression) -> Result<(), ValidationError> {
        match value {
            Expression::Literal(lit) => {
                if let crate::core::Value::Int(n) = lit {
                    if *n < 0 {
                        return Err(ValidationError::new(
                            "Priority cannot be negative".to_string(),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                }
            }
            _ => {
                return Err(ValidationError::new(
                    "Priority must be an integer literal".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    fn validate_variables(&self) -> Result<(), ValidationError> {
        for (name, value) in &self.variables {
            if name.is_empty() {
                return Err(ValidationError::new(
                    "Variable name cannot be empty".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
            // 验证变量值是否有效
            self.validate_expression(value)?;
        }
        Ok(())
    }

    fn validate_expression(&self, expression: &Expression) -> Result<(), ValidationError> {
        match expression {
            Expression::Binary { left, right, .. } => {
                self.validate_expression(left)?;
                self.validate_expression(right)?;
            }
            Expression::Unary { operand, .. } => {
                self.validate_expression(operand)?;
            }
            Expression::Function { args, .. } => {
                for arg in args {
                    self.validate_expression(arg)?;
                }
            }
            Expression::List(items) => {
                for item in items {
                    self.validate_expression(item)?;
                }
            }
            Expression::Map(pairs) => {
                for (_, value) in pairs {
                    self.validate_expression(value)?;
                }
            }
            Expression::Case { conditions, default, .. } => {
                for (condition, expression) in conditions {
                    self.validate_expression(condition)?;
                    self.validate_expression(expression)?;
                }
                if let Some(default_expression) = default {
                    self.validate_expression(default_expression)?;
                }
            }
            Expression::TypeCast { expression, .. } => {
                self.validate_expression(expression)?;
            }
            Expression::Subscript { collection, index } => {
                self.validate_expression(collection)?;
                self.validate_expression(index)?;
            }
            Expression::Range { collection, start, end } => {
                self.validate_expression(collection)?;
                if let Some(start_expression) = start {
                    self.validate_expression(start_expression)?;
                }
                if let Some(end_expression) = end {
                    self.validate_expression(end_expression)?;
                }
            }
            Expression::Path(items) => {
                for item in items {
                    self.validate_expression(item)?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn add_set_item(&mut self, item: SetItem) {
        self.set_items.push(item);
    }

    pub fn set_variable(&mut self, name: String, value: Expression) {
        self.variables.insert(name, value);
    }
}

impl Validator {
    pub fn validate_set(
        &mut self,
        set_items: &[SetItem],
    ) -> Result<(), ValidationError> {
        let mut validator = SetValidator::new(self.context().clone());
        for item in set_items {
            validator.add_set_item(item.clone());
        }
        validator.validate()
    }
}

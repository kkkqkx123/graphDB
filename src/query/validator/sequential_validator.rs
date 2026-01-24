//! Sequential 语句验证器
//! 对应 NebulaGraph SequentialValidator.h/.cpp 的功能
//! 验证多语句查询（使用分号分隔）的合法性

use super::base_validator::Validator;
use super::ValidationContext;
use crate::query::validator::ValidationError;
use crate::query::validator::ValidationErrorType;
use std::collections::HashMap;

#[derive(Clone)]
pub struct SequentialStatement {
    pub statement: String,
    pub parameters: HashMap<String, crate::core::Expression>,
}

pub struct SequentialValidator {
    statements: Vec<SequentialStatement>,
    max_statements: usize,
    variables: HashMap<String, crate::core::DataType>,
}

impl SequentialValidator {
    pub fn new(_context: ValidationContext) -> Self {
        Self {
            statements: Vec::new(),
            max_statements: 100,
            variables: HashMap::new(),
        }
    }

    pub fn validate(&mut self) -> Result<(), ValidationError> {
        self.validate_impl()
    }

    fn validate_impl(&mut self) -> Result<(), ValidationError> {
        self.validate_statement_count()?;
        self.validate_statement_order()?;
        self.validate_variables()?;
        Ok(())
    }

    fn validate_statement_count(&self) -> Result<(), ValidationError> {
        if self.statements.is_empty() {
            return Err(ValidationError::new(
                "Sequential statement must have at least one statement".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        if self.statements.len() > self.max_statements {
            return Err(ValidationError::new(
                format!(
                    "Too many statements in sequential query (max: {})",
                    self.max_statements
                ),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    fn validate_statement_order(&self) -> Result<(), ValidationError> {
        let mut has_ddl = false;
        let mut has_dml = false;

        for (i, stmt) in self.statements.iter().enumerate() {
            let stmt_upper = stmt.statement.to_uppercase();
            if self.is_ddl_statement(&stmt_upper) {
                if has_dml {
                    return Err(ValidationError::new(
                        format!(
                            "DDL statement cannot follow DML statement at position {}",
                            i + 1
                        ),
                        ValidationErrorType::SemanticError,
                    ));
                }
                if has_ddl {
                    return Err(ValidationError::new(
                        format!(
                            "Multiple DDL statements are not allowed, found at position {}",
                            i + 1
                        ),
                        ValidationErrorType::SemanticError,
                    ));
                }
                has_ddl = true;
            }
            if self.is_dml_statement(&stmt_upper) {
                has_dml = true;
            }
        }
        Ok(())
    }

    fn is_ddl_statement(&self, stmt: &str) -> bool {
        stmt.starts_with("CREATE") || stmt.starts_with("ALTER") || stmt.starts_with("DROP")
    }

    fn is_dml_statement(&self, stmt: &str) -> bool {
        stmt.starts_with("INSERT") || stmt.starts_with("UPDATE") || stmt.starts_with("DELETE")
            || stmt.starts_with("UPSERT")
    }

    fn validate_variables(&self) -> Result<(), ValidationError> {
        for (name, _) in &self.variables {
            if name.is_empty() {
                return Err(ValidationError::new(
                    "Variable name cannot be empty".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
            if !name.starts_with('$') && !name.starts_with('@') {
                return Err(ValidationError::new(
                    format!("Invalid variable name '{}': must start with '$' or '@'", name),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    pub fn add_statement(&mut self, statement: SequentialStatement) {
        self.statements.push(statement);
    }

    pub fn set_variable(&mut self, name: String, type_: crate::core::DataType) {
        self.variables.insert(name, type_);
    }

    pub fn set_max_statements(&mut self, max: usize) {
        self.max_statements = max;
    }
}

impl Validator {
    pub fn validate_sequential(
        &mut self,
        statements: &[SequentialStatement],
    ) -> Result<(), ValidationError> {
        let mut validator = SequentialValidator::new(self.context().clone());
        for stmt in statements {
            validator.add_statement(stmt.clone());
        }
        validator.validate()
    }
}

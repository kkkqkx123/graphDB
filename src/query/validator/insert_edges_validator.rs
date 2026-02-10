//! Insert Edges 语句验证器
//! 对应 NebulaGraph InsertEdgesValidator 的功能
//! 验证 INSERT EDGES 语句的语义正确性

use crate::core::error::{DBResult, ValidationError as CoreValidationError, ValidationErrorType};
use crate::query::context::ast::AstContext;
use crate::query::context::execution::QueryContext;
use crate::query::parser::ast::stmt::InsertStmt;
use crate::query::validator::base_validator::{Validator, ValueType};

pub struct InsertEdgesValidator {
    base: Validator,
}

impl InsertEdgesValidator {
    pub fn new() -> Self {
        Self {
            base: Validator::new(),
        }
    }

    pub fn validate(&mut self, stmt: &InsertStmt) -> Result<(), CoreValidationError> {
        match &stmt.target {
            crate::query::parser::ast::stmt::InsertTarget::Edge {
                edge_name,
                prop_names,
                src,
                dst,
                rank,
                values,
            } => {
                self.validate_edge_type_exists(edge_name)?;
                self.validate_property_names(edge_name, prop_names)?;
                self.validate_vertex_id_format(src, "source")?;
                self.validate_vertex_id_format(dst, "destination")?;
                self.validate_rank(rank)?;
                self.validate_values_count(prop_names, values)?;
                self.validate_property_values(edge_name, prop_names, values)?;
            }
            crate::query::parser::ast::stmt::InsertTarget::Vertices { .. } => {
                return Err(CoreValidationError::new(
                    "Expected INSERT EDGES but got INSERT VERTICES".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    pub fn validate_with_ast(
        &mut self,
        stmt: &InsertStmt,
        _query_context: Option<&QueryContext>,
        ast: &mut AstContext,
    ) -> DBResult<()> {
        self.validate_space_chosen(ast)?;
        self.validate(stmt)?;
        self.generate_output_columns(ast);
        Ok(())
    }

    fn validate_space_chosen(&self, ast: &AstContext) -> Result<(), CoreValidationError> {
        if ast.space().space_id.is_none() {
            return Err(CoreValidationError::new(
                "No space selected. Use `USE <space>` to select a graph space first.".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    fn validate_edge_type_exists(&self, edge_name: &str) -> Result<(), CoreValidationError> {
        if edge_name.is_empty() {
            return Err(CoreValidationError::new(
                "Edge type name cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    fn validate_property_names(
        &self,
        _edge_name: &str,
        prop_names: &[String],
    ) -> Result<(), CoreValidationError> {
        let mut seen = std::collections::HashSet::new();
        for prop_name in prop_names {
            if !seen.insert(prop_name) {
                return Err(CoreValidationError::new(
                    format!("Duplicate property name '{}' in INSERT EDGES", prop_name),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    fn validate_vertex_id_format(
        &self,
        expr: &crate::core::Expression,
        role: &str,
    ) -> Result<(), CoreValidationError> {
        match expr {
            crate::core::Expression::Literal(crate::core::Value::String(s)) => {
                if s.is_empty() {
                    return Err(CoreValidationError::new(
                        format!("{} vertex ID cannot be empty", role),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            crate::core::Expression::Variable(_) => {}
            _ => {
                return Err(CoreValidationError::new(
                    format!(
                        "{} vertex ID must be a string constant or variable",
                        role
                    ),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    fn validate_rank(&self, rank: &Option<crate::core::Expression>) -> Result<(), CoreValidationError> {
        if let Some(rank_expr) = rank {
            match rank_expr {
                crate::core::Expression::Literal(crate::core::Value::Int(_)) => Ok(()),
                crate::core::Expression::Variable(_) => Ok(()),
                _ => Err(CoreValidationError::new(
                    "Rank must be an integer constant or variable".to_string(),
                    ValidationErrorType::SemanticError,
                )),
            }
        } else {
            Ok(())
        }
    }

    fn validate_values_count(
        &self,
        prop_names: &[String],
        values: &[crate::core::Expression],
    ) -> Result<(), CoreValidationError> {
        if values.len() != prop_names.len() {
            return Err(CoreValidationError::new(
                format!(
                    "Value count mismatch: expected {} values, got {}",
                    prop_names.len(),
                    values.len()
                ),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    fn validate_property_values(
        &self,
        edge_name: &str,
        prop_names: &[String],
        values: &[crate::core::Expression],
    ) -> Result<(), CoreValidationError> {
        for (prop_idx, value) in values.iter().enumerate() {
            if let Err(e) = self.validate_property_value(edge_name, &prop_names[prop_idx], value) {
                return Err(CoreValidationError::new(
                    format!(
                        "Error in edge property '{}': {}",
                        prop_names[prop_idx],
                        e.message
                    ),
                    e.error_type,
                ));
            }
        }
        Ok(())
    }

    fn validate_property_value(
        &self,
        _edge_name: &str,
        _prop_name: &str,
        value: &crate::core::Expression,
    ) -> Result<(), CoreValidationError> {
        match value {
            crate::core::Expression::Literal(_) => Ok(()),
            crate::core::Expression::Variable(_) => Ok(()),
            crate::core::Expression::Function { args, .. } => {
                if args.is_empty() {
                    return Err(CoreValidationError::new(
                        "Function call must have arguments".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn generate_output_columns(&mut self, _ast: &mut AstContext) {
        self.base.add_output("INSERTED_EDGES".to_string(), ValueType::List);
    }
}

impl Default for InsertEdgesValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;
    use crate::query::parser::ast::stmt::{InsertStmt, InsertTarget};
    use crate::query::parser::ast::Span;

    fn create_insert_edge_stmt(
        edge_name: String,
        prop_names: Vec<String>,
        src: Expression,
        dst: Expression,
        rank: Option<Expression>,
        values: Vec<Expression>,
    ) -> InsertStmt {
        InsertStmt {
            span: Span::default(),
            target: InsertTarget::Edge {
                edge_name,
                prop_names,
                src,
                dst,
                rank,
                values,
            },
        }
    }

    #[test]
    fn test_validate_edge_name_not_empty() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "".to_string(),
            vec!["prop".to_string()],
            Expression::literal("v1"),
            Expression::literal("v2"),
            None,
            vec![Expression::literal("value")],
        );
        let result = validator.validate(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "Edge type name cannot be empty");
    }

    #[test]
    fn test_validate_duplicate_property_names() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec!["prop1".to_string(), "prop1".to_string()],
            Expression::literal("v1"),
            Expression::literal("v2"),
            None,
            vec![Expression::literal("val1"), Expression::literal("val2")],
        );
        let result = validator.validate(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Duplicate property name"));
    }

    #[test]
    fn test_validate_value_count_mismatch() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec!["prop1".to_string(), "prop2".to_string()],
            Expression::literal("v1"),
            Expression::literal("v2"),
            None,
            vec![Expression::literal("val1")],
        );
        let result = validator.validate(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Value count mismatch"));
    }

    #[test]
    fn test_validate_source_vertex_id_empty() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec![],
            Expression::literal(""),
            Expression::literal("v2"),
            None,
            vec![],
        );
        let result = validator.validate(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("source vertex ID cannot be empty"));
    }

    #[test]
    fn test_validate_destination_vertex_id_empty() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec![],
            Expression::literal("v1"),
            Expression::literal(""),
            None,
            vec![],
        );
        let result = validator.validate(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("destination vertex ID cannot be empty"));
    }

    #[test]
    fn test_validate_vertex_ids_valid() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec![],
            Expression::literal("v1"),
            Expression::literal("v2"),
            None,
            vec![],
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_vertex_ids_variable() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec![],
            Expression::variable("$src"),
            Expression::variable("$dst"),
            None,
            vec![],
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_source_vertex_id() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec![],
            Expression::literal(123),
            Expression::literal("v2"),
            None,
            vec![],
        );
        let result = validator.validate(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("source vertex ID must be a string constant or variable"));
    }

    #[test]
    fn test_validate_rank_valid_integer() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec![],
            Expression::literal("v1"),
            Expression::literal("v2"),
            Some(Expression::literal(0)),
            vec![],
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_rank_valid_variable() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec![],
            Expression::literal("v1"),
            Expression::literal("v2"),
            Some(Expression::variable("$rank")),
            vec![],
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_rank() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec![],
            Expression::literal("v1"),
            Expression::literal("v2"),
            Some(Expression::literal("invalid")),
            vec![],
        );
        let result = validator.validate(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Rank must be an integer constant or variable"));
    }

    #[test]
    fn test_validate_property_values() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec!["since".to_string(), "type".to_string()],
            Expression::literal("v1"),
            Expression::literal("v2"),
            None,
            vec![Expression::literal(2020), Expression::literal("best")],
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_wrong_target_type() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = InsertStmt {
            span: Span::default(),
            target: InsertTarget::Vertices {
                tag_name: "person".to_string(),
                prop_names: vec![],
                values: vec![],
            },
        };
        let result = validator.validate(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "Expected INSERT EDGES but got INSERT VERTICES");
    }
}

//! Insert Vertices 语句验证器
//! 对应 NebulaGraph InsertVerticesValidator 的功能
//! 验证 INSERT VERTICES 语句的语义正确性

use crate::core::error::{DBResult, ValidationError as CoreValidationError, ValidationErrorType};
use crate::query::context::ast::AstContext;
use crate::query::context::execution::QueryContext;
use crate::query::parser::ast::stmt::InsertStmt;
use crate::query::validator::base_validator::{Validator, ValueType};

pub struct InsertVerticesValidator {
    base: Validator,
}

impl InsertVerticesValidator {
    pub fn new() -> Self {
        Self {
            base: Validator::new(),
        }
    }

    pub fn validate(&mut self, stmt: &InsertStmt) -> Result<(), CoreValidationError> {
        match &stmt.target {
            crate::query::parser::ast::stmt::InsertTarget::Vertices {
                tag_name,
                prop_names,
                values,
            } => {
                self.validate_tag_exists(tag_name)?;
                self.validate_property_names(tag_name, prop_names)?;
                self.validate_values_count(prop_names, values)?;
                self.validate_vertex_id_format(values)?;
                self.validate_property_values(tag_name, prop_names, values)?;
            }
            crate::query::parser::ast::stmt::InsertTarget::Edge { .. } => {
                return Err(CoreValidationError::new(
                    "Expected INSERT VERTICES but got INSERT EDGES".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    pub fn validate_with_ast(
        &mut self,
        stmt: &InsertStmt,
        query_context: Option<&QueryContext>,
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

    fn validate_tag_exists(&self, tag_name: &str) -> Result<(), CoreValidationError> {
        if tag_name.is_empty() {
            return Err(CoreValidationError::new(
                "Tag name cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    fn validate_property_names(
        &self,
        tag_name: &str,
        prop_names: &[String],
    ) -> Result<(), CoreValidationError> {
        let mut seen = std::collections::HashSet::new();
        for prop_name in prop_names {
            if !seen.insert(prop_name) {
                return Err(CoreValidationError::new(
                    format!("Duplicate property name '{}' in INSERT VERTICES", prop_name),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    fn validate_values_count(
        &self,
        prop_names: &[String],
        values: &[(crate::core::Expression, Vec<crate::core::Expression>)],
    ) -> Result<(), CoreValidationError> {
        for (idx, (_, props)) in values.iter().enumerate() {
            if props.len() != prop_names.len() {
                return Err(CoreValidationError::new(
                    format!(
                        "Value count mismatch for vertex {}: expected {} values, got {}",
                        idx + 1,
                        prop_names.len(),
                        props.len()
                    ),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    fn validate_vertex_id_format(
        &self,
        values: &[(crate::core::Expression, Vec<crate::core::Expression>)],
    ) -> Result<(), CoreValidationError> {
        for (idx, (vid_expr, _)) in values.iter().enumerate() {
            match vid_expr {
                crate::core::Expression::Literal(crate::core::Value::String(s)) => {
                    if s.is_empty() {
                        return Err(CoreValidationError::new(
                            format!("Vertex ID cannot be empty for vertex {}", idx + 1),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                }
                crate::core::Expression::Variable(_) => {
                }
                _ => {
                    return Err(CoreValidationError::new(
                        format!(
                            "Vertex ID must be a string constant or variable for vertex {}",
                            idx + 1
                        ),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_property_values(
        &self,
        tag_name: &str,
        prop_names: &[String],
        values: &[(crate::core::Expression, Vec<crate::core::Expression>)],
    ) -> Result<(), CoreValidationError> {
        for (idx, (_, props)) in values.iter().enumerate() {
            for (prop_idx, value) in props.iter().enumerate() {
                if let Err(e) = self.validate_property_value(tag_name, &prop_names[prop_idx], value) {
                    return Err(CoreValidationError::new(
                        format!("Error in vertex {} property '{}': {}", idx + 1, prop_names[prop_idx], e.message),
                        e.error_type,
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_property_value(
        &self,
        _tag_name: &str,
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

    fn generate_output_columns(&self, ast: &mut AstContext) {
        ast.add_output(
            "INSERTED_VERTICES".to_string(),
            ValueType::List,
        );
    }
}

impl Default for InsertVerticesValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;
    use crate::core::Value;
    use crate::query::parser::ast::stmt::{InsertStmt, InsertTarget};
    use crate::query::parser::ast::Span;

    fn create_insert_stmt(tag_name: String, prop_names: Vec<String>, values: Vec<(Expression, Vec<Expression>)>) -> InsertStmt {
        InsertStmt {
            span: Span::default(),
            target: InsertTarget::Vertices {
                tag_name,
                prop_names,
                values,
            },
        }
    }

    #[test]
    fn test_validate_tag_name_not_empty() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_stmt(
            "".to_string(),
            vec!["name".to_string()],
            vec![(Expression::literal("v1"), vec![Expression::literal("test")])],
        );
        let result = validator.validate(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "Tag name cannot be empty");
    }

    #[test]
    fn test_validate_duplicate_property_names() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_stmt(
            "person".to_string(),
            vec!["name".to_string(), "name".to_string()],
            vec![(Expression::literal("v1"), vec![Expression::literal("test1"), Expression::literal("test2")])],
        );
        let result = validator.validate(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Duplicate property name"));
    }

    #[test]
    fn test_validate_value_count_mismatch() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_stmt(
            "person".to_string(),
            vec!["name".to_string(), "age".to_string()],
            vec![(Expression::literal("v1"), vec![Expression::literal("test")])],
        );
        let result = validator.validate(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Value count mismatch"));
    }

    #[test]
    fn test_validate_vertex_id_empty() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_stmt(
            "person".to_string(),
            vec!["name".to_string()],
            vec![(Expression::literal(""), vec![Expression::literal("test")])],
        );
        let result = validator.validate(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Vertex ID cannot be empty"));
    }

    #[test]
    fn test_validate_vertex_id_valid() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_stmt(
            "person".to_string(),
            vec!["name".to_string()],
            vec![(Expression::literal("v1"), vec![Expression::literal("test")])],
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_vertex_id_variable() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_stmt(
            "person".to_string(),
            vec!["name".to_string()],
            vec![(Expression::variable("vid_param"), vec![Expression::literal("test")])],
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_vertex_id_type() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_stmt(
            "person".to_string(),
            vec!["name".to_string()],
            vec![(Expression::literal(123), vec![Expression::literal("test")])],
        );
        let result = validator.validate(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Vertex ID must be a string constant or variable"));
    }

    #[test]
    fn test_validate_multiple_vertices() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_stmt(
            "person".to_string(),
            vec!["name".to_string()],
            vec![
                (Expression::literal("v1"), vec![Expression::literal("test1")]),
                (Expression::literal("v2"), vec![Expression::literal("test2")]),
                (Expression::literal("v3"), vec![Expression::literal("test3")]),
            ],
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_property_values_literal() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_stmt(
            "person".to_string(),
            vec!["name".to_string(), "age".to_string()],
            vec![(
                Expression::literal("v1"),
                vec![Expression::literal("John"), Expression::literal(25)],
            )],
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_property_values_variable() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_stmt(
            "person".to_string(),
            vec!["name".to_string()],
            vec![(
                Expression::literal("v1"),
                vec![Expression::variable("$name_param")],
            )],
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_property_values_function() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_stmt(
            "person".to_string(),
            vec!["name".to_string()],
            vec![(
                Expression::literal("v1"),
                vec![Expression::Function {
                    name: "to_string".to_string(),
                    args: vec![Expression::literal(123)],
                }],
            )],
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_wrong_target_type() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = InsertStmt {
            span: Span::default(),
            target: InsertTarget::Edge {
                edge_name: "friend".to_string(),
                prop_names: vec![],
                src: Expression::literal("v1"),
                dst: Expression::literal("v2"),
                rank: None,
                values: vec![],
            },
        };
        let result = validator.validate(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "Expected INSERT VERTICES but got INSERT EDGES");
    }
}

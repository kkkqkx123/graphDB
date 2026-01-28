//! Delete 语句验证器
//! 对应 NebulaGraph DeleteValidator 的功能
//! 验证 DELETE 语句的语义正确性

use crate::core::error::{DBResult, ValidationError as CoreValidationError, ValidationErrorType};
use crate::core::Expression;
use crate::query::context::ast::AstContext;
use crate::query::context::execution::QueryContext;
use crate::query::parser::ast::stmt::DeleteStmt;
use crate::query::validator::base_validator::{Validator, ValueType};

pub struct DeleteValidator {
    base: Validator,
}

impl DeleteValidator {
    pub fn new() -> Self {
        Self {
            base: Validator::new(),
        }
    }

    pub fn validate(&mut self, stmt: &DeleteStmt) -> Result<(), CoreValidationError> {
        self.validate_target(&stmt.target)?;
        self.validate_where_clause(stmt.where_clause.as_ref())?;
        Ok(())
    }

    pub fn validate_with_ast(
        &mut self,
        stmt: &DeleteStmt,
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

    fn validate_target(
        &self,
        target: &crate::query::parser::ast::stmt::DeleteTarget,
    ) -> Result<(), CoreValidationError> {
        match target {
            crate::query::parser::ast::stmt::DeleteTarget::Vertices(vids) => {
                if vids.is_empty() {
                    return Err(CoreValidationError::new(
                        "DELETE VERTICES must specify at least one vertex".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
                for (idx, vid) in vids.iter().enumerate() {
                    self.validate_vertex_id(vid, idx + 1)?;
                }
            }
            crate::query::parser::ast::stmt::DeleteTarget::Edges { src, dst, edge_type, rank } => {
                self.validate_vertex_id(src, 0)?;
                self.validate_vertex_id(dst, 1)?;
                if let Some(rank_expr) = rank {
                    self.validate_rank(rank_expr)?;
                }
                if let Some(et) = edge_type {
                    if et.is_empty() {
                        return Err(CoreValidationError::new(
                            "Edge type name cannot be empty".to_string(),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                }
            }
            crate::query::parser::ast::stmt::DeleteTarget::Tag(tag_name) => {
                if tag_name.is_empty() {
                    return Err(CoreValidationError::new(
                        "Tag name cannot be empty".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            crate::query::parser::ast::stmt::DeleteTarget::Index(index_name) => {
                if index_name.is_empty() {
                    return Err(CoreValidationError::new(
                        "Index name cannot be empty".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_vertex_id(
        &self,
        expr: &Expression,
        idx: usize,
    ) -> Result<(), CoreValidationError> {
        match expr {
            Expression::Literal(crate::core::Value::String(s)) => {
                if s.is_empty() {
                    return Err(CoreValidationError::new(
                        format!("Vertex ID at position {} cannot be empty", idx + 1),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            Expression::Variable(_) => {}
            _ => {
                return Err(CoreValidationError::new(
                    format!(
                        "Vertex ID at position {} must be a string constant or variable",
                        idx + 1
                    ),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    fn validate_rank(&self, expr: &Expression) -> Result<(), CoreValidationError> {
        match expr {
            Expression::Literal(crate::core::Value::Int(_)) => Ok(()),
            Expression::Variable(_) => Ok(()),
            _ => Err(CoreValidationError::new(
                "Rank must be an integer constant or variable".to_string(),
                ValidationErrorType::SemanticError,
            )),
        }
    }

    fn validate_where_clause(
        &self,
        where_clause: Option<&Expression>,
    ) -> Result<(), CoreValidationError> {
        if let Some(where_expr) = where_clause {
            self.validate_expression(where_expr)?;
        }
        Ok(())
    }

    fn validate_expression(&self, expr: &Expression) -> Result<(), CoreValidationError> {
        match expr {
            Expression::Literal(_) => Ok(()),
            Expression::Variable(_) => Ok(()),
            Expression::Property { .. } => Ok(()),
            Expression::Function { args, .. } => {
                for arg in args {
                    self.validate_expression(arg)?;
                }
                Ok(())
            }
            Expression::Unary { operand, .. } => self.validate_expression(operand),
            Expression::Binary { left, right, .. } => {
                self.validate_expression(left)?;
                self.validate_expression(right)?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn generate_output_columns(&self, ast: &mut AstContext) {
        ast.add_output("DELETED".to_string(), ValueType::Bool);
    }
}

impl Default for DeleteValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;
    use crate::query::parser::ast::stmt::{DeleteStmt, DeleteTarget};
    use crate::query::parser::ast::Span;

    fn create_delete_stmt(target: DeleteTarget, where_clause: Option<Expression>) -> DeleteStmt {
        DeleteStmt {
            span: Span::default(),
            target,
            where_clause,
        }
    }

    #[test]
    fn test_validate_vertices_empty_list() {
        let mut validator = DeleteValidator::new();
        let stmt = create_delete_stmt(DeleteTarget::Vertices(vec![]), None);
        let result = validator.validate(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "DELETE VERTICES must specify at least one vertex");
    }

    #[test]
    fn test_validate_vertices_valid() {
        let mut validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Vertices(vec![
                Expression::literal("v1"),
                Expression::literal("v2"),
            ]),
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_vertices_with_variable() {
        let mut validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Vertices(vec![Expression::variable("$vids")]),
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_vertex_id_empty() {
        let mut validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Vertices(vec![
                Expression::literal("v1"),
                Expression::literal(""),
            ]),
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("cannot be empty"));
    }

    #[test]
    fn test_validate_edges_valid() {
        let mut validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Edges {
                src: Expression::literal("v1"),
                dst: Expression::literal("v2"),
                edge_type: Some("friend".to_string()),
                rank: None,
            },
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_edges_with_rank() {
        let mut validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Edges {
                src: Expression::literal("v1"),
                dst: Expression::literal("v2"),
                edge_type: Some("friend".to_string()),
                rank: Some(Expression::literal(0)),
            },
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_edges_empty_edge_type() {
        let mut validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Edges {
                src: Expression::literal("v1"),
                dst: Expression::literal("v2"),
                edge_type: Some("".to_string()),
                rank: None,
            },
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "Edge type name cannot be empty");
    }

    #[test]
    fn test_validate_edges_invalid_rank() {
        let mut validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Edges {
                src: Expression::literal("v1"),
                dst: Expression::literal("v2"),
                edge_type: Some("friend".to_string()),
                rank: Some(Expression::literal("invalid")),
            },
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Rank must be an integer constant or variable"));
    }

    #[test]
    fn test_validate_tag_valid() {
        let mut validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Tag("person".to_string()),
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_tag_empty() {
        let mut validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Tag("".to_string()),
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "Tag name cannot be empty");
    }

    #[test]
    fn test_validate_index_valid() {
        let mut validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Index("idx_person".to_string()),
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_index_empty() {
        let mut validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Index("".to_string()),
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "Index name cannot be empty");
    }

    #[test]
    fn test_validate_where_clause_literal() {
        let mut validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Vertices(vec![Expression::literal("v1")]),
            Some(Expression::Binary {
                left: Box::new(Expression::Property {
                    object: Box::new(Expression::Variable("n".to_string())),
                    property: "status".to_string(),
                }),
                op: crate::core::types::BinaryOperator::Equal,
                right: Box::new(Expression::literal("deleted")),
            }),
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_where_clause_variable() {
        let mut validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Vertices(vec![Expression::literal("v1")]),
            Some(Expression::Variable("$condition".to_string())),
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_where_clause_function() {
        let mut validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Vertices(vec![Expression::literal("v1")]),
            Some(Expression::Function {
                name: "exists".to_string(),
                args: vec![Expression::variable("$cond")],
            }),
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_where_clause_binary() {
        let mut validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Vertices(vec![Expression::literal("v1")]),
            Some(Expression::Binary {
                left: Box::new(Expression::Variable("n".to_string())),
                op: crate::core::types::BinaryOperator::And,
                right: Box::new(Expression::Variable("m".to_string())),
            }),
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }
}

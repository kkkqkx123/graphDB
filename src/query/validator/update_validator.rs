//! Update 语句验证器
//! 对应 NebulaGraph UpdateValidator 的功能
//! 验证 UPDATE 语句的语义正确性

use crate::core::error::{DBResult, ValidationError as CoreValidationError, ValidationErrorType};
use crate::core::Expression;
use crate::query::context::ast::AstContext;
use crate::query::context::execution::QueryContext;
use crate::query::parser::ast::stmt::{SetClause, UpdateStmt};
use crate::query::validator::base_validator::{Validator, ValueType};

pub struct UpdateValidator {
    base: Validator,
}

impl UpdateValidator {
    pub fn new() -> Self {
        Self {
            base: Validator::new(),
        }
    }

    pub fn validate(&mut self, stmt: &UpdateStmt) -> Result<(), CoreValidationError> {
        self.validate_target(&stmt.target)?;
        self.validate_set_clause(&stmt.set_clause)?;
        self.validate_where_clause(stmt.where_clause.as_ref())?;
        self.validate_assignments(&stmt.set_clause)?;
        Ok(())
    }

    pub fn validate_with_ast(
        &mut self,
        stmt: &UpdateStmt,
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
        target: &crate::query::parser::ast::stmt::UpdateTarget,
    ) -> Result<(), CoreValidationError> {
        match target {
            crate::query::parser::ast::stmt::UpdateTarget::Vertex(vid_expr) => {
                self.validate_vertex_id(vid_expr, "vertex")?;
            }
            crate::query::parser::ast::stmt::UpdateTarget::Edge { src, dst, edge_type, rank } => {
                self.validate_vertex_id(src, "source")?;
                self.validate_vertex_id(dst, "destination")?;
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
            crate::query::parser::ast::stmt::UpdateTarget::Tag(tag_name) => {
                if tag_name.is_empty() {
                    return Err(CoreValidationError::new(
                        "Tag name cannot be empty".to_string(),
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
        role: &str,
    ) -> Result<(), CoreValidationError> {
        match expr {
            Expression::Literal(crate::core::Value::String(s)) => {
                if s.is_empty() {
                    return Err(CoreValidationError::new(
                        format!("{} vertex ID cannot be empty", role),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            Expression::Variable(_) => {}
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

    fn validate_set_clause(&self, set_clause: &SetClause) -> Result<(), CoreValidationError> {
        if set_clause.assignments.is_empty() {
            return Err(CoreValidationError::new(
                "UPDATE statement must have at least one SET clause".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    fn validate_assignments(&self, set_clause: &SetClause) -> Result<(), CoreValidationError> {
        let mut seen = std::collections::HashSet::new();
        for assignment in &set_clause.assignments {
            if !seen.insert(assignment.property.clone()) {
                return Err(CoreValidationError::new(
                    format!(
                        "Duplicate property assignment for '{}'",
                        assignment.property
                    ),
                    ValidationErrorType::SemanticError,
                ));
            }
            self.validate_property_value(&assignment.value)?;
        }
        Ok(())
    }

    fn validate_property_value(&self, value: &Expression) -> Result<(), CoreValidationError> {
        match value {
            Expression::Literal(_) => Ok(()),
            Expression::Variable(_) => Ok(()),
            Expression::Function { args, .. } => {
                if args.is_empty() {
                    return Err(CoreValidationError::new(
                        "Function call must have arguments".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
                self.validate_function_args(args)?;
                Ok(())
            }
            Expression::Unary { op, operand } => {
                self.validate_property_value(operand)?;
                Ok(())
            }
            Expression::Binary { left, right, .. } => {
                self.validate_property_value(left)?;
                self.validate_property_value(right)?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn validate_function_args(&self, args: &[Expression]) -> Result<(), CoreValidationError> {
        for arg in args {
            self.validate_property_value(arg)?;
        }
        Ok(())
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
        ast.add_output("UPDATED".to_string(), ValueType::Bool);
    }
}

impl Default for UpdateValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;
    use crate::query::parser::ast::stmt::{UpdateStmt, UpdateTarget, SetClause, Assignment};
    use crate::query::parser::ast::Span;

    fn create_update_stmt(target: UpdateTarget, assignments: Vec<Assignment>, where_clause: Option<Expression>) -> UpdateStmt {
        UpdateStmt {
            span: Span::default(),
            target,
            set_clause: SetClause {
                span: Span::default(),
                assignments,
            },
            where_clause,
        }
    }

    #[test]
    fn test_validate_vertex_target_valid() {
        let mut validator = UpdateValidator::new();
        let stmt = create_update_stmt(
            UpdateTarget::Vertex(Expression::literal("v1")),
            vec![Assignment {
                property: "name".to_string(),
                value: Expression::literal("new_name"),
            }],
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_vertex_target_variable() {
        let mut validator = UpdateValidator::new();
        let stmt = create_update_stmt(
            UpdateTarget::Vertex(Expression::variable("$vid")),
            vec![Assignment {
                property: "name".to_string(),
                value: Expression::literal("new_name"),
            }],
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_vertex_id_empty() {
        let mut validator = UpdateValidator::new();
        let stmt = create_update_stmt(
            UpdateTarget::Vertex(Expression::literal("")),
            vec![Assignment {
                property: "name".to_string(),
                value: Expression::literal("new_name"),
            }],
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("vertex ID cannot be empty"));
    }

    #[test]
    fn test_validate_edge_target_valid() {
        let mut validator = UpdateValidator::new();
        let stmt = create_update_stmt(
            UpdateTarget::Edge {
                src: Expression::literal("v1"),
                dst: Expression::literal("v2"),
                edge_type: Some("friend".to_string()),
                rank: None,
            },
            vec![Assignment {
                property: "since".to_string(),
                value: Expression::literal(2020),
            }],
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_edge_target_empty_edge_type() {
        let mut validator = UpdateValidator::new();
        let stmt = create_update_stmt(
            UpdateTarget::Edge {
                src: Expression::literal("v1"),
                dst: Expression::literal("v2"),
                edge_type: Some("".to_string()),
                rank: None,
            },
            vec![],
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "Edge type name cannot be empty");
    }

    #[test]
    fn test_validate_tag_target_valid() {
        let mut validator = UpdateValidator::new();
        let stmt = create_update_stmt(
            UpdateTarget::Tag("person".to_string()),
            vec![Assignment {
                property: "name".to_string(),
                value: Expression::literal("new_name"),
            }],
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_tag_target_empty() {
        let mut validator = UpdateValidator::new();
        let stmt = create_update_stmt(
            UpdateTarget::Tag("".to_string()),
            vec![],
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "Tag name cannot be empty");
    }

    #[test]
    fn test_validate_empty_set_clause() {
        let mut validator = UpdateValidator::new();
        let stmt = create_update_stmt(
            UpdateTarget::Vertex(Expression::literal("v1")),
            vec![],
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "UPDATE statement must have at least one SET clause");
    }

    #[test]
    fn test_validate_duplicate_assignments() {
        let mut validator = UpdateValidator::new();
        let stmt = create_update_stmt(
            UpdateTarget::Vertex(Expression::literal("v1")),
            vec![
                Assignment {
                    property: "name".to_string(),
                    value: Expression::literal("new_name"),
                },
                Assignment {
                    property: "name".to_string(),
                    value: Expression::literal("another_name"),
                },
            ],
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Duplicate property assignment"));
    }

    #[test]
    fn test_validate_assignment_literal_value() {
        let mut validator = UpdateValidator::new();
        let stmt = create_update_stmt(
            UpdateTarget::Vertex(Expression::literal("v1")),
            vec![Assignment {
                property: "age".to_string(),
                value: Expression::literal(25),
            }],
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_assignment_variable_value() {
        let mut validator = UpdateValidator::new();
        let stmt = create_update_stmt(
            UpdateTarget::Vertex(Expression::literal("v1")),
            vec![Assignment {
                property: "name".to_string(),
                value: Expression::variable("$new_name"),
            }],
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_assignment_function_value() {
        let mut validator = UpdateValidator::new();
        let stmt = create_update_stmt(
            UpdateTarget::Vertex(Expression::literal("v1")),
            vec![Assignment {
                property: "name".to_string(),
                value: Expression::Function {
                    name: "upper".to_string(),
                    args: vec![Expression::variable("$name")],
                },
            }],
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_assignment_binary_expression() {
        let mut validator = UpdateValidator::new();
        let stmt = create_update_stmt(
            UpdateTarget::Vertex(Expression::literal("v1")),
            vec![Assignment {
                property: "age".to_string(),
                value: Expression::Binary {
                    left: Box::new(Expression::variable("$age")),
                    op: crate::core::types::BinaryOperator::Add,
                    right: Box::new(Expression::literal(1)),
                },
            }],
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_assignment_unary_expression() {
        let mut validator = UpdateValidator::new();
        let stmt = create_update_stmt(
            UpdateTarget::Vertex(Expression::literal("v1")),
            vec![Assignment {
                property: "active".to_string(),
                value: Expression::Unary {
                    op: crate::core::types::UnaryOperator::Not,
                    operand: Box::new(Expression::variable("$active")),
                },
            }],
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_where_clause() {
        let mut validator = UpdateValidator::new();
        let stmt = create_update_stmt(
            UpdateTarget::Vertex(Expression::literal("v1")),
            vec![Assignment {
                property: "name".to_string(),
                value: Expression::literal("new_name"),
            }],
            Some(Expression::Binary {
                left: Box::new(Expression::Property {
                    object: Box::new(Expression::Variable("n".to_string())),
                    property: "status".to_string(),
                }),
                op: crate::core::types::BinaryOperator::Equal,
                right: Box::new(Expression::literal("active")),
            }),
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_edge_rank() {
        let mut validator = UpdateValidator::new();
        let stmt = create_update_stmt(
            UpdateTarget::Edge {
                src: Expression::literal("v1"),
                dst: Expression::literal("v2"),
                edge_type: Some("friend".to_string()),
                rank: Some(Expression::literal(0)),
            },
            vec![Assignment {
                property: "since".to_string(),
                value: Expression::literal(2020),
            }],
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_edge_rank() {
        let mut validator = UpdateValidator::new();
        let stmt = create_update_stmt(
            UpdateTarget::Edge {
                src: Expression::literal("v1"),
                dst: Expression::literal("v2"),
                edge_type: Some("friend".to_string()),
                rank: Some(Expression::literal("invalid")),
            },
            vec![],
            None,
        );
        let result = validator.validate(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Rank must be an integer constant or variable"));
    }
}

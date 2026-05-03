//! Edges Statement Validator
//! Corresponding to the functionality of NebulaGraph InsertEdgesValidator
//! Verify the semantic correctness of the INSERT EDGES statement.

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::types::expr::contextual::ContextualExpression;
use crate::core::{NullType, Value};
use crate::query::parser::ast::stmt::{Ast, InsertTarget};
use crate::query::parser::ast::Stmt;
use crate::query::validator::structs::validation_info::ValidationInfo;
use crate::query::validator::validator_trait::{
    ColumnDef, ExpressionProps, StatementType, StatementValidator, ValidationResult, ValueType,
};
use crate::query::QueryContext;
use crate::storage::metadata::inmemory_schema_manager::InMemorySchemaManager;
use crate::storage::metadata::schema_manager::SchemaManager;
use std::collections::HashSet;
use std::sync::Arc;

/// Verified edge insertion information
#[derive(Debug, Clone)]
pub struct ValidatedInsertEdges {
    pub space_id: u64,
    pub edge_name: String,
    pub edge_type_id: Option<i32>,
    pub prop_names: Vec<String>,
    pub edges: Vec<ValidatedEdgeInsert>,
    pub if_not_exists: bool,
}

#[derive(Debug, Clone)]
pub struct ValidatedEdgeInsert {
    pub src_id: Value,
    pub dst_id: Value,
    pub rank: i64,
    pub values: Vec<Value>,
}

#[derive(Debug)]
pub struct InsertEdgesValidator {
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expression_props: ExpressionProps,
    user_defined_vars: Vec<String>,
    validated_result: Option<ValidatedInsertEdges>,
    schema_manager: Option<Arc<RedbSchemaManager>>,
}

impl InsertEdgesValidator {
    pub fn new() -> Self {
        Self {
            inputs: Vec::new(),
            outputs: Vec::new(),
            expression_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
            validated_result: None,
            schema_manager: None,
        }
    }

    pub fn with_schema_manager(mut self, schema_manager: Arc<RedbSchemaManager>) -> Self {
        self.schema_manager = Some(schema_manager);
        self
    }

    pub fn set_schema_manager(&mut self, schema_manager: Arc<RedbSchemaManager>) {
        self.schema_manager = Some(schema_manager);
    }

    /// Verify the existence of the edge type.
    fn validate_edge_type_exists(&self, edge_name: &str) -> Result<(), ValidationError> {
        if edge_name.is_empty() {
            return Err(ValidationError::new(
                "Edge type name cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    /// Verify the attribute name.
    fn validate_property_names(&self, prop_names: &[String]) -> Result<(), ValidationError> {
        let mut seen = HashSet::new();
        for prop_name in prop_names {
            if !seen.insert(prop_name) {
                return Err(ValidationError::new(
                    format!("Duplicate property name '{}' in INSERT EDGES", prop_name),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    /// Verify the vertex ID format.
    /// Using the unified validation method of SchemaValidator
    fn validate_vertex_id_format(
        &self,
        expr: &ContextualExpression,
        role: &str,
        space_name: Option<&str>,
    ) -> Result<(), ValidationError> {
        // Get vid_type from schema_manager if available, otherwise default to String
        let vid_type = if let (Some(ref schema_manager), Some(space_name)) = (&self.schema_manager, space_name) {
            match schema_manager.get_space(space_name) {
                Ok(Some(space_info)) => space_info.vid_type,
                _ => crate::core::types::DataType::String,
            }
        } else {
            crate::core::types::DataType::String
        };

        if let Some(ref schema_manager) = self.schema_manager {
            let schema_validator =
                crate::query::validator::SchemaValidator::new(schema_manager.clone());
            schema_validator
                .validate_vid_expr(expr, &vid_type, role)
                .map_err(|e| ValidationError::new(e.message, e.error_type))
        } else {
            // Performing basic validation in the absence of the schema_manager
            Self::basic_validate_vertex_id_format(expr, role)
        }
    }

    /// Basic vertex ID verification (when no SchemaManager is available)
    /// Accepts both string and integer vertex IDs
    fn basic_validate_vertex_id_format(
        expr: &ContextualExpression,
        role: &str,
    ) -> Result<(), ValidationError> {
        if expr.expression().is_none() {
            return Err(ValidationError::new(
                format!("{} vertex ID expression is invalid", role),
                ValidationErrorType::SemanticError,
            ));
        }

        if expr.is_variable() {
            return Ok(());
        }

        if expr.is_literal() {
            if let Some(value) = expr.as_literal() {
                if value.is_null() || value.is_empty() {
                    return Err(ValidationError::new(
                        format!("{} vertex ID cannot be empty", role),
                        ValidationErrorType::SemanticError,
                    ));
                }
                // Check the empty string.
                if let Value::String(s) = value {
                    if s.is_empty() {
                        return Err(ValidationError::new(
                            format!("{} vertex ID cannot be empty", role),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                    return Ok(());
                }
                // Accept integer vertex IDs for Int64 vid_type spaces
                if let Value::Int(_) = value {
                    return Ok(());
                }
                // Accept BigInt vertex IDs
                if let Value::BigInt(_) = value {
                    return Ok(());
                }
                return Err(ValidationError::new(
                    format!("{} vertex ID must be a string, integer, or variable", role),
                    ValidationErrorType::SemanticError,
                ));
            }
        }

        Err(ValidationError::new(
            format!("{} vertex ID must be a string, integer, or variable", role),
            ValidationErrorType::SemanticError,
        ))
    }

    /// Verify the rank.
    fn validate_rank(&self, rank: &Option<ContextualExpression>) -> Result<(), ValidationError> {
        if let Some(rank_expr) = rank {
            if rank_expr.expression().is_none() {
                return Err(ValidationError::new(
                    "Rank expression is invalid".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
            if !rank_expr.is_variable() && !rank_expr.is_literal() {
                return Err(ValidationError::new(
                    "Rank must be an integer constant or variable".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
            // Check whether the literal value is an integer.
            if rank_expr.is_literal() {
                if let Some(value) = rank_expr.as_literal() {
                    if !matches!(value, Value::Int(_)) {
                        return Err(ValidationError::new(
                            "Rank must be an integer constant or variable".to_string(),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// Number of validation values
    fn validate_values_count(
        &self,
        prop_names: &[String],
        values: &[ContextualExpression],
    ) -> Result<(), ValidationError> {
        if values.len() != prop_names.len() {
            return Err(ValidationError::new(
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

    /// Verify the attribute values
    fn validate_property_values(
        &self,
        _edge_name: &str,
        prop_names: &[String],
        values: &[ContextualExpression],
    ) -> Result<(), ValidationError> {
        for (prop_idx, value) in values.iter().enumerate() {
            if let Err(e) = self.validate_property_value(&prop_names[prop_idx], value) {
                return Err(ValidationError::new(
                    format!(
                        "Error in edge property '{}': {}",
                        prop_names[prop_idx], e.message
                    ),
                    e.error_type,
                ));
            }
        }
        Ok(())
    }

    /// Verify the value of a single attribute
    fn validate_property_value(
        &self,
        _prop_name: &str,
        value: &ContextualExpression,
    ) -> Result<(), ValidationError> {
        if value.expression().is_none() {
            return Err(ValidationError::new(
                "Invalid attribute value expression".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // Literals and variables are always valid.
        if value.is_literal() || value.is_variable() {
            return Ok(());
        }

        // The function call requires parameters.
        // Note: More detailed verification is required in this case, but ContextualExpression does not provide a way to access the function parameters.
        // Temporarily accept all other types of expressions.
        Ok(())
    }

    /// Generate a column of outputs.
    fn generate_output_columns(&mut self) {
        self.outputs.clear();
        self.outputs.push(ColumnDef {
            name: "INSERTED_EDGES".to_string(),
            type_: ValueType::List,
        });
    }

    /// Evaluating an expression to obtain a value
    fn evaluate_expression(&self, expr: &ContextualExpression) -> Result<Value, ValidationError> {
        if expr.expression().is_none() {
            return Ok(Value::Null(NullType::Null));
        }

        if let Some(value) = expr.as_literal() {
            return Ok(value.clone());
        }

        if let Some(name) = expr.as_variable() {
            return Ok(Value::String(format!("${}", name)));
        }

        Ok(Value::Null(NullType::Null))
    }

    /// Evaluating the rank expression
    fn evaluate_rank(&self, rank: &Option<ContextualExpression>) -> Result<i64, ValidationError> {
        if let Some(rank_expr) = rank {
            if let Some(Value::BigInt(n)) = rank_expr.as_literal() {
                return Ok(n);
            }
        }
        Ok(0)
    }
}

impl Default for InsertEdgesValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementing the StatementValidator trait
///
/// # Refactoring changes
/// The `validate` method accepts `Arc<Ast>` and `Arc<QueryContext>` as parameters.
impl StatementValidator for InsertEdgesValidator {
    fn validate(
        &mut self,
        ast: Arc<Ast>,
        qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        // 1. Check whether additional space is needed.
        if !self.is_global_statement() && qctx.space_id().is_none() {
            return Err(ValidationError::new(
                "No image space selected, please execute first USE <space>".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 2. Obtain the INSERT statement
        let insert_stmt = match &ast.stmt {
            Stmt::Insert(insert_stmt) => insert_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected INSERT statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        // 3. Verify the statement type
        let (edge_name, prop_names, edges) = match &insert_stmt.target {
            InsertTarget::Edge {
                edge_name,
                prop_names,
                edges,
            } => (edge_name.clone(), prop_names.clone(), edges.clone()),
            InsertTarget::Vertices { .. } => {
                return Err(ValidationError::new(
                    "Expected INSERT EDGES but got INSERT VERTICES".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        // 4. Verify that the edge type exists.
        self.validate_edge_type_exists(&edge_name)?;

        // 5. Verify the attribute names
        self.validate_property_names(&prop_names)?;

        // Get space_name for vid_type lookup
        let space_name = qctx.space_name();

        // 6. Verify each edge.
        let mut validated_edges = Vec::new();
        for (src, dst, rank, values) in &edges {
            self.validate_vertex_id_format(src, "source", space_name.as_deref())?;
            self.validate_vertex_id_format(dst, "destination", space_name.as_deref())?;
            self.validate_rank(rank)?;
            self.validate_values_count(&prop_names, values)?;
            self.validate_property_values(&edge_name, &prop_names, values)?;

            // Evaluate and convert
            let src_id = self.evaluate_expression(src)?;
            let dst_id = self.evaluate_expression(dst)?;
            let rank_val = self.evaluate_rank(rank)?;
            let mut value_list = Vec::new();
            for v in values {
                value_list.push(self.evaluate_expression(v)?);
            }

            validated_edges.push(ValidatedEdgeInsert {
                src_id,
                dst_id,
                rank: rank_val,
                values: value_list,
            });
        }

        // 7. Obtain the space_id
        let space_id = qctx.space_id().unwrap_or(0);

        // 8. Create the verification results
        let validated = ValidatedInsertEdges {
            space_id,
            edge_name: edge_name.clone(),
            edge_type_id: None,
            prop_names,
            edges: validated_edges,
            if_not_exists: insert_stmt.if_not_exists,
        };

        self.validated_result = Some(validated);

        // 9. Generate an output column
        self.generate_output_columns();

        // 10. Constructing detailed ValidationInfo
        let mut info = ValidationInfo::new();

        // Add semantic information
        if !info.semantic_info.referenced_edges.contains(&edge_name) {
            info.semantic_info.referenced_edges.push(edge_name.clone());
        }

        // 11. Return the verification results containing detailed information.
        Ok(ValidationResult::success_with_info(info))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::InsertEdges
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        // The `INSERT EDGES` command is not a global statement; it is necessary to select a specific space in advance before using it.
        false
    }

    fn expression_props(&self) -> &ExpressionProps {
        &self.expression_props
    }

    fn user_defined_vars(&self) -> &[String] {
        &self.user_defined_vars
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::expr::contextual::ContextualExpression;
    use crate::core::types::expr::Expression;
    use crate::query::parser::ast::stmt::InsertStmt;
    use crate::query::parser::ast::Span;
    use crate::query::QueryRequestContext;
    use crate::query::validator::context::expression_context::ExpressionAnalysisContext;
    use std::sync::Arc;

    fn create_contextual_expr(expr: Expression) -> ContextualExpression {
        let ctx = std::sync::Arc::new(ExpressionAnalysisContext::new());
        let meta = crate::core::types::expr::ExpressionMeta::new(expr);
        let id = ctx.register_expression(meta);
        ContextualExpression::new(id, ctx)
    }

    /// Create a QueryContext for testing purposes, which should contain a valid space_id.
    fn create_test_query_context() -> Arc<QueryContext> {
        let rctx = Arc::new(QueryRequestContext::new("TEST".to_string()));
        let mut qctx = QueryContext::new(rctx);
        let space_info = crate::core::types::SpaceInfo::new("test_space".to_string());
        qctx.set_space_info(space_info);
        Arc::new(qctx)
    }

    fn create_test_ast(stmt: Stmt) -> Arc<Ast> {
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        Arc::new(Ast::new(stmt, ctx))
    }

    fn create_insert_edge_stmt(
        edge_name: String,
        prop_names: Vec<String>,
        src: ContextualExpression,
        dst: ContextualExpression,
        rank: Option<ContextualExpression>,
        values: Vec<ContextualExpression>,
    ) -> InsertStmt {
        InsertStmt {
            span: Span::default(),
            target: InsertTarget::Edge {
                edge_name,
                prop_names,
                edges: vec![(src, dst, rank, values)],
            },
            if_not_exists: false,
        }
    }

    #[test]
    fn test_validate_edge_name_not_empty() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "".to_string(),
            vec!["prop".to_string()],
            create_contextual_expr(Expression::Literal(Value::String("v1".to_string()))),
            create_contextual_expr(Expression::Literal(Value::String("v2".to_string()))),
            None,
            vec![create_contextual_expr(Expression::Literal(Value::String(
                "value".to_string(),
            )))],
        );

        let qctx = create_test_query_context();
        let result = validator.validate(create_test_ast(Stmt::Insert(stmt)), qctx);
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
            create_contextual_expr(Expression::Literal(Value::String("v1".to_string()))),
            create_contextual_expr(Expression::Literal(Value::String("v2".to_string()))),
            None,
            vec![
                create_contextual_expr(Expression::Literal(Value::String("val1".to_string()))),
                create_contextual_expr(Expression::Literal(Value::String("val2".to_string()))),
            ],
        );

        let qctx = create_test_query_context();
        let result = validator.validate(create_test_ast(Stmt::Insert(stmt)), qctx);
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
            create_contextual_expr(Expression::Literal(Value::String("v1".to_string()))),
            create_contextual_expr(Expression::Literal(Value::String("v2".to_string()))),
            None,
            vec![create_contextual_expr(Expression::Literal(Value::String(
                "val1".to_string(),
            )))],
        );

        let qctx = create_test_query_context();
        let result = validator.validate(create_test_ast(Stmt::Insert(stmt)), qctx);
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
            create_contextual_expr(Expression::Literal(Value::String("".to_string()))),
            create_contextual_expr(Expression::Literal(Value::String("v2".to_string()))),
            None,
            vec![],
        );

        let qctx = create_test_query_context();
        let result = validator.validate(create_test_ast(Stmt::Insert(stmt)), qctx);
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
            create_contextual_expr(Expression::Literal(Value::String("v1".to_string()))),
            create_contextual_expr(Expression::Literal(Value::String("".to_string()))),
            None,
            vec![],
        );

        let qctx = create_test_query_context();
        let result = validator.validate(create_test_ast(Stmt::Insert(stmt)), qctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err
            .message
            .contains("destination vertex ID cannot be empty"));
    }

    #[test]
    fn test_validate_vertex_ids_valid() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec![],
            create_contextual_expr(Expression::Literal(Value::String("v1".to_string()))),
            create_contextual_expr(Expression::Literal(Value::String("v2".to_string()))),
            None,
            vec![],
        );

        let qctx = create_test_query_context();
        let result = validator.validate(create_test_ast(Stmt::Insert(stmt)), qctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_vertex_ids_variable() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec![],
            create_contextual_expr(Expression::Variable("$src".to_string())),
            create_contextual_expr(Expression::Variable("$dst".to_string())),
            None,
            vec![],
        );

        let qctx = create_test_query_context();
        let result = validator.validate(create_test_ast(Stmt::Insert(stmt)), qctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_source_vertex_id() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec![],
            create_contextual_expr(Expression::Literal(Value::Float(123.45))),
            create_contextual_expr(Expression::Literal(Value::String("v2".to_string()))),
            None,
            vec![],
        );

        let qctx = create_test_query_context();
        let result = validator.validate(create_test_ast(Stmt::Insert(stmt)), qctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err
            .message
            .contains("must be a string, integer, or variable"));
    }

    #[test]
    fn test_validate_rank_valid_integer() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec![],
            create_contextual_expr(Expression::Literal(Value::String("v1".to_string()))),
            create_contextual_expr(Expression::Literal(Value::String("v2".to_string()))),
            Some(create_contextual_expr(Expression::Literal(Value::Int(0)))),
            vec![],
        );

        let qctx = create_test_query_context();
        let result = validator.validate(create_test_ast(Stmt::Insert(stmt)), qctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_rank_valid_variable() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec![],
            create_contextual_expr(Expression::Literal(Value::String("v1".to_string()))),
            create_contextual_expr(Expression::Literal(Value::String("v2".to_string()))),
            Some(create_contextual_expr(Expression::Variable(
                "$rank".to_string(),
            ))),
            vec![],
        );

        let qctx = create_test_query_context();
        let result = validator.validate(create_test_ast(Stmt::Insert(stmt)), qctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_rank() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec![],
            create_contextual_expr(Expression::Literal(Value::String("v1".to_string()))),
            create_contextual_expr(Expression::Literal(Value::String("v2".to_string()))),
            Some(create_contextual_expr(Expression::Literal(Value::String(
                "invalid".to_string(),
            )))),
            vec![],
        );

        let qctx = create_test_query_context();
        let result = validator.validate(create_test_ast(Stmt::Insert(stmt)), qctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err
            .message
            .contains("Rank must be an integer constant or variable"));
    }

    #[test]
    fn test_validate_property_values() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec!["since".to_string(), "type".to_string()],
            create_contextual_expr(Expression::Literal(Value::String("v1".to_string()))),
            create_contextual_expr(Expression::Literal(Value::String("v2".to_string()))),
            None,
            vec![
                create_contextual_expr(Expression::Literal(Value::Int(2020))),
                create_contextual_expr(Expression::Literal(Value::String("best".to_string()))),
            ],
        );

        let qctx = create_test_query_context();
        let result = validator.validate(create_test_ast(Stmt::Insert(stmt)), qctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_wrong_target_type() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = InsertStmt {
            span: Span::default(),
            target: InsertTarget::Vertices {
                tags: vec![],
                values: vec![],
            },
            if_not_exists: false,
        };

        let qctx = create_test_query_context();
        let result = validator.validate(create_test_ast(Stmt::Insert(stmt)), qctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err
            .message
            .contains("Expected INSERT EDGES but got INSERT VERTICES"));
    }

    #[test]
    fn test_insert_edges_validator_trait_interface() {
        let validator = InsertEdgesValidator::new();

        assert_eq!(validator.statement_type(), StatementType::InsertEdges);
        assert!(validator.inputs().is_empty());
        assert!(validator.user_defined_vars().is_empty());
    }
}

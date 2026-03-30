//! Vertices Insert Statement Validator
//! Corresponding to the functionality of NebulaGraph InsertVerticesValidator
//! Verify the semantic correctness of the INSERT VERTICES statement; multiple tags can be inserted.

use std::collections::HashSet;
use std::sync::Arc;

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::types::expr::contextual::ContextualExpression;
use crate::core::Expression;
use crate::core::Value;
use crate::query::parser::ast::stmt::{Ast, InsertTarget, TagInsertSpec, VertexRow};
use crate::query::parser::ast::Stmt;
use crate::query::validator::structs::validation_info::ValidationInfo;
use crate::query::validator::validator_trait::{
    ColumnDef, ExpressionProps, StatementType, StatementValidator, ValidationResult, ValueType,
};
use crate::query::QueryContext;
use crate::storage::metadata::redb_schema_manager::RedbSchemaManager;

/// Verified vertex insertion information
#[derive(Debug, Clone)]
pub struct ValidatedInsertVertices {
    pub space_id: u64,
    pub tags: Vec<ValidatedTagInsert>,
    pub vertices: Vec<ValidatedVertex>,
    pub if_not_exists: bool,
}

/// Verified Tag insertion specifications
#[derive(Debug, Clone)]
pub struct ValidatedTagInsert {
    pub tag_id: i32,
    pub tag_name: String,
    pub prop_names: Vec<String>,
}

/// Verified individual vertex
#[derive(Debug, Clone)]
pub struct ValidatedVertex {
    pub vid: Value,
    pub tag_values: Vec<Vec<Value>>,
}

#[derive(Debug)]
pub struct InsertVerticesValidator {
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expression_props: ExpressionProps,
    user_defined_vars: Vec<String>,
    validated_result: Option<ValidatedInsertVertices>,
    schema_manager: Option<Arc<RedbSchemaManager>>,
}

impl InsertVerticesValidator {
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

    /// Verify the Tag name
    fn validate_tag_name(&self, tag_name: &str) -> Result<(), ValidationError> {
        if tag_name.is_empty() {
            return Err(ValidationError::new(
                "Tag name cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    /// Verify the attribute name
    fn validate_property_names(&self, prop_names: &[String]) -> Result<(), ValidationError> {
        let mut seen = HashSet::new();
        for prop_name in prop_names {
            if !seen.insert(prop_name) {
                return Err(ValidationError::new(
                    format!("Duplicate property name '{}' in INSERT VERTICES", prop_name),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    /// Verify the data in the vertex row.
    fn validate_vertex_rows(
        &self,
        tags: &[TagInsertSpec],
        rows: &[VertexRow],
    ) -> Result<(), ValidationError> {
        for (row_idx, row) in rows.iter().enumerate() {
            // Verify the VID format.
            self.validate_vid_expression(&row.vid, row_idx)?;

            // The number of verification values matches the number of tags.
            if row.tag_values.len() != tags.len() {
                return Err(ValidationError::new(
                    format!(
                        "Value count mismatch for vertex {}: expected {} tag value groups, got {}",
                        row_idx + 1,
                        tags.len(),
                        row.tag_values.len()
                    ),
                    ValidationErrorType::SemanticError,
                ));
            }

            // Verify the number of values for each Tag.
            for (tag_idx, (tag_spec, values)) in tags.iter().zip(row.tag_values.iter()).enumerate()
            {
                if values.len() != tag_spec.prop_names.len() {
                    return Err(ValidationError::new(
                        format!(
                            "Value count mismatch for vertex {}, tag {}: expected {} values, got {}",
                            row_idx + 1,
                            tag_idx + 1,
                            tag_spec.prop_names.len(),
                            values.len()
                        ),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
        }
        Ok(())
    }

    /// Verify the VID expression
    fn validate_vid_expression(
        &self,
        vid_expr: &ContextualExpression,
        idx: usize,
    ) -> Result<(), ValidationError> {
        self.validate_vid_expression_internal(vid_expr, idx)
    }

    /// Internal method: Verification of the VID expression
    fn validate_vid_expression_internal(
        &self,
        vid_expr: &ContextualExpression,
        idx: usize,
    ) -> Result<(), ValidationError> {
        let expr_meta = match vid_expr.expression() {
            Some(m) => m,
            None => {
                return Err(ValidationError::new(
                    format!("Vertex ID expression is invalid for vertex {}", idx + 1),
                    ValidationErrorType::SemanticError,
                ))
            }
        };
        let expr = expr_meta.inner();

        match expr {
            Expression::Literal(Value::String(s)) => {
                if s.is_empty() {
                    return Err(ValidationError::new(
                        format!("Vertex ID cannot be empty for vertex {}", idx + 1),
                        ValidationErrorType::SemanticError,
                    ));
                }
                Ok(())
            }
            Expression::Literal(Value::Int(_)) => Ok(()),
            Expression::Variable(_) => Ok(()),
            _ => Err(ValidationError::new(
                format!("Invalid vertex ID expression type for vertex {}", idx + 1),
                ValidationErrorType::SemanticError,
            )),
        }
    }

    /// Evaluating an expression to obtain a value
    fn evaluate_expression(&self, expr: &ContextualExpression) -> Result<Value, ValidationError> {
        if let Some(e) = expr.get_expression() {
            self.evaluate_expression_internal(&e)
        } else {
            Ok(Value::Null(crate::core::NullType::Null))
        }
    }

    /// Internal method: Evaluating an expression to determine its value
    fn evaluate_expression_internal(
        &self,
        expr: &crate::core::types::expr::Expression,
    ) -> Result<Value, ValidationError> {
        use crate::core::types::expr::Expression;

        match expr {
            Expression::Literal(val) => Ok(val.clone()),
            Expression::Variable(name) => {
                // Variables are parsed at runtime.
                Ok(Value::String(format!("${}", name)))
            }
            _ => Ok(Value::Null(crate::core::NullType::Null)),
        }
    }

    /// Generate a column of outputs.
    fn generate_output_columns(&mut self) {
        self.outputs.clear();
        self.outputs.push(ColumnDef {
            name: "INSERTED_VERTICES".to_string(),
            type_: ValueType::List,
        });
    }
}

impl Default for InsertVerticesValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementing the StatementValidator trait
///
/// # Refactoring Changes
/// The `validate` method accepts `Arc<Ast>` and `Arc<QueryContext>` as arguments.
impl StatementValidator for InsertVerticesValidator {
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

        // 3. Verify the type of the statement
        let (tags, values) = match &insert_stmt.target {
            InsertTarget::Vertices { tags, values } => {
                if tags.is_empty() {
                    return Err(ValidationError::new(
                        "INSERT VERTEX must specify at least one tag".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
                (tags.clone(), values.clone())
            }
            InsertTarget::Edge { .. } => {
                return Err(ValidationError::new(
                    "Expected INSERT VERTICES but got INSERT EDGES".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        // 4. Verify all tags.
        for tag_spec in &tags {
            self.validate_tag_name(&tag_spec.tag_name)?;
            self.validate_property_names(&tag_spec.prop_names)?;
        }

        // 5. Verify the data in the vertex row.
        self.validate_vertex_rows(&tags, &values)?;

        // 6. Convert the verified data
        let mut validated_tags = Vec::new();
        for tag_spec in &tags {
            validated_tags.push(ValidatedTagInsert {
                tag_id: 0, // Obtaining data from the schema at runtime
                tag_name: tag_spec.tag_name.clone(),
                prop_names: tag_spec.prop_names.clone(),
            });
        }

        let mut validated_vertices = Vec::new();
        for row in &values {
            let vid = self.evaluate_expression(&row.vid)?;
            let mut tag_values = Vec::new();
            for tag_vals in &row.tag_values {
                let mut values = Vec::new();
                for v in tag_vals {
                    values.push(self.evaluate_expression(v)?);
                }
                tag_values.push(values);
            }
            validated_vertices.push(ValidatedVertex { vid, tag_values });
        }

        // 7. Obtain the space_id
        let space_id = qctx.space_id().unwrap_or(0);

        // 8. Create the verification results
        let validated = ValidatedInsertVertices {
            space_id,
            tags: validated_tags.clone(),
            vertices: validated_vertices,
            if_not_exists: insert_stmt.if_not_exists,
        };

        self.validated_result = Some(validated);

        // 9. Generate the output column
        self.generate_output_columns();

        // 10. Constructing detailed ValidationInfo
        let mut info = ValidationInfo::new();

        // Add semantic information
        for tag in &validated_tags {
            if !info.semantic_info.referenced_tags.contains(&tag.tag_name) {
                info.semantic_info
                    .referenced_tags
                    .push(tag.tag_name.clone());
            }
        }

        // 11. Return the verification results containing detailed information.
        Ok(ValidationResult::success_with_info(info))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::InsertVertices
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        // The `INSERT VERTICES` command is not a global statement; therefore, the space (the database or table in which the vertices are to be inserted) must be selected in advance.
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
    use crate::core::Expression;
    use crate::core::Value;
    use crate::query::parser::ast::stmt::InsertStmt;
    use crate::query::parser::ast::Span;
    use crate::query::query_request_context::QueryRequestContext;
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

    fn create_insert_vertices_stmt(
        tags: Vec<TagInsertSpec>,
        values: Vec<VertexRow>,
        if_not_exists: bool,
    ) -> InsertStmt {
        InsertStmt {
            span: Span::default(),
            target: InsertTarget::Vertices { tags, values },
            if_not_exists,
        }
    }

    fn create_tag_spec(tag_name: &str, prop_names: Vec<&str>) -> TagInsertSpec {
        TagInsertSpec {
            tag_name: tag_name.to_string(),
            prop_names: prop_names.iter().map(|s| s.to_string()).collect(),
            is_default_props: false,
        }
    }

    fn create_vertex_row(
        vid: ContextualExpression,
        tag_values: Vec<Vec<ContextualExpression>>,
    ) -> VertexRow {
        VertexRow { vid, tag_values }
    }

    #[test]
    fn test_validate_empty_tags() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_vertices_stmt(vec![], vec![], false);

        let qctx = create_test_query_context();
        let result = validator.validate(create_test_ast(Stmt::Insert(stmt)), qctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err
            .message
            .contains("INSERT VERTEX must specify at least one tag"));
    }

    #[test]
    fn test_validate_empty_tag_name() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_vertices_stmt(
            vec![create_tag_spec("", vec!["name"])],
            vec![create_vertex_row(
                create_contextual_expr(Expression::Literal(Value::String("vid1".to_string()))),
                vec![vec![create_contextual_expr(Expression::Literal(
                    Value::String("Alice".to_string()),
                ))]],
            )],
            false,
        );

        let qctx = create_test_query_context();
        let result = validator.validate(create_test_ast(Stmt::Insert(stmt)), qctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Tag name cannot be empty"));
    }

    #[test]
    fn test_validate_duplicate_property_names() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_vertices_stmt(
            vec![create_tag_spec("person", vec!["name", "name"])],
            vec![create_vertex_row(
                create_contextual_expr(Expression::Literal(Value::String("vid1".to_string()))),
                vec![vec![
                    create_contextual_expr(Expression::Literal(Value::String("Alice".to_string()))),
                    create_contextual_expr(Expression::Literal(Value::String("Bob".to_string()))),
                ]],
            )],
            false,
        );

        let qctx = create_test_query_context();
        let result = validator.validate(create_test_ast(Stmt::Insert(stmt)), qctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Duplicate property name"));
    }

    #[test]
    fn test_validate_value_count_mismatch() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_vertices_stmt(
            vec![create_tag_spec("person", vec!["name", "age"])],
            vec![create_vertex_row(
                create_contextual_expr(Expression::Literal(Value::String("vid1".to_string()))),
                vec![vec![create_contextual_expr(Expression::Literal(
                    Value::String("Alice".to_string()),
                ))]],
            )],
            false,
        );

        let qctx = create_test_query_context();
        let result = validator.validate(create_test_ast(Stmt::Insert(stmt)), qctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Value count mismatch"));
    }

    #[test]
    fn test_validate_empty_vid() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_vertices_stmt(
            vec![create_tag_spec("person", vec!["name"])],
            vec![create_vertex_row(
                create_contextual_expr(Expression::Literal(Value::String("".to_string()))),
                vec![vec![create_contextual_expr(Expression::Literal(
                    Value::String("Alice".to_string()),
                ))]],
            )],
            false,
        );

        let qctx = create_test_query_context();
        let result = validator.validate(create_test_ast(Stmt::Insert(stmt)), qctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Vertex ID cannot be empty"));
    }

    #[test]
    fn test_validate_valid_single_tag() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_vertices_stmt(
            vec![create_tag_spec("person", vec!["name", "age"])],
            vec![create_vertex_row(
                create_contextual_expr(Expression::Literal(Value::String("vid1".to_string()))),
                vec![vec![
                    create_contextual_expr(Expression::Literal(Value::String("Alice".to_string()))),
                    create_contextual_expr(Expression::Literal(Value::Int(30))),
                ]],
            )],
            false,
        );

        let qctx = create_test_query_context();
        let result = validator.validate(create_test_ast(Stmt::Insert(stmt)), qctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_valid_multiple_tags() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_vertices_stmt(
            vec![
                create_tag_spec("person", vec!["name"]),
                create_tag_spec("employee", vec!["department", "salary"]),
            ],
            vec![create_vertex_row(
                create_contextual_expr(Expression::Literal(Value::String("vid1".to_string()))),
                vec![
                    vec![create_contextual_expr(Expression::Literal(Value::String(
                        "Alice".to_string(),
                    )))],
                    vec![
                        create_contextual_expr(Expression::Literal(Value::String(
                            "Engineering".to_string(),
                        ))),
                        create_contextual_expr(Expression::Literal(Value::Int(50000))),
                    ],
                ],
            )],
            false,
        );

        let qctx = create_test_query_context();
        let result = validator.validate(create_test_ast(Stmt::Insert(stmt)), qctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_multiple_vertices() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_vertices_stmt(
            vec![create_tag_spec("person", vec!["name"])],
            vec![
                create_vertex_row(
                    create_contextual_expr(Expression::Literal(Value::String("vid1".to_string()))),
                    vec![vec![create_contextual_expr(Expression::Literal(
                        Value::String("Alice".to_string()),
                    ))]],
                ),
                create_vertex_row(
                    create_contextual_expr(Expression::Literal(Value::String("vid2".to_string()))),
                    vec![vec![create_contextual_expr(Expression::Literal(
                        Value::String("Bob".to_string()),
                    ))]],
                ),
            ],
            false,
        );

        let qctx = create_test_query_context();
        let result = validator.validate(create_test_ast(Stmt::Insert(stmt)), qctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_variable_vid() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_vertices_stmt(
            vec![create_tag_spec("person", vec!["name"])],
            vec![create_vertex_row(
                create_contextual_expr(Expression::Variable("$vid".to_string())),
                vec![vec![create_contextual_expr(Expression::Literal(
                    Value::String("Alice".to_string()),
                ))]],
            )],
            false,
        );

        let qctx = create_test_query_context();
        let result = validator.validate(create_test_ast(Stmt::Insert(stmt)), qctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_integer_vid() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_vertices_stmt(
            vec![create_tag_spec("person", vec!["name"])],
            vec![create_vertex_row(
                create_contextual_expr(Expression::Literal(Value::Int(123))),
                vec![vec![create_contextual_expr(Expression::Literal(
                    Value::String("Alice".to_string()),
                ))]],
            )],
            false,
        );

        let qctx = create_test_query_context();
        let result = validator.validate(create_test_ast(Stmt::Insert(stmt)), qctx);
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
                edges: vec![],
            },
            if_not_exists: false,
        };

        let qctx = create_test_query_context();
        let result = validator.validate(create_test_ast(Stmt::Insert(stmt)), qctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "Expected INSERT VERTICES but got INSERT EDGES");
    }

    #[test]
    fn test_insert_vertices_validator_trait_interface() {
        let validator = InsertVerticesValidator::new();

        assert_eq!(validator.statement_type(), StatementType::InsertVertices);
        assert!(validator.inputs().is_empty());
        assert!(validator.user_defined_vars().is_empty());
    }

    #[test]
    fn test_validate_if_not_exists() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_vertices_stmt(
            vec![create_tag_spec("person", vec!["name"])],
            vec![create_vertex_row(
                create_contextual_expr(Expression::Literal(Value::String("vid1".to_string()))),
                vec![vec![create_contextual_expr(Expression::Literal(
                    Value::String("Alice".to_string()),
                ))]],
            )],
            true, // if_not_exists = true
        );

        let qctx = create_test_query_context();
        let result = validator.validate(create_test_ast(Stmt::Insert(stmt)), qctx);
        assert!(result.is_ok());

        // Verify whether `if_not_exists` has been saved correctly.
        assert!(
            validator
                .validated_result
                .as_ref()
                .expect("Failed to get validated result")
                .if_not_exists
        );
    }
}

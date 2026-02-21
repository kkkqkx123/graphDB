//! Insert Vertices 语句验证器
//! 对应 NebulaGraph InsertVerticesValidator 的功能
//! 验证 INSERT VERTICES 语句的语义正确性，支持多 Tag 插入

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::{Expression, Value};
use crate::query::context::ast::AstContext;
use crate::query::parser::ast::stmt::{InsertTarget, TagInsertSpec, VertexRow};
use crate::query::parser::ast::Stmt;
use crate::query::validator::validator_trait::{
    ColumnDef, ExpressionProps, StatementType, StatementValidator, ValidationResult, ValueType,
};
use std::collections::HashSet;
use std::sync::Arc;
use crate::storage::metadata::schema_manager::SchemaManager;

/// 验证后的顶点插入信息
#[derive(Debug, Clone)]
pub struct ValidatedInsertVertices {
    pub space_id: u64,
    pub tags: Vec<ValidatedTagInsert>,
    pub vertices: Vec<ValidatedVertex>,
    pub if_not_exists: bool,
}

/// 验证后的 Tag 插入规范
#[derive(Debug, Clone)]
pub struct ValidatedTagInsert {
    pub tag_id: i32,
    pub tag_name: String,
    pub prop_names: Vec<String>,
}

/// 验证后的单个顶点
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
    schema_manager: Option<Arc<dyn SchemaManager>>,
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

    pub fn with_schema_manager(mut self, schema_manager: Arc<dyn SchemaManager>) -> Self {
        self.schema_manager = Some(schema_manager);
        self
    }

    /// 验证 Tag 名称
    fn validate_tag_name(&self, tag_name: &str) -> Result<(), ValidationError> {
        if tag_name.is_empty() {
            return Err(ValidationError::new(
                "Tag name cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    /// 验证属性名
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

    /// 验证顶点行数据
    fn validate_vertex_rows(
        &self,
        tags: &[TagInsertSpec],
        rows: &[VertexRow],
    ) -> Result<(), ValidationError> {
        for (row_idx, row) in rows.iter().enumerate() {
            // 验证 VID 格式
            self.validate_vid_expression(&row.vid, row_idx)?;

            // 验证值数量与 Tag 数量匹配
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

            // 验证每个 Tag 的值数量
            for (tag_idx, (tag_spec, values)) in
                tags.iter().zip(row.tag_values.iter()).enumerate() {
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

    /// 验证 VID 表达式
    fn validate_vid_expression(
        &self,
        vid_expr: &Expression,
        idx: usize,
    ) -> Result<(), ValidationError> {
        match vid_expr {
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
                format!(
                    "Vertex ID must be a string constant or variable for vertex {}",
                    idx + 1
                ),
                ValidationErrorType::SemanticError,
            )),
        }
    }

    /// 评估表达式为值
    fn evaluate_expression(&self, expr: &Expression) -> Result<Value, ValidationError> {
        match expr {
            Expression::Literal(val) => Ok(val.clone()),
            Expression::Variable(name) => {
                // 变量在运行时解析
                Ok(Value::String(format!("${}", name)))
            }
            _ => Ok(Value::Null(crate::core::NullType::Null)),
        }
    }

    /// 生成输出列
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

impl StatementValidator for InsertVerticesValidator {
    fn validate(&mut self, ast: &mut AstContext) -> Result<ValidationResult, ValidationError> {
        // 1. 检查是否需要空间
        let query_context = ast.query_context();
        if !self.is_global_statement() && query_context.is_none() {
            return Err(ValidationError::new(
                "未选择图空间，请先执行 USE <space>".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 2. 获取 INSERT 语句
        let stmt = ast.sentence()
            .ok_or_else(|| ValidationError::new(
                "No statement found in AST context".to_string(),
                ValidationErrorType::SemanticError,
            ))?;

        let insert_stmt = match stmt {
            Stmt::Insert(insert_stmt) => insert_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected INSERT statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        // 3. 验证语句类型
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

        // 4. 验证所有 Tag
        for tag_spec in &tags {
            self.validate_tag_name(&tag_spec.tag_name)?;
            self.validate_property_names(&tag_spec.prop_names)?;
        }

        // 5. 验证顶点行数据
        self.validate_vertex_rows(&tags, &values)?;

        // 6. 转换验证后的数据
        let mut validated_tags = Vec::new();
        for tag_spec in &tags {
            validated_tags.push(ValidatedTagInsert {
                tag_id: 0, // 运行时从 schema 获取
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

        // 7. 获取 space_id
        let space_id = query_context
            .map(|qc| qc.space_id())
            .filter(|&id| id != 0)
            .or_else(|| ast.space().space_id.map(|id| id as u64))
            .unwrap_or(0);

        // 8. 创建验证结果
        let validated = ValidatedInsertVertices {
            space_id,
            tags: validated_tags,
            vertices: validated_vertices,
            if_not_exists: insert_stmt.if_not_exists,
        };

        self.validated_result = Some(validated);

        // 9. 生成输出列
        self.generate_output_columns();

        // 10. 返回验证结果
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
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
        // INSERT VERTICES 不是全局语句，需要预先选择空间
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
    use crate::query::parser::ast::stmt::InsertStmt;
    use crate::query::parser::ast::Span;

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

    fn create_vertex_row(vid: Expression, tag_values: Vec<Vec<Expression>>) -> VertexRow {
        VertexRow { vid, tag_values }
    }

    #[test]
    fn test_validate_empty_tags() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_vertices_stmt(
            vec![],
            vec![],
            false,
        );

        let mut ast = AstContext::default();
        ast.set_sentence(Stmt::Insert(stmt));

        let result = validator.validate(&mut ast);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("INSERT VERTEX must specify at least one tag"));
    }

    #[test]
    fn test_validate_empty_tag_name() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_vertices_stmt(
            vec![create_tag_spec("", vec!["name"])],
            vec![create_vertex_row(
                Expression::literal("vid1"),
                vec![vec![Expression::literal("Alice")]],
            )],
            false,
        );

        let mut ast = AstContext::default();
        ast.set_sentence(Stmt::Insert(stmt));

        let result = validator.validate(&mut ast);
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
                Expression::literal("vid1"),
                vec![vec![Expression::literal("Alice"), Expression::literal("Bob")]],
            )],
            false,
        );

        let mut ast = AstContext::default();
        ast.set_sentence(Stmt::Insert(stmt));

        let result = validator.validate(&mut ast);
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
                Expression::literal("vid1"),
                vec![vec![Expression::literal("Alice")]], // 只提供了一个值，但期望两个
            )],
            false,
        );

        let mut ast = AstContext::default();
        ast.set_sentence(Stmt::Insert(stmt));

        let result = validator.validate(&mut ast);
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
                Expression::literal(""),
                vec![vec![Expression::literal("Alice")]],
            )],
            false,
        );

        let mut ast = AstContext::default();
        ast.set_sentence(Stmt::Insert(stmt));

        let result = validator.validate(&mut ast);
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
                Expression::literal("vid1"),
                vec![vec![Expression::literal("Alice"), Expression::literal(30)]],
            )],
            false,
        );

        let mut ast = AstContext::default();
        ast.set_sentence(Stmt::Insert(stmt));

        let result = validator.validate(&mut ast);
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
                Expression::literal("vid1"),
                vec![
                    vec![Expression::literal("Alice")],
                    vec![Expression::literal("Engineering"), Expression::literal(50000)],
                ],
            )],
            false,
        );

        let mut ast = AstContext::default();
        ast.set_sentence(Stmt::Insert(stmt));

        let result = validator.validate(&mut ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_multiple_vertices() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_vertices_stmt(
            vec![create_tag_spec("person", vec!["name"])],
            vec![
                create_vertex_row(
                    Expression::literal("vid1"),
                    vec![vec![Expression::literal("Alice")]],
                ),
                create_vertex_row(
                    Expression::literal("vid2"),
                    vec![vec![Expression::literal("Bob")]],
                ),
            ],
            false,
        );

        let mut ast = AstContext::default();
        ast.set_sentence(Stmt::Insert(stmt));

        let result = validator.validate(&mut ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_variable_vid() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_vertices_stmt(
            vec![create_tag_spec("person", vec!["name"])],
            vec![create_vertex_row(
                Expression::variable("$vid"),
                vec![vec![Expression::literal("Alice")]],
            )],
            false,
        );

        let mut ast = AstContext::default();
        ast.set_sentence(Stmt::Insert(stmt));

        let result = validator.validate(&mut ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_integer_vid() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_insert_vertices_stmt(
            vec![create_tag_spec("person", vec!["name"])],
            vec![create_vertex_row(
                Expression::literal(123),
                vec![vec![Expression::literal("Alice")]],
            )],
            false,
        );

        let mut ast = AstContext::default();
        ast.set_sentence(Stmt::Insert(stmt));

        let result = validator.validate(&mut ast);
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

        let mut ast = AstContext::default();
        ast.set_sentence(Stmt::Insert(stmt));

        let result = validator.validate(&mut ast);
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
                Expression::literal("vid1"),
                vec![vec![Expression::literal("Alice")]],
            )],
            true, // if_not_exists = true
        );

        let mut ast = AstContext::default();
        ast.set_sentence(Stmt::Insert(stmt));

        let result = validator.validate(&mut ast);
        assert!(result.is_ok());

        // 验证 if_not_exists 被正确保存
        assert!(validator.validated_result.as_ref().unwrap().if_not_exists);
    }
}

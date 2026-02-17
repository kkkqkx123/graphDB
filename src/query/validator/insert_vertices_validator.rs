//! Insert Vertices 语句验证器（增强版）
//! 对应 NebulaGraph InsertVerticesValidator 的功能
//! 验证 INSERT VERTICES 语句的语义正确性，支持多 Tag 插入

use crate::core::error::{DBResult, ValidationError as CoreValidationError, ValidationErrorType};
use crate::core::Value;
use crate::query::context::ast::AstContext;
use crate::query::context::execution::QueryContext;
use crate::query::parser::ast::stmt::{InsertStmt, InsertTarget, TagInsertSpec, VertexRow};
use crate::query::validator::base_validator::{Validator, ValueType};
use crate::query::validator::schema_validator::SchemaValidator;
use crate::storage::metadata::schema_manager::SchemaManager;

/// 验证后的顶点插入信息
#[derive(Debug, Clone)]
pub struct ValidatedVertexInsert {
    pub space_id: i32,
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

pub struct InsertVerticesValidator<'a> {
    base: Validator,
    schema_validator: Option<SchemaValidator<'a>>,
}

impl<'a> InsertVerticesValidator<'a> {
    pub fn new() -> Self {
        Self {
            base: Validator::new(),
            schema_validator: None,
        }
    }

    pub fn with_schema_manager(mut self, schema_manager: &'a dyn SchemaManager) -> Self {
        self.schema_validator = Some(SchemaValidator::new(schema_manager));
        self
    }

    /// 验证 INSERT VERTICES 语句
    /// 返回验证后的插入信息，供执行层使用
    pub fn validate_with_schema(
        &mut self,
        stmt: &InsertStmt,
        space_name: &str,
    ) -> Result<ValidatedVertexInsert, CoreValidationError> {
        let schema_validator = self.schema_validator.as_ref().ok_or_else(|| {
            CoreValidationError::new(
                "Schema validator not initialized".to_string(),
                ValidationErrorType::SemanticError,
            )
        })?;

        let space = schema_validator
            .schema_manager
            .get_space(space_name)
            .map_err(|e| {
                CoreValidationError::new(
                    format!("Failed to get space '{}': {}", space_name, e),
                    ValidationErrorType::SemanticError,
                )
            })?
            .ok_or_else(|| {
                CoreValidationError::new(
                    format!("Space '{}' not found", space_name),
                    ValidationErrorType::SemanticError,
                )
            })?;

        match &stmt.target {
            InsertTarget::Vertices { tags, values } => {
                // 验证所有 Tag
                let mut validated_tags = Vec::new();
                for tag_spec in tags {
                    let validated_tag = self.validate_tag_spec(
                        tag_spec,
                        space_name,
                        schema_validator,
                    )?;
                    validated_tags.push(validated_tag);
                }

                // 验证并转换所有顶点数据
                let mut validated_vertices = Vec::new();
                for (idx, row) in values.iter().enumerate() {
                    // 验证 VID
                    let vid = self.validate_and_evaluate_vid(
                        &row.vid,
                        &space.vid_type,
                        schema_validator,
                        idx,
                    )?;

                    // 验证每个 Tag 的属性值
                    let mut tag_values = Vec::new();
                    for (tag_idx, (tag_spec, validated_tag)) in 
                        row.tag_values.iter().zip(validated_tags.iter()).enumerate() {
                        let values = self.validate_and_convert_props(
                            &validated_tag.prop_names,
                            tag_spec,
                            schema_validator,
                            idx,
                            tag_idx,
                        )?;
                        tag_values.push(values);
                    }

                    validated_vertices.push(ValidatedVertex { vid, tag_values });
                }

                Ok(ValidatedVertexInsert {
                    space_id: space.space_id,
                    tags: validated_tags,
                    vertices: validated_vertices,
                    if_not_exists: stmt.if_not_exists,
                })
            }
            InsertTarget::Edge { .. } => Err(CoreValidationError::new(
                "Expected INSERT VERTICES but got INSERT EDGES".to_string(),
                ValidationErrorType::SemanticError,
            )),
        }
    }

    /// 验证 Tag 规范
    fn validate_tag_spec(
        &self,
        tag_spec: &TagInsertSpec,
        space_name: &str,
        schema_validator: &SchemaValidator,
    ) -> Result<ValidatedTagInsert, CoreValidationError> {
        let tag_info = schema_validator
            .get_tag(space_name, &tag_spec.tag_name)
            .map_err(|e| {
                CoreValidationError::new(
                    format!("Failed to get tag '{}': {}", tag_spec.tag_name, e),
                    ValidationErrorType::SemanticError,
                )
            })?
            .ok_or_else(|| {
                CoreValidationError::new(
                    format!("Tag '{}' not found in space '{}'", tag_spec.tag_name, space_name),
                    ValidationErrorType::SemanticError,
                )
            })?;

        // 验证属性名
        self.validate_property_names_with_schema(&tag_info.properties, &tag_spec.prop_names)?;

        Ok(ValidatedTagInsert {
            tag_id: tag_info.tag_id,
            tag_name: tag_spec.tag_name.clone(),
            prop_names: tag_spec.prop_names.clone(),
        })
    }

    /// 基础验证（不依赖 Schema）
    pub fn validate(&mut self, stmt: &InsertStmt) -> Result<(), CoreValidationError> {
        match &stmt.target {
            InsertTarget::Vertices { tags, values } => {
                if tags.is_empty() {
                    return Err(CoreValidationError::new(
                        "INSERT VERTEX must specify at least one tag".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
                
                for tag_spec in tags {
                    self.validate_tag_name(&tag_spec.tag_name)?;
                    self.validate_property_names_basic(&tag_spec.prop_names)?;
                }
                
                self.validate_vertex_rows(tags, values)?;
            }
            InsertTarget::Edge { .. } => {
                return Err(CoreValidationError::new(
                    "Expected INSERT VERTICES but got INSERT EDGES".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    /// 完整验证（包含 AST 上下文）
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

    fn validate_tag_name(&self, tag_name: &str) -> Result<(), CoreValidationError> {
        if tag_name.is_empty() {
            return Err(CoreValidationError::new(
                "Tag name cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    /// 验证顶点行数据
    fn validate_vertex_rows(
        &self,
        tags: &[TagInsertSpec],
        rows: &[VertexRow],
    ) -> Result<(), CoreValidationError> {
        for (row_idx, row) in rows.iter().enumerate() {
            // 验证 VID 格式
            self.validate_vid_expression(&row.vid, row_idx)?;
            
            // 验证值数量与 Tag 数量匹配
            if row.tag_values.len() != tags.len() {
                return Err(CoreValidationError::new(
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
                    return Err(CoreValidationError::new(
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

    fn validate_vid_expression(
        &self,
        vid_expr: &crate::core::Expression,
        idx: usize,
    ) -> Result<(), CoreValidationError> {
        match vid_expr {
            crate::core::Expression::Literal(crate::core::Value::String(s)) => {
                if s.is_empty() {
                    return Err(CoreValidationError::new(
                        format!("Vertex ID cannot be empty for vertex {}", idx + 1),
                        ValidationErrorType::SemanticError,
                    ));
                }
                Ok(())
            }
            crate::core::Expression::Literal(crate::core::Value::Int(_)) => Ok(()),
            crate::core::Expression::Variable(_) => Ok(()),
            _ => Err(CoreValidationError::new(
                format!(
                    "Vertex ID must be a string constant or variable for vertex {}",
                    idx + 1
                ),
                ValidationErrorType::SemanticError,
            )),
        }
    }

    /// 使用 Schema 验证属性名
    fn validate_property_names_with_schema(
        &self,
        schema_props: &[crate::core::types::PropertyDef],
        prop_names: &[String],
    ) -> Result<(), CoreValidationError> {
        // 检查重复属性名
        let mut seen = std::collections::HashSet::new();
        for prop_name in prop_names {
            if !seen.insert(prop_name) {
                return Err(CoreValidationError::new(
                    format!("Duplicate property name '{}' in INSERT VERTICES", prop_name),
                    ValidationErrorType::SemanticError,
                ));
            }

            // 检查属性是否存在于 Schema 中
            if !schema_props.iter().any(|p| &p.name == prop_name) {
                return Err(CoreValidationError::new(
                    format!(
                        "Property '{}' does not exist in tag schema",
                        prop_name
                    ),
                    ValidationErrorType::SemanticError,
                ));
            }
        }

        Ok(())
    }

    fn validate_property_names_basic(
        &self,
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

    /// 验证并评估 VID
    fn validate_and_evaluate_vid(
        &self,
        vid_expr: &crate::core::Expression,
        vid_type: &crate::core::types::DataType,
        schema_validator: &SchemaValidator,
        vertex_idx: usize,
    ) -> Result<Value, CoreValidationError> {
        // 评估表达式为值
        let vid = schema_validator
            .evaluate_expression(vid_expr)
            .map_err(|e| {
                CoreValidationError::new(
                    format!("Failed to evaluate vertex ID for vertex {}: {}", vertex_idx + 1, e.message),
                    e.error_type,
                )
            })?;

        // 验证 VID 类型
        schema_validator
            .validate_vid(&vid, vid_type)
            .map_err(|e| {
                CoreValidationError::new(
                    format!("Invalid vertex ID for vertex {}: {}", vertex_idx + 1, e.message),
                    e.error_type,
                )
            })?;

        Ok(vid)
    }

    /// 验证并转换属性值
    fn validate_and_convert_props(
        &self,
        prop_names: &[String],
        prop_values: &[crate::core::Expression],
        schema_validator: &SchemaValidator,
        vertex_idx: usize,
        tag_idx: usize,
    ) -> Result<Vec<Value>, CoreValidationError> {
        let mut result = Vec::new();

        for (_prop_idx, (prop_name, value_expr)) in
            prop_names.iter().zip(prop_values.iter()).enumerate()
        {
            // 评估表达式
            let value = schema_validator
                .evaluate_expression(value_expr)
                .map_err(|e| {
                    CoreValidationError::new(
                        format!(
                            "Failed to evaluate property '{}' for vertex {}, tag {}: {}",
                            prop_name,
                            vertex_idx + 1,
                            tag_idx + 1,
                            e.message
                        ),
                        e.error_type,
                    )
                })?;

            result.push(value);
        }

        Ok(result)
    }

    fn generate_output_columns(&mut self, _ast: &mut AstContext) {
        self.base.add_output("INSERTED_VERTICES".to_string(), ValueType::List);
    }
}

impl Default for InsertVerticesValidator<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{DataType, PropertyDef, TagInfo};
    use crate::query::parser::Span;

    // 模拟 SchemaManager 用于测试
    #[derive(Debug)]
    #[allow(dead_code)]
    struct MockSchemaManager;

    impl SchemaManager for MockSchemaManager {
        fn create_space(
            &self,
            _space: &crate::core::types::SpaceInfo,
        ) -> crate::storage::StorageResult<bool> {
            Ok(true)
        }
        fn drop_space(&self, _space_name: &str) -> crate::storage::StorageResult<bool> {
            Ok(true)
        }
        fn get_space(
            &self,
            _space_name: &str,
        ) -> crate::storage::StorageResult<Option<crate::core::types::SpaceInfo>> {
            Ok(Some(crate::core::types::SpaceInfo {
                space_id: 1,
                space_name: "test_space".to_string(),
                partition_num: 1,
                replica_factor: 1,
                vid_type: DataType::String,
                tags: vec![],
                edge_types: vec![],
                version: crate::core::types::metadata::MetadataVersion {
                    version: 1,
                    timestamp: 0,
                    description: String::new(),
                },
                comment: None,
            }))
        }
        fn get_space_by_id(
            &self,
            _space_id: i32,
        ) -> crate::storage::StorageResult<Option<crate::core::types::SpaceInfo>> {
            Ok(None)
        }
        fn list_spaces(&self) -> crate::storage::StorageResult<Vec<crate::core::types::SpaceInfo>> {
            Ok(vec![])
        }
        fn create_tag(&self, _space: &str, _tag: &TagInfo) -> crate::storage::StorageResult<bool> {
            Ok(true)
        }
        fn get_tag(
            &self,
            _space: &str,
            tag_name: &str,
        ) -> crate::storage::StorageResult<Option<TagInfo>> {
            if tag_name == "person" {
                Ok(Some(TagInfo {
                    tag_id: 1,
                    tag_name: "person".to_string(),
                    properties: vec![
                        PropertyDef::new("name".to_string(), DataType::String).with_nullable(false),
                        PropertyDef::new("age".to_string(), DataType::Int).with_nullable(true),
                    ],
                    comment: None,
                }))
            } else if tag_name == "employee" {
                Ok(Some(TagInfo {
                    tag_id: 2,
                    tag_name: "employee".to_string(),
                    properties: vec![
                        PropertyDef::new("department".to_string(), DataType::String).with_nullable(false),
                        PropertyDef::new("salary".to_string(), DataType::Int).with_nullable(true),
                    ],
                    comment: None,
                }))
            } else {
                Ok(None)
            }
        }
        fn drop_tag(&self, _space: &str, _tag_name: &str) -> crate::storage::StorageResult<bool> {
            Ok(true)
        }
        fn list_tags(&self, _space: &str) -> crate::storage::StorageResult<Vec<TagInfo>> {
            Ok(vec![])
        }
        fn create_edge_type(
            &self,
            _space: &str,
            _edge: &crate::core::types::EdgeTypeInfo,
        ) -> crate::storage::StorageResult<bool> {
            Ok(true)
        }
        fn get_edge_type(
            &self,
            _space: &str,
            _edge_name: &str,
        ) -> crate::storage::StorageResult<Option<crate::core::types::EdgeTypeInfo>> {
            Ok(None)
        }
        fn drop_edge_type(&self, _space: &str, _edge_name: &str) -> crate::storage::StorageResult<bool> {
            Ok(true)
        }
        fn list_edge_types(&self, _space: &str) -> crate::storage::StorageResult<Vec<crate::core::types::EdgeTypeInfo>> {
            Ok(vec![])
        }
        fn get_tag_schema(&self, _space: &str, tag: &str) -> crate::storage::StorageResult<crate::storage::Schema> {
            Ok(crate::storage::Schema::new(tag.to_string(), 1))
        }
        fn get_edge_type_schema(&self, _space: &str, edge: &str) -> crate::storage::StorageResult<crate::storage::Schema> {
            Ok(crate::storage::Schema::new(edge.to_string(), 1))
        }
    }

    fn create_test_stmt(if_not_exists: bool) -> InsertStmt {
        InsertStmt {
            span: Span::default(),
            target: InsertTarget::Vertices {
                tags: vec![
                    TagInsertSpec {
                        tag_name: "person".to_string(),
                        prop_names: vec!["name".to_string(), "age".to_string()],
                        is_default_props: false,
                    },
                ],
                values: vec![
                    VertexRow {
                        vid: crate::core::Expression::literal("vid1"),
                        tag_values: vec![
                            vec![
                                crate::core::Expression::literal("Alice"),
                                crate::core::Expression::literal(30i64),
                            ],
                        ],
                    },
                ],
            },
            if_not_exists,
        }
    }

    #[test]
    fn test_validate_single_tag() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_test_stmt(false);
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_if_not_exists() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = create_test_stmt(true);
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_empty_tag_name() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = InsertStmt {
            span: Span::default(),
            target: InsertTarget::Vertices {
                tags: vec![
                    TagInsertSpec {
                        tag_name: "".to_string(),
                        prop_names: vec![],
                        is_default_props: true,
                    },
                ],
                values: vec![],
            },
            if_not_exists: false,
        };
        let result = validator.validate(&stmt);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_duplicate_property() {
        let mut validator = InsertVerticesValidator::new();
        let stmt = InsertStmt {
            span: Span::default(),
            target: InsertTarget::Vertices {
                tags: vec![
                    TagInsertSpec {
                        tag_name: "person".to_string(),
                        prop_names: vec!["name".to_string(), "name".to_string()],
                        is_default_props: false,
                    },
                ],
                values: vec![],
            },
            if_not_exists: false,
        };
        let result = validator.validate(&stmt);
        assert!(result.is_err());
    }
}

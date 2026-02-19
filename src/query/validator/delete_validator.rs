//! Delete 语句验证器（增强版）
//! 对应 NebulaGraph DeleteValidator 的功能
//! 验证 DELETE 语句的语义正确性

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::{Expression, Value};
use crate::query::context::validate::ValidationContext;
use crate::query::parser::ast::stmt::{DeleteStmt, DeleteTarget};
use crate::query::validator::core::{ColumnDef, StatementType, StatementValidator};
use crate::query::validator::schema_validator::SchemaValidator;
use crate::storage::metadata::schema_manager::SchemaManager;

/// 验证后的删除信息
#[derive(Debug, Clone)]
pub struct ValidatedDelete {
    pub space_id: i32,
    pub target_type: DeleteTargetType,
    pub with_edge: bool,
    pub where_clause: Option<Expression>,
}

/// 删除目标类型
#[derive(Debug, Clone)]
pub enum DeleteTargetType {
    Vertices(Vec<Value>),
    Edges {
        edge_type: Option<String>,
        edge_type_id: Option<i32>,
        edges: Vec<EdgeKey>,
    },
    Tags {
        tag_names: Vec<String>,
        tag_ids: Vec<i32>,
        vertex_ids: Vec<Value>,
    },
    Index(String),
}

/// 边的唯一标识
#[derive(Debug, Clone)]
pub struct EdgeKey {
    pub src: Value,
    pub dst: Value,
    pub rank: i64,
}

/// DELETE 语句验证器
pub struct DeleteValidator<'a> {
    schema_validator: Option<SchemaValidator<'a>>,
    stmt: Option<DeleteStmt>,
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    validated_result: Option<ValidatedDelete>,
}

impl<'a> DeleteValidator<'a> {
    /// 创建新的验证器
    pub fn new() -> Self {
        Self {
            schema_validator: None,
            stmt: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            validated_result: None,
        }
    }

    /// 设置 Schema 管理器
    pub fn with_schema_manager(mut self, schema_manager: &'a dyn SchemaManager) -> Self {
        self.schema_validator = Some(SchemaValidator::new(schema_manager));
        self
    }

    /// 设置要验证的语句
    pub fn with_statement(mut self, stmt: DeleteStmt) -> Self {
        self.stmt = Some(stmt);
        self
    }

    /// 获取验证结果
    pub fn validated_result(&self) -> Option<&ValidatedDelete> {
        self.validated_result.as_ref()
    }

    /// 验证 DELETE 语句并返回验证后的信息
    pub fn validate_with_schema(
        &mut self,
        stmt: &DeleteStmt,
        space_name: &str,
    ) -> Result<ValidatedDelete, ValidationError> {
        // 基础验证（不依赖 schema_validator）
        self.validate_basic(stmt)?;

        let schema_validator = self.schema_validator.as_ref().ok_or_else(|| {
            ValidationError::new(
                "Schema validator not initialized".to_string(),
                ValidationErrorType::SemanticError,
            )
        })?;

        let space = schema_validator
            .schema_manager
            .get_space(space_name)
            .map_err(|e| {
                ValidationError::new(
                    format!("Failed to get space '{}': {}", space_name, e),
                    ValidationErrorType::SemanticError,
                )
            })?
            .ok_or_else(|| {
                ValidationError::new(
                    format!("Space '{}' not found", space_name),
                    ValidationErrorType::SemanticError,
                )
            })?;

        // 验证并转换目标
        let target_type =
            self.validate_and_convert_target_with_schema(&stmt.target, &space.vid_type, schema_validator)?;

        Ok(ValidatedDelete {
            space_id: space.space_id,
            target_type,
            with_edge: stmt.with_edge,
            where_clause: stmt.where_clause.clone(),
        })
    }

    /// 基础验证（不依赖 Schema）
    pub fn validate_basic(&self, stmt: &DeleteStmt) -> Result<(), ValidationError> {
        self.validate_target(&stmt.target)?;
        self.validate_where_clause(stmt.where_clause.as_ref())?;
        Ok(())
    }

    /// 验证目标
    fn validate_target(&self, target: &DeleteTarget) -> Result<(), ValidationError> {
        match target {
            DeleteTarget::Vertices(vids) => {
                if vids.is_empty() {
                    return Err(ValidationError::new(
                        "DELETE VERTICES must specify at least one vertex".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
                for (idx, vid) in vids.iter().enumerate() {
                    self.validate_vertex_id(vid, idx + 1)?;
                }
            }
            DeleteTarget::Edges { edge_type, edges } => {
                for (idx, (src, dst, rank)) in edges.iter().enumerate() {
                    self.validate_vertex_id(src, idx * 2)?;
                    self.validate_vertex_id(dst, idx * 2 + 1)?;
                    if let Some(rank_expr) = rank {
                        self.validate_rank(rank_expr)?;
                    }
                }
                if let Some(et) = edge_type {
                    if et.is_empty() {
                        return Err(ValidationError::new(
                            "Edge type name cannot be empty".to_string(),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                }
            }
            DeleteTarget::Tags { tag_names, vertex_ids, is_all_tags } => {
                // 如果不是删除所有 Tag，则需要指定至少一个 Tag 名
                if !is_all_tags && tag_names.is_empty() {
                    return Err(ValidationError::new(
                        "DELETE TAG must specify at least one tag name or use *".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
                for tag_name in tag_names {
                    if tag_name.is_empty() {
                        return Err(ValidationError::new(
                            "Tag name cannot be empty".to_string(),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                }
                if vertex_ids.is_empty() {
                    return Err(ValidationError::new(
                        "DELETE TAG must specify at least one vertex ID".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
                for (idx, vid) in vertex_ids.iter().enumerate() {
                    self.validate_vertex_id(vid, idx + 1)?;
                }
            }
            DeleteTarget::Index(index_name) => {
                if index_name.is_empty() {
                    return Err(ValidationError::new(
                        "Index name cannot be empty".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
        }
        Ok(())
    }

    /// 验证并转换目标（使用 Schema）
    fn validate_and_convert_target_with_schema(
        &self,
        target: &DeleteTarget,
        vid_type: &crate::core::DataType,
        schema_validator: &SchemaValidator,
    ) -> Result<DeleteTargetType, ValidationError> {
        match target {
            DeleteTarget::Vertices(vids) => {
                let mut validated_vids = Vec::new();
                for (idx, vid_expr) in vids.iter().enumerate() {
                    let vid = self.validate_and_evaluate_vid(
                        vid_expr,
                        vid_type,
                        schema_validator,
                        idx + 1,
                    )?;
                    validated_vids.push(vid);
                }
                Ok(DeleteTargetType::Vertices(validated_vids))
            }
            DeleteTarget::Edges { edge_type, edges } => {
                // 获取 EdgeType ID
                let edge_type_id = if let Some(et) = edge_type {
                    let edge_info = schema_validator
                        .get_edge_type("", et)
                        .map_err(|e| {
                            ValidationError::new(
                                format!("Failed to get edge type '{}': {}", et, e),
                                ValidationErrorType::SemanticError,
                            )
                        })?;
                    edge_info.map(|e| e.edge_type_id)
                } else {
                    None
                };

                let mut validated_edges = Vec::new();
                for (idx, (src, dst, rank)) in edges.iter().enumerate() {
                    let src_vid = self.validate_and_evaluate_vid(
                        src,
                        vid_type,
                        schema_validator,
                        idx * 2,
                    )?;
                    let dst_vid = self.validate_and_evaluate_vid(
                        dst,
                        vid_type,
                        schema_validator,
                        idx * 2 + 1,
                    )?;
                    let rank_val = if let Some(rank_expr) = rank {
                        self.evaluate_rank(rank_expr, schema_validator)?
                    } else {
                        0
                    };
                    validated_edges.push(EdgeKey {
                        src: src_vid,
                        dst: dst_vid,
                        rank: rank_val,
                    });
                }

                Ok(DeleteTargetType::Edges {
                    edge_type: edge_type.clone(),
                    edge_type_id,
                    edges: validated_edges,
                })
            }
            DeleteTarget::Tags { tag_names, vertex_ids, is_all_tags } => {
                // 获取 Tag IDs
                let mut tag_ids = Vec::new();
                let final_tag_names = if *is_all_tags {
                    // 如果是删除所有 Tag，获取该 Space 下的所有 Tag
                    vec![] // 执行层会处理获取所有 Tag 的逻辑
                } else {
                    for tag_name in tag_names {
                        let tag_info = schema_validator
                            .get_tag("", tag_name)
                            .map_err(|e| {
                                ValidationError::new(
                                    format!("Failed to get tag '{}': {}", tag_name, e),
                                    ValidationErrorType::SemanticError,
                                )
                            })?;
                        if let Some(tag) = tag_info {
                            tag_ids.push(tag.tag_id);
                        }
                    }
                    tag_names.clone()
                };

                let mut validated_vids = Vec::new();
                for (idx, vid_expr) in vertex_ids.iter().enumerate() {
                    let vid = self.validate_and_evaluate_vid(
                        vid_expr,
                        vid_type,
                        schema_validator,
                        idx + 1,
                    )?;
                    validated_vids.push(vid);
                }

                Ok(DeleteTargetType::Tags {
                    tag_names: final_tag_names,
                    tag_ids,
                    vertex_ids: validated_vids,
                })
            }
            DeleteTarget::Index(index_name) => Ok(DeleteTargetType::Index(index_name.clone())),
        }
    }

    fn validate_vertex_id(&self, expr: &Expression, idx: usize) -> Result<(), ValidationError> {
        match expr {
            Expression::Literal(crate::core::Value::String(s)) => {
                if s.is_empty() {
                    return Err(ValidationError::new(
                        format!("Vertex ID at position {} cannot be empty", idx + 1),
                        ValidationErrorType::SemanticError,
                    ));
                }
                Ok(())
            }
            Expression::Literal(crate::core::Value::Int(_)) => Ok(()),
            Expression::Variable(_) => Ok(()),
            _ => Err(ValidationError::new(
                format!(
                    "Vertex ID at position {} must be a string constant or variable",
                    idx + 1
                ),
                ValidationErrorType::SemanticError,
            )),
        }
    }

    /// 验证并评估 VID
    fn validate_and_evaluate_vid(
        &self,
        vid_expr: &Expression,
        vid_type: &crate::core::DataType,
        schema_validator: &SchemaValidator,
        idx: usize,
    ) -> Result<Value, ValidationError> {
        let vid = schema_validator
            .evaluate_expression(vid_expr)
            .map_err(|e| {
                ValidationError::new(
                    format!("Failed to evaluate vertex ID at position {}: {}", idx, e.message),
                    e.error_type,
                )
            })?;

        schema_validator
            .validate_vid(&vid, vid_type)
            .map_err(|e| {
                ValidationError::new(
                    format!("Invalid vertex ID at position {}: {}", idx, e.message),
                    e.error_type,
                )
            })?;

        Ok(vid)
    }

    fn validate_rank(&self, expr: &Expression) -> Result<(), ValidationError> {
        match expr {
            Expression::Literal(crate::core::Value::Int(_)) => Ok(()),
            Expression::Variable(_) => Ok(()),
            _ => Err(ValidationError::new(
                "Rank must be an integer constant or variable".to_string(),
                ValidationErrorType::SemanticError,
            )),
        }
    }

    /// 评估 rank 表达式
    fn evaluate_rank(
        &self,
        expr: &Expression,
        schema_validator: &SchemaValidator,
    ) -> Result<i64, ValidationError> {
        let value = schema_validator
            .evaluate_expression(expr)
            .map_err(|e| {
                ValidationError::new(
                    format!("Failed to evaluate rank: {}", e.message),
                    e.error_type,
                )
            })?;

        match value {
            Value::Int(i) => Ok(i),
            _ => Err(ValidationError::new(
                "Rank must be an integer".to_string(),
                ValidationErrorType::TypeMismatch,
            )),
        }
    }

    fn validate_where_clause(
        &self,
        where_clause: Option<&Expression>,
    ) -> Result<(), ValidationError> {
        if let Some(where_expr) = where_clause {
            self.validate_expression(where_expr)?;
        }
        Ok(())
    }

    fn validate_expression(&self, expr: &Expression) -> Result<(), ValidationError> {
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
}

impl Default for DeleteValidator<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl StatementValidator for DeleteValidator<'_> {
    fn validate(&mut self, ctx: &mut ValidationContext) -> Result<(), ValidationError> {
        let stmt = self.stmt.as_ref().ok_or_else(|| {
            ValidationError::new(
                "DELETE statement not set".to_string(),
                ValidationErrorType::SemanticError,
            )
        })?;

        // 检查是否选择了图空间
        if ctx.space().space_id.is_none() {
            return Err(ValidationError::new(
                "No space selected. Use `USE <space>` to select a graph space first.".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        let space_name = ctx.space().name.as_str();

        match self.validate_with_schema(stmt, space_name) {
            Ok(result) => {
                self.validated_result = Some(result);
                // 添加输出列
                self.add_output(ColumnDef::new("DELETED", crate::core::DataType::Bool));
                Ok(())
            }
            Err(e) => {
                ctx.add_error(e.clone());
                Err(e)
            }
        }
    }

    fn statement_type(&self) -> StatementType {
        StatementType::Delete
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn add_input(&mut self, col: ColumnDef) {
        self.inputs.push(col);
    }

    fn add_output(&mut self, col: ColumnDef) {
        self.outputs.push(col);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;
    use crate::core::types::{DataType, TagInfo};
    use crate::query::parser::ast::stmt::{DeleteStmt, DeleteTarget};
    use crate::query::parser::ast::Span;

    fn create_delete_stmt(target: DeleteTarget, where_clause: Option<Expression>) -> DeleteStmt {
        DeleteStmt {
            span: Span::default(),
            target,
            where_clause,
            with_edge: false,
        }
    }

    // 模拟 SchemaManager 用于测试
    #[derive(Debug)]
    struct MockSchemaManager;

    impl SchemaManager for MockSchemaManager {
        fn create_space(&self, _space: &crate::core::types::SpaceInfo) -> crate::storage::StorageResult<bool> {
            Ok(true)
        }
        fn drop_space(&self, _space_name: &str) -> crate::storage::StorageResult<bool> {
            Ok(true)
        }
        fn get_space(&self, _space_name: &str) -> crate::storage::StorageResult<Option<crate::core::types::SpaceInfo>> {
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
        fn get_space_by_id(&self, _space_id: i32) -> crate::storage::StorageResult<Option<crate::core::types::SpaceInfo>> {
            Ok(None)
        }
        fn list_spaces(&self) -> crate::storage::StorageResult<Vec<crate::core::types::SpaceInfo>> {
            Ok(vec![])
        }
        fn create_tag(&self, _space: &str, _tag: &TagInfo) -> crate::storage::StorageResult<bool> {
            Ok(true)
        }
        fn get_tag(&self, _space: &str, tag_name: &str) -> crate::storage::StorageResult<Option<TagInfo>> {
            if tag_name == "person" {
                Ok(Some(TagInfo {
                    tag_id: 1,
                    tag_name: "person".to_string(),
                    properties: vec![],
                    comment: None,
                    ttl_duration: None,
                    ttl_col: None,
                }))
            } else {
                Ok(None)
            }
        }
        fn list_tags(&self, _space: &str) -> crate::storage::StorageResult<Vec<TagInfo>> {
            Ok(vec![])
        }
        fn drop_tag(&self, _space: &str, _tag_name: &str) -> crate::storage::StorageResult<bool> {
            Ok(true)
        }
        fn create_edge_type(&self, _space: &str, _edge: &crate::core::types::EdgeTypeInfo) -> crate::storage::StorageResult<bool> {
            Ok(true)
        }
        fn get_edge_type(&self, _space: &str, _edge_type_name: &str) -> crate::storage::StorageResult<Option<crate::core::types::EdgeTypeInfo>> {
            Ok(None)
        }
        fn list_edge_types(&self, _space: &str) -> crate::storage::StorageResult<Vec<crate::core::types::EdgeTypeInfo>> {
            Ok(vec![])
        }
        fn drop_edge_type(&self, _space: &str, _edge_type_name: &str) -> crate::storage::StorageResult<bool> {
            Ok(true)
        }
        fn get_tag_schema(&self, _space: &str, _tag: &str) -> crate::storage::StorageResult<crate::storage::Schema> {
            Ok(crate::storage::Schema::new("test".to_string(), 1))
        }
        fn get_edge_type_schema(&self, _space: &str, _edge: &str) -> crate::storage::StorageResult<crate::storage::Schema> {
            Ok(crate::storage::Schema::new("test".to_string(), 1))
        }
    }

    #[test]
    fn test_validate_vertices_empty_list() {
        let validator = DeleteValidator::new();
        let stmt = create_delete_stmt(DeleteTarget::Vertices(vec![]), None);
        let result = validator.validate_basic(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "DELETE VERTICES must specify at least one vertex");
    }

    #[test]
    fn test_validate_vertices_valid() {
        let validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Vertices(vec![
                Expression::Literal(crate::core::Value::String("v1".to_string())),
                Expression::Literal(crate::core::Value::String("v2".to_string())),
            ]),
            None,
        );
        let result = validator.validate_basic(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_vertices_with_variable() {
        let validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Vertices(vec![Expression::Variable("$vids".to_string())]),
            None,
        );
        let result = validator.validate_basic(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_vertex_id_empty() {
        let validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Vertices(vec![
                Expression::Literal(crate::core::Value::String("v1".to_string())),
                Expression::Literal(crate::core::Value::String("".to_string())),
            ]),
            None,
        );
        let result = validator.validate_basic(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("cannot be empty"));
    }

    #[test]
    fn test_validate_edges_valid() {
        let validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Edges {
                edge_type: Some("friend".to_string()),
                edges: vec![(Expression::Literal(crate::core::Value::String("v1".to_string())), Expression::Literal(crate::core::Value::String("v2".to_string())), None)],
            },
            None,
        );
        let result = validator.validate_basic(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_edges_with_rank() {
        let validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Edges {
                edge_type: Some("friend".to_string()),
                edges: vec![(Expression::Literal(crate::core::Value::String("v1".to_string())), Expression::Literal(crate::core::Value::String("v2".to_string())), Some(Expression::Literal(crate::core::Value::Int(0))))],
            },
            None,
        );
        let result = validator.validate_basic(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_with_schema() {
        static MOCK: MockSchemaManager = MockSchemaManager;
        let mut validator = DeleteValidator::new().with_schema_manager(&MOCK);

        let stmt = create_delete_stmt(
            DeleteTarget::Vertices(vec![
                Expression::Literal(crate::core::Value::String("v1".to_string())),
                Expression::Literal(crate::core::Value::String("v2".to_string())),
            ]),
            None,
        );

        let result = validator.validate_with_schema(&stmt, "test_space");
        assert!(result.is_ok());

        let validated = result.unwrap();
        assert_eq!(validated.space_id, 1);
        match validated.target_type {
            DeleteTargetType::Vertices(vids) => {
                assert_eq!(vids.len(), 2);
            }
            _ => panic!("Expected Vertices target type"),
        }
    }
}

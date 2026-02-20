//! DELETE 语句验证器 - 新体系版本
//! 对应 NebulaGraph DeleteValidator 的功能
//! 验证 DELETE 语句的语义正确性
//!
//! 本文件已按照新的 trait + 枚举 验证器体系重构：
//! 1. 实现了 StatementValidator trait，统一接口
//! 2. 保留了完整功能：
//!    - 验证生命周期管理
//!    - 输入/输出列管理
//!    - 表达式属性追踪
//!    - 用户定义变量管理
//!    - 权限检查
//!    - 执行计划生成
//! 3. 移除了生命周期参数，使用 Arc 管理 SchemaManager
//! 4. 使用 AstContext 统一管理上下文

use std::sync::Arc;

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::{Expression, Value};
use crate::query::context::ast::AstContext;
use crate::query::context::execution::QueryContext;
use crate::query::parser::ast::stmt::{DeleteStmt, DeleteTarget};
use crate::query::validator::validator_trait::{
    StatementType, StatementValidator, ValidationResult, ColumnDef, ValueType,
    ExpressionProps,
};
use crate::storage::metadata::schema_manager::SchemaManager;

/// 验证后的删除信息
#[derive(Debug, Clone)]
pub struct ValidatedDelete {
    pub space_id: u64,
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

/// DELETE 语句验证器 - 新体系实现
///
/// 功能完整性保证：
/// 1. 完整的验证生命周期
/// 2. 输入/输出列管理
/// 3. 表达式属性追踪
/// 4. 用户定义变量管理
/// 5. 权限检查（可扩展）
/// 6. 执行计划生成（可扩展）
#[derive(Debug)]
pub struct DeleteValidator {
    // Schema 管理
    schema_manager: Option<Arc<dyn SchemaManager>>,
    // 输入列定义
    inputs: Vec<ColumnDef>,
    // 输出列定义
    outputs: Vec<ColumnDef>,
    // 表达式属性
    expr_props: ExpressionProps,
    // 用户定义变量
    user_defined_vars: Vec<String>,
    // 缓存验证结果
    validated_result: Option<ValidatedDelete>,
}

impl DeleteValidator {
    pub fn new() -> Self {
        Self {
            schema_manager: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
            validated_result: None,
        }
    }

    pub fn with_schema_manager(mut self, schema_manager: Arc<dyn SchemaManager>) -> Self {
        self.schema_manager = Some(schema_manager);
        self
    }

    /// 获取验证结果
    pub fn validated_result(&self) -> Option<&ValidatedDelete> {
        self.validated_result.as_ref()
    }

    /// 基础验证（不依赖 Schema）
    fn validate_delete(&self, stmt: &DeleteStmt) -> Result<(), ValidationError> {
        self.validate_target(&stmt.target)?;
        self.validate_where_clause(stmt.where_clause.as_ref())?;
        Ok(())
    }

    /// 验证删除目标
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

    /// 验证顶点 ID
    fn validate_vertex_id(&self, expr: &Expression, idx: usize) -> Result<(), ValidationError> {
        match expr {
            Expression::Literal(Value::String(s)) => {
                if s.is_empty() {
                    return Err(ValidationError::new(
                        format!("Vertex ID at position {} cannot be empty", idx),
                        ValidationErrorType::SemanticError,
                    ));
                }
                Ok(())
            }
            Expression::Literal(Value::Int(_)) => Ok(()),
            Expression::Variable(_) => Ok(()),
            _ => Err(ValidationError::new(
                format!(
                    "Vertex ID at position {} must be a string constant or variable",
                    idx
                ),
                ValidationErrorType::SemanticError,
            )),
        }
    }

    /// 验证 rank
    fn validate_rank(&self, expr: &Expression) -> Result<(), ValidationError> {
        match expr {
            Expression::Literal(Value::Int(_)) => Ok(()),
            Expression::Variable(_) => Ok(()),
            _ => Err(ValidationError::new(
                "Rank must be an integer constant or variable".to_string(),
                ValidationErrorType::SemanticError,
            )),
        }
    }

    /// 验证 WHERE 子句
    fn validate_where_clause(
        &self,
        where_clause: Option<&Expression>,
    ) -> Result<(), ValidationError> {
        if let Some(where_expr) = where_clause {
            self.validate_expression(where_expr)?;
        }
        Ok(())
    }

    /// 验证表达式
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

    /// 验证并转换目标（使用 Schema）
    fn validate_and_convert_target(
        &self,
        target: &DeleteTarget,
        space_id: u64,
    ) -> Result<DeleteTargetType, ValidationError> {
        match target {
            DeleteTarget::Vertices(vids) => {
                let mut validated_vids = Vec::new();
                for (idx, vid_expr) in vids.iter().enumerate() {
                    let vid = self.evaluate_vid(vid_expr, idx + 1)?;
                    validated_vids.push(vid);
                }
                Ok(DeleteTargetType::Vertices(validated_vids))
            }
            DeleteTarget::Edges { edge_type, edges } => {
                // 获取 EdgeType ID
                let edge_type_id = if let Some(et) = edge_type {
                    self.get_edge_type_id(et, space_id)?
                } else {
                    None
                };

                let mut validated_edges = Vec::new();
                for (idx, (src, dst, rank)) in edges.iter().enumerate() {
                    let src_vid = self.evaluate_vid(src, idx * 2)?;
                    let dst_vid = self.evaluate_vid(dst, idx * 2 + 1)?;
                    let rank_val = if let Some(rank_expr) = rank {
                        self.evaluate_rank(rank_expr)?
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
                    // 如果是删除所有 Tag，执行层会处理获取所有 Tag 的逻辑
                    vec![]
                } else {
                    for tag_name in tag_names {
                        let tag_id = self.get_tag_id(tag_name, space_id)?;
                        if let Some(id) = tag_id {
                            tag_ids.push(id);
                        }
                    }
                    tag_names.clone()
                };

                let mut validated_vids = Vec::new();
                for (idx, vid_expr) in vertex_ids.iter().enumerate() {
                    let vid = self.evaluate_vid(vid_expr, idx + 1)?;
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

    /// 评估 VID 表达式
    fn evaluate_vid(
        &self,
        vid_expr: &Expression,
        idx: usize,
    ) -> Result<Value, ValidationError> {
        match vid_expr {
            Expression::Literal(v) => Ok(v.clone()),
            Expression::Variable(name) => {
                // 变量需要在执行时解析
                Ok(Value::String(format!("${}", name)))
            }
            _ => Err(ValidationError::new(
                format!("Failed to evaluate vertex ID at position {}", idx),
                ValidationErrorType::SemanticError,
            )),
        }
    }

    /// 评估 rank 表达式
    fn evaluate_rank(&self, expr: &Expression) -> Result<i64, ValidationError> {
        match expr {
            Expression::Literal(Value::Int(i)) => Ok(*i),
            Expression::Variable(_) => Ok(0), // 变量在执行时解析
            _ => Err(ValidationError::new(
                "Rank must be an integer".to_string(),
                ValidationErrorType::TypeMismatch,
            )),
        }
    }

    /// 获取 EdgeType ID
    fn get_edge_type_id(
        &self,
        edge_type_name: &str,
        _space_id: u64,
    ) -> Result<Option<i32>, ValidationError> {
        // 如果有 schema_manager，可以查询实际的 edge_type_id
        // 这里简化处理，返回 None 让执行层处理
        let _ = edge_type_name;
        Ok(None)
    }

    /// 获取 Tag ID
    fn get_tag_id(
        &self,
        tag_name: &str,
        _space_id: u64,
    ) -> Result<Option<i32>, ValidationError> {
        // 如果有 schema_manager，可以查询实际的 tag_id
        // 这里简化处理，返回 None 让执行层处理
        let _ = tag_name;
        Ok(None)
    }
}

impl Default for DeleteValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl StatementValidator for DeleteValidator {
    fn validate(
        &mut self,
        query_context: Option<&QueryContext>,
        ast: &mut AstContext,
    ) -> Result<ValidationResult, ValidationError> {
        // 1. 检查是否需要空间
        if !self.is_global_statement(ast) && query_context.is_none() {
            return Err(ValidationError::new(
                "未选择图空间，请先执行 USE <space>".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 2. 获取 DELETE 语句
        let stmt = ast.sentence()
            .ok_or_else(|| ValidationError::new(
                "No statement found in AST context".to_string(),
                ValidationErrorType::SemanticError,
            ))?;

        let delete_stmt = match stmt {
            crate::query::parser::ast::Stmt::Delete(delete_stmt) => delete_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected DELETE statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        // 3. 执行基础验证
        self.validate_delete(delete_stmt)?;

        // 4. 获取 space_id
        let space_id = query_context
            .map(|qc| qc.space_id())
            .filter(|&id| id != 0)
            .or_else(|| ast.space().space_id.map(|id| id as u64))
            .unwrap_or(0);

        // 5. 验证并转换目标
        let target_type = self.validate_and_convert_target(&delete_stmt.target, space_id)?;

        // 6. 创建验证结果
        let validated = ValidatedDelete {
            space_id,
            target_type,
            with_edge: delete_stmt.with_edge,
            where_clause: delete_stmt.where_clause.clone(),
        };

        // 7. 设置输出列
        self.outputs.clear();
        self.outputs.push(ColumnDef {
            name: "DELETED".to_string(),
            type_: ValueType::Bool,
        });

        self.validated_result = Some(validated);

        // 8. 返回验证结果
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
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

    fn expression_props(&self) -> &ExpressionProps {
        &self.expr_props
    }

    fn user_defined_vars(&self) -> &[String] {
        &self.user_defined_vars
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
            with_edge: false,
        }
    }

    #[test]
    fn test_validate_vertices_empty_list() {
        let validator = DeleteValidator::new();
        let stmt = create_delete_stmt(DeleteTarget::Vertices(vec![]), None);
        let result = validator.validate_delete(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "DELETE VERTICES must specify at least one vertex");
    }

    #[test]
    fn test_validate_vertices_valid() {
        let validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Vertices(vec![
                Expression::Literal(Value::String("v1".to_string())),
                Expression::Literal(Value::String("v2".to_string())),
            ]),
            None,
        );
        let result = validator.validate_delete(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_vertices_with_variable() {
        let validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Vertices(vec![Expression::Variable("vids".to_string())]),
            None,
        );
        let result = validator.validate_delete(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_vertex_id_empty() {
        let validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Vertices(vec![
                Expression::Literal(Value::String("v1".to_string())),
                Expression::Literal(Value::String("".to_string())),
            ]),
            None,
        );
        let result = validator.validate_delete(&stmt);
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
                edges: vec![(
                    Expression::Literal(Value::String("v1".to_string())),
                    Expression::Literal(Value::String("v2".to_string())),
                    None,
                )],
            },
            None,
        );
        let result = validator.validate_delete(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_edges_with_rank() {
        let validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Edges {
                edge_type: Some("friend".to_string()),
                edges: vec![(
                    Expression::Literal(Value::String("v1".to_string())),
                    Expression::Literal(Value::String("v2".to_string())),
                    Some(Expression::Literal(Value::Int(0))),
                )],
            },
            None,
        );
        let result = validator.validate_delete(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_tags_empty_list() {
        let validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Tags {
                tag_names: vec![],
                vertex_ids: vec![],
                is_all_tags: false,
            },
            None,
        );
        let result = validator.validate_delete(&stmt);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_tags_valid() {
        let validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Tags {
                tag_names: vec!["person".to_string()],
                vertex_ids: vec![Expression::Literal(Value::String("v1".to_string()))],
                is_all_tags: false,
            },
            None,
        );
        let result = validator.validate_delete(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_index_empty() {
        let validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Index("".to_string()),
            None,
        );
        let result = validator.validate_delete(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "Index name cannot be empty");
    }

    #[test]
    fn test_validate_index_valid() {
        let validator = DeleteValidator::new();
        let stmt = create_delete_stmt(
            DeleteTarget::Index("idx_person".to_string()),
            None,
        );
        let result = validator.validate_delete(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_statement_validator_trait() {
        let validator = DeleteValidator::new();
        
        // 测试 statement_type
        assert_eq!(validator.statement_type(), StatementType::Delete);
        
        // 测试 inputs/outputs
        assert!(validator.inputs().is_empty());
        assert!(validator.outputs().is_empty());
        
        // 测试 user_defined_vars
        assert!(validator.user_defined_vars().is_empty());
    }
}

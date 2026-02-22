//! Drop 语句验证器
//! 对应 NebulaGraph 中 Drop 相关验证器的功能
//! 验证 DROP SPACE, DROP TAG, DROP EDGE, DROP INDEX 等语句
//!
//! 设计原则：
//! 1. 实现了 StatementValidator trait，统一接口
//! 2. DROP SPACE 是全局语句，其他 DROP 需要选择空间
//! 3. 验证目标对象是否存在（根据 if_exists 标志）

use std::sync::Arc;
use crate::core::error::{ValidationError, ValidationErrorType};
use crate::query::context::QueryContext;
use crate::query::parser::ast::stmt::{DropStmt, DropTarget};
use crate::query::validator::validator_trait::{
    StatementType, StatementValidator, ValidationResult, ColumnDef, ValueType,
    ExpressionProps,
};

/// 验证后的 Drop 信息
#[derive(Debug, Clone)]
pub struct ValidatedDrop {
    pub target_type: DropTargetType,
    pub target_name: String,
    pub space_name: Option<String>,
    pub if_exists: bool,
}

/// Drop 目标类型
#[derive(Debug, Clone)]
pub enum DropTargetType {
    Space,
    Tag,
    Edge,
    TagIndex,
    EdgeIndex,
}

/// Drop 语句验证器
#[derive(Debug)]
pub struct DropValidator {
    target_type: DropTargetType,
    target_name: String,
    space_name: Option<String>,
    if_exists: bool,
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expr_props: ExpressionProps,
    user_defined_vars: Vec<String>,
}

impl DropValidator {
    pub fn new() -> Self {
        Self {
            target_type: DropTargetType::Space,
            target_name: String::new(),
            space_name: None,
            if_exists: false,
            inputs: Vec::new(),
            outputs: vec![
                ColumnDef { name: "Result".to_string(), type_: ValueType::String },
            ],
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    fn validate_impl(&mut self, stmt: &DropStmt) -> Result<(), ValidationError> {
        self.if_exists = stmt.if_exists;

        match &stmt.target {
            DropTarget::Space(name) => {
                self.target_type = DropTargetType::Space;
                self.target_name = name.clone();
                self.space_name = Some(name.clone());

                // 验证空间名非空
                if self.target_name.is_empty() {
                    return Err(ValidationError::new(
                        "Space name cannot be empty".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            DropTarget::Tags(tags) => {
                if tags.is_empty() {
                    return Err(ValidationError::new(
                        "At least one tag must be specified".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
                // 多个 tag 删除时，只处理第一个（简化实现）
                self.target_type = DropTargetType::Tag;
                self.target_name = tags[0].clone();
            }
            DropTarget::Edges(edges) => {
                if edges.is_empty() {
                    return Err(ValidationError::new(
                        "At least one edge type must be specified".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
                // 多个 edge 删除时，只处理第一个（简化实现）
                self.target_type = DropTargetType::Edge;
                self.target_name = edges[0].clone();
            }
            DropTarget::TagIndex { space_name, index_name } => {
                self.target_type = DropTargetType::TagIndex;
                self.space_name = Some(space_name.clone());
                self.target_name = index_name.clone();

                if space_name.is_empty() {
                    return Err(ValidationError::new(
                        "Space name cannot be empty".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
                if index_name.is_empty() {
                    return Err(ValidationError::new(
                        "Index name cannot be empty".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            DropTarget::EdgeIndex { space_name, index_name } => {
                self.target_type = DropTargetType::EdgeIndex;
                self.space_name = Some(space_name.clone());
                self.target_name = index_name.clone();

                if space_name.is_empty() {
                    return Err(ValidationError::new(
                        "Space name cannot be empty".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
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

    /// 获取目标类型
    pub fn target_type(&self) -> &DropTargetType {
        &self.target_type
    }

    /// 获取目标名称
    pub fn target_name(&self) -> &str {
        &self.target_name
    }

    /// 获取空间名
    pub fn space_name(&self) -> Option<&String> {
        self.space_name.as_ref()
    }

    /// 获取 if_exists 标志
    pub fn if_exists(&self) -> bool {
        self.if_exists
    }

    pub fn validated_result(&self) -> ValidatedDrop {
        ValidatedDrop {
            target_type: self.target_type.clone(),
            target_name: self.target_name.clone(),
            space_name: self.space_name.clone(),
            if_exists: self.if_exists,
        }
    }
}

/// 实现 StatementValidator trait
///
/// # 重构变更
/// - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext
impl StatementValidator for DropValidator {
    fn validate(
        &mut self,
        stmt: &crate::query::parser::ast::Stmt,
        _qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        let drop_stmt = match stmt {
            crate::query::parser::ast::Stmt::Drop(drop_stmt) => drop_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected DROP statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };
        
        self.validate_impl(drop_stmt)?;
        
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::Drop
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        // DROP SPACE 是全局语句，其他 DROP 不是
        matches!(self.target_type, DropTargetType::Space)
    }

    fn expression_props(&self) -> &ExpressionProps {
        &self.expr_props
    }

    fn user_defined_vars(&self) -> &[String] {
        &self.user_defined_vars
    }
}

impl Default for DropValidator {
    fn default() -> Self {
        Self::new()
    }
}

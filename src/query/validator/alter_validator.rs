//! Alter 语句验证器
//! 对应 NebulaGraph 中 Alter 相关验证器的功能
//! 验证 ALTER TAG, ALTER EDGE, ALTER SPACE 等语句
//!
//! 设计原则：
//! 1. 实现了 StatementValidator trait，统一接口
//! 2. ALTER SPACE 是全局语句，其他 ALTER 需要选择空间
//! 3. 验证属性修改的合法性（添加、删除、修改）

use std::sync::Arc;
use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::types::PropertyDef;
use crate::query::context::QueryContext;
use crate::query::parser::ast::stmt::{AlterStmt, AlterTarget, PropertyChange};
use crate::query::validator::validator_trait::{
    StatementType, StatementValidator, ValidationResult, ColumnDef, ValueType,
    ExpressionProps,
};

/// 验证后的 Alter 信息
#[derive(Debug, Clone)]
pub struct ValidatedAlter {
    pub target_type: AlterTargetType,
    pub target_name: String,
    pub space_name: Option<String>,
    pub additions: Vec<PropertyDef>,
    pub deletions: Vec<String>,
    pub changes: Vec<PropertyChange>,
}

/// Alter 目标类型
#[derive(Debug, Clone)]
pub enum AlterTargetType {
    Tag,
    Edge,
    Space,
}

/// Alter 语句验证器
#[derive(Debug)]
pub struct AlterValidator {
    target_type: AlterTargetType,
    target_name: String,
    space_name: Option<String>,
    additions: Vec<PropertyDef>,
    deletions: Vec<String>,
    changes: Vec<PropertyChange>,
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expr_props: ExpressionProps,
    user_defined_vars: Vec<String>,
}

impl AlterValidator {
    pub fn new() -> Self {
        Self {
            target_type: AlterTargetType::Tag,
            target_name: String::new(),
            space_name: None,
            additions: Vec::new(),
            deletions: Vec::new(),
            changes: Vec::new(),
            inputs: Vec::new(),
            outputs: vec![
                ColumnDef { name: "Result".to_string(), type_: ValueType::String },
            ],
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    fn validate_impl(&mut self, stmt: &AlterStmt) -> Result<(), ValidationError> {
        match &stmt.target {
            AlterTarget::Tag { tag_name, additions, deletions, changes } => {
                self.target_type = AlterTargetType::Tag;
                self.target_name = tag_name.clone();
                self.additions = additions.clone();
                self.deletions = deletions.clone();
                self.changes = changes.clone();

                // 验证 tag 名非空
                if self.target_name.is_empty() {
                    return Err(ValidationError::new(
                        "Tag name cannot be empty".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }

                // 验证至少有一个修改操作
                if additions.is_empty() && deletions.is_empty() && changes.is_empty() {
                    return Err(ValidationError::new(
                        "At least one alter operation is required".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }

                // 验证属性修改
                self.validate_property_changes(additions, deletions, changes)?;
            }
            AlterTarget::Edge { edge_name, additions, deletions, changes } => {
                self.target_type = AlterTargetType::Edge;
                self.target_name = edge_name.clone();
                self.additions = additions.clone();
                self.deletions = deletions.clone();
                self.changes = changes.clone();

                // 验证 edge 名非空
                if self.target_name.is_empty() {
                    return Err(ValidationError::new(
                        "Edge name cannot be empty".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }

                // 验证至少有一个修改操作
                if additions.is_empty() && deletions.is_empty() && changes.is_empty() {
                    return Err(ValidationError::new(
                        "At least one alter operation is required".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }

                // 验证属性修改
                self.validate_property_changes(additions, deletions, changes)?;
            }
            AlterTarget::Space { space_name, comment } => {
                self.target_type = AlterTargetType::Space;
                self.target_name = space_name.clone();
                self.space_name = Some(space_name.clone());

                // 验证 space 名非空
                if self.target_name.is_empty() {
                    return Err(ValidationError::new(
                        "Space name cannot be empty".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }

                // 验证至少有一个修改参数
                if comment.is_none() {
                    return Err(ValidationError::new(
                        "At least one alter parameter is required".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate_property_changes(
        &self,
        additions: &[PropertyDef],
        deletions: &[String],
        changes: &[PropertyChange],
    ) -> Result<(), ValidationError> {
        // 验证添加的属性
        for prop in additions {
            self.validate_property_name(&prop.name)?;
        }

        // 验证删除的属性名
        for name in deletions {
            self.validate_property_name(name)?;
        }

        // 验证修改的属性
        for change in changes {
            self.validate_property_name(&change.old_name)?;
            self.validate_property_name(&change.new_name)?;

            // 新旧名称不能相同
            if change.old_name == change.new_name {
                return Err(ValidationError::new(
                    format!(
                        "Old name and new name cannot be the same: {}",
                        change.old_name
                    ),
                    ValidationErrorType::SemanticError,
                ));
            }
        }

        // 检查冲突：不能同时添加和删除同一个属性
        for added in additions {
            if deletions.contains(&added.name) {
                return Err(ValidationError::new(
                    format!(
                        "Cannot add and delete property '{}' at the same time",
                        added.name
                    ),
                    ValidationErrorType::SemanticError,
                ));
            }
        }

        // 检查冲突：不能同时删除和修改同一个属性
        for deleted in deletions {
            for changed in changes {
                if deleted == &changed.old_name {
                    return Err(ValidationError::new(
                        format!(
                            "Cannot delete and change property '{}' at the same time",
                            deleted
                        ),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate_property_name(&self, name: &str) -> Result<(), ValidationError> {
        if name.is_empty() {
            return Err(ValidationError::new(
                "Property name cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 属性名必须以字母或下划线开头
        let first_char = name.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() && first_char != '_' {
            return Err(ValidationError::new(
                format!(
                    "Property name '{}' must start with a letter or underscore",
                    name
                ),
                ValidationErrorType::SemanticError,
            ));
        }

        // 属性名只能包含字母、数字和下划线
        for (i, c) in name.chars().enumerate() {
            if i > 0 && !c.is_ascii_alphanumeric() && c != '_' {
                return Err(ValidationError::new(
                    format!(
                        "Property name '{}' contains invalid character '{}'",
                        name, c
                    ),
                    ValidationErrorType::SemanticError,
                ));
            }
        }

        Ok(())
    }

    /// 获取目标类型
    pub fn target_type(&self) -> &AlterTargetType {
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

    /// 获取添加的属性
    pub fn additions(&self) -> &[PropertyDef] {
        &self.additions
    }

    /// 获取删除的属性
    pub fn deletions(&self) -> &[String] {
        &self.deletions
    }

    /// 获取修改的属性
    pub fn changes(&self) -> &[PropertyChange] {
        &self.changes
    }

    pub fn validated_result(&self) -> ValidatedAlter {
        ValidatedAlter {
            target_type: self.target_type.clone(),
            target_name: self.target_name.clone(),
            space_name: self.space_name.clone(),
            additions: self.additions.clone(),
            deletions: self.deletions.clone(),
            changes: self.changes.clone(),
        }
    }
}

/// 实现 StatementValidator trait
///
/// # 重构变更
/// - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext
impl StatementValidator for AlterValidator {
    fn validate(
        &mut self,
        stmt: &crate::query::parser::ast::Stmt,
        _qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        let alter_stmt = match stmt {
            crate::query::parser::ast::Stmt::Alter(alter_stmt) => alter_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected ALTER statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };
        
        self.validate_impl(alter_stmt)?;
        
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::Alter
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        // ALTER SPACE 是全局语句，其他 ALTER 不是
        matches!(self.target_type, AlterTargetType::Space)
    }

    fn expression_props(&self) -> &ExpressionProps {
        &self.expr_props
    }

    fn user_defined_vars(&self) -> &[String] {
        &self.user_defined_vars
    }
}

impl Default for AlterValidator {
    fn default() -> Self {
        Self::new()
    }
}

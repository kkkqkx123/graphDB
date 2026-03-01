//! 集合操作语句验证器
//! 对应 NebulaGraph SetValidator 的功能
//! 验证 UNION, UNION ALL, INTERSECT, MINUS 等集合操作语句
//!
//! 设计原则：
//! 1. 实现了 StatementValidator trait，统一接口
//! 2. 验证左右子查询的列数和数据类型兼容性
//! 3. 支持多种集合操作类型

use std::sync::Arc;
use crate::core::error::{ValidationError, ValidationErrorType};
use crate::query::QueryContext;
use crate::query::parser::ast::stmt::{SetOperationStmt, SetOperationType};
use crate::query::validator::validator_trait::{
    StatementType, StatementValidator, ValidationResult, ColumnDef, ValueType,
    ExpressionProps,
};
use crate::query::validator::validator_enum::Validator;

/// 验证后的集合操作信息
#[derive(Debug, Clone)]
pub struct ValidatedSetOperation {
    pub op_type: SetOperationType,
    pub left_outputs: Vec<ColumnDef>,
    pub right_outputs: Vec<ColumnDef>,
    pub output_col_names: Vec<String>,
}

/// 集合操作验证器
#[derive(Debug)]
pub struct SetOperationValidator {
    op_type: SetOperationType,
    left_validator: Option<Box<Validator>>,
    right_validator: Option<Box<Validator>>,
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expr_props: ExpressionProps,
    user_defined_vars: Vec<String>,
}

impl SetOperationValidator {
    pub fn new() -> Self {
        Self {
            op_type: SetOperationType::Union,
            left_validator: None,
            right_validator: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    fn validate_impl(&mut self, stmt: &SetOperationStmt) -> Result<(), ValidationError> {
        self.op_type = stmt.op_type.clone();

        // 创建左子查询验证器
        self.left_validator = Some(Box::new(
            Validator::create_from_stmt(&stmt.left)
                .ok_or_else(|| ValidationError::new(
                    "Failed to create validator for left statement".to_string(),
                    ValidationErrorType::SemanticError,
                ))?
        ));

        // 创建右子查询验证器
        self.right_validator = Some(Box::new(
            Validator::create_from_stmt(&stmt.right)
                .ok_or_else(|| ValidationError::new(
                    "Failed to create validator for right statement".to_string(),
                    ValidationErrorType::SemanticError,
                ))?
        ));

        Ok(())
    }

    fn validate_compatibility(
        &self,
        left_outputs: &[ColumnDef],
        right_outputs: &[ColumnDef],
    ) -> Result<(), ValidationError> {
        // 验证列数相同
        if left_outputs.len() != right_outputs.len() {
            return Err(ValidationError::new(
                format!(
                    "Set operation requires same number of columns: left has {}, right has {}",
                    left_outputs.len(),
                    right_outputs.len()
                ),
                ValidationErrorType::SemanticError,
            ));
        }

        // 验证列名兼容性（可选：可以要求列名相同或自动映射）
        for (i, (left, right)) in left_outputs.iter().zip(right_outputs.iter()).enumerate() {
            // 验证数据类型兼容性
            if !Self::types_compatible(&left.type_, &right.type_) {
                return Err(ValidationError::new(
                    format!(
                        "Type mismatch at column {}: left is {:?}, right is {:?}",
                        i + 1,
                        left.type_,
                        right.type_
                    ),
                    ValidationErrorType::TypeError,
                ));
            }
        }

        Ok(())
    }

    fn types_compatible(left: &ValueType, right: &ValueType) -> bool {
        // 相同类型兼容
        if left == right {
            return true;
        }

        // Unknown 类型与任何类型兼容
        if matches!(left, ValueType::Unknown) || matches!(right, ValueType::Unknown) {
            return true;
        }

        // Null 类型与任何类型兼容
        if matches!(left, ValueType::Null) || matches!(right, ValueType::Null) {
            return true;
        }

        // 数值类型之间兼容
        let left_is_numeric = matches!(left, ValueType::Int | ValueType::Float);
        let right_is_numeric = matches!(right, ValueType::Int | ValueType::Float);
        if left_is_numeric && right_is_numeric {
            return true;
        }

        false
    }

    fn merge_outputs(&mut self, left_outputs: &[ColumnDef], right_outputs: &[ColumnDef]) {
        // 集合操作的输出列使用左子查询的列名
        // 但类型需要是兼容后的类型
        self.outputs = left_outputs
            .iter()
            .zip(right_outputs.iter())
            .map(|(left, right)| ColumnDef {
                name: left.name.clone(),
                type_: Self::merge_types(&left.type_, &right.type_),
            })
            .collect();
    }

    fn merge_types(left: &ValueType, right: &ValueType) -> ValueType {
        if left == right {
            return left.clone();
        }

        // 如果一个是 Unknown，使用另一个
        if matches!(left, ValueType::Unknown) {
            return right.clone();
        }
        if matches!(right, ValueType::Unknown) {
            return left.clone();
        }

        // 数值类型合并为 Float
        let left_is_numeric = matches!(left, ValueType::Int | ValueType::Float);
        let right_is_numeric = matches!(right, ValueType::Int | ValueType::Float);
        if left_is_numeric && right_is_numeric {
            return ValueType::Float;
        }

        // 默认返回 Unknown
        ValueType::Unknown
    }

    /// 获取操作类型
    pub fn op_type(&self) -> &SetOperationType {
        &self.op_type
    }

    /// 获取左子查询验证器
    pub fn left_validator(&self) -> Option<&Validator> {
        self.left_validator.as_deref()
    }

    /// 获取右子查询验证器
    pub fn right_validator(&self) -> Option<&Validator> {
        self.right_validator.as_deref()
    }

    pub fn validated_result(&self) -> ValidatedSetOperation {
        ValidatedSetOperation {
            op_type: self.op_type.clone(),
            left_outputs: self.left_validator.as_ref()
                .map(|v| v.get_outputs().to_vec())
                .unwrap_or_default(),
            right_outputs: self.right_validator.as_ref()
                .map(|v| v.get_outputs().to_vec())
                .unwrap_or_default(),
            output_col_names: self.outputs.iter().map(|c| c.name.clone()).collect(),
        }
    }
}

/// 实现 StatementValidator trait
///
/// # 重构变更
/// - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext
impl StatementValidator for SetOperationValidator {
    fn validate(
        &mut self,
        stmt: &crate::query::parser::ast::Stmt,
        qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        let set_op_stmt = match stmt {
            crate::query::parser::ast::Stmt::SetOperation(set_op_stmt) => set_op_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected SET OPERATION statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };
        
        self.validate_impl(set_op_stmt)?;
        
        // 验证左右子查询
        let left_outputs = if let Some(ref mut left) = self.left_validator {
            let result = left.validate(&set_op_stmt.left, qctx.clone());
            if result.success {
                result.outputs
            } else {
                return Err(result.errors.first().cloned().unwrap_or_else(|| {
                    ValidationError::new(
                        "Left subquery validation failed".to_string(),
                        ValidationErrorType::SemanticError,
                    )
                }));
            }
        } else {
            Vec::new()
        };

        let right_outputs = if let Some(ref mut right) = self.right_validator {
            let result = right.validate(&set_op_stmt.right, qctx.clone());
            if result.success {
                result.outputs
            } else {
                return Err(result.errors.first().cloned().unwrap_or_else(|| {
                    ValidationError::new(
                        "Right subquery validation failed".to_string(),
                        ValidationErrorType::SemanticError,
                    )
                }));
            }
        } else {
            Vec::new()
        };

        // 验证兼容性
        self.validate_compatibility(&left_outputs, &right_outputs)?;

        // 合并输出列
        self.merge_outputs(&left_outputs, &right_outputs);

        // 收集用户定义变量
        if let Some(ref left) = self.left_validator {
            for var in left.get_user_defined_vars() {
                if !self.user_defined_vars.contains(&var.to_string()) {
                    self.user_defined_vars.push(var.to_string());
                }
            }
        }
        if let Some(ref right) = self.right_validator {
            for var in right.get_user_defined_vars() {
                if !self.user_defined_vars.contains(&var.to_string()) {
                    self.user_defined_vars.push(var.to_string());
                }
            }
        }
        
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::Set
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        // 集合操作是否为全局语句取决于左右子查询
        let left_global = self.left_validator.as_ref()
            .map(|v| v.get_type().is_global_statement())
            .unwrap_or(false);
        let right_global = self.right_validator.as_ref()
            .map(|v| v.get_type().is_global_statement())
            .unwrap_or(false);
        left_global && right_global
    }

    fn expression_props(&self) -> &ExpressionProps {
        &self.expr_props
    }

    fn user_defined_vars(&self) -> &[String] {
        &self.user_defined_vars
    }
}

impl Default for SetOperationValidator {
    fn default() -> Self {
        Self::new()
    }
}

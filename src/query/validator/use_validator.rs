//! USE 语句验证器 - 新体系版本
//! 对应 NebulaGraph UseValidator.h/.cpp 的功能
//! 验证 USE <space> 语句
//!
//! 本文件已按照新的 trait + 枚举 验证器体系重构：
//! 1. 实现了 StatementValidator trait，统一接口
//! 2. 保留了原有完整功能：
//!    - 空间名验证（非空、不以数字开头、长度限制等）
//!    - 特殊字符检查
//! 3. 使用 QueryContext 统一管理上下文

use std::sync::Arc;

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::query::context::QueryContext;
use crate::query::parser::ast::Stmt;
use crate::query::validator::validator_trait::{
    StatementType, StatementValidator, ValidationResult, ColumnDef,
    ExpressionProps,
};

/// 验证后的 USE 信息
#[derive(Debug, Clone)]
pub struct ValidatedUse {
    pub space_name: String,
}

/// USE 验证器 - 新体系实现
///
/// 功能完整性保证：
/// 1. 完整的验证生命周期
/// 2. 输入/输出列管理
/// 3. 表达式属性追踪
/// 4. 全局语句支持（不需要预先选择空间）
#[derive(Debug)]
pub struct UseValidator {
    // 空间名
    space_name: String,
    // 输入列定义
    inputs: Vec<ColumnDef>,
    // 输出列定义
    outputs: Vec<ColumnDef>,
    // 表达式属性
    expr_props: ExpressionProps,
    // 用户定义变量
    user_defined_vars: Vec<String>,
    // 验证错误列表
    validation_errors: Vec<ValidationError>,
    // 缓存验证结果
    validated_result: Option<ValidatedUse>,
}

impl UseValidator {
    /// 创建新的验证器实例
    pub fn new() -> Self {
        Self {
            space_name: String::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
            validation_errors: Vec::new(),
            validated_result: None,
        }
    }

    /// 获取验证结果
    pub fn validated_result(&self) -> Option<&ValidatedUse> {
        self.validated_result.as_ref()
    }

    /// 获取验证错误列表
    pub fn validation_errors(&self) -> &[ValidationError] {
        &self.validation_errors
    }

    /// 添加验证错误
    fn add_error(&mut self, error: ValidationError) {
        self.validation_errors.push(error);
    }

    /// 清空验证错误
    fn clear_errors(&mut self) {
        self.validation_errors.clear();
    }

    /// 检查是否有验证错误
    fn has_errors(&self) -> bool {
        !self.validation_errors.is_empty()
    }

    /// 设置空间名
    pub fn set_space_name(&mut self, name: String) {
        self.space_name = name;
    }

    /// 获取空间名
    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    /// 验证 USE 语句（传统方式，保持向后兼容）
    pub fn validate_use(&mut self) -> Result<ValidatedUse, ValidationError> {
        self.validate_space_name()?;
        self.validate_space_exists()?;

        let result = ValidatedUse {
            space_name: self.space_name.clone(),
        };

        self.validated_result = Some(result.clone());
        Ok(result)
    }

    /// 验证空间名
    fn validate_space_name(&self) -> Result<(), ValidationError> {
        if self.space_name.is_empty() {
            return Err(ValidationError::new(
                "USE 语句需要指定空间名".to_string(),
                ValidationErrorType::SyntaxError,
            ));
        }

        if self.space_name.starts_with('_') {
            return Err(ValidationError::new(
                format!(
                    "空间名 '{}' 不能以下划线开头",
                    self.space_name
                ),
                ValidationErrorType::SemanticError,
            ));
        }

        if self.space_name.chars().next().unwrap_or_default().is_ascii_digit() {
            return Err(ValidationError::new(
                format!(
                    "空间名 '{}' 不能以数字开头",
                    self.space_name
                ),
                ValidationErrorType::SemanticError,
            ));
        }

        let invalid_chars: Vec<char> = vec![' ', '\t', '\n', '\r', ',', ';', '(', ')', '[', ']'];
        for c in self.space_name.chars() {
            if invalid_chars.contains(&c) {
                return Err(ValidationError::new(
                    format!(
                        "空间名 '{}' 包含非法字符 '{}'",
                        self.space_name, c
                    ),
                    ValidationErrorType::SemanticError,
                ));
            }
        }

        if self.space_name.len() > 64 {
            return Err(ValidationError::new(
                format!(
                    "空间名 '{}' 超过最大长度 64 个字符",
                    self.space_name
                ),
                ValidationErrorType::SemanticError,
            ));
        }

        Ok(())
    }

    /// 验证空间是否存在
    fn validate_space_exists(&self) -> Result<(), ValidationError> {
        // 在实际实现中，这里应该检查 SchemaManager
        // 但由于 USE 语句的特殊性（用于选择空间），
        // 我们可能在验证时还没有连接到具体的空间
        // 因此这里暂时返回 Ok，实际检查在执行阶段进行
        Ok(())
    }
}

impl Default for UseValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// 实现 StatementValidator trait
///
/// # 重构变更
/// - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext
impl StatementValidator for UseValidator {
    fn validate(
        &mut self,
        stmt: &Stmt,
        _qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        // 清空之前的状态
        self.outputs.clear();
        self.inputs.clear();
        self.expr_props = ExpressionProps::default();
        self.clear_errors();

        // 从 Stmt 中提取 USE 语句信息
        if let Stmt::Use(use_stmt) = stmt {
            self.space_name = use_stmt.space.clone();
        } else {
            return Err(ValidationError::new(
                "期望 USE 语句".to_string(),
                crate::core::error::ValidationErrorType::SemanticError,
            ));
        }

        // 执行具体验证逻辑
        if let Err(e) = self.validate_use() {
            self.add_error(e);
        }

        // 如果有验证错误，返回失败结果
        if self.has_errors() {
            let errors = self.validation_errors.clone();
            return Ok(ValidationResult::failure(errors));
        }

        // 返回成功的验证结果
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::Use
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        // USE 是全局语句，不需要预先选择空间
        true
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

    #[test]
    fn test_use_validator_new() {
        let validator = UseValidator::new();
        assert!(validator.inputs().is_empty());
        assert!(validator.outputs().is_empty());
        assert!(validator.validated_result().is_none());
        assert!(validator.validation_errors().is_empty());
    }

    #[test]
    fn test_use_validator_default() {
        let validator: UseValidator = Default::default();
        assert!(validator.inputs().is_empty());
        assert!(validator.outputs().is_empty());
    }

    #[test]
    fn test_statement_type() {
        let validator = UseValidator::new();
        assert_eq!(validator.statement_type(), StatementType::Use);
    }

    #[test]
    fn test_use_validation() {
        let mut validator = UseValidator::new();
        
        // 设置有效的空间名
        validator.set_space_name("test_space".to_string());
        
        let result = validator.validate_use();
        assert!(result.is_ok());
        
        let validated = result.unwrap();
        assert_eq!(validated.space_name, "test_space");
    }

    #[test]
    fn test_use_empty_space_name() {
        let mut validator = UseValidator::new();
        
        // 不设置空间名
        let result = validator.validate_use();
        assert!(result.is_err());
    }

    #[test]
    fn test_use_invalid_space_name_start_with_digit() {
        let mut validator = UseValidator::new();
        
        // 以数字开头的空间名
        validator.set_space_name("1space".to_string());
        let result = validator.validate_use();
        assert!(result.is_err());
    }

    #[test]
    fn test_use_invalid_space_name_start_with_underscore() {
        let mut validator = UseValidator::new();
        
        // 以下划线开头的空间名
        validator.set_space_name("_space".to_string());
        let result = validator.validate_use();
        assert!(result.is_err());
    }

    #[test]
    fn test_use_invalid_space_name_with_space() {
        let mut validator = UseValidator::new();
        
        // 包含空格的空间名
        validator.set_space_name("test space".to_string());
        let result = validator.validate_use();
        assert!(result.is_err());
    }

    #[test]
    fn test_use_invalid_space_name_too_long() {
        let mut validator = UseValidator::new();
        
        // 超过 64 个字符的空间名
        let long_name = "a".repeat(65);
        validator.set_space_name(long_name);
        let result = validator.validate_use();
        assert!(result.is_err());
    }

    #[test]
    fn test_is_global_statement() {
        let validator = UseValidator::new();
        assert!(validator.is_global_statement());
    }
}

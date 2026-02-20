//! 管道操作验证器
//! 对应 NebulaGraph PipeValidator.h/.cpp 的功能
//! 验证管道操作符 `|` 连接的前后查询兼容性
//!
//! 本文件已按照新的 trait + 枚举 验证器体系重构：
//! 1. 实现了 StatementValidator trait，统一接口
//! 2. 保留了原有完整功能：
//!    - 左侧输出验证
//!    - 右侧输入验证
//!    - 列兼容性检查
//!    - 管道连接验证
//!    - 类型匹配验证
//! 3. 使用 AstContext 统一管理上下文

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::query::context::ast::AstContext;
use crate::query::context::execution::QueryContext;
use crate::query::validator::validator_trait::{
    StatementType, StatementValidator, ValidationResult, ColumnDef, ValueType,
    ExpressionProps,
};

/// 列信息定义
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub type_: ValueType,
    pub alias: Option<String>,
}

impl ColumnInfo {
    /// 创建新的列信息
    pub fn new(name: String, type_: ValueType) -> Self {
        Self {
            name,
            type_,
            alias: None,
        }
    }

    /// 创建带别名的列信息
    pub fn with_alias(name: String, type_: ValueType, alias: String) -> Self {
        Self {
            name,
            type_,
            alias: Some(alias),
        }
    }
}

/// 管道验证器 - 新体系实现
///
/// 功能完整性保证：
/// 1. 完整的验证生命周期
/// 2. 输入/输出列管理
/// 3. 表达式属性追踪
/// 4. 管道连接兼容性验证
/// 5. 列类型匹配检查
#[derive(Debug)]
pub struct PipeValidator {
    // 左侧查询的输出列
    left_output_cols: Vec<ColumnInfo>,
    // 右侧查询的输入列
    right_input_cols: Vec<ColumnInfo>,
    // 输入列定义（用于 trait 接口）
    inputs: Vec<ColumnDef>,
    // 输出列定义（管道操作的输出为右侧查询的输出）
    outputs: Vec<ColumnDef>,
    // 表达式属性
    expr_props: ExpressionProps,
    // 用户定义变量
    user_defined_vars: Vec<String>,
    // 验证错误列表
    validation_errors: Vec<ValidationError>,
}

impl PipeValidator {
    /// 创建新的验证器实例
    pub fn new() -> Self {
        Self {
            left_output_cols: Vec::new(),
            right_input_cols: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
            validation_errors: Vec::new(),
        }
    }

    /// 设置左侧输出列
    pub fn set_left_output(&mut self, cols: Vec<ColumnInfo>) {
        self.left_output_cols = cols;
        // 同步到 inputs
        self.inputs = self.left_output_cols
            .iter()
            .map(|col| ColumnDef {
                name: col.name.clone(),
                type_: col.type_.clone(),
            })
            .collect();
    }

    /// 设置右侧输入列
    pub fn set_right_input(&mut self, cols: Vec<ColumnInfo>) {
        self.right_input_cols = cols;
    }

    /// 添加左侧输出列
    pub fn add_left_output(&mut self, col: ColumnInfo) {
        self.left_output_cols.push(col.clone());
        self.inputs.push(ColumnDef {
            name: col.name,
            type_: col.type_,
        });
    }

    /// 添加右侧输入列
    pub fn add_right_input(&mut self, col: ColumnInfo) {
        self.right_input_cols.push(col);
    }

    /// 获取左侧输出列
    pub fn left_output_cols(&self) -> &[ColumnInfo] {
        &self.left_output_cols
    }

    /// 获取右侧输入列
    pub fn right_input_cols(&self) -> &[ColumnInfo] {
        &self.right_input_cols
    }

    /// 添加验证错误
    fn add_error(&mut self, error: ValidationError) {
        self.validation_errors.push(error);
    }

    /// 检查是否有验证错误
    fn has_errors(&self) -> bool {
        !self.validation_errors.is_empty()
    }

    /// 清空验证错误
    fn clear_errors(&mut self) {
        self.validation_errors.clear();
    }

    /// 执行验证（传统方式，保持向后兼容）
    pub fn validate_pipe(&mut self) -> Result<(), ValidationError> {
        self.clear_errors();
        self.validate_impl()?;
        Ok(())
    }

    fn validate_impl(&mut self) -> Result<(), ValidationError> {
        self.validate_left_output()?;
        self.validate_right_input()?;
        self.validate_compatibility()?;
        self.validate_pipe_connection()?;
        Ok(())
    }

    fn validate_left_output(&self) -> Result<(), ValidationError> {
        for col in &self.left_output_cols {
            if col.name.is_empty() {
                return Err(ValidationError::new(
                    "Pipe left side has empty column name".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    fn validate_right_input(&self) -> Result<(), ValidationError> {
        for col in &self.right_input_cols {
            if col.name.is_empty() {
                return Err(ValidationError::new(
                    "Pipe right side has empty column reference".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    fn validate_compatibility(&self) -> Result<(), ValidationError> {
        if self.left_output_cols.is_empty() && !self.right_input_cols.is_empty() {
            return Err(ValidationError::new(
                "Pipe left side has no output columns but right side requires input".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        for right_col in &self.right_input_cols {
            let mut found = false;
            for left_col in &self.left_output_cols {
                if right_col.name == left_col.name {
                    if right_col.type_ != left_col.type_ && left_col.type_ != ValueType::Unknown {
                        return Err(ValidationError::new(
                            format!(
                                "Column type mismatch for '{}': left output is {:?}, right input requires {:?}",
                                right_col.name, left_col.type_, right_col.type_
                            ),
                            ValidationErrorType::TypeError,
                        ));
                    }
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(ValidationError::new(
                    format!(
                        "Column '{}' referenced in pipe right side not found in left output",
                        right_col.name
                    ),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    fn validate_pipe_connection(&self) -> Result<(), ValidationError> {
        if self.left_output_cols.is_empty() && self.right_input_cols.is_empty() {
            return Ok(());
        }

        if !self.right_input_cols.is_empty() && self.left_output_cols.is_empty() {
            return Err(ValidationError::new(
                "Pipe requires input from previous query but previous query has no output".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    /// 验证管道兼容性（静态方法，便于直接使用）
    pub fn validate_pipe_compatibility(
        left_outputs: &[ColumnInfo],
        right_inputs: &[ColumnInfo],
    ) -> Result<(), ValidationError> {
        let mut validator = Self::new();
        validator.set_left_output(left_outputs.to_vec());
        validator.set_right_input(right_inputs.to_vec());
        validator.validate_pipe()
    }
}

impl Default for PipeValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl StatementValidator for PipeValidator {
    fn validate(
        &mut self,
        _query_context: Option<&QueryContext>,
        ast: &mut AstContext,
    ) -> Result<ValidationResult, ValidationError> {
        self.clear_errors();

        // 执行验证
        if let Err(e) = self.validate_impl() {
            return Ok(ValidationResult::failure(vec![e]));
        }

        // 管道操作的输出为右侧查询的输出
        // 如果没有右侧输入列，则输出左侧输出列
        self.outputs = if self.right_input_cols.is_empty() {
            self.inputs.clone()
        } else {
            self.right_input_cols
                .iter()
                .map(|col| ColumnDef {
                    name: col.name.clone(),
                    type_: col.type_.clone(),
                })
                .collect()
        };

        // 同步到 AstContext
        ast.set_inputs(self.inputs.clone());
        ast.set_outputs(self.outputs.clone());

        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::Pipe
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

    #[test]
    fn test_pipe_validator_new() {
        let validator = PipeValidator::new();
        assert!(validator.left_output_cols().is_empty());
        assert!(validator.right_input_cols().is_empty());
    }

    #[test]
    fn test_set_left_output() {
        let mut validator = PipeValidator::new();
        let cols = vec![
            ColumnInfo::new("col1".to_string(), ValueType::String),
            ColumnInfo::new("col2".to_string(), ValueType::Int),
        ];
        validator.set_left_output(cols);
        assert_eq!(validator.left_output_cols().len(), 2);
    }

    #[test]
    fn test_validate_empty_columns() {
        let mut validator = PipeValidator::new();
        // 空管道是允许的
        let result = validator.validate_pipe();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_compatible_columns() {
        let mut validator = PipeValidator::new();
        let left_cols = vec![
            ColumnInfo::new("name".to_string(), ValueType::String),
            ColumnInfo::new("age".to_string(), ValueType::Int),
        ];
        let right_cols = vec![
            ColumnInfo::new("name".to_string(), ValueType::String),
        ];
        validator.set_left_output(left_cols);
        validator.set_right_input(right_cols);

        let result = validator.validate_pipe();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_incompatible_type() {
        let mut validator = PipeValidator::new();
        let left_cols = vec![
            ColumnInfo::new("age".to_string(), ValueType::Int),
        ];
        let right_cols = vec![
            ColumnInfo::new("age".to_string(), ValueType::String),
        ];
        validator.set_left_output(left_cols);
        validator.set_right_input(right_cols);

        let result = validator.validate_pipe();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_missing_column() {
        let mut validator = PipeValidator::new();
        let left_cols = vec![
            ColumnInfo::new("name".to_string(), ValueType::String),
        ];
        let right_cols = vec![
            ColumnInfo::new("age".to_string(), ValueType::Int),
        ];
        validator.set_left_output(left_cols);
        validator.set_right_input(right_cols);

        let result = validator.validate_pipe();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_static_method() {
        let left_cols = vec![
            ColumnInfo::new("name".to_string(), ValueType::String),
        ];
        let right_cols = vec![
            ColumnInfo::new("name".to_string(), ValueType::String),
        ];

        let result = PipeValidator::validate_pipe_compatibility(&left_cols, &right_cols);
        assert!(result.is_ok());
    }
}

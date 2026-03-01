//! SET/GET/SHOW 语句验证器 - 新体系版本
//! 对应 NebulaGraph SetValidator.h/.cpp 的功能
//! 验证 SET/GET/SHOW 语句的合法性
//!
//! 本文件已按照新的 trait + 枚举 验证器体系重构：
//! 1. 实现了 StatementValidator trait，统一接口
//! 2. 保留了原有完整功能：
//!    - SET 变量验证
//!    - SET Tag/Edge 属性验证
//!    - SET 优先级验证
//!    - 表达式验证
//! 3. 使用 AstContext 统一管理上下文

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::types::expression::contextual::ContextualExpression;
use crate::query::QueryContext;
use crate::query::validator::validator_trait::{
    StatementType, StatementValidator, ValidationResult, ColumnDef, ValueType,
    ExpressionProps,
};

/// SET 语句类型
#[derive(Debug, Clone, PartialEq)]
pub enum SetStatementType {
    SetVariable,
    SetTag,
    SetEdge,
    SetPriority,
}

/// SET 项定义
#[derive(Debug, Clone)]
pub struct SetItem {
    pub statement_type: SetStatementType,
    pub target: ContextualExpression,
    pub value: ContextualExpression,
}

impl SetItem {
    /// 创建新的 SET 项
    pub fn new(statement_type: SetStatementType, target: ContextualExpression, value: ContextualExpression) -> Self {
        Self {
            statement_type,
            target,
            value,
        }
    }
}

/// 验证后的 SET 信息
#[derive(Debug, Clone)]
pub struct ValidatedSet {
    pub items: Vec<ValidatedSetItem>,
    pub variables: HashMap<String, ContextualExpression>,
}

/// 验证后的 SET 项
#[derive(Debug, Clone)]
pub struct ValidatedSetItem {
    pub statement_type: SetStatementType,
    pub target: ContextualExpression,
    pub value: ContextualExpression,
}

/// SET 验证器 - 新体系实现
///
/// 功能完整性保证：
/// 1. 完整的验证生命周期
/// 2. 输入/输出列管理
/// 3. 表达式属性追踪
/// 4. 变量管理
#[derive(Debug)]
pub struct SetValidator {
    // SET 项列表
    set_items: Vec<SetItem>,
    // 变量映射
    variables: HashMap<String, ContextualExpression>,
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
    validated_result: Option<ValidatedSet>,
}

impl SetValidator {
    /// 创建新的验证器实例
    pub fn new() -> Self {
        Self {
            set_items: Vec::new(),
            variables: HashMap::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
            validation_errors: Vec::new(),
            validated_result: None,
        }
    }

    /// 获取验证结果
    pub fn validated_result(&self) -> Option<&ValidatedSet> {
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

    /// 添加 SET 项
    pub fn add_set_item(&mut self, item: SetItem) {
        self.set_items.push(item);
    }

    /// 设置变量
    pub fn set_variable(&mut self, name: String, value: ContextualExpression) {
        self.variables.insert(name.clone(), value);
        if !self.user_defined_vars.contains(&name) {
            self.user_defined_vars.push(name);
        }
    }

    /// 获取 SET 项列表
    pub fn set_items(&self) -> &[SetItem] {
        &self.set_items
    }

    /// 获取变量映射
    pub fn variables(&self) -> &HashMap<String, ContextualExpression> {
        &self.variables
    }

    /// 验证 SET 语句（传统方式，保持向后兼容）
    pub fn validate_set(&mut self) -> Result<ValidatedSet, ValidationError> {
        let mut validated_items = Vec::new();

        for item in &self.set_items {
            self.validate_set_item(item)?;
            validated_items.push(ValidatedSetItem {
                statement_type: item.statement_type.clone(),
                target: item.target.clone(),
                value: item.value.clone(),
            });
        }

        self.validate_variables()?;

        let result = ValidatedSet {
            items: validated_items,
            variables: self.variables.clone(),
        };

        self.validated_result = Some(result.clone());
        Ok(result)
    }

    /// 验证单个 SET 项
    fn validate_set_item(&self, item: &SetItem) -> Result<(), ValidationError> {
        match item.statement_type {
            SetStatementType::SetVariable => {
                self.validate_set_variable(&item.target, &item.value)?;
            }
            SetStatementType::SetTag => {
                self.validate_set_tag(&item.target, &item.value)?;
            }
            SetStatementType::SetEdge => {
                self.validate_set_edge(&item.target, &item.value)?;
            }
            SetStatementType::SetPriority => {
                self.validate_set_priority(&item.value)?;
            }
        }
        Ok(())
    }

    /// 验证 SET 变量
    fn validate_set_variable(
        &self,
        target: &Expression,
        _value: &Expression,
    ) -> Result<(), ValidationError> {
        if let Expression::Variable(name) = target {
            if name.is_empty() {
                return Err(ValidationError::new(
                    "变量名不能为空".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
            if !name.starts_with('$') {
                return Err(ValidationError::new(
                    format!("变量名 '{}' 必须以 '$' 开头", name),
                    ValidationErrorType::SemanticError,
                ));
            }
            Ok(())
        } else {
            Err(ValidationError::new(
                "SET 变量必须目标是一个变量".to_string(),
                ValidationErrorType::SemanticError,
            ))
        }
    }

    /// 验证 SET Tag
    fn validate_set_tag(
        &self,
        target: &Expression,
        _value: &Expression,
    ) -> Result<(), ValidationError> {
        if !matches!(target, Expression::Property { .. }) {
            return Err(ValidationError::new(
                "SET Tag 必须目标是一个属性表达式".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    /// 验证 SET Edge
    fn validate_set_edge(
        &self,
        target: &Expression,
        _value: &Expression,
    ) -> Result<(), ValidationError> {
        if !matches!(target, Expression::Property { .. }) {
            return Err(ValidationError::new(
                "SET Edge 必须目标是一个属性表达式".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    /// 验证 SET 优先级
    fn validate_set_priority(&self, value: &Expression) -> Result<(), ValidationError> {
        match value {
            Expression::Literal(lit) => {
                if let crate::core::Value::Int(n) = lit {
                    if *n < 0 {
                        return Err(ValidationError::new(
                            "优先级不能为负数".to_string(),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                    Ok(())
                } else {
                    Err(ValidationError::new(
                        "优先级必须是整数".to_string(),
                        ValidationErrorType::TypeError,
                    ))
                }
            }
            _ => Err(ValidationError::new(
                "优先级必须是整数字面量".to_string(),
                ValidationErrorType::SemanticError,
            )),
        }
    }

    /// 验证变量
    fn validate_variables(&self) -> Result<(), ValidationError> {
        for (name, value) in &self.variables {
            if name.is_empty() {
                return Err(ValidationError::new(
                    "变量名不能为空".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
            if !name.starts_with('$') && !name.starts_with('@') {
                return Err(ValidationError::new(
                    format!("无效的变量名 '{}': 必须以 '$' 或 '@' 开头", name),
                    ValidationErrorType::SemanticError,
                ));
            }
            // 验证变量值表达式
            self.validate_expression(value)?;
        }
        Ok(())
    }

    /// 验证表达式
    fn validate_expression(&self, expression: &Expression) -> Result<(), ValidationError> {
        match expression {
            Expression::Binary { left, right, .. } => {
                self.validate_expression(left)?;
                self.validate_expression(right)?;
            }
            Expression::Unary { operand, .. } => {
                self.validate_expression(operand)?;
            }
            Expression::Function { args, .. } => {
                for arg in args {
                    self.validate_expression(arg)?;
                }
            }
            Expression::List(items) => {
                for item in items {
                    self.validate_expression(item)?;
                }
            }
            Expression::Map(pairs) => {
                for (_, value) in pairs {
                    self.validate_expression(value)?;
                }
            }
            Expression::Case { conditions, default, .. } => {
                for (condition, expr) in conditions {
                    self.validate_expression(condition)?;
                    self.validate_expression(expr)?;
                }
                if let Some(default_expr) = default {
                    self.validate_expression(default_expr)?;
                }
            }
            Expression::TypeCast { expression, .. } => {
                self.validate_expression(expression)?;
            }
            Expression::Subscript { collection, index } => {
                self.validate_expression(collection)?;
                self.validate_expression(index)?;
            }
            Expression::Range { collection, start, end } => {
                self.validate_expression(collection)?;
                if let Some(start_expr) = start {
                    self.validate_expression(start_expr)?;
                }
                if let Some(end_expr) = end {
                    self.validate_expression(end_expr)?;
                }
            }
            Expression::Path(items) => {
                for item in items {
                    self.validate_expression(item)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// 验证具体语句
    ///
    /// # 重构变更
    /// - 移除 AstContext 参数
    /// - 接收 Arc<QueryContext> 参数
    fn validate_impl(
        &mut self,
        _qctx: Arc<QueryContext>,
    ) -> Result<(), ValidationError> {
        // 执行 SET 验证
        self.validate_set()?;

        // SET 语句的输出是设置的变量
        self.outputs.clear();
        for (name, _) in &self.variables {
            self.outputs.push(ColumnDef {
                name: name.clone(),
                type_: ValueType::Unknown,
            });
        }

        Ok(())
    }
}

impl Default for SetValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// 实现 StatementValidator trait
///
/// # 重构变更
/// - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext
/// - 移除 AstContext 相关操作
impl StatementValidator for SetValidator {
    fn validate(
        &mut self,
        _stmt: &crate::query::parser::ast::Stmt,
        qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        // 清空之前的状态
        self.outputs.clear();
        self.inputs.clear();
        self.expr_props = ExpressionProps::default();
        self.clear_errors();

        // 执行具体验证逻辑
        if let Err(e) = self.validate_impl(qctx) {
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
        StatementType::Set
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        // SET 是全局语句，不需要预先选择空间
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
    use crate::core::Value;

    #[test]
    fn test_set_validator_new() {
        let validator = SetValidator::new();
        assert!(validator.inputs().is_empty());
        assert!(validator.outputs().is_empty());
        assert!(validator.validated_result().is_none());
        assert!(validator.validation_errors().is_empty());
    }

    #[test]
    fn test_set_validator_default() {
        let validator: SetValidator = Default::default();
        assert!(validator.inputs().is_empty());
        assert!(validator.outputs().is_empty());
    }

    #[test]
    fn test_statement_type() {
        let validator = SetValidator::new();
        assert_eq!(validator.statement_type(), StatementType::Set);
    }

    #[test]
    fn test_set_variable_validation() {
        let mut validator = SetValidator::new();
        
        // 测试有效的变量设置
        let item = SetItem::new(
            SetStatementType::SetVariable,
            Expression::Variable("$var".to_string()),
            Expression::Literal(Value::Int(42)),
        );
        validator.add_set_item(item);
        
        let result = validator.validate_set();
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_variable_invalid_name() {
        let mut validator = SetValidator::new();
        
        // 测试无效的变量名（不以 $ 开头）
        let item = SetItem::new(
            SetStatementType::SetVariable,
            Expression::Variable("var".to_string()),
            Expression::Literal(Value::Int(42)),
        );
        validator.add_set_item(item);
        
        let result = validator.validate_set();
        assert!(result.is_err());
    }

    #[test]
    fn test_set_priority_validation() {
        let mut validator = SetValidator::new();
        
        // 测试有效的优先级设置
        let item = SetItem::new(
            SetStatementType::SetPriority,
            Expression::Variable("$priority".to_string()),
            Expression::Literal(Value::Int(5)),
        );
        validator.add_set_item(item);
        
        let result = validator.validate_set();
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_priority_negative() {
        let mut validator = SetValidator::new();
        
        // 测试无效的优先级（负数）
        let item = SetItem::new(
            SetStatementType::SetPriority,
            Expression::Variable("$priority".to_string()),
            Expression::Literal(Value::Int(-1)),
        );
        validator.add_set_item(item);
        
        let result = validator.validate_set();
        assert!(result.is_err());
    }
}

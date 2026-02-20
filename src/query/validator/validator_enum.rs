//! 验证器枚举
//! 使用枚举统一管理所有验证器类型
//! 这是新验证器体系的核心组件，替代 Box<dyn> 的动态分发
//!
//! 设计原则：
//! 1. 保留 base_validator.rs 的完整功能
//! 2. 使用枚举避免动态分发开销
//! 3. 统一接口，便于管理和扩展

use crate::core::error::ValidationError;
use crate::query::context::ast::AstContext;
use crate::query::context::execution::QueryContext;
use crate::query::validator::validator_trait::{
    StatementType, StatementValidator, ValidationResult, ColumnDef, ExpressionProps,
};

// 导入具体验证器
use crate::query::validator::create_validator::CreateValidator;

/// 统一验证器枚举
/// 
/// 设计优势：
/// 1. 编译期确定类型，避免动态分发开销
/// 2. 统一接口，便于管理和扩展
/// 3. 模式匹配支持，便于针对特定验证器处理
/// 4. 保留完整的验证生命周期功能
#[derive(Debug)]
pub enum Validator {
    /// CREATE 语句验证器
    Create(CreateValidator),
    // TODO: 后续添加其他验证器
    // Match(MatchValidator),
    // Go(GoValidator),
    // ...
}

impl Validator {
    /// 创建 CREATE 验证器
    pub fn create(validator: CreateValidator) -> Self {
        Validator::Create(validator)
    }

    /// 获取语句类型
    pub fn statement_type(&self) -> StatementType {
        match self {
            Validator::Create(_) => StatementType::Create,
        }
    }

    /// 执行完整验证生命周期
    /// 
    /// 验证生命周期：
    /// 1. 检查是否需要空间（is_global_statement）
    /// 2. 执行具体验证逻辑（validate_impl）
    /// 3. 权限检查（check_permission）
    /// 4. 生成执行计划（to_plan）
    /// 5. 同步输入/输出到 AstContext
    pub fn validate(
        &mut self,
        query_context: Option<&QueryContext>,
        ast: &mut AstContext,
    ) -> Result<ValidationResult, ValidationError> {
        match self {
            Validator::Create(v) => v.validate(query_context, ast),
        }
    }

    /// 获取输入列
    pub fn inputs(&self) -> &[ColumnDef] {
        match self {
            Validator::Create(v) => v.inputs(),
        }
    }

    /// 获取输出列
    pub fn outputs(&self) -> &[ColumnDef] {
        match self {
            Validator::Create(v) => v.outputs(),
        }
    }

    /// 判断是否为全局语句（不需要预先选择空间）
    pub fn is_global_statement(&self, ast: &AstContext) -> bool {
        match self {
            Validator::Create(v) => v.is_global_statement(ast),
        }
    }

    /// 获取验证器名称
    pub fn validator_name(&self) -> String {
        match self {
            Validator::Create(v) => v.validator_name(),
        }
    }

    /// 获取表达式属性
    pub fn expression_props(&self) -> &ExpressionProps {
        match self {
            Validator::Create(v) => v.expression_props(),
        }
    }

    /// 获取用户定义变量列表
    pub fn user_defined_vars(&self) -> &[String] {
        match self {
            Validator::Create(v) => v.user_defined_vars(),
        }
    }
}

impl StatementValidator for Validator {
    fn validate(
        &mut self,
        query_context: Option<&QueryContext>,
        ast: &mut AstContext,
    ) -> Result<ValidationResult, ValidationError> {
        self.validate(query_context, ast)
    }

    fn statement_type(&self) -> StatementType {
        self.statement_type()
    }

    fn inputs(&self) -> &[ColumnDef] {
        self.inputs()
    }

    fn outputs(&self) -> &[ColumnDef] {
        self.outputs()
    }

    fn is_global_statement(&self, ast: &AstContext) -> bool {
        self.is_global_statement(ast)
    }

    fn expression_props(&self) -> &ExpressionProps {
        self.expression_props()
    }

    fn user_defined_vars(&self) -> &[String] {
        self.user_defined_vars()
    }
}

/// 验证器工厂
/// 用于创建不同类型的验证器
pub struct ValidatorFactory;

impl ValidatorFactory {
    /// 根据语句类型创建对应的验证器
    /// 
    /// # Arguments
    /// * `stmt_type` - 语句类型
    /// 
    /// # Returns
    /// * `Some(Validator)` - 成功创建验证器
    /// * `None` - 不支持的语句类型
    pub fn create(stmt_type: StatementType) -> Option<Validator> {
        match stmt_type {
            StatementType::Create => Some(Validator::Create(CreateValidator::new())),
            // TODO: 添加其他语句类型的支持
            _ => None,
        }
    }

    /// 获取支持的语句类型列表
    pub fn supported_types() -> Vec<StatementType> {
        vec![
            StatementType::Create,
            // TODO: 添加其他支持的类型
        ]
    }
}

/// 验证器集合
/// 用于管理多个验证器
#[derive(Debug, Default)]
pub struct ValidatorCollection {
    validators: Vec<Validator>,
}

impl ValidatorCollection {
    /// 创建空的验证器集合
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }

    /// 添加验证器
    pub fn add(&mut self, validator: Validator) {
        self.validators.push(validator);
    }

    /// 获取验证器数量
    pub fn len(&self) -> usize {
        self.validators.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.validators.is_empty()
    }

    /// 执行所有验证器的验证
    /// 
    /// 返回所有验证结果的集合
    pub fn validate_all(
        &mut self,
        query_context: Option<&QueryContext>,
        ast: &mut AstContext,
    ) -> Vec<Result<ValidationResult, ValidationError>> {
        self.validators
            .iter_mut()
            .map(|v| v.validate(query_context, ast))
            .collect()
    }

    /// 获取第一个验证器
    pub fn first(&self) -> Option<&Validator> {
        self.validators.first()
    }

    /// 获取第一个可变验证器
    pub fn first_mut(&mut self) -> Option<&mut Validator> {
        self.validators.first_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_enum() {
        let validator = ValidatorFactory::create(StatementType::Create);
        assert!(validator.is_some());
        
        let v = validator.unwrap();
        assert_eq!(v.statement_type(), StatementType::Create);
        assert_eq!(v.validator_name(), "CREATEValidator");
    }

    #[test]
    fn test_validator_collection() {
        let mut collection = ValidatorCollection::new();
        assert!(collection.is_empty());
        
        let validator = Validator::Create(CreateValidator::new());
        collection.add(validator);
        
        assert_eq!(collection.len(), 1);
        assert!(!collection.is_empty());
    }

    #[test]
    fn test_supported_types() {
        let types = ValidatorFactory::supported_types();
        assert!(types.contains(&StatementType::Create));
    }
}

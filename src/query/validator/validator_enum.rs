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
use crate::query::validator::order_by_validator::OrderByValidator;
use crate::query::validator::pipe_validator::PipeValidator;
use crate::query::validator::sequential_validator::SequentialValidator;

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
    /// ORDER BY 子句验证器
    OrderBy(OrderByValidator),
    /// 管道操作验证器
    Pipe(PipeValidator),
    /// Sequential 语句验证器
    Sequential(SequentialValidator),
    // TODO: 后续添加其他验证器
    // Match(MatchValidator),
    // Go(GoValidator),
    // ...
}

impl Validator {
    /// 创建默认验证器（使用 SequentialValidator 作为默认）
    pub fn new() -> Self {
        Validator::Sequential(SequentialValidator::new())
    }

    /// 创建 CREATE 验证器
    pub fn create_validator(validator: CreateValidator) -> Self {
        Validator::Create(validator)
    }

    /// 创建 ORDER BY 验证器
    pub fn order_by(validator: OrderByValidator) -> Self {
        Validator::OrderBy(validator)
    }

    /// 创建 Pipe 验证器
    pub fn pipe(validator: PipeValidator) -> Self {
        Validator::Pipe(validator)
    }

    /// 创建 Sequential 验证器
    pub fn sequential(validator: SequentialValidator) -> Self {
        Validator::Sequential(validator)
    }

    /// 获取语句类型
    pub fn statement_type(&self) -> StatementType {
        match self {
            Validator::Create(_) => StatementType::Create,
            Validator::OrderBy(_) => StatementType::OrderBy,
            Validator::Pipe(_) => StatementType::Pipe,
            Validator::Sequential(_) => StatementType::Sequential,
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
            Validator::OrderBy(v) => v.validate(query_context, ast),
            Validator::Pipe(v) => v.validate(query_context, ast),
            Validator::Sequential(v) => v.validate(query_context, ast),
        }
    }

    /// 使用 AstContext 进行验证（兼容旧接口）
    pub fn validate_with_ast_context(
        &mut self,
        query_context: Option<&QueryContext>,
        ast: &mut AstContext,
    ) -> crate::core::error::DBResult<()> {
        match self.validate(query_context, ast) {
            Ok(result) => {
                if result.success {
                    Ok(())
                } else {
                    let error_msg = result.errors.iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join("; ");
                    Err(crate::core::error::DBError::from(
                        crate::core::error::QueryError::InvalidQuery(error_msg)
                    ))
                }
            }
            Err(e) => Err(crate::core::error::DBError::from(
                crate::core::error::QueryError::InvalidQuery(e.to_string())
            )),
        }
    }

    /// 获取输入列
    pub fn inputs(&self) -> &[ColumnDef] {
        match self {
            Validator::Create(v) => v.inputs(),
            Validator::OrderBy(v) => v.inputs(),
            Validator::Pipe(v) => v.inputs(),
            Validator::Sequential(v) => v.inputs(),
        }
    }

    /// 获取输出列
    pub fn outputs(&self) -> &[ColumnDef] {
        match self {
            Validator::Create(v) => v.outputs(),
            Validator::OrderBy(v) => v.outputs(),
            Validator::Pipe(v) => v.outputs(),
            Validator::Sequential(v) => v.outputs(),
        }
    }

    /// 判断是否为全局语句（不需要预先选择空间）
    pub fn is_global_statement(&self, ast: &AstContext) -> bool {
        match self {
            Validator::Create(v) => v.is_global_statement(ast),
            Validator::OrderBy(v) => v.is_global_statement(ast),
            Validator::Pipe(v) => v.is_global_statement(ast),
            Validator::Sequential(v) => v.is_global_statement(ast),
        }
    }

    /// 获取验证器名称
    pub fn validator_name(&self) -> String {
        match self {
            Validator::Create(v) => v.validator_name(),
            Validator::OrderBy(v) => v.validator_name(),
            Validator::Pipe(v) => v.validator_name(),
            Validator::Sequential(v) => v.validator_name(),
        }
    }

    /// 获取表达式属性
    pub fn expression_props(&self) -> &ExpressionProps {
        match self {
            Validator::Create(v) => v.expression_props(),
            Validator::OrderBy(v) => v.expression_props(),
            Validator::Pipe(v) => v.expression_props(),
            Validator::Sequential(v) => v.expression_props(),
        }
    }

    /// 获取用户定义变量列表
    pub fn user_defined_vars(&self) -> &[String] {
        match self {
            Validator::Create(v) => v.user_defined_vars(),
            Validator::OrderBy(v) => v.user_defined_vars(),
            Validator::Pipe(v) => v.user_defined_vars(),
            Validator::Sequential(v) => v.user_defined_vars(),
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
            StatementType::OrderBy => Some(Validator::OrderBy(OrderByValidator::new())),
            StatementType::Pipe => Some(Validator::Pipe(PipeValidator::new())),
            StatementType::Sequential => Some(Validator::Sequential(SequentialValidator::new())),
            // TODO: 添加其他语句类型的支持
            _ => None,
        }
    }

    /// 获取支持的语句类型列表
    pub fn supported_types() -> Vec<StatementType> {
        vec![
            StatementType::Create,
            StatementType::OrderBy,
            StatementType::Pipe,
            StatementType::Sequential,
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

    /// 获取指定索引的验证器
    pub fn get(&self, index: usize) -> Option<&Validator> {
        self.validators.get(index)
    }

    /// 获取指定索引的可变验证器
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Validator> {
        self.validators.get_mut(index)
    }

    /// 迭代器
    pub fn iter(&self) -> impl Iterator<Item = &Validator> {
        self.validators.iter()
    }

    /// 可变迭代器
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Validator> {
        self.validators.iter_mut()
    }

    /// 清空验证器集合
    pub fn clear(&mut self) {
        self.validators.clear();
    }

    /// 验证所有验证器
    pub fn validate_all(
        &mut self,
        query_context: Option<&QueryContext>,
        ast: &mut AstContext,
    ) -> Result<Vec<ValidationResult>, ValidationError> {
        let mut results = Vec::new();
        for validator in &mut self.validators {
            let result = validator.validate(query_context, ast)?;
            results.push(result);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_factory_create() {
        assert!(ValidatorFactory::create(StatementType::Create).is_some());
        assert!(ValidatorFactory::create(StatementType::OrderBy).is_some());
        assert!(ValidatorFactory::create(StatementType::Pipe).is_some());
        assert!(ValidatorFactory::create(StatementType::Sequential).is_some());
    }

    #[test]
    fn test_validator_statement_type() {
        let create_validator = Validator::create_validator(CreateValidator::new());
        assert_eq!(create_validator.statement_type(), StatementType::Create);

        let order_by_validator = Validator::order_by(OrderByValidator::new());
        assert_eq!(order_by_validator.statement_type(), StatementType::OrderBy);

        let pipe_validator = Validator::pipe(PipeValidator::new());
        assert_eq!(pipe_validator.statement_type(), StatementType::Pipe);

        let sequential_validator = Validator::sequential(SequentialValidator::new());
        assert_eq!(sequential_validator.statement_type(), StatementType::Sequential);
    }

    #[test]
    fn test_validator_collection() {
        let mut collection = ValidatorCollection::new();
        assert!(collection.is_empty());

        collection.add(Validator::create_validator(CreateValidator::new()));
        collection.add(Validator::order_by(OrderByValidator::new()));

        assert_eq!(collection.len(), 2);
        assert!(!collection.is_empty());

        let validator = collection.get(0);
        assert!(validator.is_some());
        assert_eq!(validator.unwrap().statement_type(), StatementType::Create);
    }
}

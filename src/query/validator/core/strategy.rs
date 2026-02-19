//! 验证策略 trait 定义
//!
//! 定义泛型验证策略接口，避免使用 dyn 开销

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::query::context::validate::ValidationContext;

/// 验证策略类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValidationStrategyType {
    /// 别名验证
    Alias,
    /// 表达式验证
    Expression,
    /// 子句验证
    Clause,
    /// 聚合函数验证
    Aggregate,
    /// 分页验证
    Pagination,
    /// 类型推导
    TypeDeduce,
    /// 变量验证
    Variable,
}

impl ValidationStrategyType {
    /// 获取策略名称
    pub fn name(&self) -> &'static str {
        match self {
            ValidationStrategyType::Alias => "Alias",
            ValidationStrategyType::Expression => "Expression",
            ValidationStrategyType::Clause => "Clause",
            ValidationStrategyType::Aggregate => "Aggregate",
            ValidationStrategyType::Pagination => "Pagination",
            ValidationStrategyType::TypeDeduce => "TypeDeduce",
            ValidationStrategyType::Variable => "Variable",
        }
    }
}

/// 验证策略 trait
///
/// 所有验证策略必须实现此 trait
///
/// 使用泛型参数 C 替代 dyn ValidationContext，实现静态分发
///
/// # 类型参数
///
/// * `C` - 上下文类型，必须实现 ValidationContext trait
///
/// # 示例
///
/// ```rust,ignore
/// pub struct AliasValidationStrategy;
///
/// impl ValidationStrategy for AliasValidationStrategy {
///     fn validate<C: ValidationContext>(&self, ctx: &mut C) -> Result<(), ValidationError> {
///         // 执行别名验证
///     }
///
///     fn strategy_type(&self) -> ValidationStrategyType {
///         ValidationStrategyType::Alias
///     }
/// }
/// ```
pub trait ValidationStrategy {
    /// 执行验证
    ///
    /// # 参数
    ///
    /// * `ctx` - 验证上下文
    ///
    /// # 返回
    ///
    /// 验证成功返回 Ok，失败返回 ValidationError
    fn validate(&self, ctx: &mut ValidationContext) -> Result<(), ValidationError>;

    /// 获取策略类型
    fn strategy_type(&self) -> ValidationStrategyType;

    /// 获取策略名称
    fn strategy_name(&self) -> &'static str {
        self.strategy_type().name()
    }
}

/// 验证策略集合
///
/// 管理一组验证策略，按顺序执行
#[derive(Debug, Default)]
pub struct StrategySet {
    strategies: Vec<Box<dyn ValidationStrategy>>,
}

impl StrategySet {
    /// 创建空的策略集合
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
        }
    }

    /// 添加策略
    pub fn add<S: ValidationStrategy + 'static>(&mut self, strategy: S) {
        self.strategies.push(Box::new(strategy));
    }

    /// 执行所有策略
    ///
    /// 按顺序执行所有策略，遇到第一个错误时停止
    pub fn validate_all(&self, ctx: &mut ValidationContext) -> Result<(), ValidationError> {
        for strategy in &self.strategies {
            if let Err(e) = strategy.validate(ctx) {
                return Err(e);
            }
        }
        Ok(())
    }

    /// 执行所有策略，收集所有错误
    ///
    /// 执行所有策略，收集所有错误到上下文中
    pub fn validate_collect(&self, ctx: &mut ValidationContext) {
        for strategy in &self.strategies {
            if let Err(e) = strategy.validate(ctx) {
                ctx.add_error(e);
            }
        }
    }

    /// 获取策略数量
    pub fn len(&self) -> usize {
        self.strategies.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.strategies.is_empty()
    }

    /// 清空策略
    pub fn clear(&mut self) {
        self.strategies.clear();
    }
}

/// 默认策略集合
///
/// 包含常用的验证策略
pub struct DefaultStrategySet;

impl DefaultStrategySet {
    /// 创建默认策略集合
    pub fn new() -> StrategySet {
        let mut set = StrategySet::new();
        // 后续可以在这里添加默认策略
        // set.add(AliasValidationStrategy::new());
        // set.add(ExpressionValidationStrategy::new());
        set
    }
}

impl Default for DefaultStrategySet {
    fn default() -> Self {
        Self
    }
}

/// 验证策略结果
pub type StrategyResult = Result<(), ValidationError>;

/// 验证策略辅助函数

/// 创建语法错误
pub fn syntax_error(message: impl Into<String>) -> ValidationError {
    ValidationError::new(message.into(), ValidationErrorType::SyntaxError)
}

/// 创建语义错误
pub fn semantic_error(message: impl Into<String>) -> ValidationError {
    ValidationError::new(message.into(), ValidationErrorType::SemanticError)
}

/// 创建类型错误
pub fn type_error(message: impl Into<String>) -> ValidationError {
    ValidationError::new(message.into(), ValidationErrorType::TypeError)
}

/// 创建类型不匹配错误
pub fn type_mismatch(expected: &str, actual: &str) -> ValidationError {
    ValidationError::new(
        format!("类型不匹配: 期望 {}, 实际 {}", expected, actual),
        ValidationErrorType::TypeMismatch,
    )
}

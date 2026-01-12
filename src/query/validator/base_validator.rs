//! 基础验证器
//! 对应 NebulaGraph Validator.h/.cpp 的功能
//! 所有验证器的基类

use crate::query::validator::ValidationContext;

pub struct Validator {
    context: ValidationContext,
}

impl Validator {
    pub fn new(context: ValidationContext) -> Self {
        Self { context }
    }

    /// 验证实现 - 子类需要实现此方法
    pub fn validate(&mut self) -> Result<(), String> {
        self.validate_impl()
    }

    /// 子类需要重写的验证实现
    fn validate_impl(&mut self) -> Result<(), String> {
        // 基类默认实现，子类应该重写此方法
        Ok(())
    }

    /// 获取验证上下文的可变引用
    pub fn context_mut(&mut self) -> &mut ValidationContext {
        &mut self.context
    }

    /// 获取验证上下文的引用
    pub fn context(&self) -> &ValidationContext {
        &self.context
    }

    /// 添加验证错误
    pub fn add_error(&mut self, error: String) {
        self.context.add_error(error);
    }

    /// 使用统一错误类型的验证方法
    pub fn validate_unified(&mut self) -> Result<(), crate::core::error::DBError> {
        self.validate().map_err(|e| {
            crate::core::error::DBError::Query(crate::core::error::QueryError::InvalidQuery(
                format!("验证失败: {}", e),
            ))
        })
    }
}

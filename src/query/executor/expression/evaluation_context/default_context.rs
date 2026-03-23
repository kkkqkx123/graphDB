//! 默认表达式求值上下文实现
//!
//! 提供表达式求值过程中的上下文管理
//!
//! **注意：** 此上下文用于运行时表达式求值。
//! 编译时分析请使用 `ExpressionAnalysisContext`。

use crate::core::Value;
use crate::query::executor::expression::functions::global_registry_ref;
use std::collections::HashMap;

/// 默认表达式求值上下文
///
/// 提供表达式求值所需的上下文环境，包括：
/// - 变量存储
/// - 函数注册（使用全局函数注册表）
///
/// **注意：** 此上下文用于运行时表达式求值。
/// 编译时分析请使用 `ExpressionAnalysisContext`。
#[derive(Debug)]
pub struct DefaultExpressionContext {
    /// 变量存储
    variables: HashMap<String, Value>,
}

impl DefaultExpressionContext {
    /// 创建新的上下文
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// 添加变量
    pub fn add_variable(mut self, name: String, value: Value) -> Self {
        self.variables.insert(name, value);
        self
    }

    /// 批量添加变量
    pub fn with_variables<I>(mut self, variables: I) -> Self
    where
        I: IntoIterator<Item = (String, Value)>,
    {
        for (name, value) in variables {
            self.variables.insert(name, value);
        }
        self
    }

    /// 从 ExecutionContext 创建 DefaultExpressionContext
    ///
    /// 复制 ExecutionContext 中的所有变量到新的 DefaultExpressionContext
    pub fn from_execution_context(ctx: &crate::query::executor::base::ExecutionContext) -> Self {
        Self {
            variables: ctx.variables.clone(),
        }
    }

    /// 将变量同步回 ExecutionContext
    ///
    /// 将当前 DefaultExpressionContext 中的所有变量同步到 ExecutionContext
    pub fn sync_to_execution_context(
        self,
        ctx: &mut crate::query::executor::base::ExecutionContext,
    ) {
        for (name, value) in self.variables {
            ctx.set_variable(name, value);
        }
    }
}

impl Default for DefaultExpressionContext {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::query::executor::expression::evaluator::traits::ExpressionContext
    for DefaultExpressionContext
{
    fn get_variable(&self, name: &str) -> Option<Value> {
        self.variables.get(name).cloned()
    }

    fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    fn get_function(
        &self,
        name: &str,
    ) -> Option<crate::query::executor::expression::functions::OwnedFunctionRef> {
        let registry = global_registry_ref();
        registry
            .get_builtin(name)
            .map(|f| crate::query::executor::expression::functions::OwnedFunctionRef::Builtin(f.clone()))
            .or_else(|| {
                registry
                    .get_custom(name)
                    .map(|f| crate::query::executor::expression::functions::OwnedFunctionRef::Custom(f.clone()))
            })
    }
}

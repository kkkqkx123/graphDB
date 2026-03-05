//! 执行上下文
//!
//! 管理执行器执行过程中的中间结果和变量。

use std::collections::HashMap;
use std::sync::Arc;

use super::execution_result::ExecutionResult;
use crate::query::validator::context::ExpressionAnalysisContext;

/// 执行上下文
///
/// 用于在执行器执行过程中存储中间结果和变量，支持执行器之间的数据传递。
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// 中间结果存储
    pub results: HashMap<String, ExecutionResult>,
    /// 变量存储
    pub variables: HashMap<String, crate::core::Value>,
    /// 表达式上下文，用于跨阶段共享表达式信息和缓存
    pub expression_context: Arc<ExpressionAnalysisContext>,
}

impl ExecutionContext {
    /// 创建新的执行上下文
    pub fn new(expression_context: Arc<ExpressionAnalysisContext>) -> Self {
        Self {
            results: HashMap::new(),
            variables: HashMap::new(),
            expression_context,
        }
    }

    /// 设置中间结果
    pub fn set_result(&mut self, name: String, result: ExecutionResult) {
        self.results.insert(name, result);
    }

    /// 获取中间结果
    pub fn get_result(&self, name: &str) -> Option<&ExecutionResult> {
        self.results.get(name)
    }

    /// 设置变量
    pub fn set_variable(&mut self, name: String, value: crate::core::Value) {
        self.variables.insert(name, value);
    }

    /// 获取变量
    pub fn get_variable(&self, name: &str) -> Option<&crate::core::Value> {
        self.variables.get(name)
    }

    /// 获取表达式上下文
    pub fn expression_context(&self) -> &Arc<ExpressionAnalysisContext> {
        &self.expression_context
    }
}

impl Default for ExecutionContext {
    /// 默认实现，创建一个新的 ExpressionContext
    fn default() -> Self {
        Self {
            results: HashMap::new(),
            variables: HashMap::new(),
            expression_context: Arc::new(ExpressionAnalysisContext::new()),
        }
    }
}

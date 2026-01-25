//! 执行上下文
//!
//! 管理执行器执行过程中的中间结果和变量。

use std::collections::HashMap;

use super::execution_result::ExecutionResult;

/// 执行上下文
///
/// 用于在执行器执行过程中存储中间结果和变量，支持执行器之间的数据传递。
#[derive(Debug, Clone, Default)]
pub struct ExecutionContext {
    /// 中间结果存储
    pub results: HashMap<String, ExecutionResult>,
    /// 变量存储
    pub variables: HashMap<String, crate::core::Value>,
}

impl ExecutionContext {
    /// 创建新的执行上下文
    pub fn new() -> Self {
        Self::default()
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
}

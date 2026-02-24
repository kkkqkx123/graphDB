//! 行表达式上下文实现
//!
//! 为Join操作和行级表达式求值提供专用的上下文实现
//! 支持按列名和列索引访问行数据

use crate::core::Value;
use std::collections::HashMap;

/// 行表达式上下文
///
/// 专门用于在行数据上求值表达式的上下文实现
/// 支持两种访问模式：
/// 1. 按列名访问：通过 col_name_index 映射
/// 2. 按变量名访问：通过 variables 映射
#[derive(Debug, Clone)]
pub struct RowExpressionContext {
    /// 当前行数据
    row: Vec<Value>,
    /// 列名索引映射（快速查找）
    col_name_index: HashMap<String, usize>,
    /// 额外变量（用于存储计算中间结果）
    variables: HashMap<String, Value>,
}

impl RowExpressionContext {
    /// 创建新的行上下文
    pub fn new(row: Vec<Value>, col_names: Vec<String>) -> Self {
        let col_name_index: HashMap<String, usize> = col_names
            .into_iter()
            .enumerate()
            .map(|(i, name)| (name, i))
            .collect();

        Self {
            row,
            col_name_index,
            variables: HashMap::new(),
        }
    }

    /// 从现有数据创建上下文
    pub fn from_dataset(row: &[Value], col_names: &[String]) -> Self {
        Self::new(row.to_vec(), col_names.to_vec())
    }

    /// 按列名获取值
    pub fn get_value_by_name(&self, name: &str) -> Option<&Value> {
        self.col_name_index
            .get(name)
            .and_then(|&idx| self.row.get(idx))
    }
}

impl crate::expression::evaluator::traits::ExpressionContext for RowExpressionContext {
    fn get_variable(&self, name: &str) -> Option<Value> {
        // 首先检查变量映射
        if let Some(value) = self.variables.get(name) {
            return Some(value.clone());
        }

        // 然后检查列名（支持将列名作为变量访问）
        if let Some(value) = self.get_value_by_name(name) {
            return Some(value.clone());
        }

        None
    }

    fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    fn get_function(&self, _name: &str) -> Option<crate::expression::functions::FunctionRef> {
        None
    }

    fn supports_cache(&self) -> bool {
        false
    }

    fn get_cache(&mut self) -> Option<&mut crate::expression::context::CacheManager> {
        None
    }
}

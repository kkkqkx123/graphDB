//! 匿名变量生成器模块 - 提供匿名变量生成功能
//! 对应原C++中的AnonVarGenerator.h/cpp

use crate::core::SymbolTable;
use crate::graph::utils::IdGenerator;
use std::sync::Arc;

/// 匿名变量生成器
pub struct AnonVarGenerator {
    symbol_table: Arc<SymbolTable>,
    id_generator: IdGenerator,
}

impl AnonVarGenerator {
    /// 创建新的匿名变量生成器
    pub fn new(symbol_table: Arc<SymbolTable>) -> Self {
        Self {
            symbol_table,
            id_generator: IdGenerator::new(0),
        }
    }

    /// 生成一个新的匿名变量名
    pub fn get_var(&self) -> String {
        let var_name = format!("__VAR_{}", self.id_generator.id());
        self.symbol_table
            .new_variable(&var_name)
            .expect("Failed to create new variable");
        log::trace!("Build anon var: {}", var_name);
        var_name
    }

    /// 在符号表中创建指定名称的变量
    pub fn create_var(&self, var: &str) {
        self.symbol_table
            .new_variable(var)
            .expect("Failed to create variable");
    }

    /// 检查变量名是否为匿名变量
    /// 解析器不允许用户使用以'_'开头的变量名，
    /// 以'_'开头的变量名仅由图数据库内部生成。
    pub fn is_anno_var(var: &str) -> bool {
        !var.is_empty() && var.chars().next() == Some('_')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_anno_var() {
        assert!(AnonVarGenerator::is_anno_var("_anon_var"));
        assert!(AnonVarGenerator::is_anno_var("_"));
        assert!(!AnonVarGenerator::is_anno_var("regular_var"));
        assert!(!AnonVarGenerator::is_anno_var(""));
    }
}

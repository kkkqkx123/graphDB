//! 符号表模块 - 管理查询中的变量和别名
//! 对应原C++中的context/Symbols.h

use std::collections::HashMap;
use std::sync::{RwLock, Arc};

/// 符号类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolType {
    Variable,
    Alias,
    Parameter,
    Function,
}

/// 符号定义
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub created_at: std::time::SystemTime,
}

/// 符号表
#[derive(Debug, Clone)]
pub struct SymbolTable {
    symbols: Arc<RwLock<HashMap<String, Symbol>>>,
}

impl SymbolTable {
    /// 创建新的符号表
    pub fn new() -> Self {
        Self {
            symbols: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 添加新变量到符号表
    pub fn new_variable(&self, name: &str) -> Result<(), String> {
        let mut symbols = self.symbols.write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;

        if symbols.contains_key(name) {
            return Err(format!("Variable '{}' already exists", name));
        }

        let symbol = Symbol {
            name: name.to_string(),
            symbol_type: SymbolType::Variable,
            created_at: std::time::SystemTime::now(),
        };

        symbols.insert(name.to_string(), symbol);
        Ok(())
    }

    /// 检查变量是否存在
    pub fn has_variable(&self, name: &str) -> bool {
        let symbols = match self.symbols.read() {
            Ok(symbols) => symbols,
            Err(_) => return false, // 如果无法获取读锁，返回false
        };

        symbols.contains_key(name)
    }

    /// 获取变量信息
    pub fn get_variable(&self, name: &str) -> Option<Symbol> {
        let symbols = match self.symbols.read() {
            Ok(symbols) => symbols,
            Err(_) => return None, // 如果无法获取读锁，返回None
        };

        symbols.get(name).cloned()
    }

    /// 获取所有变量名
    pub fn get_all_variables(&self) -> Vec<String> {
        let symbols = match self.symbols.read() {
            Ok(symbols) => symbols,
            Err(_) => return vec![], // 如果无法获取读锁，返回空向量
        };

        symbols.keys().cloned().collect()
    }

    /// 删除变量
    pub fn remove_variable(&self, name: &str) -> Result<bool, String> {
        let mut symbols = self.symbols.write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;

        Ok(symbols.remove(name).is_some())
    }

    /// 获取符号表大小
    pub fn size(&self) -> Result<usize, String> {
        let symbols = self.symbols.read()
            .map_err(|e| format!("Failed to acquire read lock: {}", e))?;

        Ok(symbols.len())
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table() {
        let table = SymbolTable::new();
        
        // 测试添加变量
        assert!(table.new_variable("test_var").is_ok());
        assert!(table.has_variable("test_var"));
        
        // 测试重复添加变量
        assert!(table.new_variable("test_var").is_err());
        
        // 测试获取变量
        let symbol = table.get_variable("test_var").unwrap();
        assert_eq!(symbol.name, "test_var");
        assert_eq!(symbol.symbol_type, SymbolType::Variable);
        
        // 测试删除变量
        assert!(table.remove_variable("test_var").unwrap());
        assert!(!table.has_variable("test_var"));
    }

    #[test]
    fn test_get_all_variables() {
        let table = SymbolTable::new();
        table.new_variable("var1").unwrap();
        table.new_variable("var2").unwrap();
        
        let vars = table.get_all_variables();
        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&"var1".to_string()));
        assert!(vars.contains(&"var2".to_string()));
    }
}
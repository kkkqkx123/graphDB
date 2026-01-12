//! 符号表模块 - 管理查询中的变量和别名
//! 对应原C++中的context/Symbols.h

use crate::core::{DependencyTracker, PlanNodeRef};

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

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

impl Symbol {
    pub fn new(name: String, symbol_type: SymbolType) -> Self {
        Self {
            name,
            symbol_type,
            created_at: std::time::SystemTime::now(),
        }
    }
}

/// 符号表
#[derive(Debug, Clone)]
pub struct SymbolTable {
    symbols: Arc<RwLock<HashMap<String, Symbol>>>,
    dependency_tracker: Arc<RwLock<DependencyTracker>>,
}

impl SymbolTable {
    /// 创建新的符号表
    pub fn new() -> Self {
        Self {
            symbols: Arc::new(RwLock::new(HashMap::new())),
            dependency_tracker: Arc::new(RwLock::new(DependencyTracker::new())),
        }
    }

    /// 添加新变量到符号表
    pub fn new_variable(&self, name: &str) -> Result<(), String> {
        let mut symbols = self
            .symbols
            .write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;

        if symbols.contains_key(name) {
            return Err(format!("Variable '{}' already exists", name));
        }

        let symbol = Symbol::new(name.to_string(), SymbolType::Variable);
        symbols.insert(name.to_string(), symbol);

        // 在依赖跟踪器中添加变量
        let mut tracker = self
            .dependency_tracker
            .write()
            .map_err(|e| format!("Failed to acquire write lock on tracker: {}", e))?;
        tracker.add_variable(name.to_string());

        Ok(())
    }

    /// 检查变量是否存在
    pub fn has_variable(&self, name: &str) -> bool {
        let symbols = match self.symbols.read() {
            Ok(symbols) => symbols,
            Err(_) => return false,
        };

        symbols.contains_key(name)
    }

    /// 获取变量信息
    pub fn get_variable(&self, name: &str) -> Option<Symbol> {
        let symbols = match self.symbols.read() {
            Ok(symbols) => symbols,
            Err(_) => return None,
        };

        symbols.get(name).cloned()
    }

    /// 获取所有变量名
    pub fn get_all_variables(&self) -> Vec<String> {
        let symbols = match self.symbols.read() {
            Ok(symbols) => symbols,
            Err(_) => return vec![],
        };

        symbols.keys().cloned().collect()
    }

    /// 删除变量
    pub fn remove_variable(&self, name: &str) -> Result<bool, String> {
        let mut symbols = self
            .symbols
            .write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;

        let removed = symbols.remove(name).is_some();

        if removed {
            // 从依赖跟踪器中删除变量
            let mut tracker = self
                .dependency_tracker
                .write()
                .map_err(|e| format!("Failed to acquire write lock on tracker: {}", e))?;
            tracker.remove_variable(name)?;
        }

        Ok(removed)
    }

    /// 获取符号表大小
    pub fn size(&self) -> Result<usize, String> {
        let symbols = self
            .symbols
            .read()
            .map_err(|e| format!("Failed to acquire read lock: {}", e))?;

        Ok(symbols.len())
    }

    /// 记录变量被计划节点读取
    pub fn read_by(&self, var_name: &str, node: PlanNodeRef) -> Result<(), String> {
        let mut tracker = self
            .dependency_tracker
            .write()
            .map_err(|e| format!("Failed to acquire write lock on tracker: {}", e))?;

        tracker.add_read_dependency(var_name, node)
    }

    /// 记录变量被计划节点写入
    pub fn written_by(&self, var_name: &str, node: PlanNodeRef) -> Result<(), String> {
        let mut tracker = self
            .dependency_tracker
            .write()
            .map_err(|e| format!("Failed to acquire write lock on tracker: {}", e))?;

        tracker.add_write_dependency(var_name, node)
    }

    /// 记录变量被计划节点读写
    pub fn read_written_by(&self, var_name: &str, node: PlanNodeRef) -> Result<(), String> {
        let mut tracker = self
            .dependency_tracker
            .write()
            .map_err(|e| format!("Failed to acquire write lock on tracker: {}", e))?;

        tracker.add_read_write_dependency(var_name, node)
    }

    /// 删除变量的读取关系
    pub fn delete_read_by(&self, var_name: &str, node: &PlanNodeRef) -> Result<bool, String> {
        let mut tracker = self
            .dependency_tracker
            .write()
            .map_err(|e| format!("Failed to acquire write lock on tracker: {}", e))?;

        tracker.remove_read_dependency(var_name, node)
    }

    /// 删除变量的写入关系
    pub fn delete_written_by(&self, var_name: &str, node: &PlanNodeRef) -> Result<bool, String> {
        let mut tracker = self
            .dependency_tracker
            .write()
            .map_err(|e| format!("Failed to acquire write lock on tracker: {}", e))?;

        tracker.remove_write_dependency(var_name, node)
    }

    /// 更新变量的读取关系（变量重命名）
    pub fn update_read_by(
        &self,
        old_var: &str,
        new_var: &str,
        node: &PlanNodeRef,
    ) -> Result<bool, String> {
        let mut tracker = self
            .dependency_tracker
            .write()
            .map_err(|e| format!("Failed to acquire write lock on tracker: {}", e))?;

        let mut success = false;
        if tracker.remove_read_dependency(old_var, node)? {
            success = true;
        }

        if tracker.add_read_dependency(new_var, node.clone()).is_ok() {
            success = true;
        }

        Ok(success)
    }

    /// 更新变量的写入关系（变量重命名）
    pub fn update_written_by(
        &self,
        old_var: &str,
        new_var: &str,
        node: &PlanNodeRef,
    ) -> Result<bool, String> {
        let mut tracker = self
            .dependency_tracker
            .write()
            .map_err(|e| format!("Failed to acquire write lock on tracker: {}", e))?;

        let mut success = false;
        if tracker.remove_write_dependency(old_var, node)? {
            success = true;
        }

        if tracker.add_write_dependency(new_var, node.clone()).is_ok() {
            success = true;
        }

        Ok(success)
    }

    /// 重命名变量
    pub fn rename_variable(&self, old_name: &str, new_name: &str) -> Result<(), String> {
        let mut symbols = self
            .symbols
            .write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;

        if let Some(mut symbol) = symbols.remove(old_name) {
            symbol.name = new_name.to_string();
            symbols.insert(new_name.to_string(), symbol);

            // 更新依赖跟踪器中的变量名
            let mut tracker = self
                .dependency_tracker
                .write()
                .map_err(|e| format!("Failed to acquire write lock on tracker: {}", e))?;
            tracker.rename_variable(old_name, new_name)?;

            Ok(())
        } else {
            Err(format!("Variable '{}' not found", old_name))
        }
    }

    /// 获取变量的读取者列表
    pub fn get_readers(&self, var_name: &str) -> Result<Vec<PlanNodeRef>, String> {
        let tracker = self
            .dependency_tracker
            .read()
            .map_err(|e| format!("Failed to acquire read lock on tracker: {}", e))?;

        Ok(tracker
            .find_readers_of(var_name)
            .into_iter()
            .cloned()
            .collect())
    }

    /// 获取变量的写入者列表
    pub fn get_writers(&self, var_name: &str) -> Result<Vec<PlanNodeRef>, String> {
        let tracker = self
            .dependency_tracker
            .read()
            .map_err(|e| format!("Failed to acquire read lock on tracker: {}", e))?;

        Ok(tracker
            .find_writers_of(var_name)
            .into_iter()
            .cloned()
            .collect())
    }

    /// 获取指定节点读取的所有变量
    pub fn get_variables_read_by(&self, node: &PlanNodeRef) -> Result<Vec<String>, String> {
        let tracker = self
            .dependency_tracker
            .read()
            .map_err(|e| format!("Failed to acquire read lock on tracker: {}", e))?;

        Ok(tracker.find_variables_read_by(node))
    }

    /// 获取指定节点写入的所有变量
    pub fn get_variables_written_by(&self, node: &PlanNodeRef) -> Result<Vec<String>, String> {
        let tracker = self
            .dependency_tracker
            .read()
            .map_err(|e| format!("Failed to acquire read lock on tracker: {}", e))?;

        Ok(tracker.find_variables_written_by(node))
    }

    /// 检测写入冲突
    pub fn detect_write_conflicts(&self) -> Result<Vec<(String, Vec<PlanNodeRef>)>, String> {
        let tracker = self
            .dependency_tracker
            .read()
            .map_err(|e| format!("Failed to acquire read lock on tracker: {}", e))?;

        Ok(tracker
            .detect_write_conflicts()
            .into_iter()
            .map(|(name, readers)| (name, readers.into_iter().cloned().collect()))
            .collect())
    }

    /// 获取依赖统计信息
    pub fn get_dependency_stats(
        &self,
    ) -> Result<Vec<super::dependency_tracker::DependencyStats>, String> {
        let tracker = self
            .dependency_tracker
            .read()
            .map_err(|e| format!("Failed to acquire read lock on tracker: {}", e))?;

        Ok(tracker.get_all_stats())
    }

    /// 生成符号表的字符串表示
    pub fn to_string(&self) -> Result<String, String> {
        let symbols = self
            .symbols
            .read()
            .map_err(|e| format!("Failed to acquire read lock: {}", e))?;
        let tracker = self
            .dependency_tracker
            .read()
            .map_err(|e| format!("Failed to acquire read lock on tracker: {}", e))?;

        let mut result = String::new();
        result.push_str("SymbolTable {\n");
        result.push_str(&format!("  symbols: {},\n", symbols.len()));
        result.push_str(&format!("  variables: {},\n", tracker.len()));

        for (name, symbol) in symbols.iter() {
            result.push_str(&format!(
                "  {}: {:?}, created_at: {:?}\n",
                name, symbol.symbol_type, symbol.created_at
            ));
        }

        result.push_str("}");
        Ok(result)
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
        let symbol = table
            .get_variable("test_var")
            .expect("get_variable should return Some in test");
        assert_eq!(symbol.name, "test_var");
        assert_eq!(symbol.symbol_type, SymbolType::Variable);

        // 测试删除变量
        assert!(table
            .remove_variable("test_var")
            .expect("remove_variable should succeed in test"));
        assert!(!table.has_variable("test_var"));
    }

    #[test]
    fn test_dependency_management() {
        let table = SymbolTable::new();
        table
            .new_variable("var1")
            .expect("new_variable should succeed in test");
        table
            .new_variable("var2")
            .expect("new_variable should succeed in test");

        let node1 = PlanNodeRef::new("node1".to_string(), 1);
        let node2 = PlanNodeRef::new("node2".to_string(), 2);

        // 设置依赖关系
        table
            .read_by("var1", node1.clone())
            .expect("read_by should succeed in test");
        table
            .written_by("var1", node2.clone())
            .expect("written_by should succeed in test");
        table
            .read_by("var2", node2.clone())
            .expect("read_by should succeed in test");

        // 验证依赖关系
        let var1_readers = table
            .get_readers("var1")
            .expect("get_readers should return Ok in test");
        let var1_writers = table
            .get_writers("var1")
            .expect("get_writers should return Ok in test");
        let var2_readers = table
            .get_readers("var2")
            .expect("get_readers should return Ok in test");

        assert_eq!(var1_readers.len(), 1);
        assert_eq!(var1_writers.len(), 1);
        assert_eq!(var2_readers.len(), 1);

        // 测试节点变量查询
        let node1_reads = table
            .get_variables_read_by(&node1)
            .expect("get_variables_read_by should return Ok in test");
        let node2_writes = table
            .get_variables_written_by(&node2)
            .expect("get_variables_written_by should return Ok in test");

        assert_eq!(node1_reads.len(), 1);
        assert_eq!(node2_writes.len(), 1);
    }

    #[test]
    fn test_write_conflict_detection() {
        let table = SymbolTable::new();
        table
            .new_variable("conflict_var")
            .expect("new_variable should succeed in test");

        let node1 = PlanNodeRef::new("node1".to_string(), 1);
        let node2 = PlanNodeRef::new("node2".to_string(), 2);

        // 多个节点写入同一变量
        table
            .written_by("conflict_var", node1.clone())
            .expect("written_by should succeed in test");
        table
            .written_by("conflict_var", node2.clone())
            .expect("written_by should succeed in test");

        let conflicts = table
            .detect_write_conflicts()
            .expect("detect_write_conflicts should return Ok in test");
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].0, "conflict_var");
        assert_eq!(conflicts[0].1.len(), 2);
    }

    #[test]
    fn test_variable_rename() {
        let table = SymbolTable::new();
        table
            .new_variable("old_var")
            .expect("new_variable should succeed in test");

        let node = PlanNodeRef::new("node1".to_string(), 1);
        table
            .read_by("old_var", node.clone())
            .expect("read_by should succeed in test");

        // 重命名变量
        table
            .rename_variable("old_var", "new_var")
            .expect("rename_variable should succeed in test");

        assert!(!table.has_variable("old_var"));
        assert!(table.has_variable("new_var"));

        // 检查依赖关系是否更新
        let new_var_readers = table
            .get_readers("new_var")
            .expect("get_readers should return Ok in test");
        assert_eq!(new_var_readers.len(), 1);
        assert_eq!(new_var_readers[0].id(), "node1");
    }

    #[test]
    fn test_to_string() {
        let table = SymbolTable::new();
        table
            .new_variable("test_var")
            .expect("new_variable should succeed in test");

        let table_str = table
            .to_string()
            .expect("to_string should return Ok in test");
        assert!(table_str.contains("SymbolTable"));
        assert!(table_str.contains("test_var"));
    }
}

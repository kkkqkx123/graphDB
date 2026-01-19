use crate::core::PlanNodeRef;

use dashmap::DashMap;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolType {
    Variable,
    Alias,
    Parameter,
    Function,
    Dataset,
    Vertex,
    Edge,
    Path,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub col_names: Vec<String>,
    pub readers: HashSet<PlanNodeRef>,
    pub writers: HashSet<PlanNodeRef>,
    pub user_count: Arc<AtomicU64>,
    pub created_at: std::time::SystemTime,
}

impl Symbol {
    pub fn new(name: String, symbol_type: SymbolType) -> Self {
        Self {
            name,
            symbol_type,
            col_names: Vec::new(),
            readers: HashSet::new(),
            writers: HashSet::new(),
            user_count: Arc::new(AtomicU64::new(0)),
            created_at: std::time::SystemTime::now(),
        }
    }

    pub fn with_col_names(mut self, col_names: Vec<String>) -> Self {
        self.col_names = col_names;
        self
    }

    pub fn with_type(mut self, symbol_type: SymbolType) -> Self {
        self.symbol_type = symbol_type;
        self
    }

    pub fn increment_user_count(&self) {
        self.user_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_user_count(&self) -> u64 {
        self.user_count.load(Ordering::Relaxed)
    }
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    symbols: Arc<DashMap<String, Symbol>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: Arc::new(DashMap::new()),
        }
    }

    pub fn new_variable(&self, name: &str) -> Result<Symbol, String> {
        if self.symbols.contains_key(name) {
            return Err(format!("变量 '{}' 已存在", name));
        }

        let symbol = Symbol::new(name.to_string(), SymbolType::Variable);
        self.symbols.insert(name.to_string(), symbol.clone());
        Ok(symbol)
    }

    pub fn new_dataset(&self, name: &str, col_names: Vec<String>) -> Result<Symbol, String> {
        if self.symbols.contains_key(name) {
            return Err(format!("变量 '{}' 已存在", name));
        }

        let symbol = Symbol::new(name.to_string(), SymbolType::Dataset)
            .with_col_names(col_names);
        self.symbols.insert(name.to_string(), symbol.clone());
        Ok(symbol)
    }

    pub fn has_variable(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    pub fn get_variable(&self, name: &str) -> Option<Symbol> {
        self.symbols.get(name).map(|v| v.clone())
    }

    pub fn remove_variable(&self, name: &str) -> Result<bool, String> {
        Ok(self.symbols.remove(name).is_some())
    }

    pub fn size(&self) -> usize {
        self.symbols.len()
    }

    pub fn read_by(&self, var_name: &str, node: PlanNodeRef) -> Result<(), String> {
        if let Some(mut symbol) = self.symbols.get_mut(var_name) {
            symbol.readers.insert(node);
            symbol.increment_user_count();
            Ok(())
        } else {
            Err(format!("变量 '{}' 不存在", var_name))
        }
    }

    pub fn written_by(&self, var_name: &str, node: PlanNodeRef) -> Result<(), String> {
        if let Some(mut symbol) = self.symbols.get_mut(var_name) {
            symbol.writers.insert(node);
            symbol.increment_user_count();
            Ok(())
        } else {
            Err(format!("变量 '{}' 不存在", var_name))
        }
    }

    pub fn delete_read_by(&self, var_name: &str, node: PlanNodeRef) -> Result<bool, String> {
        if let Some(mut symbol) = self.symbols.get_mut(var_name) {
            Ok(symbol.readers.remove(&node))
        } else {
            Err(format!("变量 '{}' 不存在", var_name))
        }
    }

    pub fn delete_written_by(&self, var_name: &str, node: PlanNodeRef) -> Result<bool, String> {
        if let Some(mut symbol) = self.symbols.get_mut(var_name) {
            Ok(symbol.writers.remove(&node))
        } else {
            Err(format!("变量 '{}' 不存在", var_name))
        }
    }

    pub fn update_read_by(
        &self,
        old_var: &str,
        new_var: &str,
        node: PlanNodeRef,
    ) -> Result<bool, String> {
        let mut success = false;

        if let Some(mut symbol) = self.symbols.get_mut(old_var) {
            if symbol.readers.remove(&node) {
                success = true;
            }
        }

        if let Some(mut symbol) = self.symbols.get_mut(new_var) {
            if symbol.readers.insert(node) {
                success = true;
            }
        }

        Ok(success)
    }

    pub fn update_written_by(
        &self,
        old_var: &str,
        new_var: &str,
        node: PlanNodeRef,
    ) -> Result<bool, String> {
        let mut success = false;

        if let Some(mut symbol) = self.symbols.get_mut(old_var) {
            if symbol.writers.remove(&node) {
                success = true;
            }
        }

        if let Some(mut symbol) = self.symbols.get_mut(new_var) {
            if symbol.writers.insert(node) {
                success = true;
            }
        }

        Ok(success)
    }

    pub fn rename_variable(&self, old_name: &str, new_name: &str) -> Result<(), String> {
        if let Some((_, mut symbol)) = self.symbols.remove(old_name) {
            symbol.name = new_name.to_string();
            self.symbols.insert(new_name.to_string(), symbol);
            Ok(())
        } else {
            Err(format!("变量 '{}' 不存在", old_name))
        }
    }

    pub fn get_readers(&self, var_name: &str) -> Result<Vec<PlanNodeRef>, String> {
        if let Some(symbol) = self.symbols.get(var_name) {
            Ok(symbol.readers.iter().cloned().collect())
        } else {
            Err(format!("变量 '{}' 不存在", var_name))
        }
    }

    pub fn get_writers(&self, var_name: &str) -> Result<Vec<PlanNodeRef>, String> {
        if let Some(symbol) = self.symbols.get(var_name) {
            Ok(symbol.writers.iter().cloned().collect())
        } else {
            Err(format!("变量 '{}' 不存在", var_name))
        }
    }

    pub fn get_variables_read_by(&self, node: PlanNodeRef) -> Vec<String> {
        self.symbols
            .iter()
            .filter(|entry| entry.value().readers.contains(&node))
            .map(|entry| entry.key().clone())
            .collect()
    }

    pub fn get_variables_written_by(&self, node: PlanNodeRef) -> Vec<String> {
        self.symbols
            .iter()
            .filter(|entry| entry.value().writers.contains(&node))
            .map(|entry| entry.key().clone())
            .collect()
    }

    pub fn detect_write_conflicts(&self) -> Vec<(String, Vec<PlanNodeRef>)> {
        self.symbols
            .iter()
            .filter(|entry| entry.value().writers.len() > 1)
            .map(|entry| {
                (
                    entry.key().clone(),
                    entry.value().writers.iter().cloned().collect(),
                )
            })
            .collect()
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        result.push_str("SymbolTable {\n");
        result.push_str(&format!("  symbols: {}\n", self.symbols.len()));

        for entry in self.symbols.iter() {
            let symbol = entry.value();
            result.push_str(&format!(
                "  {}: type={:?}, readers={}, writers={}, user_count={}\n",
                entry.key(),
                symbol.symbol_type,
                symbol.readers.len(),
                symbol.writers.len(),
                symbol.get_user_count()
            ));
        }

        result.push_str("}");
        result
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

use std::hash::Hash;
use std::hash::Hasher;

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Symbol {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table() {
        let table = SymbolTable::new();

        let symbol = table.new_variable("test_var").unwrap();
        assert_eq!(symbol.name, "test_var");
        assert_eq!(symbol.symbol_type, SymbolType::Variable);
        assert!(table.has_variable("test_var"));
        assert!(table.new_variable("test_var").is_err());

        let retrieved = table.get_variable("test_var").unwrap();
        assert_eq!(retrieved.name, "test_var");
        assert_eq!(retrieved.symbol_type, SymbolType::Variable);

        assert!(table.remove_variable("test_var").unwrap());
        assert!(!table.has_variable("test_var"));
    }

    #[test]
    fn test_dataset_creation() {
        let table = SymbolTable::new();
        let col_names = vec!["col1".to_string(), "col2".to_string()];

        let symbol = table.new_dataset("dataset_var", col_names.clone()).unwrap();
        assert_eq!(symbol.name, "dataset_var");
        assert_eq!(symbol.symbol_type, SymbolType::Dataset);
        assert_eq!(symbol.col_names, col_names);
    }

    #[test]
    fn test_dependency_management() {
        let table = SymbolTable::new();
        table.new_variable("var1").unwrap();
        table.new_variable("var2").unwrap();

        let node1 = PlanNodeRef::new(1);
        let node2 = PlanNodeRef::new(2);

        table.read_by("var1", node1).unwrap();
        table.written_by("var1", node2).unwrap();
        table.read_by("var2", node2).unwrap();

        let var1_readers = table.get_readers("var1").unwrap();
        let var1_writers = table.get_writers("var1").unwrap();
        let var2_readers = table.get_readers("var2").unwrap();

        assert_eq!(var1_readers.len(), 1);
        assert_eq!(var1_writers.len(), 1);
        assert_eq!(var2_readers.len(), 1);

        let node1_reads = table.get_variables_read_by(node1);
        let node2_writes = table.get_variables_written_by(node2);

        assert_eq!(node1_reads.len(), 1);
        assert_eq!(node2_writes.len(), 1);
    }

    #[test]
    fn test_write_conflict_detection() {
        let table = SymbolTable::new();
        table.new_variable("conflict_var").unwrap();

        let node1 = PlanNodeRef::new(1);
        let node2 = PlanNodeRef::new(2);

        table.written_by("conflict_var", node1).unwrap();
        table.written_by("conflict_var", node2).unwrap();

        let conflicts = table.detect_write_conflicts();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].0, "conflict_var");
        assert_eq!(conflicts[0].1.len(), 2);
    }

    #[test]
    fn test_variable_rename() {
        let table = SymbolTable::new();
        table.new_variable("old_var").unwrap();

        let node = PlanNodeRef::new(1);
        table.read_by("old_var", node).unwrap();

        table.rename_variable("old_var", "new_var").unwrap();

        assert!(!table.has_variable("old_var"));
        assert!(table.has_variable("new_var"));

        let new_var_readers = table.get_readers("new_var").unwrap();
        assert_eq!(new_var_readers.len(), 1);
        assert_eq!(new_var_readers[0].node_id(), 1);
    }

    #[test]
    fn test_to_string() {
        let table = SymbolTable::new();
        table.new_variable("test_var").unwrap();

        let table_str = table.to_string();
        assert!(table_str.contains("SymbolTable"));
        assert!(table_str.contains("test_var"));
    }

    #[test]
    fn test_user_count() {
        let table = SymbolTable::new();
        table.new_variable("counter_var").unwrap();

        let node = PlanNodeRef::new(1);

        let symbol = table.get_variable("counter_var").unwrap();
        assert_eq!(symbol.get_user_count(), 0);

        table.read_by("counter_var", node).unwrap();
        let symbol = table.get_variable("counter_var").unwrap();
        assert_eq!(symbol.get_user_count(), 1);

        let node2 = PlanNodeRef::new(2);
        table.written_by("counter_var", node2).unwrap();
        let symbol = table.get_variable("counter_var").unwrap();
        assert_eq!(symbol.get_user_count(), 2);
    }

    #[test]
    fn test_size() {
        let table = SymbolTable::new();
        assert_eq!(table.size(), 0);

        table.new_variable("var1").unwrap();
        assert_eq!(table.size(), 1);

        table.new_variable("var2").unwrap();
        assert_eq!(table.size(), 2);

        table.remove_variable("var1").unwrap();
        assert_eq!(table.size(), 1);
    }

    #[test]
    fn test_concurrent_access() {
        use std::thread;
        use std::sync::Arc;

        let table = Arc::new(SymbolTable::new());
        let table2 = table.clone();
        let table3 = table.clone();

        let handle1 = thread::spawn(move || {
            for i in 0..50 {
                let name = format!("var_thread1_{}", i);
                let _ = table2.new_variable(&name);
            }
        });

        let handle2 = thread::spawn(move || {
            for i in 0..50 {
                let name = format!("var_thread2_{}", i);
                let _ = table3.new_variable(&name);
            }
        });

        handle1.join().unwrap();
        handle2.join().unwrap();

        assert_eq!(table.size(), 100);
    }
}

//! 符号表实现
//!
//! 使用 RwLock<HashMap> 替代 DashMap，因为：
//! - SymbolTable 是查询级别的数据结构，不是全局共享的
//! - 每个 QueryContext 有自己的 SymbolTable，不同查询之间不存在并发竞争
//! - 数据量小，生命周期短，RwLock 更简单高效

use crate::core::DataType;

use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// 变量信息
///
/// Symbol的简化视图，用于变量信息交换
#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub variable_name: String,
    pub variable_type: DataType,
    pub source_clause: String,
    pub is_aggregated: bool,
    pub properties: Vec<String>,
}

impl VariableInfo {
    pub fn new(variable_name: impl Into<String>, variable_type: DataType) -> Self {
        Self {
            variable_name: variable_name.into(),
            variable_type,
            source_clause: String::new(),
            is_aggregated: false,
            properties: Vec::new(),
        }
    }

    pub fn with_source_clause(mut self, source_clause: impl Into<String>) -> Self {
        self.source_clause = source_clause.into();
        self
    }

    pub fn with_properties(mut self, properties: Vec<String>) -> Self {
        self.properties = properties;
        self
    }

    pub fn with_aggregated(mut self, is_aggregated: bool) -> Self {
        self.is_aggregated = is_aggregated;
        self
    }
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: Arc<str>,
    pub value_type: DataType,
    pub col_names: Vec<Arc<str>>,
    pub readers: HashSet<i64>,
    pub writers: HashSet<i64>,
    pub source_clause: Arc<str>,
    pub properties: Vec<Arc<str>>,
    pub is_aggregated: bool,
}

impl Symbol {
    pub fn new(name: impl Into<Arc<str>>, value_type: DataType) -> Self {
        Self {
            name: name.into(),
            value_type,
            col_names: Vec::new(),
            readers: HashSet::new(),
            writers: HashSet::new(),
            source_clause: Arc::from(""),
            properties: Vec::new(),
            is_aggregated: false,
        }
    }

    pub fn with_col_names(mut self, col_names: Vec<impl Into<Arc<str>>>) -> Self {
        self.col_names = col_names.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_type(mut self, value_type: DataType) -> Self {
        self.value_type = value_type;
        self
    }

    pub fn with_source_clause(mut self, source_clause: impl Into<Arc<str>>) -> Self {
        self.source_clause = source_clause.into();
        self
    }

    pub fn with_properties(mut self, properties: Vec<impl Into<Arc<str>>>) -> Self {
        self.properties = properties.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_aggregated(mut self, is_aggregated: bool) -> Self {
        self.is_aggregated = is_aggregated;
        self
    }

    pub fn to_variable_info(&self) -> VariableInfo {
        VariableInfo::new(self.name.to_string(), self.value_type.clone())
            .with_source_clause(self.source_clause.to_string())
            .with_properties(self.properties.iter().map(|p| p.to_string()).collect())
            .with_aggregated(self.is_aggregated)
    }
}

pub struct SymbolTable {
    symbols: Arc<RwLock<HashMap<String, Symbol>>>,
}

impl Clone for SymbolTable {
    fn clone(&self) -> Self {
        let new_map = self.symbols.read().clone();
        Self {
            symbols: Arc::new(RwLock::new(new_map)),
        }
    }
}

impl std::fmt::Debug for SymbolTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SymbolTable")
            .field("symbols", &self.symbols.read().len())
            .finish()
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn new_variable(&self, name: &str) -> Result<Symbol, String> {
        let mut symbols = self.symbols.write();
        if symbols.contains_key(name) {
            return Err(format!("变量 '{}' 已存在", name));
        }

        let symbol = Symbol::new(name, DataType::DataSet);
        symbols.insert(name.to_string(), symbol.clone());

        Ok(symbol)
    }

    pub fn new_variable_with_info(&self, name: &str, info: VariableInfo) -> Result<Symbol, String> {
        let mut symbols = self.symbols.write();
        if symbols.contains_key(name) {
            return Err(format!("变量 '{}' 已存在", name));
        }

        let symbol = Symbol::new(name, DataType::DataSet)
            .with_source_clause(info.source_clause)
            .with_properties(info.properties)
            .with_aggregated(info.is_aggregated);
        symbols.insert(name.to_string(), symbol.clone());

        Ok(symbol)
    }

    pub fn new_dataset(&self, name: &str, col_names: Vec<String>) -> Result<Symbol, String> {
        let mut symbols = self.symbols.write();
        if symbols.contains_key(name) {
            return Err(format!("变量 '{}' 已存在", name));
        }

        let symbol = Symbol::new(name, DataType::DataSet)
            .with_col_names(col_names);
        symbols.insert(name.to_string(), symbol.clone());

        Ok(symbol)
    }

    pub fn has_variable(&self, name: &str) -> bool {
        self.symbols.read().contains_key(name)
    }

    pub fn get_variable(&self, name: &str) -> Option<Symbol> {
        self.symbols.read().get(name).cloned()
    }

    pub fn get_variable_info(&self, name: &str) -> Option<VariableInfo> {
        self.symbols.read().get(name).map(|s| s.to_variable_info())
    }

    pub fn remove_variable(&self, name: &str) -> bool {
        self.symbols.write().remove(name).is_some()
    }

    pub fn size(&self) -> usize {
        self.symbols.read().len()
    }

    pub fn read_by(&self, var_name: &str, node_id: i64) -> bool {
        let mut symbols = self.symbols.write();
        if let Some(symbol) = symbols.get_mut(var_name) {
            symbol.readers.insert(node_id);
            true
        } else {
            false
        }
    }

    pub fn written_by(&self, var_name: &str, node_id: i64) -> bool {
        let mut symbols = self.symbols.write();
        if let Some(symbol) = symbols.get_mut(var_name) {
            symbol.writers.insert(node_id);
            true
        } else {
            false
        }
    }

    pub fn delete_read_by(&self, var_name: &str, node_id: i64) -> bool {
        let mut symbols = self.symbols.write();
        if let Some(symbol) = symbols.get_mut(var_name) {
            symbol.readers.remove(&node_id)
        } else {
            false
        }
    }

    pub fn delete_written_by(&self, var_name: &str, node_id: i64) -> bool {
        let mut symbols = self.symbols.write();
        if let Some(symbol) = symbols.get_mut(var_name) {
            symbol.writers.remove(&node_id)
        } else {
            false
        }
    }

    pub fn update_read_by(
        &self,
        old_var: &str,
        new_var: &str,
        node_id: i64,
    ) -> bool {
        let mut symbols = self.symbols.write();
        let mut success = false;

        if let Some(symbol) = symbols.get_mut(old_var) {
            if symbol.readers.remove(&node_id) {
                success = true;
            }
        }

        if let Some(symbol) = symbols.get_mut(new_var) {
            if symbol.readers.insert(node_id) {
                success = true;
            }
        }

        success
    }

    pub fn update_written_by(
        &self,
        old_var: &str,
        new_var: &str,
        node_id: i64,
    ) -> bool {
        let mut symbols = self.symbols.write();
        let mut success = false;

        if let Some(symbol) = symbols.get_mut(old_var) {
            if symbol.writers.remove(&node_id) {
                success = true;
            }
        }

        if let Some(symbol) = symbols.get_mut(new_var) {
            if symbol.writers.insert(node_id) {
                success = true;
            }
        }

        success
    }

    pub fn to_string(&self) -> String {
        let symbols = self.symbols.read();
        let mut result = String::new();
        result.push_str("SymbolTable {\n");
        result.push_str(&format!("  symbols: {}\n", symbols.len()));

        for (name, symbol) in symbols.iter() {
            result.push_str(&format!(
                "  {}: type={:?}, readers={}, writers={}\n",
                name,
                symbol.value_type,
                symbol.readers.len(),
                symbol.writers.len()
            ));
        }

        result.push_str("}");
        result
    }

    pub fn get_variables_by_type(&self, var_type: &DataType) -> Vec<VariableInfo> {
        let symbols = self.symbols.read();
        symbols
            .values()
            .filter(|s| s.value_type == *var_type)
            .map(|s| s.to_variable_info())
            .collect()
    }

    pub fn get_variables_by_source(&self, source: &str) -> Vec<VariableInfo> {
        let symbols = self.symbols.read();
        symbols
            .values()
            .filter(|s| s.source_clause.as_ref() == source)
            .map(|s| s.to_variable_info())
            .collect()
    }

    pub fn get_aggregated_variables(&self) -> Vec<VariableInfo> {
        let symbols = self.symbols.read();
        symbols
            .values()
            .filter(|s| s.is_aggregated)
            .map(|s| s.to_variable_info())
            .collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = (String, Symbol)> + '_ {
        let symbols = self.symbols.read();
        // 收集到 Vec 避免长期持有锁
        symbols
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<_>>()
            .into_iter()
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

        let symbol = table.new_variable("test_var").expect("创建变量失败");
        assert_eq!(symbol.name.as_ref(), "test_var");
        assert!(table.has_variable("test_var"));

        let retrieved = table.get_variable("test_var").expect("获取变量失败");
        assert_eq!(retrieved.name.as_ref(), "test_var");

        assert!(table.new_variable("test_var").is_err());

        let removed = table.remove_variable("test_var");
        assert!(removed);
        assert!(!table.has_variable("test_var"));
    }

    #[test]
    fn test_symbol_readers_writers() {
        let table = SymbolTable::new();
        table.new_variable("var1").expect("创建变量失败");

        assert!(table.read_by("var1", 1));
        assert!(table.written_by("var1", 2));

        let symbol = table.get_variable("var1").expect("获取变量失败");
        assert!(symbol.readers.contains(&1));
        assert!(symbol.writers.contains(&2));

        let deleted = table.delete_read_by("var1", 1);
        assert!(deleted);

        let symbol = table.get_variable("var1").expect("获取变量失败");
        assert!(!symbol.readers.contains(&1));
    }

    #[test]
    fn test_symbol_table_to_string() {
        let table = SymbolTable::new();
        table.new_variable("var1").expect("创建变量失败");
        table.new_variable("var2").expect("创建变量失败");

        let table_str = table.to_string();
        assert!(table_str.contains("SymbolTable"));
        assert!(table_str.contains("var1"));
        assert!(table_str.contains("var2"));
    }

    #[test]
    fn test_symbol_table_clone() {
        let table1 = SymbolTable::new();
        table1.new_variable("var1").expect("创建变量失败");

        let table2 = table1.clone();
        assert!(table2.has_variable("var1"));

        table2.new_variable("var2").expect("创建变量失败");
        assert!(!table1.has_variable("var2"));
    }

    #[test]
    fn test_get_variables_by_type() {
        let table = SymbolTable::new();
        table.new_variable("var1").expect("创建变量失败");

        let vars = table.get_variables_by_type(&DataType::DataSet);
        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0].variable_name, "var1");
    }

    #[test]
    fn test_get_variables_by_source() {
        let table = SymbolTable::new();
        let info = VariableInfo::new("var1".to_string(), DataType::DataSet)
            .with_source_clause("MATCH".to_string());
        table.new_variable_with_info("var1", info).expect("创建变量失败");

        let vars = table.get_variables_by_source("MATCH");
        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0].variable_name, "var1");
    }

    #[test]
    fn test_get_aggregated_variables() {
        let table = SymbolTable::new();
        let info = VariableInfo::new("var1".to_string(), DataType::DataSet)
            .with_aggregated(true);
        table.new_variable_with_info("var1", info).expect("创建变量失败");
        table.new_variable("var2").expect("创建变量失败");

        let vars = table.get_aggregated_variables();
        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0].variable_name, "var1");
    }
}

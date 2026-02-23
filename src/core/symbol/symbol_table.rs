//! 符号表实现

use crate::core::DataType;
use crate::core::types::VariableInfo;

use dashmap::DashMap;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub value_type: DataType,
    pub col_names: Vec<String>,
    pub readers: HashSet<i64>,
    pub writers: HashSet<i64>,
    pub source_clause: String,
    pub properties: Vec<String>,
    pub is_aggregated: bool,
}

impl Symbol {
    pub fn new(name: String, value_type: DataType) -> Self {
        Self {
            name,
            value_type,
            col_names: Vec::new(),
            readers: HashSet::new(),
            writers: HashSet::new(),
            source_clause: String::new(),
            properties: Vec::new(),
            is_aggregated: false,
        }
    }

    pub fn with_col_names(mut self, col_names: Vec<String>) -> Self {
        self.col_names = col_names;
        self
    }

    pub fn with_type(mut self, value_type: DataType) -> Self {
        self.value_type = value_type;
        self
    }

    pub fn with_source_clause(mut self, source_clause: String) -> Self {
        self.source_clause = source_clause;
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

    pub fn to_variable_info(&self) -> VariableInfo {
        VariableInfo::new(self.name.clone(), format!("{:?}", self.value_type))
            .with_source_clause(self.source_clause.clone())
            .with_properties(self.properties.clone())
            .with_aggregated(self.is_aggregated)
    }
}

pub struct SymbolTable {
    symbols: Arc<DashMap<String, Symbol>>,
}

impl Clone for SymbolTable {
    fn clone(&self) -> Self {
        let new_map = DashMap::new();
        for entry in self.symbols.iter() {
            new_map.insert(entry.key().clone(), entry.value().clone());
        }
        Self {
            symbols: Arc::new(new_map),
        }
    }
}

impl std::fmt::Debug for SymbolTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SymbolTable")
            .field("symbols", &self.symbols.len())
            .finish()
    }
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

        let symbol = Symbol::new(name.to_string(), DataType::DataSet);
        self.symbols.insert(name.to_string(), symbol.clone());
        
        Ok(symbol)
    }

    pub fn new_variable_with_info(&self, name: &str, info: VariableInfo) -> Result<Symbol, String> {
        if self.symbols.contains_key(name) {
            return Err(format!("变量 '{}' 已存在", name));
        }

        let symbol = Symbol::new(name.to_string(), DataType::DataSet)
            .with_source_clause(info.source_clause)
            .with_properties(info.properties)
            .with_aggregated(info.is_aggregated);
        self.symbols.insert(name.to_string(), symbol.clone());
        
        Ok(symbol)
    }

    pub fn new_dataset(&self, name: &str, col_names: Vec<String>) -> Result<Symbol, String> {
        if self.symbols.contains_key(name) {
            return Err(format!("变量 '{}' 已存在", name));
        }

        let symbol = Symbol::new(name.to_string(), DataType::DataSet)
            .with_col_names(col_names);
        self.symbols.insert(name.to_string(), symbol.clone());
        
        Ok(symbol)
    }

    pub fn has_variable(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    pub fn get_variable(&self, name: &str) -> Option<Symbol> {
        self.symbols.get(name).map(|entry| entry.clone())
    }

    pub fn get_variable_info(&self, name: &str) -> Option<VariableInfo> {
        self.symbols.get(name).map(|entry| entry.to_variable_info())
    }

    pub fn remove_variable(&self, name: &str) -> Result<bool, String> {
        let result = self.symbols.remove(name).is_some();
        Ok(result)
    }

    pub fn size(&self) -> usize {
        self.symbols.len()
    }

    pub fn read_by(&self, var_name: &str, node_id: i64) -> Result<(), String> {
        if let Some(mut entry) = self.symbols.get_mut(var_name) {
            entry.readers.insert(node_id);
            Ok(())
        } else {
            Err(format!("变量 '{}' 不存在", var_name))
        }
    }

    pub fn written_by(&self, var_name: &str, node_id: i64) -> Result<(), String> {
        if let Some(mut entry) = self.symbols.get_mut(var_name) {
            entry.writers.insert(node_id);
            Ok(())
        } else {
            Err(format!("变量 '{}' 不存在", var_name))
        }
    }

    pub fn delete_read_by(&self, var_name: &str, node_id: i64) -> Result<bool, String> {
        if let Some(mut entry) = self.symbols.get_mut(var_name) {
            let result = entry.readers.remove(&node_id);
            Ok(result)
        } else {
            Err(format!("变量 '{}' 不存在", var_name))
        }
    }

    pub fn delete_written_by(&self, var_name: &str, node_id: i64) -> Result<bool, String> {
        if let Some(mut entry) = self.symbols.get_mut(var_name) {
            let result = entry.writers.remove(&node_id);
            Ok(result)
        } else {
            Err(format!("变量 '{}' 不存在", var_name))
        }
    }

    pub fn update_read_by(
        &self,
        old_var: &str,
        new_var: &str,
        node_id: i64,
    ) -> Result<bool, String> {
        let mut success = false;

        if let Some(mut entry) = self.symbols.get_mut(old_var) {
            if entry.readers.remove(&node_id) {
                success = true;
            }
        }

        if let Some(mut entry) = self.symbols.get_mut(new_var) {
            if entry.writers.insert(node_id) {
                success = true;
            }
        }

        Ok(success)
    }

    pub fn update_written_by(
        &self,
        old_var: &str,
        new_var: &str,
        node_id: i64,
    ) -> Result<bool, String> {
        let mut success = false;

        if let Some(mut entry) = self.symbols.get_mut(old_var) {
            if entry.writers.remove(&node_id) {
                success = true;
            }
        }

        if let Some(mut entry) = self.symbols.get_mut(new_var) {
            if entry.writers.insert(node_id) {
                success = true;
            }
        }

        Ok(success)
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        result.push_str("SymbolTable {\n");
        result.push_str(&format!("  symbols: {}\n", self.symbols.len()));

        for entry in self.symbols.iter() {
            let name = entry.key();
            let symbol = entry.value();
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

    pub fn get_variables_by_type(&self, var_type: &str) -> Vec<VariableInfo> {
        self.symbols
            .iter()
            .filter(|entry| format!("{:?}", entry.value().value_type).to_lowercase().contains(&var_type.to_lowercase()))
            .map(|entry| entry.value().to_variable_info())
            .collect()
    }

    pub fn get_variables_by_source(&self, source: &str) -> Vec<VariableInfo> {
        self.symbols
            .iter()
            .filter(|entry| entry.value().source_clause == source)
            .map(|entry| entry.value().to_variable_info())
            .collect()
    }

    pub fn get_aggregated_variables(&self) -> Vec<VariableInfo> {
        self.symbols
            .iter()
            .filter(|entry| entry.value().is_aggregated)
            .map(|entry| entry.value().to_variable_info())
            .collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = (String, Symbol)> + '_ {
        self.symbols.iter().map(|entry| (entry.key().clone(), entry.value().clone()))
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
        assert_eq!(symbol.name, "test_var");
        assert!(table.has_variable("test_var"));

        let retrieved = table.get_variable("test_var").expect("获取变量失败");
        assert_eq!(retrieved.name, "test_var");

        assert!(table.new_variable("test_var").is_err());

        let removed = table.remove_variable("test_var").expect("删除变量失败");
        assert!(removed);
        assert!(!table.has_variable("test_var"));
    }

    #[test]
    fn test_symbol_readers_writers() {
        let table = SymbolTable::new();
        table.new_variable("var1").expect("创建变量失败");

        table.read_by("var1", 1).expect("添加读者失败");
        table.written_by("var1", 2).expect("添加写者失败");

        let symbol = table.get_variable("var1").expect("获取变量失败");
        assert!(symbol.readers.contains(&1));
        assert!(symbol.writers.contains(&2));

        let deleted = table.delete_read_by("var1", 1).expect("删除读者失败");
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

        let vars = table.get_variables_by_type("dataset");
        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0].variable_name, "var1");
    }

    #[test]
    fn test_get_variables_by_source() {
        let table = SymbolTable::new();
        let info = VariableInfo::new("var1".to_string(), "DataSet".to_string())
            .with_source_clause("MATCH".to_string());
        table.new_variable_with_info("var1", info).expect("创建变量失败");

        let vars = table.get_variables_by_source("MATCH");
        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0].variable_name, "var1");
    }

    #[test]
    fn test_get_aggregated_variables() {
        let table = SymbolTable::new();
        let info = VariableInfo::new("var1".to_string(), "DataSet".to_string())
            .with_aggregated(true);
        table.new_variable_with_info("var1", info).expect("创建变量失败");
        table.new_variable("var2").expect("创建变量失败");

        let vars = table.get_aggregated_variables();
        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0].variable_name, "var1");
    }
}

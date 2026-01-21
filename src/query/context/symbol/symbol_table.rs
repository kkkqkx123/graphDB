use crate::core::PlanNodeRef;
use crate::core::value::types::ValueTypeDef;
use crate::query::context::ast::VariableInfo;

use dashmap::DashMap;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub value_type: ValueTypeDef,
    pub col_names: Vec<String>,
    pub readers: HashSet<PlanNodeRef>,
    pub writers: HashSet<PlanNodeRef>,
    pub source_clause: String,
    pub properties: Vec<String>,
    pub is_aggregated: bool,
}

impl Symbol {
    pub fn new(name: String, value_type: ValueTypeDef) -> Self {
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

    pub fn with_type(mut self, value_type: ValueTypeDef) -> Self {
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

#[derive(Debug, Clone)]
pub struct SymbolTable {
    symbols: Arc<DashMap<String, Symbol>>,
    pub scope_stack: Vec<HashSet<String>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: Arc::new(DashMap::new()),
            scope_stack: vec![HashSet::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scope_stack.push(HashSet::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scope_stack.len() > 1 {
            let popped_scope = self.scope_stack.pop().unwrap();
            for name in popped_scope {
                let _ = self.symbols.remove(&name);
            }
        }
    }

    pub fn current_scope(&self) -> &HashSet<String> {
        self.scope_stack.last().expect("Scope stack is empty")
    }

    pub fn new_variable(&self, name: &str) -> Result<Symbol, String> {
        if self.symbols.contains_key(name) {
            return Err(format!("变量 '{}' 已存在", name));
        }

        let symbol = Symbol::new(name.to_string(), ValueTypeDef::DataSet);
        self.symbols.insert(name.to_string(), symbol.clone());
        Ok(symbol)
    }

    pub fn new_variable_with_info(&self, name: &str, info: VariableInfo) -> Result<Symbol, String> {
        if self.symbols.contains_key(name) {
            return Err(format!("变量 '{}' 已存在", name));
        }

        let symbol = Symbol::new(name.to_string(), ValueTypeDef::DataSet)
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

        let symbol = Symbol::new(name.to_string(), ValueTypeDef::DataSet)
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

    pub fn get_variable_info(&self, name: &str) -> Option<VariableInfo> {
        self.symbols.get(name).map(|s| s.to_variable_info())
    }

    pub fn remove_variable(&self, name: &str) -> Result<bool, String> {
        Ok(self.symbols.remove(name).is_some())
    }

    pub fn size(&self) -> usize {
        self.symbols.len()
    }

    pub fn current_scope_size(&self) -> usize {
        self.current_scope().len()
    }

    pub fn read_by(&self, var_name: &str, node: PlanNodeRef) -> Result<(), String> {
        if let Some(mut symbol) = self.symbols.get_mut(var_name) {
            symbol.readers.insert(node);
            Ok(())
        } else {
            Err(format!("变量 '{}' 不存在", var_name))
        }
    }

    pub fn written_by(&self, var_name: &str, node: PlanNodeRef) -> Result<(), String> {
        if let Some(mut symbol) = self.symbols.get_mut(var_name) {
            symbol.writers.insert(node);
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
            if symbol.writers.insert(node) {
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

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        result.push_str("SymbolTable {\n");
        result.push_str(&format!("  symbols: {}\n", self.symbols.len()));
        result.push_str(&format!("  scope_depth: {}\n", self.scope_stack.len()));

        for entry in self.symbols.iter() {
            let symbol = entry.value();
            result.push_str(&format!(
                "  {}: type={:?}, readers={}, writers={}\n",
                entry.key(),
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
            .filter(|s| format!("{:?}", s.value_type).to_lowercase().contains(&var_type.to_lowercase()))
            .map(|s| s.to_variable_info())
            .collect()
    }

    pub fn get_variables_by_source(&self, source: &str) -> Vec<VariableInfo> {
        self.symbols
            .iter()
            .filter(|s| s.source_clause == source)
            .map(|s| s.to_variable_info())
            .collect()
    }

    pub fn get_aggregated_variables(&self) -> Vec<VariableInfo> {
        self.symbols
            .iter()
            .filter(|s| s.is_aggregated)
            .map(|s| s.to_variable_info())
            .collect()
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
        assert_eq!(symbol.value_type, ValueTypeDef::DataSet);
        assert!(table.has_variable("test_var"));
        assert!(table.new_variable("test_var").is_err());

        let retrieved = table.get_variable("test_var").unwrap();
        assert_eq!(retrieved.name, "test_var");
        assert_eq!(retrieved.value_type, ValueTypeDef::DataSet);

        assert!(table.remove_variable("test_var").unwrap());
        assert!(!table.has_variable("test_var"));
    }

    #[test]
    fn test_dataset_creation() {
        let table = SymbolTable::new();
        let col_names = vec!["col1".to_string(), "col2".to_string()];

        let symbol = table.new_dataset("dataset_var", col_names.clone()).unwrap();
        assert_eq!(symbol.name, "dataset_var");
        assert_eq!(symbol.value_type, ValueTypeDef::DataSet);
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

        let var1 = table.get_variable("var1").unwrap();
        let var2 = table.get_variable("var2").unwrap();

        assert_eq!(var1.readers.len(), 1);
        assert_eq!(var1.writers.len(), 1);
        assert_eq!(var2.readers.len(), 1);
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

    #[test]
    fn test_variable_info_conversion() {
        let table = SymbolTable::new();

        let var_info = VariableInfo::new("dst".to_string(), "vertex".to_string())
            .with_source_clause("GO".to_string())
            .with_properties(vec!["_dst".to_string()])
            .with_aggregated(false);

        table.new_variable_with_info("dst", var_info).unwrap();

        let info = table.get_variable_info("dst").unwrap();
        assert_eq!(info.variable_name, "dst");
        assert_eq!(info.source_clause, "GO");
        assert_eq!(info.properties, vec!["_dst"]);
    }

    #[test]
    fn test_scope_management() {
        let mut table = SymbolTable::new();

        table.new_variable("var1").unwrap();
        assert_eq!(table.size(), 1);

        table.push_scope();
        table.new_variable("var2").unwrap();
        assert_eq!(table.size(), 2);

        table.pop_scope();
        assert_eq!(table.size(), 1);
        assert!(!table.has_variable("var2"));
    }

    #[test]
    fn test_get_aggregated_variables() {
        let table = SymbolTable::new();

        let var1 = VariableInfo::new("var1".to_string(), "vertex".to_string())
            .with_aggregated(false);
        let var2 = VariableInfo::new("var2".to_string(), "integer".to_string())
            .with_aggregated(true);

        table.new_variable_with_info("var1", var1).unwrap();
        table.new_variable_with_info("var2", var2).unwrap();

        let aggregated = table.get_aggregated_variables();
        assert_eq!(aggregated.len(), 1);
        assert_eq!(aggregated[0].variable_name, "var2");
    }
}

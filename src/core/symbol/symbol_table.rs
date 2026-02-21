//! 符号表实现

use crate::core::DataType;
use crate::core::types::VariableInfo;

use std::collections::HashMap;
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
    symbols: Arc<HashMap<String, Symbol>>,
}

impl Clone for SymbolTable {
    fn clone(&self) -> Self {
        Self {
            symbols: Arc::clone(&self.symbols),
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
            symbols: Arc::new(HashMap::new()),
        }
    }

    pub fn new_variable(&mut self, name: &str) -> Result<Symbol, String> {
        if self.symbols.contains_key(name) {
            return Err(format!("变量 '{}' 已存在", name));
        }

        let symbol = Symbol::new(name.to_string(), DataType::DataSet);
        let mut new_symbols = (*self.symbols).clone();
        new_symbols.insert(name.to_string(), symbol.clone());
        self.symbols = Arc::new(new_symbols);
        
        Ok(symbol)
    }

    pub fn new_variable_with_info(&mut self, name: &str, info: VariableInfo) -> Result<Symbol, String> {
        if self.symbols.contains_key(name) {
            return Err(format!("变量 '{}' 已存在", name));
        }

        let symbol = Symbol::new(name.to_string(), DataType::DataSet)
            .with_source_clause(info.source_clause)
            .with_properties(info.properties)
            .with_aggregated(info.is_aggregated);
        let mut new_symbols = (*self.symbols).clone();
        new_symbols.insert(name.to_string(), symbol.clone());
        self.symbols = Arc::new(new_symbols);
        
        Ok(symbol)
    }

    pub fn new_dataset(&mut self, name: &str, col_names: Vec<String>) -> Result<Symbol, String> {
        if self.symbols.contains_key(name) {
            return Err(format!("变量 '{}' 已存在", name));
        }

        let symbol = Symbol::new(name.to_string(), DataType::DataSet)
            .with_col_names(col_names);
        let mut new_symbols = (*self.symbols).clone();
        new_symbols.insert(name.to_string(), symbol.clone());
        self.symbols = Arc::new(new_symbols);
        
        Ok(symbol)
    }

    pub fn has_variable(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    pub fn get_variable(&self, name: &str) -> Option<Symbol> {
        self.symbols.get(name).cloned()
    }

    pub fn get_variable_info(&self, name: &str) -> Option<VariableInfo> {
        self.symbols.get(name).map(|s| s.to_variable_info())
    }

    pub fn remove_variable(&mut self, name: &str) -> Result<bool, String> {
        let mut new_symbols = (*self.symbols).clone();
        let result = new_symbols.remove(name).is_some();
        self.symbols = Arc::new(new_symbols);
        Ok(result)
    }

    pub fn size(&self) -> usize {
        self.symbols.len()
    }

    pub fn read_by(&mut self, var_name: &str, node_id: i64) -> Result<(), String> {
        if self.symbols.contains_key(var_name) {
            let mut new_symbols = (*self.symbols).clone();
            if let Some(symbol) = new_symbols.get_mut(var_name) {
                symbol.readers.insert(node_id);
                self.symbols = Arc::new(new_symbols);
                return Ok(());
            }
        }
        Err(format!("变量 '{}' 不存在", var_name))
    }

    pub fn written_by(&mut self, var_name: &str, node_id: i64) -> Result<(), String> {
        if self.symbols.contains_key(var_name) {
            let mut new_symbols = (*self.symbols).clone();
            if let Some(symbol) = new_symbols.get_mut(var_name) {
                symbol.writers.insert(node_id);
                self.symbols = Arc::new(new_symbols);
                return Ok(());
            }
        }
        Err(format!("变量 '{}' 不存在", var_name))
    }

    pub fn delete_read_by(&mut self, var_name: &str, node_id: i64) -> Result<bool, String> {
        let mut new_symbols = (*self.symbols).clone();
        if let Some(symbol) = new_symbols.get_mut(var_name) {
            let result = symbol.readers.remove(&node_id);
            self.symbols = Arc::new(new_symbols);
            return Ok(result);
        }
        Err(format!("变量 '{}' 不存在", var_name))
    }

    pub fn delete_written_by(&mut self, var_name: &str, node_id: i64) -> Result<bool, String> {
        let mut new_symbols = (*self.symbols).clone();
        if let Some(symbol) = new_symbols.get_mut(var_name) {
            let result = symbol.writers.remove(&node_id);
            self.symbols = Arc::new(new_symbols);
            return Ok(result);
        }
        Err(format!("变量 '{}' 不存在", var_name))
    }

    pub fn update_read_by(
        &mut self,
        old_var: &str,
        new_var: &str,
        node_id: i64,
    ) -> Result<bool, String> {
        let mut success = false;
        let mut new_symbols = (*self.symbols).clone();

        if let Some(symbol) = new_symbols.get_mut(old_var) {
            if symbol.readers.remove(&node_id) {
                success = true;
            }
        }

        if let Some(symbol) = new_symbols.get_mut(new_var) {
            if symbol.writers.insert(node_id) {
                success = true;
            }
        }

        self.symbols = Arc::new(new_symbols);
        Ok(success)
    }

    pub fn update_written_by(
        &mut self,
        old_var: &str,
        new_var: &str,
        node_id: i64,
    ) -> Result<bool, String> {
        let mut success = false;
        let mut new_symbols = (*self.symbols).clone();

        if let Some(symbol) = new_symbols.get_mut(old_var) {
            if symbol.writers.remove(&node_id) {
                success = true;
            }
        }

        if let Some(symbol) = new_symbols.get_mut(new_var) {
            if symbol.writers.insert(node_id) {
                success = true;
            }
        }

        self.symbols = Arc::new(new_symbols);
        Ok(success)
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        result.push_str("SymbolTable {\n");
        result.push_str(&format!("  symbols: {}\n", self.symbols.len()));

        for (name, symbol) in self.symbols.iter() {
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
            .filter(|(_, s)| format!("{:?}", s.value_type).to_lowercase().contains(&var_type.to_lowercase()))
            .map(|(_, s)| s.to_variable_info())
            .collect()
    }

    pub fn get_variables_by_source(&self, source: &str) -> Vec<VariableInfo> {
        self.symbols
            .iter()
            .filter(|(_, s)| s.source_clause == source)
            .map(|(_, s)| s.to_variable_info())
            .collect()
    }

    pub fn get_aggregated_variables(&self) -> Vec<VariableInfo> {
        self.symbols
            .iter()
            .filter(|(_, s)| s.is_aggregated)
            .map(|(_, s)| s.to_variable_info())
            .collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Symbol)> {
        self.symbols.iter()
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

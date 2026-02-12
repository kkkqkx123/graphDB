//! 代价模型核心类型
//! 定义 Cost、Statistics 等代价估算相关的核心结构体

use std::collections::HashMap;

/// 代价值（单精度浮点数）
/// 与 nebula-graph 保持一致，使用简单的单一代价值
pub type Cost = f64;

#[derive(Debug, Clone)]
pub struct TableStats {
    pub row_count: u64,
    pub column_stats: HashMap<String, ColumnStats>,
}

#[derive(Debug, Clone)]
pub struct ColumnStats {
    pub distinct_count: u64,
    pub min_value: Option<String>,
    pub max_value: Option<String>,
    pub null_count: u64,
}

impl Default for TableStats {
    fn default() -> Self {
        Self {
            row_count: 0,
            column_stats: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Statistics {
    pub table_stats: HashMap<String, TableStats>,
    pub estimated_row_counts: HashMap<usize, u64>,
}

impl Default for Statistics {
    fn default() -> Self {
        Self {
            table_stats: HashMap::new(),
            estimated_row_counts: HashMap::new(),
        }
    }
}

impl Statistics {
    pub fn get_table_stats(&self, table_name: &str) -> Option<&TableStats> {
        self.table_stats.get(table_name)
    }

    pub fn get_estimated_rows(&self, node_id: usize) -> Option<&u64> {
        self.estimated_row_counts.get(&node_id)
    }

    pub fn set_estimated_rows(&mut self, node_id: usize, rows: u64) {
        self.estimated_row_counts.insert(node_id, rows);
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PlanNodeProperties {
    pub output_vars: Vec<String>,
    pub required_props: Vec<String>,
    pub input_vars: Vec<String>,
    pub output_cols: Vec<String>,
    pub is_agg: bool,
    pub group_keys: Vec<String>,
}

impl PlanNodeProperties {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_output_vars(output_vars: Vec<String>) -> Self {
        Self {
            output_vars,
            ..Self::default()
        }
    }

    pub fn add_output_var(&mut self, var: String) {
        self.output_vars.push(var);
    }

    pub fn add_required_prop(&mut self, prop: String) {
        self.required_props.push(prop);
    }

    pub fn add_input_var(&mut self, var: String) {
        self.input_vars.push(var);
    }

    pub fn add_output_col(&mut self, col: String) {
        self.output_cols.push(col);
    }

    pub fn set_agg(&mut self, is_agg: bool) {
        self.is_agg = is_agg;
    }

    pub fn add_group_key(&mut self, key: String) {
        self.group_keys.push(key);
    }
}

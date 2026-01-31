//! 代价模型核心类型
//! 定义 Cost、Statistics 等代价估算相关的核心结构体

use std::collections::HashMap;
use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cost {
    pub cpu_cost: f64,
    pub io_cost: f64,
    pub memory_cost: f64,
    pub network_cost: f64,
}

impl fmt::Display for Cost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cost(cpu={:.2}, io={:.2}, memory={:.2}, network={:.2}, total={:.2})",
            self.cpu_cost, self.io_cost, self.memory_cost, self.network_cost, self.total()
        )
    }
}

impl Default for Cost {
    fn default() -> Self {
        Self {
            cpu_cost: 0.0,
            io_cost: 0.0,
            memory_cost: 0.0,
            network_cost: 0.0,
        }
    }
}

impl Cost {
    pub fn new(cpu: f64, io: f64, memory: f64, network: f64) -> Self {
        Self {
            cpu_cost: cpu,
            io_cost: io,
            memory_cost: memory,
            network_cost: network,
        }
    }

    pub fn total(&self) -> f64 {
        self.cpu_cost + self.io_cost + self.memory_cost + self.network_cost
    }

    pub fn is_zero(&self) -> bool {
        self.cpu_cost == 0.0 && self.io_cost == 0.0 && self.memory_cost == 0.0 && self.network_cost == 0.0
    }
}

impl PartialOrd for Cost {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.total().partial_cmp(&other.total())
    }
}

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

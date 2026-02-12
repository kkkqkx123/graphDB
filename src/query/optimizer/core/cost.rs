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

/// 反馈统计
/// 用于记录估算值与实际值的对比，校准后续估算
#[derive(Debug, Clone, Default)]
pub struct FeedbackStats {
    /// 总估算行数
    pub total_estimates: u64,
    /// 总实际行数
    pub total_actual: u64,
    /// 样本数量
    pub sample_count: u64,
}

impl FeedbackStats {
    /// 创建新的反馈统计
    pub fn new() -> Self {
        Self::default()
    }

    /// 记录一次估算与实际值的对比
    pub fn record(&mut self, estimated: u64, actual: u64) {
        self.total_estimates += estimated;
        self.total_actual += actual;
        self.sample_count += 1;
    }

    /// 计算平均误差率
    /// 返回值范围 [0.0, 1.0]，0.0 表示完全准确
    pub fn average_error_rate(&self) -> f64 {
        if self.sample_count == 0 {
            return 0.0;
        }

        let avg_estimate = self.total_estimates as f64 / self.sample_count as f64;
        let avg_actual = self.total_actual as f64 / self.sample_count as f64;

        if avg_actual == 0.0 {
            return 0.0;
        }

        ((avg_estimate - avg_actual).abs() / avg_actual).min(1.0)
    }

    /// 判断估算是否系统性偏高
    pub fn is_consistently_overestimating(&self) -> bool {
        if self.sample_count < 10 {
            return false;
        }
        self.total_estimates > self.total_actual
    }

    /// 判断估算是否系统性偏低
    pub fn is_consistently_underestimating(&self) -> bool {
        if self.sample_count < 10 {
            return false;
        }
        self.total_estimates < self.total_actual
    }

    /// 获取校准因子
    /// 用于调整后续估算值
    pub fn get_calibration_factor(&self) -> f64 {
        if self.sample_count < 5 || self.total_actual == 0 {
            return 1.0;
        }

        let avg_estimate = self.total_estimates as f64 / self.sample_count as f64;
        let avg_actual = self.total_actual as f64 / self.sample_count as f64;

        avg_actual / avg_estimate
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

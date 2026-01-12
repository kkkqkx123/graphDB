//! 结果集结构定义
//!
//! 包含查询结果集和执行统计的相关数据结构

use serde::{Deserialize, Serialize};

/// 表示可能包含多种类型数据的结果集
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultSet {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<crate::core::Value>>,
    pub stats: ExecutionStats,
}

/// 查询执行的统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStats {
    pub total_vertices: usize,
    pub total_edges: usize,
    pub vertices_scanned: usize,
    pub edges_scanned: usize,
    pub execution_time_ms: u64,
    pub memory_used_bytes: usize,
}

impl ExecutionStats {
    pub fn new() -> Self {
        Self {
            total_vertices: 0,
            total_edges: 0,
            vertices_scanned: 0,
            edges_scanned: 0,
            execution_time_ms: 0,
            memory_used_bytes: 0,
        }
    }
}

impl Default for ExecutionStats {
    fn default() -> Self {
        Self::new()
    }
}

impl ResultSet {
    pub fn new(columns: Vec<String>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
            stats: ExecutionStats::new(),
        }
    }

    pub fn add_row(&mut self, row: Vec<crate::core::Value>) {
        if row.len() == self.columns.len() {
            self.rows.push(row);
        }
    }

    pub fn with_stats(mut self, stats: ExecutionStats) -> Self {
        self.stats = stats;
        self
    }
}

//! 统计信息节点实现
//!
//! 提供统计信息查询相关的计划节点定义

use crate::define_plan_node;

define_plan_node! {
    pub struct ShowStatsNode {
        stats_type: ShowStatsType,
    }
    enum: ShowStats
    input: ZeroInputNode
}

impl ShowStatsNode {
    pub fn new(id: i64, stats_type: ShowStatsType) -> Self {
        Self {
            id,
            stats_type,
            output_var: None,
            col_names: Vec::new(),
        }
    }

    pub fn stats_type(&self) -> &ShowStatsType {
        &self.stats_type
    }
}

/// 显示统计类型
#[derive(Debug, Clone)]
pub enum ShowStatsType {
    /// 显示存储统计
    Storage,
    /// 显示空间统计
    Space,
}

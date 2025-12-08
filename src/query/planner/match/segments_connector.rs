//! 计划段连接器
//! 连接多个计划段形成完整的执行计划

use crate::query::planner::plan::SubPlan;

/// 计划段连接器
/// 负责将多个计划段连接成完整的执行计划
#[derive(Debug)]
pub struct SegmentsConnector;

impl SegmentsConnector {
    /// 创建新的段连接器
    pub fn new() -> Self {
        Self
    }

    /// 连接多个子计划段
    pub fn connect_segments(&self, segments: Vec<SubPlan>) -> SubPlan {
        // TODO: 实现段连接逻辑
        SubPlan::new(None, None)
    }
}

impl Default for SegmentsConnector {
    fn default() -> Self {
        Self::new()
    }
}

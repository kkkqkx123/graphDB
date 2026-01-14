//! 扫描查找策略
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;

#[derive(Debug)]
pub struct ScanSeek;

impl ScanSeek {
    pub fn new() -> Self {
        Self
    }
}

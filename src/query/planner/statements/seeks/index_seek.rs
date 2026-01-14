//! 索引查找策略
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;

#[derive(Debug)]
pub struct IndexSeek;

impl IndexSeek {
    pub fn new() -> Self {
        Self
    }
}

pub type IndexScanMetadata = ();
pub type IndexSeekType = ();

//! 顶点查找策略
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;

#[derive(Debug)]
pub struct VertexSeek;

impl VertexSeek {
    pub fn new() -> Self {
        Self
    }
}

pub type VertexSeekType = ();

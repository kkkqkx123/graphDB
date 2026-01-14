//! 查找策略
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;

#[derive(Debug)]
pub struct SeekStrategy;

impl SeekStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug)]
pub struct SeekStrategySelector;

impl SeekStrategySelector {
    pub fn new() -> Self {
        Self
    }
}

pub type SeekStrategyType = ();

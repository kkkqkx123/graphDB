//! MATCH 路径规划器
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;

#[derive(Debug)]
pub struct MatchPathPlanner;

impl MatchPathPlanner {
    pub fn new() -> Self {
        Self
    }
}

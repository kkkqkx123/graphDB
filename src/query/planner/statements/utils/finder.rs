//! 查找器
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;

#[derive(Debug)]
pub struct Finder;

impl Finder {
    pub fn new() -> Self {
        Self
    }
}

pub type FinderResult = ();

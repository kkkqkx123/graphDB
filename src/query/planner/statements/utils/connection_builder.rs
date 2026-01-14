//! 连接构建器
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;

#[derive(Debug)]
pub struct ConnectionBuilder;

impl ConnectionBuilder {
    pub fn new() -> Self {
        Self
    }
}

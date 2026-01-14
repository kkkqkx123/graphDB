//! 连接策略
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;

#[derive(Debug)]
pub enum ConnectionType {
    Cartesian,
    InnerJoin,
    LeftJoin,
    PatternApply,
    RollUpApply,
    Sequential,
}

#[derive(Debug)]
pub struct ConnectionStrategy;

impl ConnectionStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug)]
pub struct CartesianStrategy;

impl CartesianStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug)]
pub struct InnerJoinStrategy;

impl InnerJoinStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug)]
pub struct LeftJoinStrategy;

impl LeftJoinStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug)]
pub struct PatternApplyStrategy;

impl PatternApplyStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug)]
pub struct RollUpApplyStrategy;

impl RollUpApplyStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug)]
pub struct SequentialStrategy;

impl SequentialStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug)]
pub struct UnifiedConnector;

impl UnifiedConnector {
    pub fn new() -> Self {
        Self
    }
}

//! 连接策略
use crate::query::context::ast::base::AstContext;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::plan::core::nodes::join_node::CrossJoinNode;
use crate::query::planner::plan::PlanNodeEnum;
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

    pub fn cartesian_product(
        _qctx: &AstContext,
        left: &SubPlan,
        right: &SubPlan,
    ) -> Result<SubPlan, PlannerError> {
        if left.root.is_none() || right.root.is_none() {
            return Ok(if left.root.is_some() { left.clone() } else { right.clone() });
        }

        let left_root = left.root.as_ref().unwrap();
        let right_root = right.root.as_ref().unwrap();

        let cross_join_node = CrossJoinNode::new(left_root.clone(), right_root.clone())
            .map_err(|e| PlannerError::PlanGenerationFailed(format!("Failed to create cross join node: {}", e)))?;

        let cross_join_enum = PlanNodeEnum::CrossJoin(cross_join_node);

        Ok(SubPlan::new(
            Some(cross_join_enum.clone()),
            Some(cross_join_enum),
        ))
    }
}

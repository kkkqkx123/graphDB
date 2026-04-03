use crate::query::executor::{ExecutionContext, ExecutionResult, ExecutionError};
use crate::core::Value;

#[derive(Debug, Clone)]
pub struct FulltextSearchNode {
    pub space_id: u64,
    pub tag_name: String,
    pub field_name: String,
    pub query: String,
    pub limit: Option<usize>,
    pub child: Option<Box<PhysicalPlan>>,
}

#[derive(Debug, Clone)]
pub struct FulltextJoinNode {
    pub search_result_column: String,
    pub vertex_id_column: String,
    pub child: Box<PhysicalPlan>,
}

#[derive(Debug, Clone)]
pub struct ScoreCalcNode {
    pub score_column: String,
    pub child: Box<PhysicalPlan>,
}

#[derive(Debug, Clone)]
pub enum PhysicalPlan {
    FulltextSearch(FulltextSearchNode),
    FulltextJoin(FulltextJoinNode),
    ScoreCalc(ScoreCalcNode),
}

impl PhysicalPlan {
    pub fn space_id(&self) -> Option<u64> {
        match self {
            PhysicalPlan::FulltextSearch(node) => Some(node.space_id),
            _ => None,
        }
    }
}

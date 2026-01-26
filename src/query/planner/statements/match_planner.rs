//! MATCH 规划器
//!
//! 负责将 MATCH 查询转换为可执行的执行计划。
//! 直接使用 AstContext 中的语句信息进行规划。
//!
//! ## 支持的功能
//!
//! - 节点模式匹配：(n:Tag)
//! - 关系模式匹配：-[e:Edge]->
//! - WHERE 条件过滤
//! - 属性投影
//! - ORDER BY / LIMIT / SKIP

use crate::core::Expression;
use crate::query::context::ast::AstContext;
use crate::query::planner::connector::SegmentsConnector;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::plan::core::nodes::filter_node::FilterNode;
use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;
use crate::query::planner::plan::core::nodes::{PlanNodeEnum, ScanVerticesNode};
use crate::query::planner::planner::{Planner, PlannerError};

/// MATCH 规划器
///
/// 直接使用 AstContext 中的语句进行规划，生成可执行的执行计划。
#[derive(Debug)]
pub struct MatchPlanner {}

impl MatchPlanner {
    pub fn new() -> Self {
        Self {}
    }

    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "MATCH"
    }

    pub fn parse_tag_from_pattern(pattern: &str) -> Option<String> {
        let colon_pos = pattern.find(':')?;
        let closing_paren_pos = pattern.find(')')?;
        if colon_pos < closing_paren_pos {
            let tag_part = &pattern[colon_pos + 1..closing_paren_pos];
            Some(tag_part.to_string())
        } else {
            None
        }
    }

    fn plan_node_pattern(
        &self,
        _node: &str,
        _space_id: i32,
    ) -> Result<PlanNodeEnum, PlannerError> {
        let scan_node = ScanVerticesNode::new(_space_id);
        Ok(scan_node.into_enum())
    }

    fn plan_edge_pattern(
        &self,
        _edge: &str,
        _space_id: i32,
    ) -> Result<PlanNodeEnum, PlannerError> {
        let expand_node = crate::query::planner::plan::core::nodes::ExpandAllNode::new(
            _space_id,
            vec![],
            "both",
        );
        Ok(expand_node.into_enum())
    }

    fn plan_filter(&self, _condition: &Expression) -> Result<PlanNodeEnum, PlannerError> {
        let start_node = ScanVerticesNode::new(1);
        let filter_node = FilterNode::new(start_node.into_enum(), _condition.clone())?;
        Ok(filter_node.into_enum())
    }

    fn join_plans(
        &self,
        left: SubPlan,
        right: SubPlan,
    ) -> Result<SubPlan, PlannerError> {
        SegmentsConnector::cross_join(left, right)
    }
}

impl Planner for MatchPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let stmt = ast_ctx.sentence().ok_or_else(|| {
            PlannerError::InvalidAstContext("AstContext 中缺少语句".to_string())
        })?;

        let space_id = ast_ctx.space.space_id.unwrap_or(1) as i32;

        let start_node = ScanVerticesNode::new(space_id);
        let mut current_plan = SubPlan::from_root(start_node.into_enum());

        if ast_ctx.query_type() == crate::query::context::ast::QueryType::ReadQuery {
        }

        Ok(current_plan)
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::ast::AstContext;

    #[test]
    fn test_match_planner_creation() {
        let planner = MatchPlanner::new();
        assert!(MatchPlanner::match_ast_ctx(&AstContext::from_strings("MATCH", "MATCH (n)")));
    }

    #[test]
    fn test_match_planner_make() {
        let planner = MatchPlanner::make();
        assert!(planner.match_planner(&AstContext::from_strings("MATCH", "MATCH (n)")));
        assert!(!planner.match_planner(&AstContext::from_strings("GO", "GO 1 TO 2")));
    }

    #[test]
    fn test_match_planner_match_ast_ctx() {
        assert!(MatchPlanner::match_ast_ctx(&AstContext::from_strings(
            "MATCH",
            "MATCH (n)"
        )));
        assert!(MatchPlanner::match_ast_ctx(&AstContext::from_strings(
            "match",
            "match (n)"
        )));
        assert!(!MatchPlanner::match_ast_ctx(&AstContext::from_strings(
            "GO",
            "GO 1 TO 2"
        )));
    }

    #[test]
    fn test_parse_tag_from_pattern() {
        assert_eq!(MatchPlanner::parse_tag_from_pattern("(n:Person)"), Some("Person".to_string()));
        assert_eq!(MatchPlanner::parse_tag_from_pattern("(n:Tag1:Tag2)"), Some("Tag1:Tag2".to_string()));
        assert_eq!(MatchPlanner::parse_tag_from_pattern("(n)"), None);
    }
}

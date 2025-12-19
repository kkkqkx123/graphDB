//! GO语句规划器
//! 处理Nebula GO查询的规划

use crate::query::context::ast_context::AstContext;
use crate::query::planner::plan::core::{
    ArgumentNode, DedupNode, ExpandAllNode, ExpandNode, FilterNode, InnerJoinNode, ProjectNode,
};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};
use std::sync::Arc;

/// GO查询规划器
/// 负责将GO语句转换为执行计划
#[derive(Debug)]
pub struct GoPlanner;

impl GoPlanner {
    /// 创建新的GO规划器
    pub fn new() -> Self {
        Self
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查AST上下文是否匹配GO查询
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "GO"
    }

    /// 获取匹配和实例化函数
    pub fn get_match_and_instantiate() -> crate::query::planner::planner::MatchAndInstantiate {
        crate::query::planner::planner::MatchAndInstantiate {
            match_func: Self::match_ast_ctx,
            instantiate_func: Self::make,
            priority: 100,
        }
    }
}

impl Planner for GoPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // 实现GO查询的规划逻辑
        println!("Processing GO query planning: {:?}", ast_ctx);

        // 创建执行计划节点
        // 1. 创建参数节点（如果需要）
        let arg_node = Arc::new(ArgumentNode::new(1, "start_vertices"));

        // 2. 创建扩展节点
        let _edge_types = vec!["DEFAULT_EDGE".to_string()]; // 使用默认边类型
        let _expand_node = Arc::new(ExpandNode::new(1, _edge_types.clone(), "out"));

        // 3. 创建ExpandAll节点进行多步扩展
        let direction = "out"; // 默认方向
        let edge_types = vec!["DEFAULT_EDGE".to_string()]; // 使用默认边类型
        let expand_all_node = Arc::new(ExpandAllNode::new(1, edge_types, direction));

        // 4. 如果有JOIN操作，创建JOIN节点
        let join_node: Option<Arc<dyn crate::query::planner::plan::core::PlanNode>> = None; // 简化处理，不使用JOIN

        // 5. 创建过滤节点（如果有过滤条件） - 简化处理
        let filter_node: Option<Arc<dyn crate::query::planner::plan::core::PlanNode>> = None;

        // 6. 创建投影节点
        use crate::graph::expression::Expression;
        use crate::query::validator::YieldColumn;
        let yield_columns = vec![YieldColumn::with_alias(
            Expression::Variable("DEFAULT".to_string()),
            "project_result".to_string(),
        )];

        let last_node: Arc<dyn crate::query::planner::plan::core::PlanNode> =
            if let Some(ref filter_ref) = filter_node {
                filter_ref.clone()
            } else if let Some(ref join_ref) = join_node {
                join_ref.clone()
            } else {
                expand_all_node.clone()
            };

        let project_node = Arc::new(
            ProjectNode::new(last_node, yield_columns)
                .expect("ProjectNode creation should succeed with valid input"),
        );

        // 7. 如果需要去重，创建去重节点
        let final_node: Arc<dyn crate::query::planner::plan::core::PlanNode> = {
            let dedup_node = Arc::new(
                DedupNode::new(project_node)
                    .expect("DedupNode creation should succeed with valid input"),
            );
            dedup_node
        };

        // 创建SubPlan
        let sub_plan = SubPlan {
            root: Some(final_node),
            tail: Some(arg_node),
        };

        Ok(sub_plan)
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl Default for GoPlanner {
    fn default() -> Self {
        Self::new()
    }
}

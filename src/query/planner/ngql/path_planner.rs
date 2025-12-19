//! PATH查询规划器
//! 处理Nebula PATH查询的规划

use crate::query::context::ast_context::AstContext;
use crate::query::planner::plan::core::{
    ArgumentNode, DedupNode, ExpandAllNode, ExpandNode, FilterNode, GetVerticesNode, ProjectNode,
};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};
use std::sync::Arc;

/// PATH查询规划器
/// 负责将PATH查询转换为执行计划
#[derive(Debug)]
pub struct PathPlanner;

impl PathPlanner {
    /// 创建新的PATH规划器
    pub fn new() -> Self {
        Self
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查AST上下文是否匹配PATH查询
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "PATH"
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

impl Planner for PathPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // 实现PATH查询的规划逻辑
        println!("Processing PATH query planning: {:?}", ast_ctx);

        // 1. 创建参数节点，获取起始和结束顶点
        let start_arg_node = Arc::new(ArgumentNode::new(1, "start_vertices"));

        // 2. 创建GetVertices节点来获取顶点
        let _get_vertices_node = Arc::new(GetVerticesNode::new(
            1,
            "start_vertices",
        ));

        // 3. 创建扩展节点进行路径搜索 - 简化实现，使用默认方向和边类型
        let expand_direction = "out"; // 默认方向

        let edge_types = vec!["DEFAULT_EDGE".to_string()];
        let _expand_node = Arc::new(ExpandNode::new(1, edge_types.clone(), expand_direction));

        // 5. 创建ExpandAll节点进行多步扩展
        let expand_all_node = Arc::new(ExpandAllNode::new(1, edge_types, expand_direction));

        // 6. 创建过滤节点（如果有过滤条件）- 简化处理
        use crate::graph::expression::Expression;
        let expr = Expression::Variable("*".to_string());
        let filter_node: Arc<dyn crate::query::planner::plan::core::PlanNode> =
            Arc::new(
                FilterNode::new(expand_all_node.clone(), expr)
                    .expect("FilterNode creation should succeed with valid input"),
            );

        // 7. 创建投影节点
        use crate::graph::expression::Expression;
        use crate::query::validator::YieldColumn;
        let yield_columns = vec![YieldColumn::with_alias(
            Expression::Variable("DEFAULT".to_string()),
            "projected_path".to_string(),
        )];

        let project_node = Arc::new(
            ProjectNode::new(filter_node.clone(), yield_columns)
                .expect("ProjectNode creation should succeed with valid input"),
        );

        // 8. 如果是查找最短路径，可能需要额外的处理
        let final_node: Arc<dyn crate::query::planner::plan::core::PlanNode> =
            {
                // 需要额外的节点来处理最短路径算法
                let dedup_node = Arc::new(
                    DedupNode::new(project_node)
                        .expect("DedupNode creation should succeed with valid input"),
                );
                dedup_node
            };

        // 创建SubPlan
        let sub_plan = SubPlan {
            root: Some(final_node),
            tail: Some(start_arg_node),
        };

        Ok(sub_plan)
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl Default for PathPlanner {
    fn default() -> Self {
        Self::new()
    }
}

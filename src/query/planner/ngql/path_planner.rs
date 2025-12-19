//! PATH查询规划器
//! 处理Nebula PATH查询的规划

use crate::query::context::ast_context::{AstContext, PathContext};
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
        // 从ast_ctx创建PathContext
        let path_ctx = PathContext::new(ast_ctx.clone());

        // 实现PATH查询的规划逻辑
        println!("Processing PATH query planning: {:?}", path_ctx);

        // 1. 创建参数节点，获取起始和结束顶点
        let start_arg_node = Arc::new(ArgumentNode::new(1, &path_ctx.from.user_defined_var_name));
        let _end_arg_node = Arc::new(ArgumentNode::new(2, &path_ctx.to.user_defined_var_name));

        // 2. 创建GetVertices节点来获取顶点
        let _get_vertices_node = Arc::new(GetVerticesNode::new(
            1,
            &path_ctx.from.user_defined_var_name,
        ));

        // 3. 创建扩展节点进行路径搜索
        let expand_direction = if path_ctx.over.direction == "both" {
            "both"
        } else if path_ctx.over.direction == "in" {
            "in"
        } else {
            "out"
        };

        let mut edge_types = path_ctx.over.edge_types.clone();
        // 如果是双向边，设置方向
        if path_ctx.over.direction == "both" {
            edge_types.extend(path_ctx.over.edge_types.iter().map(|et| format!("-{}", et)));
        } else if path_ctx.over.direction == "in" {
            edge_types = path_ctx
                .over
                .edge_types
                .iter()
                .map(|et| format!("-{}", et))
                .collect();
        }

        let _expand_node = Arc::new(ExpandNode::new(1, edge_types.clone(), expand_direction));

        // 5. 创建ExpandAll节点进行多步扩展
        let expand_all_direction = if path_ctx.over.direction == "both" {
            "both"
        } else if path_ctx.over.direction == "in" {
            "in"
        } else {
            "out"
        };

        let expand_all_node = Arc::new(ExpandAllNode::new(1, edge_types, expand_all_direction));

        // 6. 创建过滤节点（如果有过滤条件）
        let filter_node: Arc<dyn crate::query::planner::plan::core::PlanNode> =
            if let Some(ref condition) = path_ctx.filter {
                use crate::graph::expression::Expression;
                let expr = Expression::Variable(condition.clone());
                let filter = Arc::new(
                    FilterNode::new(expand_all_node.clone(), expr)
                        .expect("FilterNode creation should succeed with valid input"),
                );
                filter
            } else {
                expand_all_node
            };

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
            if path_ctx.is_shortest {
                // 需要额外的节点来处理最短路径算法
                let dedup_node = Arc::new(
                    DedupNode::new(project_node)
                        .expect("DedupNode creation should succeed with valid input"),
                );
                dedup_node
            } else {
                project_node
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

//! PATH查询规划器
//! 处理Nebula PATH查询的规划

use crate::core::types::EdgeDirection;
use crate::query::context::ast::{AstContext, PathContext};
use crate::query::planner::plan::core::{
    ArgumentNode, DedupNode, ExpandAllNode, ExpandNode, FilterNode, GetVerticesNode, PlanNodeEnum,
    ProjectNode,
};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};

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
        let path_ctx = PathContext::new(ast_ctx.clone());

        println!("Processing PATH query planning: {:?}", path_ctx);

        // 1. 创建参数节点，获取起始和结束顶点
        let start_arg_node = ArgumentNode::new(1, &path_ctx.from.user_defined_var_name);
        let _end_arg_node = ArgumentNode::new(2, &path_ctx.to.user_defined_var_name);

        // 2. 创建GetVertices节点来获取顶点
        let _get_vertices_node = GetVerticesNode::new(1, &path_ctx.from.user_defined_var_name);

        // 3. 创建扩展节点进行路径搜索
        let expand_direction: EdgeDirection = path_ctx.over.direction.into();

        let mut edge_types = path_ctx.over.edge_types.clone();
        if path_ctx.over.direction == EdgeDirection::Both {
            edge_types.extend(path_ctx.over.edge_types.iter().map(|et| format!("-{}", et)));
        } else if path_ctx.over.direction == EdgeDirection::Incoming {
            edge_types = path_ctx
                .over
                .edge_types
                .iter()
                .map(|et| format!("-{}", et))
                .collect();
        }

        let _expand_node = ExpandNode::new(1, edge_types.clone(), expand_direction);

        let direction_str = match path_ctx.over.direction {
            EdgeDirection::Outgoing => "out",
            EdgeDirection::Incoming => "in",
            EdgeDirection::Both => "both",
        };
        let expand_all_node =
            PlanNodeEnum::ExpandAll(ExpandAllNode::new(2, edge_types, direction_str));

        // 6. 创建过滤节点（如果有过滤条件）
        let filter_node: PlanNodeEnum = if let Some(ref condition) = path_ctx.filter {
            use crate::core::Expression;
            let expr = Expression::Variable(condition.clone());
            match FilterNode::new(expand_all_node.clone(), expr) {
                Ok(node) => PlanNodeEnum::Filter(node),
                Err(_) => expand_all_node.clone(),
            }
        } else {
            expand_all_node.clone()
        };

        // 7. 创建投影节点
        use crate::core::Expression;
        use crate::query::validator::YieldColumn;
        let yield_columns = vec![YieldColumn {
            expr: Expression::Variable("DEFAULT".to_string()),
            alias: "projected_path".to_string(),
            is_matched: false,
        }];

        let project_node: PlanNodeEnum = match ProjectNode::new(filter_node.clone(), yield_columns)
        {
            Ok(node) => PlanNodeEnum::Project(node),
            Err(_) => filter_node.clone(),
        };

        // 8. 如果是查找最短路径，可能需要额外的处理
        let final_node: PlanNodeEnum = if path_ctx.is_shortest {
            match DedupNode::new(project_node.clone()) {
                Ok(node) => PlanNodeEnum::Dedup(node),
                Err(_) => project_node.clone(),
            }
        } else {
            project_node
        };

        // 创建SubPlan
        let sub_plan = SubPlan::new(
            Some(final_node),
            Some(PlanNodeEnum::Argument(start_arg_node)),
        );

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

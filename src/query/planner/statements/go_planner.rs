//! GO语句规划器
//! 处理Nebula GO查询的规划

use crate::core::types::EdgeDirection;
use crate::query::context::ast::{AstContext, GoContext};
use crate::query::planner::plan::core::PlanNodeEnum;
use crate::query::planner::plan::core::{
    ArgumentNode, DedupNode, ExpandAllNode, ExpandNode, FilterNode, InnerJoinNode, ProjectNode,
};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};

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
        let go_ctx = GoContext::new(ast_ctx.clone());
        println!("Processing GO query planning: {:?}", go_ctx);

        let arg_node = ArgumentNode::new(1, &go_ctx.from.user_defined_var_name);
        let arg_node_enum = PlanNodeEnum::Argument(arg_node);

        let mut edge_types = go_ctx.over.edge_types.clone();
        if go_ctx.over.direction == EdgeDirection::Both {
            edge_types = go_ctx.over.edge_types.clone();
        } else if go_ctx.over.direction == EdgeDirection::Incoming {
            edge_types = edge_types.iter().map(|et| format!("-{}", et)).collect();
        }

        let expand_direction: EdgeDirection = go_ctx.over.direction.into();

        let _expand_node = ExpandNode::new(1, edge_types.clone(), expand_direction);

        let direction_str = match go_ctx.over.direction {
            EdgeDirection::Outgoing => "out",
            EdgeDirection::Incoming => "in",
            EdgeDirection::Both => "both",
        };
        let expand_all_node = ExpandAllNode::new(1, go_ctx.over.edge_types.clone(), direction_str);

        let join_node_opt: Option<PlanNodeEnum> = if go_ctx.join_dst {
            let join_key = crate::core::Expression::Variable("_expandall_vid".to_string());
            let left_input =
                PlanNodeEnum::Argument(ArgumentNode::new(1, &go_ctx.from.user_defined_var_name));
            let right_input = PlanNodeEnum::ExpandAll(expand_all_node.clone());

            match InnerJoinNode::new(
                left_input,
                right_input,
                vec![join_key.clone()],
                vec![join_key],
            ) {
                Ok(join) => Some(PlanNodeEnum::InnerJoin(join)),
                Err(_) => None,
            }
        } else {
            None
        };

        let filter_node_opt: Option<PlanNodeEnum> = if let Some(ref condition) = go_ctx.filter {
            let expr = crate::core::Expression::Variable(condition.clone());
            let input_node = if let Some(ref join_node) = join_node_opt {
                join_node.clone()
            } else {
                PlanNodeEnum::ExpandAll(expand_all_node.clone())
            };

            match FilterNode::new(input_node, expr) {
                Ok(filter) => Some(PlanNodeEnum::Filter(filter)),
                Err(_) => None,
            }
        } else {
            None
        };

        let yield_columns = if let Some(ref yield_expr) = go_ctx.yield_expr {
            yield_expr
                .columns
                .iter()
                .map(|col| crate::query::validator::YieldColumn {
                    expr: crate::core::Expression::Variable(col.alias.clone()),
                    alias: col.alias.clone(),
                    is_matched: false,
                })
                .collect()
        } else {
            vec![crate::query::validator::YieldColumn {
                expr: crate::core::Expression::Variable("DEFAULT".to_string()),
                alias: "project_result".to_string(),
                is_matched: false,
            }]
        };

        let last_node: PlanNodeEnum = if let Some(ref filter_node) = filter_node_opt {
            filter_node.clone()
        } else if let Some(ref join_node) = join_node_opt {
            join_node.clone()
        } else {
            PlanNodeEnum::ExpandAll(expand_all_node)
        };

        let project_node_enum = match ProjectNode::new(last_node, yield_columns.clone()) {
            Ok(project) => PlanNodeEnum::Project(project),
            Err(_) => {
                let fallback_project = ProjectNode::new(
                    PlanNodeEnum::Argument(ArgumentNode::new(
                        1,
                        &go_ctx.from.user_defined_var_name,
                    )),
                    yield_columns.clone(),
                )
                .expect("Fallback ProjectNode creation should succeed");
                PlanNodeEnum::Project(fallback_project)
            }
        };

        let final_node: PlanNodeEnum = if go_ctx.distinct {
            match DedupNode::new(project_node_enum.clone()) {
                Ok(dedup) => PlanNodeEnum::Dedup(dedup),
                Err(_) => project_node_enum.clone(),
            }
        } else {
            project_node_enum
        };

        let sub_plan = SubPlan {
            root: Some(final_node),
            tail: Some(arg_node_enum),
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

//! GO语句规划器
//! 处理Nebula GO查询的规划
//!
//! ## 改进说明
//!
//! - 实现完整的表达式过滤逻辑
//! - 改进 JOIN 键处理
//! - 添加属性投影支持

use crate::core::types::EdgeDirection;
use crate::core::types::expression::Expression;
use crate::query::context::ast::{AstContext, GoContext};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};

pub use crate::query::planner::plan::core::nodes::{
    ArgumentNode, DedupNode, ExpandAllNode, FilterNode, GetNeighborsNode, HashInnerJoinNode,
    ProjectNode,
};
pub use crate::query::planner::plan::core::PlanNodeEnum;

/// GO查询规划器
/// 负责将GO语句转换为执行计划
#[derive(Debug)]
pub struct GoPlanner {
    query_context: AstContext,
}

impl GoPlanner {
    /// 创建新的GO规划器
    pub fn new() -> Self {
        Self {
            query_context: AstContext::from_strings("GO", "GO FROM $src"),
        }
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查AST上下文是否匹配GO查询
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "GO"
    }
}

impl Planner for GoPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let go_ctx = GoContext::new(ast_ctx.clone());

        let arg_node = ArgumentNode::new(0, &go_ctx.traverse.from.user_defined_var_name);
        let arg_node_enum = PlanNodeEnum::Argument(arg_node);

        let direction_str = match go_ctx.traverse.over.direction {
            EdgeDirection::Out => "out",
            EdgeDirection::In => "in",
            EdgeDirection::Both => "both",
        };

        let expand_all_node = ExpandAllNode::new(
            1,
            go_ctx.traverse.over.edge_types.clone(),
            direction_str,
        );

        let input_for_join = if go_ctx.join_dst {
            let expand_enum = PlanNodeEnum::ExpandAll(expand_all_node.clone());

            let join_key_left = Expression::Variable("_expandall_vid".to_string());
            let join_key_right = Expression::Variable("_expandall_vid".to_string());

            let left_input =
                PlanNodeEnum::Argument(ArgumentNode::new(0, &go_ctx.traverse.from.user_defined_var_name));

            match HashInnerJoinNode::new(
                left_input,
                expand_enum,
                vec![join_key_left],
                vec![join_key_right],
            ) {
                Ok(join) => PlanNodeEnum::HashInnerJoin(join),
                Err(e) => {
                    return Err(PlannerError::PlanGenerationFailed(format!(
                        "Failed to create join node: {}",
                        e
                    )));
                }
            }
        } else {
            PlanNodeEnum::ExpandAll(expand_all_node)
        };

        let filter_node = if let Some(ref condition) = go_ctx.traverse.filter {
            match FilterNode::new(input_for_join, condition.clone()) {
                Ok(filter) => PlanNodeEnum::Filter(filter),
                Err(e) => {
                    return Err(PlannerError::PlanGenerationFailed(format!(
                        "Failed to create filter node: {}",
                        e
                    )));
                }
            }
        } else {
            input_for_join
        };

        let project_columns = Self::build_yield_columns(&go_ctx)?;
        let project_node = match ProjectNode::new(filter_node, project_columns) {
            Ok(project) => PlanNodeEnum::Project(project),
            Err(e) => {
                return Err(PlannerError::PlanGenerationFailed(format!(
                    "Failed to create project node: {}",
                    e
                )));
            }
        };

        let final_node = if go_ctx.distinct {
            match DedupNode::new(project_node.clone()) {
                Ok(dedup) => PlanNodeEnum::Dedup(dedup),
                Err(_) => project_node,
            }
        } else {
            project_node
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

impl GoPlanner {
    /// 构建YIELD列
    fn build_yield_columns(
        go_ctx: &GoContext,
    ) -> Result<Vec<crate::query::validator::YieldColumn>, PlannerError> {
        let mut columns = Vec::new();

        if let Some(ref yield_expression) = go_ctx.yield_expression {
            for col in &yield_expression.columns {
                columns.push(crate::query::validator::YieldColumn {
                    expression: crate::core::Expression::Variable(col.alias.clone()),
                    alias: col.alias.clone(),
                    is_matched: false,
                });
            }
        } else {
            columns.push(crate::query::validator::YieldColumn {
                expression: crate::core::Expression::Variable("_expandall_dst".to_string()),
                alias: "dst".to_string(),
                is_matched: false,
            });

            columns.push(crate::query::validator::YieldColumn {
                expression: crate::core::Expression::Variable("_expandall_props".to_string()),
                alias: "properties".to_string(),
                is_matched: false,
            });
        }

        if columns.is_empty() {
            columns.push(crate::query::validator::YieldColumn {
                expression: crate::core::Expression::Variable("*".to_string()),
                alias: "result".to_string(),
                is_matched: false,
            });
        }

        Ok(columns)
    }
}

impl Default for GoPlanner {
    fn default() -> Self {
        Self::new()
    }
}

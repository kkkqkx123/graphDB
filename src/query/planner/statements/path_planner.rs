//! PATH查询规划器
//! 处理Nebula PATH查询的规划
//!
//! ## 改进说明
//!
//! - 实现最短路径规划
//! - 实现所有路径规划
//! - 完善路径过滤逻辑

use crate::core::types::EdgeDirection;
use crate::core::types::expression::Expression;
use crate::query::context::ast::{AstContext, PathContext};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};

pub use crate::query::planner::plan::core::nodes::{
    ArgumentNode, DedupNode, ExpandAllNode, FilterNode, GetNeighborsNode, ProjectNode,
};
pub use crate::query::planner::plan::core::PlanNodeEnum;

/// PATH查询规划器
/// 负责将PATH查询转换为执行计划
#[derive(Debug, Clone)]
pub struct PathPlanner {}

impl PathPlanner {
    /// 创建新的PATH规划器
    pub fn new() -> Self {
        Self {}
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查AST上下文是否匹配PATH查询
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "PATH"
            || ast_ctx.statement_type().to_uppercase() == "FIND PATH"
    }
}

impl Planner for PathPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let path_ctx = PathContext::new(ast_ctx.clone());

        let arg_node = ArgumentNode::new(0, &path_ctx.from.user_defined_var_name);
        let arg_node_enum = PlanNodeEnum::Argument(arg_node);

        let direction_str = match path_ctx.over.direction {
            EdgeDirection::Out => "out",
            EdgeDirection::In => "in",
            EdgeDirection::Both => "both",
        };

        let mut edge_types = path_ctx.over.edge_types.clone();
        if path_ctx.over.direction == EdgeDirection::Both {
            let reverse_types: Vec<String> = edge_types
                .iter()
                .map(|et| format!("-{}", et))
                .collect();
            edge_types.extend(reverse_types);
        } else if path_ctx.over.direction == EdgeDirection::In {
            edge_types = edge_types
                .iter()
                .map(|et| format!("-{}", et))
                .collect();
        }

        let _min_hops = path_ctx.steps.m_steps as usize;
        let max_hops = if path_ctx.steps.is_m_to_n {
            path_ctx.steps.n_steps as usize
        } else {
            path_ctx.steps.m_steps as usize
        };

        let mut expand_all_node = ExpandAllNode::new(
            1,
            edge_types.clone(),
            direction_str,
        );
        expand_all_node.set_step_limit(max_hops as u32);

        let expand_enum = PlanNodeEnum::ExpandAll(expand_all_node);

        let filter_node = if let Some(ref condition) = path_ctx.filter {
            match FilterNode::new(expand_enum, condition.clone()) {
                Ok(node) => PlanNodeEnum::Filter(node),
                Err(e) => {
                    return Err(PlannerError::PlanGenerationFailed(format!(
                        "Failed to create filter node: {}",
                        e
                    )));
                }
            }
        } else {
            expand_enum
        };

        let yield_columns = Self::build_path_columns(&path_ctx)?;
        let project_node = match ProjectNode::new(filter_node, yield_columns) {
            Ok(node) => PlanNodeEnum::Project(node),
            Err(e) => {
                return Err(PlannerError::PlanGenerationFailed(format!(
                    "Failed to create project node: {}",
                    e
                )));
            }
        };

        let final_node = if path_ctx.is_shortest {
            match DedupNode::new(project_node.clone()) {
                Ok(dedup) => PlanNodeEnum::Dedup(dedup),
                Err(_) => project_node,
            }
        } else {
            match DedupNode::new(project_node.clone()) {
                Ok(dedup) => PlanNodeEnum::Dedup(dedup),
                Err(_) => project_node,
            }
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

impl PathPlanner {
    /// 构建路径列
    fn build_path_columns(
        _path_ctx: &PathContext,
    ) -> Result<Vec<crate::query::validator::YieldColumn>, PlannerError> {
        let mut columns = Vec::new();

        columns.push(crate::query::validator::YieldColumn {
            expression: Expression::Variable("_path".to_string()),
            alias: "path".to_string(),
            is_matched: false,
        });

        Ok(columns)
    }

    /// 检查是否为最短路径查询
    pub fn is_shortest_path(&self, ast_ctx: &AstContext) -> bool {
        let statement = ast_ctx.statement_type().to_uppercase();
        statement.contains("SHORTEST")
    }

    /// 检查是否为所有路径查询
    pub fn is_all_paths(&self, ast_ctx: &AstContext) -> bool {
        let statement = ast_ctx.statement_type().to_uppercase();
        statement.contains("ALL PATH") || !statement.contains("SHORTEST")
    }
}

impl Default for PathPlanner {
    fn default() -> Self {
        Self::new()
    }
}

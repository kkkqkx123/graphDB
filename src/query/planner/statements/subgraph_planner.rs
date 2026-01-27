//! SUBGRAPH查询规划器
//! 处理Nebula SUBGRAPH查询的规划

use crate::core::types::EdgeDirection;
use crate::query::context::ast::{AstContext, SubgraphContext};
use crate::query::planner::plan::core::nodes::{
    ArgumentNode as Argument, ExpandAllNode as ExpandAll, ExpandNode as Expand,
    FilterNode as Filter, PlanNodeEnum, ProjectNode as Project,
};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};

/// SUBGRAPH查询规划器
/// 负责将SUBGRAPH查询转换为执行计划
#[derive(Debug, Clone)]
pub struct SubgraphPlanner;

impl SubgraphPlanner {
    /// 创建新的SUBGRAPH规划器
    pub fn new() -> Self {
        Self
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查AST上下文是否匹配SUBGRAPH查询
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "SUBGRAPH"
    }

    /// 获取匹配和实例化函数（静态注册版本）
    pub fn get_match_and_instantiate() -> crate::query::planner::planner::MatchAndInstantiateEnum {
        crate::query::planner::planner::MatchAndInstantiateEnum::Subgraph(Self::new())
    }
}

impl Planner for SubgraphPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let subgraph_ctx = SubgraphContext::new(ast_ctx.clone());

        println!("Processing SUBGRAPH query planning: {:?}", subgraph_ctx);

        let arg_node = Argument::new(1, &subgraph_ctx.from.user_defined_var_name);

        let _expand_node = Expand::new(
            1,
            subgraph_ctx.edge_types.iter().cloned().collect::<Vec<_>>(),
            EdgeDirection::Out,
        );

        let expand_all_node = PlanNodeEnum::ExpandAll(ExpandAll::new(
            2,
            subgraph_ctx.edge_types.iter().cloned().collect::<Vec<_>>(),
            "out",
        ));

        let filter_node: PlanNodeEnum = if let Some(ref condition) = subgraph_ctx.filter {
            match Filter::new(
                expand_all_node.clone(),
                condition.clone(),
            ) {
                Ok(node) => PlanNodeEnum::Filter(node),
                Err(_) => expand_all_node.clone(),
            }
        } else {
            expand_all_node.clone()
        };

        let tag_filter_node: PlanNodeEnum = if let Some(ref tag_condition) = subgraph_ctx.tag_filter
        {
            match Filter::new(
                filter_node.clone(),
                tag_condition.clone(),
            ) {
                Ok(node) => PlanNodeEnum::Filter(node),
                Err(_) => filter_node.clone(),
            }
        } else {
            filter_node
        };

        let edge_filter_node: PlanNodeEnum =
            if let Some(ref edge_condition) = subgraph_ctx.edge_filter {
                match Filter::new(
                    tag_filter_node.clone(),
                    edge_condition.clone(),
                ) {
                    Ok(node) => PlanNodeEnum::Filter(node),
                    Err(_) => tag_filter_node.clone(),
                }
            } else {
                tag_filter_node
            };

        let project_node: PlanNodeEnum = match Project::new(edge_filter_node.clone(), vec![]) {
            Ok(node) => PlanNodeEnum::Project(node),
            Err(_) => edge_filter_node,
        };

        let sub_plan = SubPlan::new(Some(project_node), Some(PlanNodeEnum::Argument(arg_node)));

        Ok(sub_plan)
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl Default for SubgraphPlanner {
    fn default() -> Self {
        Self::new()
    }
}

//! SUBGRAPH查询规划器
//! 处理Nebula SUBGRAPH查询的规划

use crate::query::context::ast_context::AstContext;
use crate::query::planner::plan::core::nodes::{
    ArgumentNode as Argument, ExpandAllNode as ExpandAll, ExpandNode as Expand,
    FilterNode as Filter, ProjectNode as Project,
};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};
use std::sync::Arc;

/// SUBGRAPH查询规划器
/// 负责将SUBGRAPH查询转换为执行计划
#[derive(Debug)]
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

    /// 获取匹配和实例化函数
    pub fn get_match_and_instantiate() -> crate::query::planner::planner::MatchAndInstantiate {
        crate::query::planner::planner::MatchAndInstantiate {
            match_func: Self::match_ast_ctx,
            instantiate_func: Self::make,
            priority: 100,
        }
    }
}

impl Planner for SubgraphPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // 实现SUBGRAPH查询的规划逻辑
        println!("Processing SUBGRAPH query planning: {:?}", ast_ctx);

        // 1. 创建参数节点，获取起始顶点
        let arg_node = Arc::new(Argument::new(1, "start_vertices"));

        // 2. 创建扩展节点进行子图扩展
        let _expand_node = Arc::new(Expand::new(2, vec!["DEFAULT_EDGE".to_string()], "out"));

        // 3. 创建ExpandAll节点进行多步扩展
        let expand_all_node = Arc::new(ExpandAll::new(3, vec!["DEFAULT_EDGE".to_string()], "out"));

        // 4. 创建过滤节点（如果有过滤条件）- 简化处理
        let filter_node: Arc<dyn crate::query::planner::plan::core::PlanNode> =
            {
                match Filter::new(
                    expand_all_node.clone(),
                    crate::graph::expression::Expression::Variable("*".to_string()),
                ) {
                    Ok(node) => Arc::new(node),
                    Err(_) => expand_all_node.clone(),
                }
            };

        // 5. 如果有标签过滤，添加额外过滤 - 简化处理
        let tag_filter_node = filter_node;

        // 6. 如果有边过滤，添加额外过滤 - 简化处理
        let edge_filter_node = tag_filter_node;

        // 7. 创建投影节点
        let project_node = match Project::new(edge_filter_node.clone(), vec![]) {
            Ok(node) => Arc::new(node),
            Err(_) => edge_filter_node.clone(),
        };

        // 创建SubPlan
        let sub_plan = SubPlan::new(Some(project_node.clone_plan_node()), Some(arg_node));

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
